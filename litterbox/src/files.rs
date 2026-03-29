use anyhow::{Context, Result};
use log::{debug, info};
use nix::unistd::{Pid, Uid};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{
    fs::{self, File},
    io::ErrorKind,
};

use crate::env;

fn path_relative_to_lbx_root(relative_path: &str) -> Result<PathBuf> {
    let home_dir = env::home_dir()?;
    let home_path = Path::new(&home_dir);
    let full_path = home_path.join("Litterbox").join(relative_path);

    Ok(full_path)
}

pub fn dockerfile_path(lbx_name: &str) -> Result<PathBuf> {
    path_relative_to_lbx_root(&format!("definitions/{lbx_name}.Dockerfile"))
}

pub fn keyfile_path() -> Result<PathBuf> {
    path_relative_to_lbx_root("keys.ron")
}

pub fn lbx_home_path(lbx_name: &str) -> Result<PathBuf> {
    path_relative_to_lbx_root(&format!("homes/{lbx_name}"))
}

pub fn settings_path(lbx_name: &str) -> Result<PathBuf> {
    path_relative_to_lbx_root(&format!("definitions/{lbx_name}.ron"))
}

pub fn session_lock_path(lbx_name: &str) -> Result<PathBuf> {
    path_relative_to_lbx_root(&format!(".session-{lbx_name}.lock"))
}

/// Returns the runtime directory of litterbox. Any files placed there will be
/// deleted on log out.
///
/// Note: Parent directories won't be created for you.
pub fn lbx_runtime_dir() -> PathBuf {
    let runtime_dir: PathBuf = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", Uid::current()))
        .into();

    runtime_dir.join("littebox")
}

/// Returns the state directory of litterbox.
///
/// Note: Parent directories won't be created for you.
pub fn lbx_state_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable should have been set");
    let state_dir: PathBuf = std::env::var("XDG_STATE_HOME")
        .unwrap_or_else(|_| format!("{home}/.local/state"))
        .into();

    state_dir.join("litterbox")
}

pub fn append_pid_to_session_lockfile(path: &Path, pid: Pid) -> Result<()> {
    let mut pids = read_pids_from_session_lockfile(path)?;

    if !pids.contains(&pid) {
        pids.push(pid);

        write_pids_to_session_lockfile(path, &pids)?;
    }

    Ok(())
}

pub fn remove_pid_from_session_lockfile(path: &Path, pid: Pid) -> Result<()> {
    let mut pids = read_pids_from_session_lockfile(path)?;

    pids.retain(|&p| p != pid);
    write_pids_to_session_lockfile(path, &pids)?;

    Ok(())
}

pub fn write_pids_to_session_lockfile(path: &Path, pids: &[Pid]) -> Result<()> {
    let content = pids
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    write_file(path, &content)
}

pub fn read_pids_from_session_lockfile(path: &Path) -> Result<Vec<Pid>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = read_file(path)?;

    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.trim()
                .parse()
                .map(Pid::from_raw)
                .map_err(anyhow::Error::from)
        })
        .collect()
}

pub fn pipewire_socket_path() -> Result<PathBuf> {
    let mut xdg_runtime_dir = env::xdg_runtime_dir()?;
    xdg_runtime_dir.push("pipewire-0");

    Ok(xdg_runtime_dir)
}

pub fn write_file(path: &Path, contents: &str) -> Result<()> {
    let output_dir = path.parent().expect("Path should have parent.");
    fs::create_dir_all(output_dir)?;
    fs::write(path, contents)?;
    Ok(())
}

pub fn read_file(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub struct SshSockFile {
    path: PathBuf,
}

impl SshSockFile {
    pub fn new(lbx_name: &str, create_empty_placeholder: bool) -> Result<Self> {
        let path = path_relative_to_lbx_root(&format!(".ssh/{lbx_name}.sock"))?;
        let path_ref = &path;

        if fs::exists(path_ref)? {
            log::warn!("Deleting old SSH socket: {:#?}", path_ref);
            fs::remove_file(path_ref)?;
        } else {
            let ssh_dir = path_ref.parent().expect("SSH path should have parent.");
            fs::create_dir_all(ssh_dir)?;

            if create_empty_placeholder {
                fs::File::create(path_ref)?;
            }
        }

        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SshSockFile {
    fn drop(&mut self) {
        if let Ok(true) = fs::exists(self.path.clone()) {
            if let Err(e) = fs::remove_file(self.path.clone()) {
                log::error!("Failed to remove {:#?}, error: {:#?}", self.path, e);
            }
        } else {
            log::error!("No SSH socket file to clean up: {:#?}", self.path());
        }
    }
}

pub fn setup_home() -> Result<()> {
    let marker = env::home_dir()?.join(".home-built");

    if marker.exists() {
        debug!("Home already built; skipping.");
    } else {
        info!("Building home for the first time");

        Command::new("/prep-home.sh")
            .status()
            .or_else(|cause| {
                // The script is optional.
                (cause.kind() == ErrorKind::NotFound)
                    .then(Default::default)
                    .ok_or(cause)
            })
            .context("Running /prep-home.sh")?;

        File::create(&marker).context("Creating .home-built marker")?;
        info!("Home built!");
    }

    Ok(())
}
