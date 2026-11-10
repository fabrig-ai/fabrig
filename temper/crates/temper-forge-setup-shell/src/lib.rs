//! Hosts of the forge setup machines (protocol/setup/README.md, 7).
pub mod command;
pub mod fetch;
pub mod files;
pub mod parent;
pub mod prepare;
pub mod process;
pub mod run;
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
pub(crate) fn path(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}
