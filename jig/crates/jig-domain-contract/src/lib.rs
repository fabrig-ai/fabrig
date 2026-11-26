//! Values the core and its connectors exchange (jig/docs/design/domain/connectors.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod effects;
pub mod lifecycle;
pub mod ops;
pub mod parts;
pub mod procedures;
pub mod projections;
pub mod resources;
pub mod settings;
pub mod topics;
