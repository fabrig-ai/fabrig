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

/// A connector's resource path, opaque to people (domain/connectors.md, section 12).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ResourceName {
    pub connector: u16,
    pub path: Box<[Box<[u8]>]>,
}

/// How an adopted resource participates in the project.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceRole {
    Owned,
    Fork,
    Context,
}

/// A resource and its connector-specific options, opaque to people. The whole
/// path and options payload fits `Limits::amendment_bytes` at admission.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Adoption {
    pub resource: ResourceName,
    pub role: ResourceRole,
    /// Bounded connector-specific fields, interpreted by the application's root.
    pub options: Box<[Box<[u8]>]>,
}
