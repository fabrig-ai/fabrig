//! Temper's generated store records (temper/docs/design/protocol/store.md, section 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "generated head/held and ending/pending name distinct durable facts")]
#[path = "generated/v20.rs"]
pub mod v20;
