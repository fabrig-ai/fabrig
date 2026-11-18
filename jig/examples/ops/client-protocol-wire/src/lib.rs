//! Ops' family records, shared by its client and engine (examples.md, 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/ops_client.rs"]
pub mod v1;

/// The single version spoken by both connector families.
pub const VERSION: u32 = 1;

pub mod codec;
