//! The test application's client records (protocol/client/README.md, 8).
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub use jig_client_test_protocol_wire_codec::v2;
pub const VERSION: u32 = 2;

pub mod codec;
