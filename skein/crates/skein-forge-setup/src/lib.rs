//! Pure, owner-driven forge reconciliation (forge-setup.md, 4 and 6).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod call;
mod forgejo;
pub mod github;
pub mod host;
pub mod item;
pub mod plan;
pub use forgejo::{adopt, check, copy, create, given, limits, reachable, reconcile, take_over};
#[cfg(test)]
mod tests;
