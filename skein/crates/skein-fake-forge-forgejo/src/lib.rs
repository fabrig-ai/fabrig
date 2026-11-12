//! Forgejo's HTTP vocabulary over the fake forge (testing/fakes/forge.md, section 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod answers;
pub mod delivery;
pub mod image;
use alloc::boxed::Box;
use skein_lib::{List, Secret};
pub mod errors;
pub use skein_fake_forge_http::{Faults, Hosted};
pub mod observations;
pub mod response;
pub mod routes;
mod server;
mod service;
pub mod setup;
pub mod tally;
mod zip;
pub use server::{Event, Server, close, closed, down, resume, resume_with_domain, start, up, up_with_domain};
pub use service::{Error, Service, valid};
pub use tally::{Route, Tally, route};
/// Caps owned by the protocol layer, in addition to the fake domain's.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub calls: u32,
    pub connections: u32,
    pub http: skein_http::server::Limits,
    pub documents: skein_forge::forgejo::limits::Limits,
    pub raw_bytes: u32,
}
impl Limits {
    /// The private git route's bounds (testing/fakes/github.md, section 3).
    #[must_use]
    pub const fn git(&self) -> skein_fake_forge_http::git::Limits {
        skein_fake_forge_http::git::Limits { http: self.http, documents: self.documents }
    }
}
/// The repository's id, path and protected branch from the scenario.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repository {
    pub id: u32,
    pub full: Box<[u8]>,
    pub protected: Option<Box<[u8]>>,
}
/// A catalogue address and its domain source (skein/docs/design/testing/fakes/forge.md, 4).
#[derive(Debug)]
pub struct Source {
    pub address: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub private: bool,
    pub fill: skein_fake_forge_domain::api::Fill,
}
/// Scenario tables; tokens stay solely on this side of the domain boundary.
#[derive(Debug)]
pub struct Config {
    pub sources: Box<[Source]>,
    pub hooks: Option<delivery::Endpoint>,
    pub base: Box<[u8]>,
    pub tokens: List<(Secret, u64)>,
    pub users: List<(u64, Box<[u8]>)>,
    pub repositories: List<Repository>,
    pub max_response_items: u32,
    pub default_paging_num: u32,
    pub faults: Faults,
}
#[cfg(test)]
mod tests;
