//! Strict configuration decoding (hearth/docs/design/protocol/format/README.md, 2.2;
//! hearth/docs/design/domain/render/README.md, 3.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod declaration;
pub mod document;
pub mod entries;
mod error;
pub mod json;
pub mod limits;
pub mod names;
pub mod schema;
pub use error::{Declared, Error, Expected, Fault, Field, Object, Required, Shape};
pub use skein_json::tokenizer::Position;
#[cfg(test)]
mod tests;
