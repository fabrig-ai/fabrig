//! A branch's path and stock ways (temper/docs/design/domain/forge.md, 8.13;
//! temper/docs/design/protocol/configuration.md, 13.10).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod path;
#[cfg(test)]
mod tests;
mod ways;
mod words;

pub use path::*;
pub use ways::*;
pub use words::{can, cut_on_word, first_sentence};
