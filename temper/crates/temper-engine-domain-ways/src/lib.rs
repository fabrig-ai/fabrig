//! A branch's path and stock ways (temper/docs/design/domain/forge/change/branch-path.md;
//! temper/docs/design/protocol/configuration/settings.md, 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod path;
#[cfg(test)]
mod tests;
mod ways;
mod words;

pub use path::cards::role_card;
pub use path::*;
pub use ways::*;
pub use words::{can, cut_on_word, first_sentence, role_who};
