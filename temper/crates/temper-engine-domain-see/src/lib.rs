//! `see` addresses and answers (temper/docs/design/domain/agent/ask-temper.md, 1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]

extern crate alloc;

mod address;
mod answer;
pub mod forge;
pub mod inventory;
pub mod person;

pub use address::{Address, address};
pub use answer::*;
