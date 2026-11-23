//! The notes example's record codecs (examples.md, 9.2; tools/crates/design.md, 3.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

/// The current notes schema text (data/codec/golden.md, section 1).
#[expect(clippy::disallowed_macros, reason = "the codec exposes its owner's schema to tests without the UI")]
pub const SCHEMA: &str = include_str!("../../view/schema/notes-v1.schema");

#[rustfmt::skip]
#[path = "generated/notes.rs"]
pub mod records;
