//! Hosts of the forge setup machines (protocol/setup/crate-layout.md, 1).
pub mod command;
pub mod fetch;
pub mod files;
pub mod parent;
pub mod prepare;
pub mod process;
pub mod run;
pub mod setup_file;
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
pub(crate) fn path(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}
