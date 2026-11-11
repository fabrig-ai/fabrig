//! Configuring stage one: `temper/docs/design/testing/checks.md`, 5.

mod cli;
pub use cli::{TABLE, main};

pub mod check;
pub mod collect;
pub mod evidence;
pub mod golden;
pub mod judge;

#[cfg(test)]
mod tests;
