//! Core durable rows translated exhaustively (protocol/store/records.md, 1).
use super::{View, domain, task_problem, too_large, wire};
use alloc::boxed::Box;
use skein_lib::List;
pub(super) fn encode_projection_milestone(
    source: &domain::ProjectionMilestone,
    limits: &View,
) -> Result<wire::ProjectionMilestone, wire::Problem> {
    let domain::ProjectionMilestone { identity, phase, change, words } = source;
    wire::ProjectionMilestone::new(
        &limits.core,
        wire::ProjectionMilestoneParts {
            identity: encode_milestone_id(*identity, limits)?,
            phase: match phase {
                Some(value) => Some(encode_phase(value, limits)?),
                None => None,
            },
            change: match change {
                Some(value) => Some(wire::ChangeBytes::new(
                    &limits.core,
                    wire::ChangeBytesParts {
                        value: match crate::codec::tasks::carried::change(value, &limits.tasks) {
                            Ok(value) => value,
                            Err(error) => return Err(task_problem(error)),
                        },
                    },
                )?),
                None => None,
            },
            words: words.clone(),
        },
    )
}
pub(super) fn decode_projection_milestone(
    source: wire::ProjectionMilestone,
    limits: &View,
) -> Result<domain::ProjectionMilestone, wire::Problem> {
    let wire::ProjectionMilestoneParts { identity, phase, change, words } = source.into_parts();
    Ok(domain::ProjectionMilestone {
        identity: decode_milestone_id(identity),
        phase: match phase {
            Some(value) => Some(decode_phase(value, limits)?),
            None => None,
        },
        change: match change {
            Some(value) => {
                Some(match crate::codec::tasks::carried::read_change(&value.into_parts().value, &limits.tasks) {
                    Ok(value) => value,
                    Err(error) => return Err(task_problem(error)),
                })
            }
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
            wire::ProjectionRecord::GoalHot(wire::ProjectionGoalHot::new(
                &limits.core,
                wire::ProjectionGoalHotParts {
                    number: *number,
                    project: *project,
                    priority: *priority,
                    phase: encode_phase(phase, limits)?,
                    pending: encode_pending(pending, limits)?,
                    changed: *changed,
                    fed_at: *fed_at,
                },
            )?)
        }
        domain::ProjectionRecord::TaskHot { goal, task } => {
            let domain::ProjectionTaskHot { number, requester, phase } = task;
            wire::ProjectionRecord::TaskHot(wire::ProjectionTaskHot::new(
                &limits.core,
                wire::ProjectionTaskHotParts {
                    goal: *goal,
                    number: *number,
                    requester: encode_party(requester, limits)?,
                    phase: encode_phase(phase, limits)?,
                },
            )?)
        }
        domain::ProjectionRecord::GoalWords { goal, task, words } => {
            if goal != task {
                return Err(super::problem(skein_codec::Reason::Bound));
            }
            wire::ProjectionRecord::GoalWords(encode_words(*goal, *task, words, limits)?)
        }
        domain::ProjectionRecord::TaskWords { goal, task, words } => {
            wire::ProjectionRecord::TaskWords(encode_words(*goal, *task, words, limits)?)
        }
        domain::ProjectionRecord::Milestone { goal, milestone } => {
            wire::ProjectionRecord::Milestone(wire::ProjectionMilestoneRow::new(
                &limits.core,
                wire::ProjectionMilestoneRowParts {
                    goal: *goal,
                    milestone: encode_projection_milestone(milestone, limits)?,
                },
            )?)
        }
    })
}
pub(super) fn decode_projection_record(
    source: wire::ProjectionRecord,
    limits: &View,
) -> Result<domain::ProjectionRecord, wire::Problem> {
    Ok(match source {
        wire::ProjectionRecord::Goal(_) | wire::ProjectionRecord::Task(_) => {
            return Err(super::problem(skein_codec::Reason::Tag));
        }
        wire::ProjectionRecord::GoalHot(value) => {
            let wire::ProjectionGoalHotParts { number, project, priority, phase, pending, changed, fed_at } =
                value.into_parts();
            domain::ProjectionRecord::GoalHot {
                goal: domain::ProjectionGoalHot { number, project, priority, phase: decode_phase(phase, limits)? },
                pending: decode_pending(pending),
                changed,
                fed_at,
            }
        }
        wire::ProjectionRecord::TaskHot(value) => {
            let wire::ProjectionTaskHotParts { goal, number, requester, phase } = value.into_parts();
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
            let wire::ProjectionWordsParts { goal, task, words } = value.into_parts();
            if goal != task {
                return Err(super::problem(skein_codec::Reason::Bound));
            }
            domain::ProjectionRecord::GoalWords { goal, task, words }
        }
        wire::ProjectionRecord::TaskWords(value) => {
            let wire::ProjectionWordsParts { goal, task, words } = value.into_parts();
            domain::ProjectionRecord::TaskWords { goal, task, words }
        }
        wire::ProjectionRecord::Milestone(value) => {
            let wire::ProjectionMilestoneRowParts { goal, milestone } = value.into_parts();
            domain::ProjectionRecord::Milestone { goal, milestone: decode_projection_milestone(milestone, limits)? }
        }
    })
}
fn encode_words(goal: u64, task: u64, words: &[u8], limits: &View) -> Result<wire::ProjectionWords, wire::Problem> {
    wire::ProjectionWords::new(&limits.core, wire::ProjectionWordsParts { goal, task, words: Box::from(words) })
}

