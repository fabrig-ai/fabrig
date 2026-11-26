//! Projection feeds and durable rows (jig/docs/design/domain/connectors.md, 8).

pub use jig_domain_contract::projections::{
    MilestoneId, ProjectionFeed, ProjectionGoal, ProjectionGoalHot, ProjectionKey, ProjectionMilestone,
    ProjectionRecord, ProjectionTask, ProjectionTaskHot, milestone_key,
};
use jig_domain_tasks as tasks;
use skein_lib::length::len_u64;

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
