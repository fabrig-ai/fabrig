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
pub mod requests;
pub mod resources;
pub mod settings;
pub mod topics;

use alloc::boxed::Box;
use effects::EffectKind;
use resources::ResourceKind;

/// A connector's number in its application.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Connector(pub u16);

/// What a connector declares to the engine at construction (jig/docs/design/domain/connectors.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Declaration {
    pub connector: Connector,
    pub resources: Box<[ResourceKind]>,
    pub effects: Box<[EffectKind]>,
    pub projects_goals: bool,
}

/// Why the engine refuses construction (jig/docs/design/domain/engine/vocabulary.md, 2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartProblem {
    Membership(jig_domain_link::Problem),
    Declared { connector: Connector },
    ResourceKind { connector: Connector, kind: u16 },
    EffectKind { connector: Connector, kind: u16 },
    Undeclared { connector: Connector },
    HoldKinds { limit: u32 },
}

impl Declaration {
    #[must_use]
    pub fn empty(connector: Connector) -> Declaration {
        Self { connector, resources: Box::new([]), effects: Box::new([]), projects_goals: false }
    }
}

/// What the engine asks a connector (jig/docs/design/domain/engine/vocabulary.md, 2.5).
#[derive(Hash, Debug)]
pub enum Ask {
    Lifecycle(lifecycle::Ask),
    Resources(resources::Ask),
    Effects(effects::Ask),
    Topics(topics::Ask),
    Parts(parts::Ask),
    Procedures(procedures::Ask),
    Projections(projections::Ask),
    Settings(settings::Ask),
    Requests(requests::Ask),
}

/// What a connector says to the engine (jig/docs/design/domain/engine/vocabulary.md, 2.5).
#[derive(Hash, Debug)]
pub enum Up {
    Lifecycle(lifecycle::Up),
    Resources(resources::Up),
    Effects(effects::Up),
    Topics(topics::Up),
    Parts(parts::Up),
    Procedures(procedures::Up),
    Projections(projections::Up),
    Settings(settings::Up),
    Requests(requests::Up),
    /// An act in the deployment's name, answered durably by `Released::Answered` under the same key.
    Act {
        key: [u8; 16],
        act: Box<ops::Act>,
        carried: Option<ops::Carried>,
    },
}

/// What the engine releases to a connector after a commit (jig/docs/design/domain/engine/vocabulary.md, 2.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Released {
    /// Make the outbox entry; answered by `effects::Up::Outbox` or `projections::Up::Outbox`.
    Make { entry: u64 },
    /// The durable answer to the connector's act.
    Answered { key: [u8; 16], outcome: ops::Outcome },
}

#[cfg(test)]
mod tests;
