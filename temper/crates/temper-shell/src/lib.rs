//! What every binary of temper's starts from (temper/docs/design/protocol/README.md, section 5).
#![forbid(unsafe_code)]

use std::{os::unix::ffi::OsStrExt, path::Path};

pub mod commands;
pub mod directories;
pub mod help;
pub mod link_file;
pub mod rows;

fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}

#[cfg(test)]
mod tests;
