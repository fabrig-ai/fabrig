//! skein's playground kit: a simulated world run whole in a browser's worker (playground.md).
#![forbid(unsafe_code)]

extern crate alloc;

pub mod memory;
pub mod replay;
pub mod slices;
pub mod visitor;
#[cfg(target_arch = "wasm32")]
pub mod worker;
