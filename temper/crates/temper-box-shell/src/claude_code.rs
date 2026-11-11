//! Claude Code's launcher (protocol/setup/owned.md, 6).
use std::ffi::OsStr;
pub use temper_launcher::is_launcher;
use temper_shell::directories::EnvironmentEntry;

/// Fetch the pinned build if missing and replace this process, preserving argv.
#[must_use]
pub fn run(program: &OsStr, arguments: &[Box<[u8]>], environment: &[EnvironmentEntry]) -> u8 {
    temper_launcher::run("temper", program, arguments, environment)
}
