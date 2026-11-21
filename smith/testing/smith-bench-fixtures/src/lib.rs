//! Shared fixtures (smith/docs/design/testing.md, section 7; smith/docs/design/benchmarks/README.md, section 13).
//! Replay requests, recorded JSON edits and synthetic event records for benchmark tests.
//! Fixtures retain explicit inputs and expose no running agent state
//! (smith/docs/design/benchmarks/README.md, section 13).

pub mod events;

pub mod json;

pub mod request;

/// Benchmark stories shared by replaying and real binaries.
pub mod common;

/// Parity inputs and row derivation shared by both binaries.
pub mod parity;
