use clap::Subcommand;

pub mod build;
pub mod confirm;
pub mod daemon;
pub mod define;
pub mod delete;
pub mod device;
pub mod enter;
pub mod entrypoint;
pub mod keys;
pub mod list;
pub mod wait;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Define a new Litterbox using a template Dockerfile
    #[clap(visible_alias("def"))]
    Define(#[clap(flatten)] define::Command),

    /// Build a new Litterbox
    Build(#[clap(flatten)] build::Command),

    /// List all the Litterboxes that have been created
    #[clap(visible_alias("ls"))]
    List(#[clap(flatten)] list::Command),

    /// Enter an existing Litterbox
    Enter(#[clap(flatten)] enter::Command),

    /// Delete an existing Litterbox
    #[clap(visible_alias("del"), visible_alias("rm"))]
    Delete(#[clap(flatten)] delete::Command),

    /// Manage SSH keys that can be exposed to Litterboxes
    #[command(subcommand)]
    Keys(keys::Command),

    /// Attach a device to a Litterbox (the device fille be created in the home directory)
    #[clap(visible_alias("dev"))]
    Device(#[clap(flatten)] device::Command),

    /// Ask the user to confirm a request (for internal use)
    #[clap(hide = true)]
    Confirm(#[clap(flatten)] confirm::Command),

    /// Run daemon (for internal use)
    #[clap(hide = true)]
    Daemon(#[clap(flatten)] daemon::Command),

    /// Wait for the Litterbox to finish (for internal use)
    #[clap(hide = true)]
    Wait(#[clap(flatten)] wait::Command),

    /// Container entrypoint (for internal use)
    // -h and -V might conflict with a command's arguments
    #[clap(hide = true, disable_help_flag = true, disable_version_flag = true)]
    Entrypoint(#[clap(flatten)] entrypoint::Command),
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Define(command) => command.run(),
            Command::Build(command) => command.run(),
            Command::List(command) => command.run(),
            Command::Enter(command) => command.run(),
            Command::Delete(command) => command.run(),
            Command::Keys(command) => command.run(),
            Command::Device(command) => command.run(),
            Command::Confirm(command) => command.run(),
            Command::Daemon(command) => command.run(),
            Command::Wait(command) => command.run(),
            Command::Entrypoint(command) => command.run(),
        }
    }
}
