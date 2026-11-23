//! Jig's independent browser parts (protocol/browser/loop.md, 1.1).
#![forbid(unsafe_code)]
pub mod adapter;
pub mod look;
pub mod sign_in;
#[cfg(test)]
mod tests;
#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
pub mod web;
