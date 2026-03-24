//! Common items for `litterbox entrypoint`.

use clap::Args;
use std::{
    ffi::OsString,
    fmt::Display,
    str::{FromStr, ParseBoolError},
};

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

    /// When set to `true`, it will wait for background processes to finish
    /// in the foreground. When set to `false`, it will send SIGKILL to all
    /// background processes. If it's not specified, litterbox will wait for
    /// background processes in the background.
    #[arg(long)]
    pub wait: Option<bool>,

    /// The command to execute with the login shell.
    pub command: Option<OsString>,

    /// Additional arguments to pass to COMMAND.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<OsString>,
}
