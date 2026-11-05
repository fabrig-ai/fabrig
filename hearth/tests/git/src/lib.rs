//! Seeded worlds for git's stream machines (hearth/docs/design/domain/testing.md, 2.4).
pub mod fixtures;
pub mod generate;
pub mod world;

pub use hearth_test_git::expect;
pub mod mutate;
pub mod reference;
