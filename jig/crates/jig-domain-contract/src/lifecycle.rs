//! Connector restart and adoption values (jig/docs/design/domain/connectors.md, 2; engine/vocabulary.md, 2.6).

use crate::ops::Role;
use crate::resources::Name;
use alloc::boxed::Box;
use skein_lib::Token;

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
#[derive(PartialEq, Eq, Debug)]
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

/// Lifecycle asks of every connector.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Restore your records, then report `Done { stage: Restored }`.
    Restore,
    /// Operation that ends later: catch up from opening, answered by `Up::Done` once per stage.
    CatchUp,
    /// Notification: adopt this resource into the project; no answer.
    Adopt { request: Token, project: u32, adoption: Adoption },
    /// Operation that ends later: release a deleted project's adoptions and settings,
    /// answered by `Up::Released` after commit or at once if none are kept. Asked again at each start.
    ReleaseProject { project: u32 },
}

/// A connector's lifecycle answers and adoptions.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Up {
    /// Restoration or one half of catching up is done.
    Done { stage: RestartStage },
    /// A deleted project's release is durable, or the connector held nothing for it.
    Released { project: u32 },
    /// Unasked: a resource's adoption, its role and whether it is the project's home.
    Adopted { project: u32, name: Name, role: AdoptionRole, home: bool },
}
