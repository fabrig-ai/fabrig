//! The component's simulated crashing disk world (protocol/store/testing.md, 1).
#![forbid(unsafe_code)]
pub mod frame_reads;
pub mod model;
pub mod patches;
pub mod referee;
pub mod root;
pub mod world;
pub use world::{Restored, World};
