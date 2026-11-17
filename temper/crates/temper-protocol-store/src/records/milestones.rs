//! The forge's milestone row (temper/docs/design/domain/milestones.md, 8).
use super::{FITS, d, w};
use skein_lib::Wall;

pub(super) fn encode(row: &d::MilestoneRow) -> w::MilestoneRow {
    let last_landing = match &row.last_landing {
        Some(landing) => Some(
            w::MilestoneLanding::new(
                &w::CEILINGS,
                w::MilestoneLandingParts { at: landing.at.as_nanos(), goal: landing.goal, change: landing.change },
            )
            .expect(FITS),
        ),
        None => None,
    };
    let ended = match &row.ended {
        Some(end) => Some(
            w::MilestoneEnd::new(
                &w::CEILINGS,
                w::MilestoneEndParts {
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
                },
            )
            .expect(FITS),
        ),
        None => None,
    };
    w::MilestoneRow::new(
        &w::CEILINGS,
        w::MilestoneRowParts { project: row.project, task: row.task, name: row.name.clone(), last_landing, ended },
    )
    .expect(FITS)
}

pub(super) fn decode(row: w::MilestoneRow) -> d::MilestoneRow {
    let w::MilestoneRowParts { project, task, name, last_landing, ended } = row.into_parts();
    let last_landing = match last_landing {
        Some(landing) => Some(d::MilestoneLanding {
            at: Wall::from_nanos(landing.at()),
            goal: landing.goal(),
            change: landing.change(),
        }),
        None => None,
    };
    let ended = match ended {
        Some(end) => Some(d::MilestoneEnd {
            at: Wall::from_nanos(end.at()),
            outcome: match end.outcome() {
                w::MilestoneOutcome::Complete => d::MilestoneOutcome::Complete,
                w::MilestoneOutcome::Ended => d::MilestoneOutcome::Ended,
                w::MilestoneOutcome::Closed => d::MilestoneOutcome::Closed,
            },
            done: end.done(),
            failed: end.failed(),
            closed: end.closed(),
            spent: end.spent(),
        }),
        None => None,
    };
    d::MilestoneRow { project, task, name, last_landing, ended }
}
