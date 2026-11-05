//! Resolution's mmdebstrap machine (hearth/docs/design/domain/render.md, 5.2 to 5.5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod limits;
pub mod lists;
pub mod output;
pub mod packages;
pub mod tail;
#[cfg(test)]
mod tests;
pub mod vector;
