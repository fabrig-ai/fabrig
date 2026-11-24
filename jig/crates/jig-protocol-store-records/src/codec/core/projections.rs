//! Core durable rows translated exhaustively (protocol/store/records.md, 1).
use super::{View, domain, maps, phase_bytes_to_domain, phase_bytes_to_record, too_large, wire};
use alloc::boxed::Box;
use skein_lib::List;
pub(crate) fn encode_projection_record(
    source: &domain::ProjectionRecord,
    limits: &View,
) -> Result<wire::ProjectionRecord, wire::Problem> {
    Ok(match source {
        domain::ProjectionRecord::GoalHot { goal, pending, changed, fed_at } => {
            let domain::ProjectionGoalHot { number, project, priority, phase } = goal;
            wire::ProjectionRecord::GoalHot(wire::ProjectionGoalHot {
                number: *number,
                project: *project,
                priority: *priority,
                phase: phase_bytes_to_record(phase, limits)?,
                pending: encode_pending(pending, limits)?,
                changed: *changed,
                fed_at: *fed_at,
            })
        }
        domain::ProjectionRecord::TaskHot { goal, task } => {
            let domain::ProjectionTaskHot { number, requester, phase } = task;
            wire::ProjectionRecord::TaskHot(wire::ProjectionTaskHot {
                goal: *goal,
                number: *number,
                requester: maps::party_to_record(requester),
                phase: phase_bytes_to_record(phase, limits)?,
            })
        }
        domain::ProjectionRecord::GoalWords { goal, task, words } => {
            if goal != task {
                return Err(super::problem(skein_codec::Reason::Bound));
            }
            wire::ProjectionRecord::GoalWords(encode_words(*goal, *task, words))
        }
        domain::ProjectionRecord::TaskWords { goal, task, words } => {
            wire::ProjectionRecord::TaskWords(encode_words(*goal, *task, words))
        }
        domain::ProjectionRecord::Milestone { goal, milestone } => {
            wire::ProjectionRecord::Milestone(wire::ProjectionMilestoneRow {
                goal: *goal,
                milestone: maps::projection_milestone_to_record(milestone, limits)?,
            })
        }
    })
}
pub(crate) fn decode_projection_record(
    source: wire::ProjectionRecord,
    limits: &View,
) -> Result<domain::ProjectionRecord, wire::Problem> {
    Ok(match source {
        wire::ProjectionRecord::GoalHot(value) => {
            let wire::ProjectionGoalHot { number, project, priority, phase, pending, changed, fed_at } = value;
            domain::ProjectionRecord::GoalHot {
                goal: domain::ProjectionGoalHot {
                    number,
                    project,
                    priority,
                    phase: phase_bytes_to_domain(phase, limits)?,
                },
                pending: decode_pending(pending),
                changed,
                fed_at,
            }
        }
        wire::ProjectionRecord::TaskHot(value) => {
            let wire::ProjectionTaskHot { goal, number, requester, phase } = value;
            domain::ProjectionRecord::TaskHot {
                goal,
                task: domain::ProjectionTaskHot {
                    number,
                    requester: maps::party_to_domain(requester),
                    phase: phase_bytes_to_domain(phase, limits)?,
                },
            }
        }
        wire::ProjectionRecord::GoalWords(value) => {
            let wire::ProjectionWords { goal, task, words } = value;
            if goal != task {
                return Err(super::problem(skein_codec::Reason::Bound));
            }
            domain::ProjectionRecord::GoalWords { goal, task, words }
        }
        wire::ProjectionRecord::TaskWords(value) => {
            let wire::ProjectionWords { goal, task, words } = value;
            domain::ProjectionRecord::TaskWords { goal, task, words }
        }
        wire::ProjectionRecord::Milestone(value) => {
            let wire::ProjectionMilestoneRow { goal, milestone } = value;
            domain::ProjectionRecord::Milestone {
                goal,
                milestone: maps::projection_milestone_to_domain(milestone, limits)?,
            }
        }
    })
}
fn encode_words(goal: u64, task: u64, words: &[u8]) -> wire::ProjectionWords {
    wire::ProjectionWords { goal, task, words: Box::from(words) }
}

fn encode_pending(values: &[u16], limits: &View) -> Result<List<wire::Connector>, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    };
    if count > limits.core.projection_goal_hot_pending {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    let mut result = List::with_capacity(count);
    for value in values {
        result.push(wire::Connector { value: *value }).expect("same number of connectors");
    }
    Ok(result)
}
fn decode_pending(values: List<wire::Connector>) -> Box<[u16]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(value.value).expect("same number of connectors");
    }
    result.into_boxed()
}
