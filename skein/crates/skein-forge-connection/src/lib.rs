//! Forge protocol component (forge/connection.md, sections 2, 3, 4, 7 and 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod app;
pub mod boundary;
pub mod component;
mod connection;
/// Fixtures for the crate's step tests (forge/connection.md, 10).
#[cfg(test)]
mod fixtures;
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

pub mod seam;
