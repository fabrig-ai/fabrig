//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
//! The children keep visibility and pending acts. The root retains only
//! those children and the shared client (jig's client/domain/root.md, 3–10).
//! Each step routes one event; take reclaims after routing every tell.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub mod acts;
pub mod address;
pub mod amend;
pub mod approve;
pub mod asked;
pub mod authorize;
pub mod boundary;
pub mod copy;
pub mod domain;
pub mod forge;
pub mod key;
pub mod limits;
pub mod owner;
pub mod pages;
pub mod public;
pub mod question;
mod route;
pub mod sign_in;
mod sign_in_tab;
pub mod views;
pub mod words;
pub use address::{Address, ChatsFilter, Choice, GoalsFilter, ModelWay, Place, Step, WayKind};
pub use boundary::{Action, Event, Fact, Request, Tab};
pub use domain::{Domain, fire, new, next_deadline, shown, step, take, unlinked};
pub use limits::{Limits, valid};
#[cfg(test)]
mod tests;
