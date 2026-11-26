//! Connector restart and adoption values (jig/docs/design/domain/connectors.md, 2).

use crate::ops::Role;
use alloc::boxed::Box;

/// Restoration and the two halves of a connector's catching up.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RestartStage {
    Restored,
    ReadAfresh,
    Settled,
}

/// A project's durable adoption role (domain/model.md, 6.1; domain/connectors.md, 12).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AdoptionRole {
    Owned,
    Participant,
    Fork,
    Context,
}

/// One party the connector learned while adopting a resource.
#[derive(Debug)]
pub struct AdoptedParty {
    pub subject: Box<[u8]>,
    pub role: Role,
}
