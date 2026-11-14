//! Shared engine service vocabulary (temper/docs/design/protocol/services.md, 4.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;

pub mod assignment;
pub mod clients;
pub mod family;
pub mod forge;
pub mod forges;
pub mod limits;
pub mod settings;
pub mod startup;
pub mod store;
pub mod system;
pub mod tools;
pub mod translate;
pub mod values;
pub use forge::ForgeRuntime;
pub use jig_protocol_engine_clients_http::server;
pub use store::UnreadableRow;
pub use store::route::{CommitStop, RouteStop};
pub use temper_service_core::tokens;
pub use tools::{render_for_task, render_for_task_page, settings_note};
#[cfg(test)]
mod tests;
#[cfg(test)]
pub use temper_engine_service_stock::stock;
