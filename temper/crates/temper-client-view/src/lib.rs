//! Temper's own pages on skein-ui and jig-client's documents.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
mod accounts;
mod approve;
pub(crate) mod bindings;
pub mod cards;
mod choice;
pub(crate) mod column;
pub(crate) mod common;
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
pub(crate) use bindings::link;
pub use limits::{Limits, valid};
pub use root::{ShellEvent, View, act, act_event, build, event, href, opening_tab, shell_event};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod view_kit;

#[cfg(test)]
mod words_tests;

#[cfg(test)]
mod parts_tests;

#[cfg(test)]
mod setup_tests;

#[cfg(test)]
mod work_tests;
