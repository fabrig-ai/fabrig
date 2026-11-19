//! The store's families record codecs (jig/docs/design/protocol/README.md, section 5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/accounts.rs"]
pub mod accounts_v2;

#[rustfmt::skip]
#[path = "generated/component.rs"]
pub mod component_v3;

#[rustfmt::skip]
#[path = "generated/notes.rs"]
pub mod notes_v1;

#[rustfmt::skip]
#[path = "generated/secrets.rs"]
pub mod secrets_v1;

#[rustfmt::skip]
#[path = "generated/openings.rs"]
pub mod openings_v1;

#[rustfmt::skip]
#[path = "generated/registry.rs"]
pub mod registry_v1;
