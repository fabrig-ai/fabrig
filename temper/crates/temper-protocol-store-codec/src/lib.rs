//! Temper's generated store records (temper/docs/design/protocol/store.md, section 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "the generated change fields ending and pending name distinct durable facts")]
#[path = "generated/v15.rs"]
pub mod v15;
