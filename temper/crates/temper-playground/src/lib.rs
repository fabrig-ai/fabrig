//! The playground composition (temper/docs/design/playground.md, 3).
pub mod documents;
pub mod fakes;
pub mod machine;
pub mod programs;
pub mod service;
pub mod stories;
pub mod story;
#[cfg(target_arch = "wasm32")]
pub mod worker;
pub use story::{Fakes, Links, Start, world};

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/worker.rs"]
mod worker_tests;
