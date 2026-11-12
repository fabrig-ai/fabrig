//! Temper's own pages on skein-ui and jig-client's documents.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
mod accounts;
mod approve;
pub mod cards;
mod choice;
mod confirm;
mod flow;
pub mod forge;
pub mod forge_sign_in;
pub mod frame;
pub mod limits;
pub mod look;
mod outside;
pub mod pages;
pub mod paths;
pub mod public;
mod root;
pub mod shared;
pub mod sign_in;
mod views;
pub mod words;
pub use limits::{Limits, valid};
pub use root::{ShellEvent, View, act, act_event, build, event, href, opening_tab, shell_event};

/// Open a bound link with its address, leaving its contents to the caller.
pub(crate) fn link(b: &mut skein_ui::Builder, name: &[u8], to: skein_ui::Binding) {
    b.open(skein_ui::Role::Link);
    b.name(name);
    b.bind(to);
    b.link(to);
}

#[cfg(test)]
mod tests;
