//! Immutable lifecycle milestones (client/domain/objects.md, 11; domain/tasks.md, 7).
//! Position is the supplied activity position, independent of semantic revisions.
use super::{ChipEnding, ClosingStage, HoldCategory, Part, PriorPhase, RunStage};
use skein_lib::Wall;

/// One lifecycle change with its separately supplied wall clock.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MilestoneLine {
    pub task: u64,
    pub position: u64,
    pub phase: MilestonePhase,
    pub at: Option<Wall>,
}
/// The owner's lifecycle, without live chip observations or inferred prior clocks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MilestonePhase {
    /// Not yet activated.
    Waiting,
    /// A retained activation stage.
    Active(RunStage),
    /// Closing obligations and the whole retained ending.
    Closing { stage: ClosingStage, ending: ChipEnding },
    /// Held prior lifecycle, reason, raw source and independently optional ending.
    Held { before: PriorPhase, why: HoldCategory, source: Option<Part>, ending: Option<ChipEnding> },
    /// The whole terminal ending.
    Ended(ChipEnding),
}
