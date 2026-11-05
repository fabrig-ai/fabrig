//! A fixture owner's kept turn and patch (smith/docs/design/protocol/transcript.md, 12).

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

// The patch owner’s layout puts the schema compiler’s module at ../patches/mod.rs.
#[path = "../patches/mod.rs"]
pub mod patches;
