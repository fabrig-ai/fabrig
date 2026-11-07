//! The TUI's host and its loop (temper/docs/design/client/README.md, 6.13 and 6.16).
#![forbid(unsafe_code)]

pub mod host;
#[cfg(any(test, feature = "record"))]
pub mod journey;
pub mod limits;
pub mod run;

#[cfg(test)]
mod tests;
