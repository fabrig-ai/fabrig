//! The forge's milestone row (temper/docs/design/domain/milestones.md, 8).
use super::{d, w};
use skein_lib::Wall;

pub(super) fn encode(row: &d::MilestoneRow) -> w::MilestoneRow {
    let last_landing = match &row.last_landing {
        Some(landing) => {
            Some(w::MilestoneLanding { at: landing.at.as_nanos(), goal: landing.goal, change: landing.change })
        }
        None => None,
    };
    let ended = match &row.ended {
        Some(end) => Some(w::MilestoneEnd {
            at: end.at.as_nanos(),
            outcome: match end.outcome {
                d::MilestoneOutcome::Complete => w::MilestoneOutcome::Complete,
                d::MilestoneOutcome::Ended => w::MilestoneOutcome::Ended,
                d::MilestoneOutcome::Closed => w::MilestoneOutcome::Closed,
            },
            done: end.done,
            failed: end.failed,
            closed: end.closed,
            spent: end.spent,
        }),
        None => None,
    };
    w::MilestoneRow { project: row.project, task: row.task, name: row.name.clone(), last_landing, ended }
}

pub(super) fn decode(row: w::MilestoneRow) -> d::MilestoneRow {
    let w::MilestoneRow { project, task, name, last_landing, ended } = row;
    let last_landing = match last_landing {
        Some(landing) => {
            Some(d::MilestoneLanding { at: Wall::from_nanos(landing.at), goal: landing.goal, change: landing.change })
        }
        None => None,
    };
    let ended = match ended {
        Some(end) => Some(d::MilestoneEnd {
            at: Wall::from_nanos(end.at),
            outcome: match end.outcome {
                w::MilestoneOutcome::Complete => d::MilestoneOutcome::Complete,
                w::MilestoneOutcome::Ended => d::MilestoneOutcome::Ended,
                w::MilestoneOutcome::Closed => d::MilestoneOutcome::Closed,
            },
            done: end.done,
            failed: end.failed,
            closed: end.closed,
            spent: end.spent,
        }),
        None => None,
    };
    d::MilestoneRow { project, task, name, last_landing, ended }
}
