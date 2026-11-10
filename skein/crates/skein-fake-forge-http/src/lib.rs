//! HTTP shared by the fake forge's layers (skein/docs/design/testing/fakes/github.md, 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;
pub mod connection;
pub mod git;

/// Per-request chances in a thousand, drawn before and after the domain call.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Faults {
    pub reset: u32,
    pub drop: u32,
    pub server_error: u32,
}

/// The forge lent for one hosted pass, including direct object-store calls.
#[derive(Debug)]
pub struct Hosted<'a> {
    pub domain: &'a mut skein_fake_forge_domain::Domain,
    pub config: &'a skein_fake_forge_domain::Config,
}
