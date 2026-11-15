//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
//! The children keep visibility and pending acts. The root retains only
//! those children and the shared client (jig's client/domain/root.md, 3–10).
//! Each step routes one event; take reclaims after routing every tell.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub mod approve;
pub mod asked;
pub mod authorize;
pub mod boundary;
pub mod copy;
pub mod domain;
mod route;
mod sign_in_tab;
pub use address::{Address, ChatsFilter, Choice, GoalsFilter, ModelWay, Place, Step, WayKind, WorkFilter};
pub use boundary::{Action, Event, Fact, Request, Tab};
pub use domain::{Domain, fire, new, next_deadline, shown, step, take, unlinked};
pub use limits::{Limits, valid};
pub use temper_client_domain_forge::{
    address, amend, delegation, forge, key, limits, owner, public, question, sign_in, views, words,
};
pub use temper_client_domain_pages::{acts, pages};
#[cfg(test)]
mod tests;
