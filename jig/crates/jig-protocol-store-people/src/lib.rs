//! The store's people record codecs (jig/docs/design/protocol/README.md, section 5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "generated StoredActed retains its domain fields act and at")]
#[path = "generated/people_v16.rs"]
pub mod people_v16;

#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "generated StoredActed retains its domain fields act and at")]
#[path = "generated/people_v17.rs"]
pub mod people_v17;
