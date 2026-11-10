//! The reference worker root (domain/hosts/README.md, section 4, and domain/hosts/hub.md, section 8).
//! Payloads cross the agent boundary by hand-out to the protocol layer.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod admission;
pub mod boundary;
mod domain;
mod hand_out;
mod limits;
pub mod translate;
mod workspace;

pub use admission::{Class, class};
pub use boundary::{Encodable, Encoded, Event, Request};
pub use domain::{Domain, fire, step};
pub use limits::{Limits, WATERMARK, valid};
pub use workspace::Content;

#[cfg(test)]
mod tests;
