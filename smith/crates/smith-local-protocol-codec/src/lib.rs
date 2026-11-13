//! The local store's record codecs (smith/docs/design/protocol/README.md, section 5; tools/crates/design.md, 3.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/local_store_v1.rs"]
pub mod local_store_v1;
