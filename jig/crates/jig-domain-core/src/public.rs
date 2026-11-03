//! Public reading's documents and messages (jig/docs/design/domain/people.md, 13).

pub(crate) mod index;
mod line;
pub(crate) mod reads;

pub(crate) use index::{Index, follow, publish, restored};

use alloc::boxed::Box;
use skein_lib::Time;

/// One public read, without a sign-in or party.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PublicRead {
    pub owner: u16,
    pub version: u16,
    pub kind: u16,
    pub project: Option<u64>,
    pub task: Option<u64>,
}

/// A public read's terminal, held behind preceding commits.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PublicAnswer {
    Document { decided: Time, document: PublicDocument },
    Unknown,
    Busy,
}

/// A public projection or an application's record in its own envelope.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PublicDocument {
    Projects(PublicProjects),
    Board(PublicBoard),
    Card(PublicCard),
    Application(PublicApplication),
}

/// The deployment's public projects, by internal number.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicProjects {
    pub projects: Box<[PublicProject]>,
}

/// A public project and its name (jig/docs/design/client/domain/contract.md, 12).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicProject {
    pub project: u32,
    pub name: Box<str>,
}

/// A public project's bounded live and ended goals.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicBoard {
    pub project: u32,
    pub name: Box<str>,
    pub live: Box<[PublicGoal]>,
    pub live_folded: u32,
    pub ended: Box<[PublicGoal]>,
}

/// One public goal and its bounded tree.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicGoal {
    pub line: PublicLine,
    pub tree: Box<[PublicLine]>,
    pub folded: u32,
}

/// One public task, its lineage and bounded delegates.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicCard {
    pub line: PublicLine,
    pub lineage: Box<[u64]>,
    pub delegates: Box<[PublicLine]>,
    pub folded: u32,
}

/// The task title, structure and mechanics, with times in whole seconds.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicLine {
    pub task: u64,
    pub title: crate::documents::Text,
    pub parent: Option<u64>,
    pub goal: bool,
    pub priority: Option<u32>,
    pub executor: PublicExecutor,
    pub phase: PublicPhase,
    pub hold: Option<PublicHold>,
    pub made: u64,
    pub changed: u64,
    pub active: Option<u64>,
    pub ended: Option<u64>,
    pub spent: u64,
    pub budget: u64,
    pub delegates: PublicCounts,
}

/// Counts of public delegates alone, by phase class.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PublicCounts {
    pub working: u32,
    pub waiting: u32,
    pub held: u32,
    pub closing: u32,
    pub ended: u32,
}

/// An application's record and the internal task numbers it names.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicApplication {
    pub tasks: Box<[u64]>,
    pub record: Box<[u8]>,
}

/// Executor kind, without its identity or configured name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PublicExecutor {
    Agent,
    Procedure,
    Person,
    Job,
}

/// The client contract's task phases, without their payloads.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PublicPhase {
    Waiting,
    Idle,
    Due,
    Preparing,
    Claimed,
    Running,
    BackingOff,
    Active,
    Closing,
    Held,
    Ended,
    Done,
    Failed,
    Cancelled,
}

/// The client task line's hold kinds, without their reasons or identities.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PublicHold {
    Exceeded,
    Failures,
    Stalled,
    Dependency,
    Stopped,
    Placed,
    Drift,
    Effects,
    EffectFailed,
    Uncertain,
    Budget,
    Deadline,
    HoldsWaited,
    Circle,
    ResourceUnavailable,
    ResourceAccess,
    PoolLost,
    Procedure,
}

#[cfg(test)]
#[path = "tests/public.rs"]
mod tests;
