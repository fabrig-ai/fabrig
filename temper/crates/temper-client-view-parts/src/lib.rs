//! What pages draw with (temper/docs/design/client/view-crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub use temper_client_view_words::{words, look, paths, limits, Limits, valid};
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
pub mod shared;
pub mod views;
pub mod public;
pub mod sign_in;
pub use bindings::link;

#[cfg(test)]
mod tests;
