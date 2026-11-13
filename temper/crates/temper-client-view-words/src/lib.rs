//! The view’s vocabulary, look, paths and limits (temper/docs/design/client/view-crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod limits;
pub mod look;
pub mod paths;
pub mod words;
pub use limits::{Limits, valid};

#[cfg(test)]
mod tests;
