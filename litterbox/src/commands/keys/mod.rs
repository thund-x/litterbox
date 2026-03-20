use anyhow::Result;
use clap::Subcommand;

use crate::keys::Keys;

pub mod attach;
pub mod change_password;
pub mod delete;
pub mod detach;
pub mod generate;
pub mod import;
pub mod list;
pub mod print;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List all the keys are being managed
    #[clap(visible_alias("ls"))]
    List(#[clap(flatten)] list::Command),

    /// Generate a new random key
    Generate(#[clap(flatten)] generate::Command),

    /// Import a key to Litterbox
    Import(#[clap(flatten)] import::Command),

    /// Delete an existing key
    Delete(#[clap(flatten)] delete::Command),

    /// Attach an existing key to a Litterbox
    Attach(#[clap(flatten)] attach::Command),

    /// Detach an attached Litterbox from a key
    Detach(#[clap(flatten)] detach::Command),

    /// Print the key in OpenSSH public key format
    Print(#[clap(flatten)] print::Command),

    /// Change the password used to encrypt passwords for storage
    ChangePassword(#[clap(flatten)] change_password::Command),
}

impl Command {
    pub fn run(self) -> Result<()> {
        let keys = Keys::load()?;

        match self {
            Command::List(command) => command.run(keys),
            Command::Generate(command) => command.run(keys),
            Command::Import(command) => command.run(keys),
            Command::Delete(command) => command.run(keys),
            Command::Attach(command) => command.run(keys),
            Command::Detach(command) => command.run(keys),
            Command::Print(command) => command.run(keys),
            Command::ChangePassword(command) => command.run(keys),
        }
    }
}
