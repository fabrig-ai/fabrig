//! What every binary of temper's starts from (temper/docs/design/protocol/README.md, section 5).
#![forbid(unsafe_code)]

use std::{os::unix::ffi::OsStrExt, path::Path};

pub mod directories;
pub mod link_file;

fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
