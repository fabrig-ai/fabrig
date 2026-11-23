//! The engine's worker channels (protocol/channel/README.md, section 4, and protocol/channel/worker-link.md, section 1.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod boundary;
mod component;
mod connection;
pub mod host;
pub mod limits;
mod opening;
mod statement;
mod translate;
mod unencoded;
mod wires;
pub use boundary::{Bound, Event, GrantValue, Loss, Request, Scheme, Unsent, Worker};
pub use component::{Component, Unusable};
pub use limits::{Limits, Timers, valid};
pub use statement::{Stated, Statement, Unfit, judge};
pub use unencoded::Unencoded;
#[cfg(test)]
mod tests;
