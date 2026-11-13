//! The engine binary's stock settings (temper/docs/design/protocol/services.md, 4.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub mod stock;
#[cfg(test)]
mod tests;
#[cfg(test)]
pub use temper_engine_service_vocabulary::*;
