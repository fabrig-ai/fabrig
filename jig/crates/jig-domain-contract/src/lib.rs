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

/// Why the engine refuses construction (jig/docs/design/domain/engine/vocabulary.md, 2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartProblem {
    Membership(jig_domain_link::Problem),
    Declared { connector: u16 },
    ResourceKind { connector: u16, kind: u16 },
    EffectKind { connector: u16, kind: u16 },
    Undeclared { connector: u16 },
    HoldKinds { limit: u32 },
}

impl Declaration {
    #[must_use]
    pub fn empty(connector: u16) -> Declaration {
        Self { connector, resources: Box::new([]), effects: Box::new([]) }
    }
}
