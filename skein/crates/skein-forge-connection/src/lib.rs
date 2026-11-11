//! Forge protocol component (forge/connection.md, sections 2, 3, 4, 7 and 8).
#![cfg_attr(not(test), no_std)]
#![cfg_attr(
    feature = "seam",
    expect(clippy::must_use_candidate, reason = "the machine-world seam exposes the existing state operations")
)]
#![forbid(unsafe_code)]
extern crate alloc;
mod app;
pub mod boundary;
pub mod component;
mod connection;
/// What the crate's tests and the machine world's `hand` both read (forge/connection.md, 10).
#[cfg(any(test, feature = "seam"))]
pub mod fixtures;
pub mod grow;
pub mod hooks;
mod installations;
mod kept;
pub mod limits;
mod puller;
pub mod settings;
pub mod start;
pub use boundary::{Account, Answered, Change, Event, Fact, Request, StartRead, Stopped, Value};
pub use component::{Component, Diagnostics};
pub use limits::{Limits, Shared, Unusable};
pub use settings::{Access, Settings};
#[cfg(test)]
mod tests;

/// State inspected and seeded by the forge machine world (forge/connection.md, 10).
#[cfg(feature = "seam")]
pub mod seam {
    pub use crate::connection::Phase;
    pub use crate::installations::{Accepted, Clock, Diff, GitHub, HeldToken, Read, Reading, Row, Selected, Tokens};
    pub use crate::puller::State;
}
