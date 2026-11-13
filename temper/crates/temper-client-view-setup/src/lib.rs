//! The pages that set temper up (temper/docs/design/client/view-crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub use temper_client_view_words::{words, look, paths, limits, Limits, valid};
pub use temper_client_view_parts::{accounts, bindings, cards, choice, column, common, confirm, flow, forge, forge_sign_in, shared, views};
pub mod pages;

#[cfg(test)]
mod tests;
