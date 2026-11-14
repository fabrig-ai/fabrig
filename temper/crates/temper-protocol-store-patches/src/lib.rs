//! The store owner's patch module, compiled separately (protocol/store.md, 4).
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
#[cfg(test)]
extern crate std;

#[path = "../../temper-protocol-store/patches/mod.rs"]
pub mod patches;
