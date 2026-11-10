//! The box's steps, restarts and ordered shutdown (protocol/setup/box.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub mod front;
mod group_records;
pub mod host;
pub mod layout;
pub mod limits;
pub mod lines;
mod logs;
pub mod machine;
pub mod pipe;
mod ready;
mod records;
pub mod sink;
pub mod terminal;
#[cfg(test)]
mod tests;
