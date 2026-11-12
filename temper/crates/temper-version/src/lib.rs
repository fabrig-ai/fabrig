//! A binary's build identity (temper/docs/design/release.md, 3).
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{format, string::String};

/// The workspace's version (temper/docs/design/release.md, 3).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The build's commit and tracked-file dirty mark (temper/docs/design/release.md, 3).
pub const COMMIT: &str = env!("TEMPER_INTERNAL_COMMIT");

/// A binary's version line, without a newline (temper/docs/design/release.md, 3).
#[must_use]
pub fn line(name: &str) -> String {
    format!("{name} {VERSION} ({COMMIT})")
}
