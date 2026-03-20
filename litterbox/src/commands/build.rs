use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
    /// The name of the Litterbox to build
    name: String,
}

impl Command {
    pub fn run(self) -> Result<()> {
        Ok(())
    }
}
