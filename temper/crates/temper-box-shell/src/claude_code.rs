//! Claude Code's launcher (protocol/setup/owned.md, 6).
use std::ffi::OsStr;
pub use temper_launcher::is_launcher;

/// Fetch the pinned build if missing and replace this process, preserving argv.
#[must_use]
pub fn run(program: &OsStr, arguments: &[Box<[u8]>]) -> u8 {
    temper_launcher::run("temper", program, arguments)
}
