//! Shared client documents and harness helpers (temper/docs/design/domain/testing.md, 7).
#![expect(clippy::must_use_candidate, reason = "shared test fixtures return values for their callers to inspect")]

pub mod browse;
pub mod client_documents;
pub mod first_run;
pub mod memory_literal;
pub mod ports;
