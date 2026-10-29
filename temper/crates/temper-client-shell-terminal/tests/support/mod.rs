#![expect(dead_code, reason = "library tests use part of the machine world")]
mod world;
pub use world::*;
