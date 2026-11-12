//! What every binary of temper's starts from (temper/docs/design/protocol/components.md).
#![forbid(unsafe_code)]

use std::{os::unix::ffi::OsStrExt, path::Path};

pub mod commands;
pub mod directories;
pub mod environment;
pub mod help;
pub mod link_file;
pub mod table;

fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
