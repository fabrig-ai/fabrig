//! The engine's worker channels (protocol/channel/README.md, sections 4 and 9.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod boundary;
mod component;
mod connection;
pub mod host;
pub mod limits;
mod opening;
mod translate;
mod wires;
pub use boundary::{Bound, Event, GrantValue, Loss, Request, Scheme, Unsent, Worker};
pub use component::{Component, Unusable};
pub use limits::{Limits, valid};
#[cfg(test)]
mod tests;
