//! The store's families record codecs (jig/docs/design/protocol/README.md, section 5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/accounts_v2.rs"]
pub mod accounts_v2;

#[rustfmt::skip]
#[path = "generated/component_v3.rs"]
pub mod component_v3;

#[rustfmt::skip]
#[path = "generated/notes_v1.rs"]
pub mod notes_v1;

#[rustfmt::skip]
#[path = "generated/secrets_v1.rs"]
pub mod secrets_v1;

#[rustfmt::skip]
#[path = "generated/openings_v1.rs"]
pub mod openings_v1;
