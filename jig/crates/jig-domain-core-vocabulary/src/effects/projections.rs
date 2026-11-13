//! Projection feeds and durable rows (jig/docs/design/domain/connectors.md, 8).

use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::length::len_u64;

/// Current tracked-goal facts, independent of executable authority and inboxes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectionGoal {
    pub number: u64,
    pub project: u32,
    pub priority: u32,
    pub words: Box<[u8]>,
    pub phase: tasks::Phase,
}

/// One task in the whole bounded tree, including tasks that already ended.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectionTask {
    pub number: u64,
    pub requester: tasks::Party,
    pub words: Box<[u8]>,
    pub phase: tasks::Phase,
}

/// Stable identity from the task's lifecycle or semantic history family.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MilestoneId {
    /// A lifecycle transition, independently numbered by the task hub.
    Lifecycle { task: u64, position: u64 },
    /// A semantic change under its immutable task revision.
    Revision { task: u64, revision: u64 },
}

/// One newly recorded milestone; lifecycle details or semantic reason travel together.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectionMilestone {
    pub identity: MilestoneId,
    pub phase: Option<tasks::Phase>,
    pub change: Option<tasks::Change>,
    pub words: Box<[u8]>,
}

/// One connector handoff, with the whole plan and milestones since its last feed.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectionFeed {
    pub goal: ProjectionGoal,
    pub plan: Box<[ProjectionTask]>,
    pub milestones: Box<[ProjectionMilestone]>,
    pub closing: bool,
}

/// Ordered durable projection families: header before its plan and pending history.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProjectionKey {
    /// A goal's feed metadata and final settlement cohort.
    Goal(u64),
    /// One task retained in the complete plan.
    Task { goal: u64, task: u64 },
    /// One unread lifecycle milestone.
    Lifecycle { goal: u64, task: u64, position: u64 },
    /// One unread semantic history milestone.
    Revision { goal: u64, task: u64, revision: u64 },
}

/// One bounded row of a projection; the whole tree is never one journal row.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProjectionRecord {
    /// Feed metadata, retained through the last closing settlement.
    Goal { goal: ProjectionGoal, pending: Box<[u16]>, changed: bool, fed_at: Option<u64> },
    /// One current task in the bounded tree.
    Task { goal: u64, task: ProjectionTask },
    /// History recorded since the last feed.
    Milestone { goal: u64, milestone: ProjectionMilestone },
}

impl ProjectionRecord {
    /// The fixed-size address of this bounded row.
    #[must_use]
    pub fn key(&self) -> ProjectionKey {
        match self {
            Self::Goal { goal, .. } => ProjectionKey::Goal(goal.number),
            Self::Task { goal, task } => ProjectionKey::Task { goal: *goal, task: task.number },
            Self::Milestone { goal, milestone } => milestone_key(*goal, milestone.identity),
        }
    }
}

#[must_use]
pub fn milestone_key(goal: u64, identity: MilestoneId) -> ProjectionKey {
    match identity {
        MilestoneId::Lifecycle { task, position } => ProjectionKey::Lifecycle { goal, task, position },
        MilestoneId::Revision { task, revision } => ProjectionKey::Revision { goal, task, revision },
    }
}

/// Deep bytes of one durable feed row, excluding its inline record slot.
#[must_use]
pub fn projection_record_bytes(row: &ProjectionRecord) -> Option<u64> {
    record_bytes(row)
}

fn record_bytes(row: &ProjectionRecord) -> Option<u64> {
    match row {
        ProjectionRecord::Goal { goal, pending, .. } => len_u64(goal.words.len())
            .checked_add(tasks::phase_bytes(&goal.phase)?)?
            .checked_add(len_u64(pending.len()).checked_mul(2)?),
        ProjectionRecord::Task { task, .. } => len_u64(task.words.len()).checked_add(tasks::phase_bytes(&task.phase)?),
        ProjectionRecord::Milestone { milestone, .. } => {
            len_u64(milestone.words.len()).checked_add(match &milestone.phase {
                Some(phase) => tasks::phase_bytes(phase)?,
                None => 0,
            })
        }
    }
}
