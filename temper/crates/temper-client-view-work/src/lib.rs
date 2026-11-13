//! The pages of a project’s work (temper/docs/design/client/view-crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub use temper_client_view_parts::{
    accounts, bindings, cards, choice, common, confirm, flow, forge, forge_sign_in, public, shared, views,
};
pub use temper_client_view_words::{Limits, limits, look, paths, valid, words};
pub mod pages;

#[cfg(test)]
mod tests;
