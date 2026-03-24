use log::trace;
use std::process::Command;

/// Binaries that can be used to gain temporarily root privileges.
///
/// Litterbox symlinks them to itself to inform users to use the "--root"
/// argument if one wants to gain root access.
pub const SU_BINARIES: &[&str] = &["run0", "sudo", "doas"];

pub fn trace_arguments(cmd: &Command) {
    trace!(
        "Will run: {} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args().fold(String::new(), |mut acc, arg| {
            acc.push_str(&arg.to_string_lossy());
            acc.push(' ');
            acc
        })
    );
}
