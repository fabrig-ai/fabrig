//! Checked engine startup (temper/docs/design/protocol/services.md, 4.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;

pub use jig_protocol_accounts::host::Runtime as AccountsRuntime;
pub use temper_engine_service_stock::*;
pub use temper_engine_service_vocabulary::*;
pub mod startup;
pub use startup::{Named, StoreLimits, named, store_limits};
#[cfg(test)]
mod tests;
