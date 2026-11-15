//! The pages and their acts (temper/docs/design/client/domain-crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub mod acts;
pub mod pages;
pub use temper_client_domain_forge::{
    Address, ChatsFilter, Choice, GoalsFilter, Limits, ModelWay, Place, Step, WayKind, WorkFilter, address, amend,
    delegation, forge, key, limits, owner, public, question, sign_in, valid, views, words,
};
#[cfg(test)]
mod tests;
