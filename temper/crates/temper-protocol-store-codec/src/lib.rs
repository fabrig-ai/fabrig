//! Temper's generated store records (temper/docs/design/protocol/store-versions.md, section 1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/store.rs"]
#[cfg_attr(
    target_pointer_width = "64",
    expect(clippy::large_enum_variant, reason = "generated setting edits own the named repair settings value")
)]
pub mod v25;
