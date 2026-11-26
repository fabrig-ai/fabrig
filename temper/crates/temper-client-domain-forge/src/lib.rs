//! The forge's side and vocabulary (temper/docs/design/client/domain-crates.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;

/// Generated client wire translations (skein/docs/design/data/codec/maps.md, 9).
#[rustfmt::skip]
mod generated {
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "maps borrow all domain values to write a record, including small Copy values")]
    pub(crate) mod client_wire;
}

pub mod address;
pub mod amend;
pub mod delegation;
pub mod forge;
pub mod key;
pub mod limits;
pub mod milestones;
pub mod owner;
pub mod parts;
pub mod public;
pub mod question;
pub mod sign_in;
pub mod views;
pub mod words;
pub use address::{Address, ChatsFilter, Choice, GoalsFilter, ModelWay, Place, Step, WaitsFilter, WayKind, WorkFilter};
pub use limits::{Limits, valid};
#[cfg(test)]
mod tests;
