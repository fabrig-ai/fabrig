//! What pages draw with (temper/docs/design/client/view-crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub use temper_client_view_words::{Limits, limits, look, paths, valid, words};
pub mod accounts;
pub mod bindings;
pub mod cards;
pub mod choice;
pub mod column;
pub mod common;
pub mod confirm;
pub mod flow;
pub mod forge;
pub mod forge_sign_in;
pub mod public;
pub mod shared;
pub mod sign_in;
pub mod views;
pub use bindings::link;

#[cfg(test)]
mod tests;
