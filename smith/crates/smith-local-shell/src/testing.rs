//! Opened startup facts and native helpers shared with the local shell's test worlds.

pub use crate::credentials::testing::{inspect_key, read_or_draw};
pub use crate::local_host::{PrivateChat, ProductFiles, ResultDestination};

/// Write a synthetic key fixture with the requested native file mode.
#[must_use]
pub fn key_file(files: &std::path::Path, bytes: &[u8], mode: u32) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = files.join("key");
    std::fs::write(&path, bytes).expect("scratch key");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("key permissions");
    path
}
