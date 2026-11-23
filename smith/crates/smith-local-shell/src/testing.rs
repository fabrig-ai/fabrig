//! Native operations shared with the local shell's real-loop suite.

pub use crate::credentials::testing::{inspect_key, keep_key, logout, raw_key, read_or_draw};

/// Write a synthetic key fixture with the requested native file mode.
#[must_use]
pub fn key_file(files: &std::path::Path, bytes: &[u8], mode: u32) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = files.join("key");
    std::fs::write(&path, bytes).expect("scratch key");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("key permissions");
    path
}
