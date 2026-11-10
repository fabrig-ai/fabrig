//! The machine's generic Linux half (hearth/docs/design/testing/fake-machine.md, 1 and 2).
#![forbid(unsafe_code)]
extern crate alloc;
pub mod behind;
pub mod hearth;
pub mod linux;
pub use hearth::Machine;
#[cfg(test)]
mod tests;
