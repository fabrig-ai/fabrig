//! The store's people record codecs (jig/docs/design/protocol/README.md, section 5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/people.rs"]
pub mod people_v17;
