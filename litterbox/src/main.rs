use anyhow::{Result, bail};
use clap::Parser;
use inquire_derive::Selectable;
use std::{fmt::Display, process::Output};

mod agent;
mod commands;
mod daemon;
mod devices;
mod env;
mod files;
mod keys;
mod podman;
mod sandbox;
mod settings;

use crate::keys::Keys;

fn extract_stdout(output: &Output) -> Result<&str> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        bail!("Podman command failed: {stderr}");
    }

    Ok(str::from_utf8(&output.stdout)?)
}

#[derive(Debug, Copy, Clone, Selectable)]
enum Template {
    OpenSuseTumbleweed,
    UbuntuLts,
    CachyOS,
}

impl Template {
    fn contents(&self) -> &'static str {
        match self {
            Template::OpenSuseTumbleweed => include_str!("../templates/tumbleweed.Dockerfile"),
            Template::UbuntuLts => include_str!("../templates/ubuntu-latest.Dockerfile"),
            Template::CachyOS => include_str!("../templates/cachyos.Dockerfile"),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Template::OpenSuseTumbleweed => "OpenSUSE Tumbleweed",
            Template::UbuntuLts => "Ubuntu LTS",
            Template::CachyOS => "CachyOS",
        }
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

fn gen_random_name() -> String {
    let mut generator = names::Generator::with_naming(names::Name::Numbered);
    let name = generator.next().expect("Name should not be none.");

    format!("lbx-{name}")
}

/// Simple sandbox utility aimed at software development
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: crate::commands::Command,
}

fn main() -> Result<()> {
    env_logger::init();

    let argv_0 = std::env::args().next();
    if matches!(argv_0.as_deref(), Some("run0" | "sudo")) {
        eprintln!(
            "run0/sudo is not supported inside this session. Use 'litterbox enter --root <name>' to enter as root."
        );

        return Ok(());
    }

    let args = Args::parse();

    args.command.run()
}
