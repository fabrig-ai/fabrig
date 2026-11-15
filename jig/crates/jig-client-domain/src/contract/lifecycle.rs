//! Attempts and held phases (client/domain/contract/README.md, 4).
use super::{ChipEnding, HoldCategory, Part, PriorPhase, RunEnd};
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
