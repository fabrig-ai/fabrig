//! Process-tier fault injection at the shell's listen/drain boundary.
//! This module may abort its own process, never create or supervise one.
use temper_shell::directories::EnvironmentEntry;

/// Retain the cut marker from both switches at host construction.
pub(crate) fn marker(environment: &[EnvironmentEntry]) -> Option<std::path::PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    crate::WAIT_CHECK.value(environment)?;
    crate::CUT_AT_LISTEN.value(environment).map(|value| std::path::Path::new(std::ffi::OsStr::from_bytes(value)).into())
}

/// Cut one start before submitting its listener announcement. An exclusive
/// marker makes the box's next start continue without another cut.
pub(crate) fn cut_at_listen(marker: Option<&std::path::Path>) {
    let Some(marker) = marker else { return };
    match std::fs::OpenOptions::new().write(true).create_new(true).open(marker) {
        Ok(marker) => {
            drop(marker);
            std::process::abort();
        }
        Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(why) => panic!("listen cut marker: {why}"),
    }
}