/// Split a whole kept row; rows already in parts pass without a rewrite (parts.md, 4).
pub(crate) fn split(
    source: wire::ProjectionRecord,
) -> Result<Option<(wire::ProjectionRecord, wire::ProjectionRecord)>, wire::Problem> {
    Ok(match source {
        wire::ProjectionRecord::Goal(value) => {
            let wire::ProjectionGoalRowParts { goal, pending, changed, fed_at } = value.into_parts();
            let wire::ProjectionGoalParts { number, project, priority, words, phase } = goal.into_parts();
            Some((
                wire::ProjectionRecord::GoalHot(wire::ProjectionGoalHot::new(
                    &wire::CEILINGS,
                    wire::ProjectionGoalHotParts { number, project, priority, phase, pending, changed, fed_at },
                )?),
                wire::ProjectionRecord::GoalWords(wire::ProjectionWords::new(
                    &wire::CEILINGS,
                    wire::ProjectionWordsParts { goal: number, task: number, words },
                )?),
            ))
        }
        wire::ProjectionRecord::Task(value) => {
            let wire::ProjectionTaskRowParts { goal, task } = value.into_parts();
            let wire::ProjectionTaskParts { number, requester, words, phase } = task.into_parts();
            Some((
                wire::ProjectionRecord::TaskHot(wire::ProjectionTaskHot::new(
                    &wire::CEILINGS,
                    wire::ProjectionTaskHotParts { goal, number, requester, phase },
                )?),
                wire::ProjectionRecord::TaskWords(wire::ProjectionWords::new(
                    &wire::CEILINGS,
                    wire::ProjectionWordsParts { goal, task: number, words },
                )?),
            ))
        }
        wire::ProjectionRecord::Milestone(_)
        | wire::ProjectionRecord::GoalHot(_)
        | wire::ProjectionRecord::TaskHot(_)
        | wire::ProjectionRecord::GoalWords(_)
        | wire::ProjectionRecord::TaskWords(_) => None,
    })
}

fn encode_phase(source: &jig_domain_tasks::Phase, limits: &View) -> Result<wire::PhaseBytes, wire::Problem> {
    wire::PhaseBytes::new(
        &limits.core,
        wire::PhaseBytesParts {
            value: match crate::codec::tasks::carried::phase(source, &limits.tasks) {
                Ok(value) => value,
                Err(error) => return Err(task_problem(error)),
            },
        },
    )
}
fn decode_phase(source: wire::PhaseBytes, limits: &View) -> Result<jig_domain_tasks::Phase, wire::Problem> {
    match crate::codec::tasks::carried::read_phase(&source.into_parts().value, &limits.tasks) {
        Ok(value) => Ok(value),
        Err(error) => Err(task_problem(error)),
    }
}
fn encode_pending(values: &[u16], limits: &View) -> Result<List<wire::Connector>, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    };
    if count > limits.core.projection_goal_row_pending {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    let mut result = List::with_capacity(count);
    for value in values {
        result
            .push(wire::Connector::new(&limits.core, wire::ConnectorParts { value: *value })?)
            .expect("same number of connectors");
    }
    Ok(result)
}
fn decode_pending(values: List<wire::Connector>) -> Box<[u16]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(value.into_parts().value).expect("same number of connectors");
    }
    result.into_boxed()
}
fn encode_milestone_id(value: domain::MilestoneId, limits: &View) -> Result<wire::MilestoneId, wire::Problem> {
    Ok(match value {
        domain::MilestoneId::Lifecycle { task, position } => wire::MilestoneId::Lifecycle(
            wire::MilestoneLifecycle::new(&limits.core, wire::MilestoneLifecycleParts { task, position })?,
        ),
        domain::MilestoneId::Revision { task, revision } => wire::MilestoneId::Revision(wire::MilestoneRevision::new(
            &limits.core,
            wire::MilestoneRevisionParts { task, revision },
        )?),
    })
}
fn decode_milestone_id(value: wire::MilestoneId) -> domain::MilestoneId {
    match value {
        wire::MilestoneId::Lifecycle(value) => {
            let wire::MilestoneLifecycleParts { task, position } = value.into_parts();
            domain::MilestoneId::Lifecycle { task, position }
        }
        wire::MilestoneId::Revision(value) => {
            let wire::MilestoneRevisionParts { task, revision } = value.into_parts();
            domain::MilestoneId::Revision { task, revision }
        }
    }
}
fn encode_party(value: &jig_domain_tasks::Party, limits: &View) -> Result<wire::Party, wire::Problem> {
    Ok(match value {
        jig_domain_tasks::Party::Task(value) => {
            wire::Party::Task(wire::PartyTask::new(&limits.core, wire::PartyTaskParts { value: *value })?)
        }
        jig_domain_tasks::Party::Person(value) => {
            wire::Party::Person(wire::PartyPerson::new(&limits.core, wire::PartyPersonParts { value: *value })?)
        }
        jig_domain_tasks::Party::Deployment { project } => wire::Party::Deployment(wire::PartyDeployment::new(
            &limits.core,
            wire::PartyDeploymentParts { project: *project },
        )?),
    })
}
fn decode_party(value: wire::Party) -> jig_domain_tasks::Party {
    match value {
        wire::Party::Task(value) => jig_domain_tasks::Party::Task(value.into_parts().value),
        wire::Party::Person(value) => jig_domain_tasks::Party::Person(value.into_parts().value),
        wire::Party::Deployment(value) => jig_domain_tasks::Party::Deployment { project: value.into_parts().project },
    }
}
