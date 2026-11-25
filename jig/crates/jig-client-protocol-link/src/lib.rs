//! Client attempts and envelopes (protocol/client/README.md, 2, 3 and 10).
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod approval;
mod asked_link;
mod authorize;
mod batch;
pub mod boundary;
mod component;
mod generated {
    #[rustfmt::skip]
    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "maps use one borrowed signature for every to-record translation (skein/docs/design/data/codec/maps.md, 4.1)"
    )]
    pub(crate) mod client;
}
mod hello;
pub mod limits;
mod retired;
mod sign_in;
mod silence;
mod store;
pub mod translate;
mod watches;
pub use boundary::*;
/// The kept name of the preceding client's stream (protocol/client/small-store.md, 1).
pub const STREAM_KEY: &[u8] = b"stream";
pub use component::Link;
pub use limits::{BATCH_BYTES, Limits, Unusable};
pub use translate::Undecodable;
#[cfg(test)]
mod tests;

/// Whether the engine serves this client's wire version (protocol/client/versions.md, 1).
#[must_use]
pub const fn serves(versions: jig_client_domain::contract::VersionRange) -> bool {
    versions.contains(jig_client_protocol_wire::VERSION)
}
