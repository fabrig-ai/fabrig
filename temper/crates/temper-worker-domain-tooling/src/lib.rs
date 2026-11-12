//! The worker's job capability (temper/docs/design/domain/worker/jobs.md):
//! a project's command runs in its job's directory with the arguments and
//! environment of temper/docs/design/protocol/tooling.md, 2. An empty command
//! for kind 1 runs temper's own prepare, without a process (temper/docs/design/domain/worker/jobs.md).
//!
//! Sans-io: the parent routes file and process requests to the protocol
//! layer. A process deadline is data; the protocol layer kills the process
//! group when it passes. An ending precedes the terminal `Gone`, which waits
//! for every process and any result read. The parent handles delivery.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod boundary;
mod domain;
mod limits;
#[cfg(test)]
mod tests;

pub use boundary::{End, Event, Exit, Kind, Request, Spec};
pub use domain::Domain;
pub use domain::step;
pub use limits::{Invalid, Limits, valid};
