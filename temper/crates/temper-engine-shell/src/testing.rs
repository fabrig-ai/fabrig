//! Shared fixtures and adapters for the engine shell's test binaries.
pub use crate::forge::testing::setup_file;

pub mod scratch {
    pub use crate::upgrade::testing::{make, remove, sweep};
}

pub const WORKING: &[u8] = b"/work";

#[expect(clippy::disallowed_types, reason = "scratch fixtures set filesystem modification times without waiting")]
pub fn modified(path: &std::path::Path, time: std::time::SystemTime) {
    std::fs::File::open(path).expect("directory opens for time").set_modified(time).expect("directory time");
}
