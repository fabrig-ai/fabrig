//! Step-test fixtures (smith/docs/design/testing.md, section 7).
//! Replay requests, recorded JSON edits and synthetic event records for benchmark tests.
//! Fixtures retain explicit inputs and expose no running agent state
//! (smith/docs/design/benchmarks/README.md, section 13).

pub mod events;

pub mod json;

pub mod request;
