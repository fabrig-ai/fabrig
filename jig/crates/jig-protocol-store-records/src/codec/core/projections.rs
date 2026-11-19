//! Core durable rows translated exhaustively (protocol/store/records.md, 1).
use super::{View, domain, task_problem, too_large, wire};
use alloc::boxed::Box;
use skein_lib::List;
pub(super) fn encode_projection_milestone(
    source: &domain::ProjectionMilestone,
    limits: &View,
) -> Result<wire::ProjectionMilestone, wire::Problem> {
    let domain::ProjectionMilestone { identity, phase, change, words } = source;
    Ok(wire::ProjectionMilestone {
        identity: encode_milestone_id(*identity, limits),
        phase: match phase {
            Some(value) => Some(encode_phase(value, limits)?),
            None => None,
        },
        change: match change {
            Some(value) => Some(wire::ChangeBytes {
                value: match crate::codec::tasks::carried::change(value, &limits.tasks) {
                    Ok(value) => value,
                    Err(error) => return Err(task_problem(error)),
                },
            }),
            None => None,
        },
        words: words.clone(),
    })
}
pub(super) fn decode_projection_milestone(
    source: wire::ProjectionMilestone,
    limits: &View,
) -> Result<domain::ProjectionMilestone, wire::Problem> {
    let wire::ProjectionMilestone { identity, phase, change, words } = source;
    Ok(domain::ProjectionMilestone {
        identity: decode_milestone_id(identity),
        phase: match phase {
            Some(value) => Some(decode_phase(value, limits)?),
            None => None,
        },
        change: match change {
            Some(value) => Some(match crate::codec::tasks::carried::read_change(&value.value, &limits.tasks) {
                Ok(value) => value,
                Err(error) => return Err(task_problem(error)),
            }),
            None => None,
        },
        words,
    })
}
pub(super) fn encode_projection_record(
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
                phase: encode_phase(phase, limits)?,
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
                requester: encode_party(requester, limits),
                phase: encode_phase(phase, limits)?,
            })
        }
        domain::ProjectionRecord::GoalWords { goal, task, words } => {
            if goal != task {
                return Err(super::problem(skein_codec::Reason::Bound));
            }
            wire::ProjectionRecord::GoalWords(encode_words(*goal, *task, words, limits))
        }
        domain::ProjectionRecord::TaskWords { goal, task, words } => {
            wire::ProjectionRecord::TaskWords(encode_words(*goal, *task, words, limits))
        }
        domain::ProjectionRecord::Milestone { goal, milestone } => {
            wire::ProjectionRecord::Milestone(wire::ProjectionMilestoneRow {
                goal: *goal,
                milestone: encode_projection_milestone(milestone, limits)?,
            })
        }
    })
}
pub(super) fn decode_projection_record(
    source: wire::ProjectionRecord,
    limits: &View,
) -> Result<domain::ProjectionRecord, wire::Problem> {
    Ok(match source {
        wire::ProjectionRecord::GoalHot(value) => {
            let wire::ProjectionGoalHot { number, project, priority, phase, pending, changed, fed_at } = value;
            domain::ProjectionRecord::GoalHot {
                goal: domain::ProjectionGoalHot { number, project, priority, phase: decode_phase(phase, limits)? },
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
                    requester: decode_party(requester),
                    phase: decode_phase(phase, limits)?,
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
            domain::ProjectionRecord::Milestone { goal, milestone: decode_projection_milestone(milestone, limits)? }
        }
    })
}
fn encode_words(goal: u64, task: u64, words: &[u8], _limits: &View) -> wire::ProjectionWords {
    wire::ProjectionWords { goal, task, words: Box::from(words) }
}

fn encode_phase(source: &jig_domain_tasks::Phase, limits: &View) -> Result<wire::PhaseBytes, wire::Problem> {
    Ok(wire::PhaseBytes {
        value: match crate::codec::tasks::carried::phase(source, &limits.tasks) {
            Ok(value) => value,
            Err(error) => return Err(task_problem(error)),
        },
    })
}
fn decode_phase(source: wire::PhaseBytes, limits: &View) -> Result<jig_domain_tasks::Phase, wire::Problem> {
    match crate::codec::tasks::carried::read_phase(&source.value, &limits.tasks) {
        Ok(value) => Ok(value),
        Err(error) => Err(task_problem(error)),
    }
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
fn encode_milestone_id(value: domain::MilestoneId, _limits: &View) -> wire::MilestoneId {
    match value {
        domain::MilestoneId::Lifecycle { task, position } => {
            wire::MilestoneId::Lifecycle(wire::MilestoneLifecycle { task, position })
        }
        domain::MilestoneId::Revision { task, revision } => {
            wire::MilestoneId::Revision(wire::MilestoneRevision { task, revision })
        }
    }
}
fn decode_milestone_id(value: wire::MilestoneId) -> domain::MilestoneId {
    match value {
        wire::MilestoneId::Lifecycle(value) => {
            let wire::MilestoneLifecycle { task, position } = value;
            domain::MilestoneId::Lifecycle { task, position }
        }
        wire::MilestoneId::Revision(value) => {
            let wire::MilestoneRevision { task, revision } = value;
            domain::MilestoneId::Revision { task, revision }
        }
    }
}
fn encode_party(value: &jig_domain_tasks::Party, _limits: &View) -> wire::Party {
    match value {
        jig_domain_tasks::Party::Task(value) => wire::Party::Task(wire::PartyTask { value: *value }),
        jig_domain_tasks::Party::Person(value) => wire::Party::Person(wire::PartyPerson { value: *value }),
        jig_domain_tasks::Party::Deployment { project } => {
            wire::Party::Deployment(wire::PartyDeployment { project: *project })
        }
    }
}
fn decode_party(value: wire::Party) -> jig_domain_tasks::Party {
    match value {
        wire::Party::Task(value) => jig_domain_tasks::Party::Task(value.value),
        wire::Party::Person(value) => jig_domain_tasks::Party::Person(value.value),
        wire::Party::Deployment(value) => jig_domain_tasks::Party::Deployment { project: value.project },
    }
}
