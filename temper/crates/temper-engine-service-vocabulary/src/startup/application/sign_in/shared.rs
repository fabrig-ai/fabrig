//! Shared sign-in members (protocol/services.md, 4.2).
use alloc::boxed::Box;
pub use temper_service_core::startup::url::{loopback, url, visible};

#[derive(Clone, Debug)]
pub struct Member {
    pub address: Box<[u8]>,
    pub login: Box<[u8]>,
}
#[derive(Clone, Debug)]
pub struct ForgeSignIn {
    pub forge: u16,
    pub client: Box<[u8]>,
}
