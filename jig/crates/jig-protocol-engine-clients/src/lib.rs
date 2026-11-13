#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
//! Native client exchanges and streams (protocol/client/README.md, section 2 and protocol/client/sign-in.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

pub use jig_protocol_engine_clients_codec::cursor_v1;
pub use jig_protocol_transcript::Private;
pub mod boundary;
mod component;
mod exchange;
pub mod host;
pub mod limits;
pub mod public;
mod routes;
mod served;
mod sign_in;
mod stream;
pub use boundary::*;
pub use component::{Component, Unusable};
pub use limits::{Limits, PublicLimits, provider_limits, valid};
pub use public::{PublicKind, jig_kinds};

pub mod translate;

/// The first printed link's secret length (jig/docs/design/protocol/client/sign-in.md).
pub const LINK_SECRET_BYTES: usize = 32;
