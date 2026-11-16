//! A milestone's totals (temper/docs/design/client/forge/records.md).
use super::{forge, w, wire};
use alloc::boxed::Box;

pub(super) fn answer(value: Option<forge::MilestoneTotals>, limits: &w::Limits) -> Option<Box<[u8]>> {
    let result = match value {
        None => w::MilestoneResult::NotMilestone,
        Some(value) => {
            let last_landing = match value.last_landing {
                Some(landing) => Some(
                    w::MilestoneLanding::new(
                        limits,
                        w::MilestoneLandingParts {
                            at: landing.at.as_nanos(),
                            goal: landing.goal,
                            change: landing.change,
                        },
                    )
                    .ok()?,
                ),
                None => None,
            };
            let ended = match value.ended {
                Some(at) => Some(at.as_nanos()),
                None => None,
            };
            w::MilestoneResult::Totals(
                w::MilestoneTotals::new(
                    limits,
                    w::MilestoneTotalsParts {
                        task: value.task,
                        done: value.done,
                        failed: value.failed,
                        closed: value.closed,
                        live: value.live,
                        ended,
                        last_landing,
                        spent: value.spent,
                    },
                )
                .ok()?,
            )
        }
    };
    let answer = w::MilestoneAnswer::new(limits, w::MilestoneAnswerParts { result }).ok()?;
    Some(wire::milestone_answer(&answer))
}
