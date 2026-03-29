use anyhow::{Context, Result};

use crate::{env, files};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

/// Returns the absolute file path of the daemon lock file.
///
/// Note: Parent directories won't be created for you.
pub fn lock_file_path() -> PathBuf {
    files::lbx_runtime_dir().join("daemon.lock")
}

/// Returns the absolute file path of the daemon log file.
///
/// Note: Parent directories won't be created for you.
pub fn log_file_path() -> PathBuf {
    files::lbx_state_dir().join("daemon.log")
}

pub fn try_start_daemon() -> Result<()> {
    let mut cmd = Command::new(env::litterbox_binary_path());

    cmd.arg("daemon");
    cmd.stdin(Stdio::null());
    cmd.spawn()
        .context("Failed to spawn litterbox daemon")?
        .wait()
        .map(|_| ())
        .context("Failed to wait littebox daemon")
}
