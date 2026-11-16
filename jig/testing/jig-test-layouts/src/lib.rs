//! Registry and record fixtures (jig/docs/design/protocol/store/opening.md, 3).
#![no_std]
#![forbid(unsafe_code)]

pub const NAME: &str = "fixture";
pub mod layouts;
pub mod records;

/// Application key bytes; the store adds tag 2 (protocol/store/README.md, 4.2).
pub mod keys {
    // Relative to the application's secrets prefix (protocol/store/secrets.md, protocol/store/opening.md, 3).
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
