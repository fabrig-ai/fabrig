//! Agent configuration values, profile derivation and admission checks.
//! Keeps deployment quantities and policy bounds, never run state, grants or credentials.
//! `profile::derive` derives agent limits; `host_limits` derives the host's share.
//! Contracts: smith/docs/design/shell/README.md, sections 2.1 and 2.6;
//! smith/docs/design/protocol/limits.md, sections 2–4.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod checks;
pub mod events;
mod host;
pub mod profile;
pub mod programs;
mod values;

pub use checks::{effects_valid, io_valid, valid};
pub use events::event_limits;
pub use host::host_limits;
pub use programs::{ProgramBackend, program_operations, program_valid};
pub use values::{Config, Limits, ROUTE_HEADROOM};

#[cfg(test)]
mod fixture;
