//! The view’s root, frame and its pieces’ items (temper/docs/design/client/view-crates.md, 2.4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![cfg_attr(test, expect(clippy::manual_map, reason = "step code matches optional fields without closures"))]
extern crate alloc;
mod approve;
pub mod frame;
mod outside;
pub mod pages;
mod root;
pub use root::{ShellEvent, View, act, act_event, build, event, href, opening_tab, shell_event};
#[cfg_attr(
    not(test),
    expect(unused_imports, reason = "the root and its own tests name the pieces at their existing paths")
)]
pub(crate) use temper_client_view_parts::{accounts, bindings, choice, column, common, confirm, link, views};
pub use temper_client_view_parts::{cards, forge, forge_sign_in, public, shared, sign_in};
pub use temper_client_view_words::{Limits, limits, look, paths, valid, words};

#[cfg(test)]
mod tests;
