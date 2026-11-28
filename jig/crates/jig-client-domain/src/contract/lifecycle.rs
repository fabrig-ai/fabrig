//! Attempts and held phases (client/domain/contract/reads.md, 1).
use super::{ChipEnding, HoldCategory, Part, PriorPhase, RunEnd};
use alloc::boxed::Box;
use skein_lib::Wall;

/// One claimed run and the lifecycle facts recorded for it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttemptLine {
    pub task: u64,
    pub attempt: u64,
    pub claimed: Option<Wall>,
    pub started: Option<Wall>,
    pub ended: Option<Wall>,
    pub end: Option<RunEnd>,
    pub phases: Option<u32>,
    pub sources: Option<RunSources>,
}

/// A guide read from the workspace of the attempt.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Guide {
    pub repository: Box<str>,
    pub commit: Box<str>,
    pub path: Box<str>,
    pub digest: [u8; 32],
    pub bytes: u32,
    pub whole: bool,
}

/// The build, settings and project charter set the run was prepared with.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RunSources {
    pub build: Box<str>,
    pub settings: u32,
    pub charters: u64,
    pub source: Box<[u8]>,
    /// None if the host never reported; an empty report is distinct.
    pub guides: Option<Box<[Guide]>>,
}
/// One held phase, named by its task and phase change position.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HoldLine {
    pub task: u64,
    pub position: u64,
    pub at: Wall,
    pub before: PriorPhase,
    pub why: HoldCategory,
    pub source: Option<Part>,
    pub ending: Option<ChipEnding>,
}
