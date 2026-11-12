//! The native rustls entry for crates built on both platforms
//! (`skein/docs/foundation/cli.md`, 3). Empty on wasm32-unknown-unknown.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub use rustls_std::*;
