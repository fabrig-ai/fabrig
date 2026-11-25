//! Ops' family records, shared by its client and engine (examples.md, 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

/// The current family schema text (skein/docs/design/data/codec/golden.md, section 1).
#[expect(clippy::disallowed_macros, reason = "the owner embeds its current schema text once")]
pub const SCHEMA: &str = include_str!("../schema/ops-client-v1.schema");


#[rustfmt::skip]
#[path = "generated/ops_client.rs"]
pub mod v1;

/// The single version spoken by both connector families.
pub const VERSION: u32 = 1;
/// The layout spoken by both connector families.
pub use v1::LAYOUT;

pub mod codec;
