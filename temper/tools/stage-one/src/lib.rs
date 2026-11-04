//! Configuring stage one: `temper/docs/design/domain/testing.md`, 6.5.

pub mod check;
pub mod collect;
pub mod evidence;
pub mod golden;
pub mod judge;

#[cfg(test)]
mod tests;
