//! The engine clients' cursor record codecs (jig/docs/design/protocol/components.md, section 1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/cursor.rs"]
pub mod cursor_v2;
