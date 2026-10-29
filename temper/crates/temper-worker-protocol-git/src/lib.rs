//! The checkout's fixed git invocations (protocol/git.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod boundary;
pub mod capped;
mod component;
mod files;
pub mod invocation;
pub mod limits;
mod operation;
pub mod output;
pub mod ring;
pub mod sequence;
#[cfg(test)]
mod tests;

pub use boundary::{Event, Identity, Outputs, Request, Settings, Unusable, Version};
pub use component::Component;
pub use limits::{KERNEL_SUBMISSIONS, Limits, Unfit, check};
