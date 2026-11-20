//! Temper's generated store records (temper/docs/design/protocol/store.md, section 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/store.rs"]
#[expect(clippy::large_enum_variant, reason = "generated setting edits own the named repair settings value")]
pub mod v25;
