//! Common items for `litterbox entrypoint`.

use clap::Args;
use std::{
    ffi::OsString,
    fmt::Display,
    str::{FromStr, ParseBoolError},
};
use strum_macros::EnumString;

#[derive(Clone, Debug, Copy)]
pub struct Tty(pub bool);

impl Display for Tty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Tty {
    type Err = ParseBoolError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Interactive(pub bool);

impl Display for Interactive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Interactive {
    type Err = ParseBoolError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

// If you add a new field, make sure to pass it inside the container in
// `container_exec_entrypoint`.
#[derive(Args, Debug)]
pub struct CommonEntrypointOptions {
    /// Run as root instead of dropping privileges.
    #[arg(long, default_value_t = false)]
    pub root: bool,

    /// Set to "foreground" to wait for background processes to finish
    ///
    /// Set to "background" for background processes to continue in the
    /// background
    ///
    /// Set to "kill" to end all background processes
    #[arg(long, default_value_t = Default::default())]
    pub wait: WaitBehaviour,

    /// The command to execute with the login shell.
    pub command: Option<OsString>,

    /// Additional arguments to pass to COMMAND.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<OsString>,
}

#[derive(Clone, Copy, Debug, Default, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum WaitBehaviour {
    /// Wait for orphaned processes to exit.
    #[default]
    Foreground,
    /// Send orphaned processes to `litterbox wait`, ending current session.
    Background,
    /// Kill orphaned processes with `SIGTERM` and after a while `SIGKILL`.
    Kill,
}
