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

/// The goal's changing facts, without its words (protocol/store/parts.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectionGoalHot {
    pub number: u64,
    pub project: u32,
    pub priority: u32,
    pub phase: tasks::Phase,
}

impl ProjectionGoal {
    /// Copy the changing facts without copying the words.
    #[must_use]
    pub fn hot(&self) -> ProjectionGoalHot {
        let Self { number, project, priority, words: _, phase } = self;
        ProjectionGoalHot { number: *number, project: *project, priority: *priority, phase: phase.clone() }
    }

    /// Place every goal field in its durable part.
    #[must_use]
    pub fn split(self) -> (ProjectionGoalHot, Box<[u8]>) {
        let Self { number, project, priority, words, phase } = self;
        (ProjectionGoalHot { number, project, priority, phase }, words)
    }
}

impl ProjectionGoalHot {
    /// Assemble the whole goal handed to the feed.
    #[must_use]
    pub fn join(self, words: Box<[u8]>) -> ProjectionGoal {
        let Self { number, project, priority, phase } = self;
        ProjectionGoal { number, project, priority, words, phase }
    }
}

/// The plan task's changing facts, without its words (protocol/store/parts.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectionTaskHot {
    pub number: u64,
    pub requester: tasks::Party,
    pub phase: tasks::Phase,
}

impl ProjectionTask {
    /// Copy the changing facts without copying the words.
    #[must_use]
    pub fn hot(&self) -> ProjectionTaskHot {
        let Self { number, requester, words: _, phase } = self;
        ProjectionTaskHot { number: *number, requester: *requester, phase: phase.clone() }
    }

    /// Place every plan task field in its durable part.
    #[must_use]
    pub fn split(self) -> (ProjectionTaskHot, Box<[u8]>) {
        let Self { number, requester, words, phase } = self;
        (ProjectionTaskHot { number, requester, phase }, words)
    }
}

impl ProjectionTaskHot {
    /// Assemble the whole plan task handed to the feed.
    #[must_use]
    pub fn join(self, words: Box<[u8]>) -> ProjectionTask {
        let Self { number, requester, phase } = self;
        ProjectionTask { number, requester, words, phase }
    }
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
    /// The tracked goal's words.
    GoalWords(u64),
    /// One plan task's words.
    TaskWords { goal: u64, task: u64 },
}

/// One bounded row of a projection; the whole tree is never one journal row.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProjectionRecord {
    /// Feed metadata, retained through the last closing settlement.
    GoalHot { goal: ProjectionGoalHot, pending: Box<[u16]>, changed: bool, fed_at: Option<u64> },
    /// One current task in the bounded tree.
    TaskHot { goal: u64, task: ProjectionTaskHot },
    /// History recorded since the last feed.
    Milestone { goal: u64, milestone: ProjectionMilestone },
    /// The goal's words; `task` equals `goal`.
    GoalWords { goal: u64, task: u64, words: Box<[u8]> },
    /// One plan task's words.
    TaskWords { goal: u64, task: u64, words: Box<[u8]> },
}

impl ProjectionRecord {
    /// The fixed-size address of this bounded row.
    #[must_use]
    pub fn key(&self) -> ProjectionKey {
        match self {
            Self::GoalHot { goal, .. } => ProjectionKey::Goal(goal.number),
            Self::TaskHot { goal, task } => ProjectionKey::Task { goal: *goal, task: task.number },
            Self::Milestone { goal, milestone } => milestone_key(*goal, milestone.identity),
            Self::GoalWords { goal, task, .. } => {
                assert_eq!(goal, task, "goal words identify the goal");
                ProjectionKey::GoalWords(*goal)
            }
            Self::TaskWords { goal, task, .. } => ProjectionKey::TaskWords { goal: *goal, task: *task },
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
        ProjectionRecord::GoalHot { goal, pending, .. } => {
            tasks::phase_bytes(&goal.phase)?.checked_add(len_u64(pending.len()).checked_mul(2)?)
        }
        ProjectionRecord::TaskHot { task, .. } => tasks::phase_bytes(&task.phase),
        ProjectionRecord::GoalWords { words, .. } | ProjectionRecord::TaskWords { words, .. } => {
            Some(len_u64(words.len()))
        }
        ProjectionRecord::Milestone { milestone, .. } => {
            len_u64(milestone.words.len()).checked_add(match &milestone.phase {
                Some(phase) => tasks::phase_bytes(phase)?,
                None => 0,
            })
        }
    }
}
