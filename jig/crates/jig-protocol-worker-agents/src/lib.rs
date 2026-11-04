//! A worker's program sources and route (jig/docs/design/protocol/service.md, 11.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod route;
pub mod sources;
#[cfg(test)]
mod tests;
