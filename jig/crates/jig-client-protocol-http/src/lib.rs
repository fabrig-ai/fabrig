//! HTTP calls and printed links (protocol/client/default-http.md, 1, 3 and 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod call;
mod printed;
mod public;
pub use public::{Freshness, PUBLIC_KEEP, PublicHeard, PublicPath, freshness, public, public_answered, public_target};
mod stream;
use alloc::boxed::Box;
pub use call::{Call, answered, exchange, stream};
pub use printed::{Printed, Unlinked, printed};
use skein_lib::bytes;
pub use stream::{Stream, valid};
#[derive(Debug, PartialEq, Eq)]
pub struct Mapping {
    prefix: Box<[u8]>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Unmapped {
    Prefix,
}
impl Mapping {
    pub fn new(prefix: &[u8]) -> Result<Self, Unmapped> {
        if prefix.len() > 64 || !prefix.starts_with(b"/") || prefix.ends_with(b"/") {
            return Err(Unmapped::Prefix);
        }
        Ok(Self { prefix: bytes::copy_of(prefix) })
    }
}
#[cfg(test)]
mod tests;

/// Receiving limits for the transport's stream reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub record: u32,
    pub chunk: u32,
}

pub use jig_client_protocol_link::Heard;
