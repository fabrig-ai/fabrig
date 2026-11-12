//! One local chat and its spawned or colocated agent (protocol/host.md, sections 5–6
//! ; protocol/README.md, sections 5–6). The service keeps local
//! policy and the selected lower components, a credential-value
//! table, and internal routing queues. Host IO owns terminal and
//! delivery descriptors; each lower layer has its own operation tokens.
//! It never knows OAuth provider responses or token-file paths. [`iterate`]
//! runs the up, domain and down passes, including the durable store; the shell
//! settles credential requests in later iterations. Exit waits for lower closes.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(
    clippy::manual_map,
    reason = "closures are out of step code (programming-model.md 10.3), so an Option is mapped by a match"
)]

extern crate alloc;

mod credentials;

#[cfg(test)]
mod fixture;
mod host_io;
mod live;
mod service;
pub mod stream;

pub mod profile;

#[cfg(test)]
mod tests;

pub use credentials::{Accounts, CredentialRequest, Settled, Source, accounts_valid};
pub use host_io::{Launch, ProcessLimits, to_host_exit};
pub use live::{LIVE_BYTES, LIVE_SLOTS, LivePiece, RemoteTurn};
pub use service::{AgentConfig, Config, Error, Limits, Service, StartValues, iterate, valid};
