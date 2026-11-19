//! HTTP calls and printed links (protocol/client/default-http.md, 1 to 4 and 6).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod call;
mod frames;
pub use frames::Frames;
mod printed;
mod public;
pub use public::{Freshness, PUBLIC_KEEP, PublicHeard, PublicPath, freshness, public, public_answered, public_target};
mod stream;
use alloc::boxed::Box;
pub use call::{Call, answered, exchange, stream};
pub use printed::{Printed, Unlinked, link_url, printed};
use skein_lib::bytes;
pub use stream::{Stream, valid};
#[derive(Debug, PartialEq, Eq)]
pub struct Mapping {
    prefix: Box<[u8]>,
    page: Option<Box<[u8]>>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Unmapped {
    Prefix,
    Page,
}
impl Mapping {
    pub fn new(prefix: &[u8]) -> Result<Self, Unmapped> {
        if prefix.len() > 64 || !prefix.starts_with(b"/") || prefix.ends_with(b"/") {
            return Err(Unmapped::Prefix);
        }
        Ok(Self { prefix: bytes::copy_of(prefix), page: None })
    }
    pub fn with_page(prefix: &[u8], page: &[u8]) -> Result<Self, Unmapped> {
        let mut mapping = Self::new(prefix)?;
        if page.len() > 64
            || !page.starts_with(b"/")
            || page.ends_with(b"/")
            || page.contains(&b'?')
            || page.contains(&b'#')
        {
            return Err(Unmapped::Page);
        }
        if let Some(rest) = page.strip_prefix(prefix)
            && (rest.is_empty() || rest.starts_with(b"/"))
        {
            return Err(Unmapped::Page);
        }
        mapping.page = Some(bytes::copy_of(page));
        Ok(mapping)
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
