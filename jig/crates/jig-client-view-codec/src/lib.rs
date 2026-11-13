//! Reference UI record codecs (jig/docs/design/client/domain/view-state.md, 8; tools/crates/design.md, 3.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/records_v1.rs"]
pub mod records_v1;
