//! Run blame for smith's worlds (`smith/docs/design/faults.md`, 4).
//!
//! Worlds record starts, retries and ends in [`Runs`], then judge them against
//! their fault ledger. Only account identities and observed spans are known here;
//! task tries, holds, partial output and world scheduling belong to the caller.

mod answer;
mod runs;

pub use answer::{Ended, ended};
pub use runs::{Retried, Runs};
use skein_world::faults::Kind;

/// A provider refuses an account's billing arrangement.
pub const BILLING_REFUSAL: Kind = Kind { family: "provider", name: "billing refusal" };

/// A provider's edge refuses requests transiently.
pub const EDGE_REFUSAL: Kind = Kind { family: "provider", name: "edge refusal" };

/// Another client spends the account's rate window.
pub const WINDOW_ELSEWHERE: Kind = Kind { family: "provider", name: "window elsewhere" };

/// A worker's program starts without smith's tools.
pub const UNTOOLED_PROGRAM: Kind = Kind { family: "worker", name: "untooled program" };

/// A worker's check waits for a place to run.
pub const CHECK_WAITS: Kind = Kind { family: "worker", name: "check waits for a place" };

#[cfg(test)]
mod tests;
