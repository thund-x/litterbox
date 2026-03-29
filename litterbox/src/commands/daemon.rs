use std::{collections::HashSet, ffi::OsString, fs::File, time::Duration};

use anyhow::{Context, Result};
use clap::Args;
use log::debug;
use nix::{
    errno::Errno,
    fcntl::Flock,
    sys::inotify::{InitFlags, Inotify},
    unistd::{dup2_stderr, dup2_stdout},
};
use nix::{fcntl::FlockArg, sys::inotify::AddWatchFlags};
use tokio::runtime::Runtime;

use crate::{daemon, files::lbx_runtime_dir, podman::is_container_running};

/// Run daemon (for internal use)
#[derive(Args, Debug)]
pub struct Command {}

impl Command {
    pub fn run(self) -> Result<()> {
        let lock_file_path = daemon::lock_file_path();
        if let Some(parent) = lock_file_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create daemon lock file directories")?;
        }

        let lock_file =
            File::create(lock_file_path).context("Failed to create daemon lock file")?;
        let _lock = match Flock::lock(lock_file, FlockArg::LockExclusiveNonblock) {
            Ok(lock) => lock,
            Err(_) => {
                debug!("Another daemon holds the lock..");

                return Ok(());
            }
        };

        nix::unistd::daemon(true, false).context("Failed to daemonize")?;
        redirect_stdio()?;

        Runtime::new()
            .expect("Tokio runtime should start")
            .block_on(self.run_async())
    }

    async fn run_async(self) -> Result<()> {
        // TODO: Start SSH server.
        watch_litterboxes()?;
        debug!("Daemon will exit");

        Ok(())
    }
}

fn redirect_stdio() -> Result<(), anyhow::Error> {
    let log_file_path = daemon::log_file_path();

    if let Some(parent) = log_file_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create daemon log file directories")?;
    }

    let log_file_err = File::options()
        .create(true)
        .append(true)
        .open(log_file_path)
        .context("failed to create log file")?;
    let log_file_out = log_file_err.try_clone()?;

    let _ = dup2_stderr(log_file_err);
    let _ = dup2_stdout(log_file_out);

    Ok(())
}

fn watch_litterboxes() -> Result<()> {
    let inotify = Inotify::init(InitFlags::IN_NONBLOCK).context("Failed to init inotify")?;
    let mut running_litterboxes = HashSet::<OsString>::new();
    let mut can_exit = false;

    let base_dir = lbx_runtime_dir();
    let base_dir_wd = inotify
        .add_watch(
            &base_dir,
            AddWatchFlags::IN_ONLYDIR
                | AddWatchFlags::IN_CREATE
                | AddWatchFlags::IN_DELETE
                | AddWatchFlags::IN_ISDIR,
        )
        .context("Failed to watch runtime directory")?;

    // FIXME: This approach seems frail. Should I expose a Unix socket instead
    // for clients to connect to?
    loop {
        match inotify.read_events() {
            Ok(events) => {
                for event in events {
                    if let Some(lbx) = event.name
                        && event.wd == base_dir_wd
                        && event.mask.contains(AddWatchFlags::IN_ISDIR)
                    {
                        if event.mask.contains(AddWatchFlags::IN_DELETE) {
                            running_litterboxes.remove(&lbx);
                        } else if event
                            .mask
                            .intersects(AddWatchFlags::IN_CREATE | AddWatchFlags::IN_OPEN)
                        {
                            running_litterboxes.insert(lbx);
                            can_exit = true;
                        }
                    }
                }
            }

            Err(Errno::EAGAIN) => std::thread::sleep(Duration::from_secs(3)),

            Err(cause) => Err(cause).context("Failed to read events")?,
        }

        running_litterboxes.retain(|lbx| {
            if is_container_running(&lbx.to_string_lossy()).is_ok() {
                true
            } else {
                // Delete litterbox's runtime directory and the session file
                // within it.
                let _ = std::fs::remove_dir_all(base_dir.join(lbx));

                false
            }
        });

        if can_exit && running_litterboxes.is_empty() {
            break;
        }
    }

    Ok(())
}
