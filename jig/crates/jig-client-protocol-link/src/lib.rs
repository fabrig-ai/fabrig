//! Client attempts and envelopes (protocol/client/README.md, 2, 3 and 10).
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod approval;
mod authorize;
pub mod boundary;
mod component;
pub mod limits;
mod sign_in;
mod silence;
mod store;
pub mod translate;
mod watches;
pub use boundary::*;
/// The kept name of the preceding client's stream (protocol/client/README.md, 7).
pub const STREAM_KEY: &[u8] = b"stream";
pub use component::Link;
pub use limits::{Limits, Unusable};
pub use translate::Undecodable;
#[cfg(test)]
mod tests;

/// Whether the engine serves this client's wire version (protocol/client/README.md, 6).
#[must_use]
pub const fn serves(versions: jig_client_domain::contract::VersionRange) -> bool {
    versions.contains(jig_client_protocol_wire::VERSION)
}
