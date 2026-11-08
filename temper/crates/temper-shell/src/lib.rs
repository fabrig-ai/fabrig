//! Shared binary shells (temper/docs/design/protocol/README.md, 4.1).
pub mod commands;
pub mod directories;
pub mod help;
pub mod link_file;
#[cfg(test)]
mod tests;

fn path(bytes: &[u8]) -> &std::path::Path {
    use std::os::unix::ffi::OsStrExt;
    std::path::Path::new(std::ffi::OsStr::from_bytes(bytes))
}
