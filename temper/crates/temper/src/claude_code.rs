//! Claude Code's launcher (protocol/setup.md, 6.6).
use std::{ffi::OsStr, path::Path};
pub use temper_fetch_shell::launcher::is_launcher;

/// Fetch the pinned build if missing and replace this process, preserving argv.
#[must_use]
pub fn run(program: &OsStr, arguments: &[Box<[u8]>]) -> u8 {
    temper_fetch_shell::launcher::run("temper", program, arguments)
}

pub(crate) fn place(directory: &Path, cache: &Path) -> Result<(), String> {
    temper_fetch_shell::launcher::place(directory, cache)
}
