//! jig's part of the agent tools (temper/docs/design/client/ctl/tools-crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub mod jig;
#[cfg(test)]
mod tests;
