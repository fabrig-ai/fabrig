//! The pinned Forgejo's documents (forge/README.md, sections 2 and 3, and forge/forgejo.md, section 2).
pub mod client;
pub mod credential;
pub mod decode;
pub mod delivery;
pub mod document;
pub mod header;
pub mod id;
pub mod limits;
pub use crate::path;
pub mod pin;
pub mod request;
pub mod server;
#[cfg(test)]
pub(crate) mod tests;
pub use crate::time;

pub mod failure;

pub mod plans;

mod inflate;
pub mod zip;
