//! One socket and its optional TLS (skein/docs/design/net/link.md, 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod link;
#[cfg(test)]
mod tests;
mod upload;

pub use link::{Event, Limits, Link, Moved, Start};
pub use upload::Upload;
