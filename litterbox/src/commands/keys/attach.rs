use crate::keys::Keys;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
    /// The name of the key
    key_name: String,

    /// The name of the Litterbox
    litterbox_name: String,
}

impl Command {
    pub fn run(self, keys: Keys) -> Result<()> {
        Ok(())
    }
}
