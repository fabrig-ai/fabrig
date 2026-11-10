//! The deterministic mmdebstrap domain (hearth/docs/design/testing/fake-machine.md, 6).
#![forbid(unsafe_code)]
extern crate alloc;
pub mod apt;
pub mod face;
pub mod process;
pub mod resolve;
#[cfg(test)]
mod tests;
