//! Registry, record and frame fixtures (jig/docs/design/protocol/store/opening-owners.md, 1).
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

pub const NAME: &str = "fixture";
pub const JIG_OWNER: &str = "fixture-jig";
pub const SMITH_OWNER: &str = "fixture-smith";

pub mod frames;
pub mod layouts;
pub mod records;

/// Fixture key ranges (protocol/store/keys.md, 1.2).
pub mod keys {
    // Whole map keys in jig's slot.
    pub const NOTES_FROM: &[u8] = &[0x02, b'n'];
    pub const NOTES_TO: &[u8] = &[0x02, b'o'];
    pub const NOTES_ROOT: &str = "Note";

    // Application keys are relative to tag 2.
    // Relative to the application's secrets prefix (protocol/store/secrets.md, protocol/store/opening-owners.md, 1).
    pub const TOKENS_FROM: &[u8] = &[];
    pub const TOKENS_TO: &[u8] = &[0x80];
    pub const TOKENS_ROOT: &str = "Token";
    pub const SHELVES_FROM: &[u8] = b"s";
    pub const SHELVES_TO: &[u8] = b"t";
    pub const SHELVES_ROOT: &str = "Shelf";
    pub const LABELS_FROM: &[u8] = b"l";
    pub const LABELS_TO: &[u8] = b"m";
    pub const LABELS_ROOT: &str = "Label";
}
