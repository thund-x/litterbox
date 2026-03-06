use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::env;

pub fn litterbox_binary_path() -> String {
    std::env::args()
        .next()
        .expect("Binary path should be defined.")
}

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

pub fn ssh_daemon_lock_path(lbx_name: &str) -> Result<PathBuf> {
    path_relative_to_lbx_root(&format!(".ssh-daemon-{lbx_name}.lock"))
}

pub fn pipewire_socket_path() -> Result<PathBuf> {
    let xdg_runtime_dir = env::xdg_runtime_dir()?;
    let path = format!("{xdg_runtime_dir}/pipewire-0");
    Ok(Path::new(&path).to_path_buf())
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
