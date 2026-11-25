//! The test application's client records (protocol/client/README.md, 8).
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

/// The current family schema text (skein/docs/design/data/codec/golden.md, section 1).
#[expect(clippy::disallowed_macros, reason = "the owner embeds its current schema text once")]
pub const SCHEMA: &str = include_str!("../schema/test-client-v2.schema");

pub use jig_client_test_protocol_wire_codec::v2;
pub const VERSION: u32 = 2;
pub use v2::LAYOUT;

pub mod codec;
