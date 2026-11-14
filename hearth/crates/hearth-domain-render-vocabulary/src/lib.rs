//! Render's shared values (hearth/docs/design/domain/render/contracts.md, 1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod diff;
pub mod errors;
pub mod evaluate;
pub mod exports;
pub mod keys;
pub mod parts;
pub mod resolution;
pub mod revision;
pub mod vocabulary;
