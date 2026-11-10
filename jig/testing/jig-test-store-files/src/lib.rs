//! The real store in domain worlds (protocol/store/README.md, 9).
#![forbid(unsafe_code)]
mod disk;
mod files;
pub use files::Files;
mod durable;
mod store;
pub use store::{Fault, KillFrom, Pending, Store, component_limits};
#[cfg(test)]
mod tests;
