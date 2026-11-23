//! The store's settings record codecs (jig/docs/design/protocol/components.md, section 1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/settings.rs"]
pub mod settings_v3;
