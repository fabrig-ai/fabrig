//! Temper's generated store records (temper/docs/design/protocol/store.md, section 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/store.rs"]
pub mod v25;
