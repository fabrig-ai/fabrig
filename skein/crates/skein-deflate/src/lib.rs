//! Raw DEFLATE (skein/docs/design/data/deflate.md, 1 to 4; RFC 1951):
//! deterministic encoding and exactly one stream decoded under the caller's capacity.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod decoder;
mod encoder;

pub use decoder::decode;
pub use encoder::{bound, encode};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod fuzzy;
