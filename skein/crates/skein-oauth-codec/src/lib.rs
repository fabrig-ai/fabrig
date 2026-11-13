//! OAuth record codec (skein/docs/design/data/codec/README.md, section 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v3.rs"]
pub mod v3;
