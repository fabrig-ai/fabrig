//! The view’s root, frame and its pieces’ items (temper/docs/design/client/view-crates.md, 2.4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod approve;
pub mod frame;
mod outside;
pub mod pages;
mod root;
pub use temper_client_view_words::{words, look, paths, limits, Limits, valid};
pub use temper_client_view_parts::{cards, forge, forge_sign_in, public, shared, sign_in};
pub(crate) use temper_client_view_parts::{accounts, bindings, choice, column, common, confirm, flow, views, link};
pub use root::{ShellEvent, View, act, act_event, build, event, href, opening_tab, shell_event};

#[cfg(test)]
mod tests;
