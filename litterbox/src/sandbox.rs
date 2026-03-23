use crate::{env, files::setup_home};
use anyhow::{Context, Result, bail};
use landlock::{
    ABI, Access, AccessFs, Ruleset, RulesetAttr, RulesetCreatedAttr, path_beneath_rules,
};
use log::{debug, error, warn};
use nix::{
    libc::WNOWAIT,
    sys::{
        prctl::set_child_subreaper,
        wait::{WaitPidFlag, WaitStatus, wait, waitpid},
    },
    unistd::{Gid, Pid, Uid, chown, setgid, setuid},
};
use std::{
    ffi::OsString,
    os::unix::{fs::symlink, prelude::ExitStatusExt},
    path::Path,
    process::{Command, ExitStatus, Stdio},
};

pub fn apply_landlock() -> Result<()> {
    let access_all = AccessFs::from_all(ABI::V6);

    let ruleset = Ruleset::default();
    let ruleset = ruleset.handle_access(access_all)?;

    // We avoid giving full access to the container's entire root directory so that we can
    // deny access to "internal" files that Litterbox places within the root directory.
    let read_dir = std::fs::read_dir("/")?;
    let paths: Vec<_> = read_dir
        .filter_map(|e| {
            let path = e.ok()?.path();
            if path.is_dir() { Some(path) } else { None }
        })
        .collect();

    let ruleset = ruleset.create()?;
    let ruleset = ruleset.add_rules(path_beneath_rules(paths, access_all))?;
    let ruleset = ruleset.add_rules(path_beneath_rules(["/"], AccessFs::ReadDir))?;
    let ruleset = ruleset.add_rules(path_beneath_rules(
        ["/litterbox", "/prep-home.sh"],
        AccessFs::Execute | AccessFs::ReadFile,
    ))?;

    match ruleset.restrict_self() {
        Ok(status) => {
            eprintln!(
                "Landlock sandbox applied: {:?}, no_new_privs: {}",
                status.ruleset, status.no_new_privs
            );
        }
        Err(e) => {
            eprintln!("Failed to apply Landlock sandbox: {:?}", e);
        }
    }

    Ok(())
}

pub fn entrypoint(
    root: bool,
    uid: Uid,
    gid: Gid,
    prog_name: Option<OsString>,
    args: Vec<OsString>,
) -> Result<()> {
    let run0_path = Path::new("/usr/bin/run0");
    if !run0_path.exists() {
        symlink("/litterbox", run0_path)?;
    }

    let sudo_path = Path::new("/usr/bin/sudo");
    if !sudo_path.exists() {
        symlink("/litterbox", sudo_path)?;
    }

    let xdg_runtime_dir = env::xdg_runtime_dir().context("$XDG_RUNTIME_DIR is not set")?;

    chown(&xdg_runtime_dir, Some(uid), Some(gid))
        .context("Failed to set owner of $XDG_RUNTIME_DIR")?;

    if !root {
        setgid(gid)?;
        setuid(uid)?;
        debug!("Dropped permissions to non-root user");
    }

    apply_landlock()?;
    setup_home()?;

    let mut cmd = Command::new(&env::shell()?);
    cmd.stderr(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stdin(Stdio::inherit());
    cmd.arg("-l");

    if let Some(mut exec_args) = prog_name {
        // We can't use Command::args for "command" because shells generally
        // expect a single argument for the "-c" option.
        for arg in args {
            exec_args.push(" ");
            exec_args.push(arg);
        }

        cmd.arg("-c");
        cmd.arg(exec_args);
    }

    let shell_child = cmd.spawn().context("Failed to launch shell")?;
    let shell_pid = Pid::from_raw(shell_child.id() as i32);

    set_child_subreaper(true).context("failed to make process child subreaper")?;

    loop {
        match waitpid(None, None) {
            Ok(WaitStatus::Exited(pid, status)) => {
                let status = ExitStatus::from_raw(status);

                // TODO: Why aren't log messages being printed?
                // eprintln!("Child {pid} exited with status: {status:?}");
                debug!("Child {pid} exited with status: {status:?}");

                if pid == shell_pid {
                    if !status.success() {
                        bail!("Failed to execute program {:?}", cmd.get_program());
                    } else {
                        // TODO: How do I handle orphans?
                    }
                }
            }

            Ok(WaitStatus::Signaled(pid, signal, _)) => {
                // eprintln!("Child {pid} was killed with signal {signal}");
                debug!("Child {pid} was killed with signal {signal}");

                if pid == shell_pid {
                    warn!("Login shell was killed with signal {signal}");
                } else {
                    // TODO: How do I handle orphans?
                }
            }

            Ok(
                status @ (WaitStatus::PtraceEvent(..)
                | WaitStatus::PtraceSyscall(..)
                | WaitStatus::Continued(..)
                | WaitStatus::Stopped(..)
                | WaitStatus::StillAlive),
            ) => {
                // eprintln!("Child signaled with unhandled status: {status:?}");
                warn!("Child signaled with unhandled status: {status:?}");
            }

            Err(nix::errno::Errno::ECHILD) => {
                // eprintln!("Received error ECHILD");
                debug!("Received error ECHILD");
                // No calling processes are waiting anymore
                break;
            }

            Err(cause) => bail!(cause),
        }
    }

    Ok(())
}
