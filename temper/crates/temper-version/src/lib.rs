//! A binary's build identity (temper/docs/design/release.md, 3).
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{format, string::String};

/// The workspace's version (temper/docs/design/release.md, 3).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The build's commit and tracked-file dirty mark (temper/docs/design/release.md, 3).
pub const COMMIT: &str = env!("TEMPER_INTERNAL_COMMIT");

/// The compile-time commit variable (temper/docs/design/cli/README.md, 8).
pub const INTERNAL_COMMIT: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_COMMIT",
    set: skein_shell::Set::Internal,
    form: "COMMIT",
    way: skein_shell::Way::Reads,
    line: "The build's commit and tracked-file dirty mark.",
    absent: "The build script supplies unknown when git identity is unavailable.",
    notes: "Read at compile time from temper-version's build script alone; not a runtime override.",
};

/// A binary's version line, without a newline (temper/docs/design/release.md, 3).
#[must_use]
pub fn line(name: &str) -> String {
    format!("{name} {VERSION} ({COMMIT})")
}
