use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {}

impl Command {
    pub fn run(self) -> Result<()> {
        Ok(())
    }
}
