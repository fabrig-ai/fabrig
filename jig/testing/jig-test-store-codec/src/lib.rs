//! The test store's record codecs (tools/crates/design.md, section 3.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v2.rs"]
pub mod v2;
