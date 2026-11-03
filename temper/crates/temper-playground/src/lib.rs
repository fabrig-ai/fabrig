//! The playground composition (temper/docs/design/playground.md, 3).
pub mod documents;
pub mod fakes;
pub mod machine;
pub mod programs;
pub mod service;
pub mod story;
pub use story::{Fakes, Links, Start, world};

#[cfg(test)]
mod tests;
