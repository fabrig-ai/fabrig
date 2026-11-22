//! Shared client documents and harness helpers (temper/docs/design/testing/README.md, 5).
#![expect(clippy::must_use_candidate, reason = "shared test fixtures return values for their callers to inspect")]

pub mod browse;
pub mod client_documents;
pub mod first_run;
pub mod memory_literal;
pub mod milestones;
pub mod ports;

#[doc(hidden)]
pub mod testing {
    pub use crate::ports::blocks::{BLOCK, Blocks, LOWEST, RANGE_FILE};
}
