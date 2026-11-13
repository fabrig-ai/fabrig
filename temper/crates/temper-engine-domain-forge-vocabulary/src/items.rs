//! Checkout items (jig's domain/connectors.md, section 10). The connector
//! selects starts and write destinations from its adopted and procedure facts.
//! Items are returned whole with their byte size; no copy is retained here.
use alloc::boxed::Box;
use skein_forge::api;

/// An immutable checkout start, or a branch the worker resolves.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Start {
    /// The frozen base of a prepare round (temper/docs/design/domain/worker/jobs.md).
    Commit(api::Commit),
    /// The project's landing branch.
    Base(Box<[u8]>),
    /// A change's current branch.
    Branch(Box<[u8]>),
    /// The task's saved work takes precedence over its procedure's start.
    Saved(Box<[u8]>),
    /// Resolve this branch against the named base commit.
    Merge { branch: Box<[u8]>, base: api::Commit },
    /// The landing branch with each head merged into it in order
    /// (temper/docs/design/domain/worker/README.md, 4.1).
    Merged { branch: Box<[u8]>, heads: Box<[api::Commit]> },
}
/// Root-translated identities and branch names for one repository.
#[derive(Clone, Debug)]
pub struct Seed {
    pub project: u32,
    /// The run may write its goal's design (temper/docs/design/domain/forge/branches.md, 3).
    pub modify: bool,
    pub task: u64,
    pub parent: Option<u64>,
    pub repository: api::Repository,
    pub run_branch: Box<[u8]>,
    pub saved_branch: Option<Box<[u8]>>,
}
/// A sized workspace item ready to hand to the worker.
#[derive(Debug)]
pub struct Item {
    /// This item delivers a goal's design straight to its goal branch.
    pub delivers: bool,
    pub provider: api::Repository,
    pub name: Box<[u8]>,
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub start: Start,
    pub push: Option<Box<[u8]>>,
    pub landing: Option<Box<[u8]>>,
    pub holder: Option<u64>,
    /// The repository's git identity, which the run's grants carry.
    pub credential: Option<u32>,
    pub bytes: u64,
}
