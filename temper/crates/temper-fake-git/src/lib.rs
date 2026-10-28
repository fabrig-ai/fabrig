//! Git's fixed invocations, checked before a hosted program touches files.
#![forbid(unsafe_code)]
mod answer;
mod check;
pub use answer::{Answer, answer};
pub mod id;
mod program;
mod transport;
pub use check::{Checked, Refusal, check};
pub use program::{Git, Start};
pub use transport::{Transport, Typed};

/// The HTTP client and its kernel socket jobs.
pub mod connection;

mod route;
pub use route::{MAX_RUNS, Missing, Route};
mod exchange;

#[cfg(test)]
mod tests;
