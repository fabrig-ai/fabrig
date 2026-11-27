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

use alloc::boxed::Box;
use effects::EffectKind;
use resources::ResourceKind;

/// The kinds one connector gives the engine at construction (jig/docs/design/domain/connectors.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Declaration {
    pub connector: u16,
    pub resources: Box<[ResourceKind]>,
    pub effects: Box<[EffectKind]>,
}

impl Declaration {
    #[must_use]
    pub fn empty(connector: u16) -> Declaration {
        Self { connector, resources: Box::new([]), effects: Box::new([]) }
    }
}
