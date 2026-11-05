//! HTTP shared by the fake forge's layers (skein/docs/design/fake-forge.md, 12).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;
pub mod connection;
