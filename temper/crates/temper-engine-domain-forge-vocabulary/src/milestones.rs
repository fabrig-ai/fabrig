//! The forge's milestone row (temper/docs/design/domain/milestones.md, 8).
use alloc::boxed::Box;
use skein_lib::Wall;

pub const MILESTONES_PER_PROJECT: u32 = 256;
pub const MILESTONE_NAME_BYTES: u32 = 256;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MilestoneRow {
    pub project: u32,
    pub task: u64,
    pub name: Box<[u8]>,
    pub last_landing: Option<MilestoneLanding>,
    pub ended: Option<MilestoneEnd>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MilestoneLanding {
    pub at: Wall,
    pub goal: u64,
    pub change: u64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MilestoneEnd {
    pub at: Wall,
    pub outcome: MilestoneOutcome,
    pub done: u32,
    pub failed: u32,
    pub closed: u32,
    pub spent: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MilestoneOutcome {
    Complete,
    Ended,
    Closed,
}
