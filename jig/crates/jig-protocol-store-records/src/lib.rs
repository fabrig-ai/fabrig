//! Jig's records to the store's generated families and back (jig/docs/design/protocol/store/crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;

pub mod codec;

/// Generated store translations (skein/docs/design/data/codec/maps.md, 4).
#[rustfmt::skip]
pub mod generated {
    pub mod accounts;
    pub mod component;
    pub mod core;
    pub mod notes;
    pub mod people;
    pub mod settings;
    pub mod tasks;
}

use jig_protocol_store_core::core_v6;
use jig_protocol_store_families::{accounts_v2, component_v3, notes_v1};
use jig_protocol_store_people::people_v17;
use jig_protocol_store_settings::settings_v3;
use jig_protocol_store_tasks::tasks_v13;

#[cfg(test)]
mod tests;
