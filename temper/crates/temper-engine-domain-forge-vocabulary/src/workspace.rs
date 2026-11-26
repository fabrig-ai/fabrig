//! Worker workspace carriers (temper/docs/design/domain/worker/hosted-runs.md, 1.1;
//! temper/docs/design/protocol/channel.md, 4).
use alloc::boxed::Box;

/// A checkout's authoritative forge start, prepared afresh by the worker.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ForgeStart {
    /// The frozen base of a prepare round (temper/docs/design/domain/worker/jobs.md).
    Commit(skein_forge::api::Commit),
    Base(Box<[u8]>),
    Branch(Box<[u8]>),
    Saved(Box<[u8]>),
    Merge {
        branch: Box<[u8]>,
        base: skein_forge::api::Commit,
    },
    /// The landing branch with each head merged into it in order
    /// (temper/docs/design/domain/worker/hosted-runs.md, 1.1).
    Merged {
        branch: Box<[u8]>,
        heads: Box<[skein_forge::api::Commit]>,
    },
}

/// One adopted repository translated for the worker's workspace.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForgeRepository {
    /// Branches a prepare job may create, in stack order.
    pub prefixes: Box<[Box<[u8]>]>,
    /// This repository delivers for a goal run (temper/docs/design/domain/forge/branches.md, 1).
    pub delivers: bool,
    /// This task's saved-work branch, under this repository's prefix.
    pub save: Option<Box<[u8]>>,
    pub tag: u64,
    pub provider: skein_forge::api::Repository,
    pub name: Box<[u8]>,
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub start: ForgeStart,
    pub push: Option<Box<[u8]>>,
    pub landing: Option<Box<[u8]>>,
    /// This task's own checked delivery (temper/docs/design/protocol/channel.md, 4).
    pub delivered: Option<skein_forge::api::Commit>,
    /// Named check base for this hold (temper/docs/design/protocol/channel.md, 4).
    pub pin: Option<skein_forge::api::Commit>,
    pub identity: u32,
}

/// Concrete checkout roots for one assignment; the workstream that owns
/// their cache is the hub's (jig's `domain/hosts/README.md`, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForgeWorkspace {
    /// The project's rerun bound, if it states one.
    pub rerun: Option<u32>,
    pub repositories: Box<[ForgeRepository]>,
}
