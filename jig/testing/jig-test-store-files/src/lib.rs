//! The real store in domain worlds (protocol/store/README.md, 9).
#![forbid(unsafe_code)]
mod disk;
mod files;
pub use files::Files;
mod durable;
mod store;
pub use store::{Fault, KillFrom, Pending, ReadbackFinding, Store, component_limits};
#[doc(hidden)]
pub mod testing {
    pub use crate::disk::testing::fork_first_operation;
}
#[cfg(test)]
mod tests;
