//! Wire projections of owner historical facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
use super::{
    wire_funder, wire_object_key, wire_owner_activity_line, wire_owner_authority, wire_owner_executor,
    wire_owner_message_proposal_kind, wire_owner_milestone_line, wire_owner_numbers, wire_owner_person_task,
    wire_owner_proposal_choice, wire_owner_proposal_history, wire_owner_requester, wire_owner_wake_policy,
    wire_proposer, wire_question, wire_result_document, wire_specification, wire_task, wire_task_contract_document,
};
pub(in crate::codec) fn wire_owner_historical(
    value: &c::OwnerHistorical,
    limits: &w::Limits,
) -> Result<w::OwnerHistorical, w::Problem> {
    w::OwnerHistorical::new(
        limits,
        w::OwnerHistoricalParts {
            key: wire_object_key(&value.key, limits)?,
            value: wire_owner_historical_value(&value.value, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            by: value.by.as_ref().map(|item| wire_owner_requester(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_historical_value(
    value: &c::OwnerHistoricalValue,
    limits: &w::Limits,
) -> Result<w::OwnerHistoricalValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHistoricalValue::Task { document, result } => {
            w::OwnerHistoricalValue::Task(w::OwnerHistoricalValueTask::new(
                limits,
                w::OwnerHistoricalValueTaskParts {
                    document: wire_task(document, limits)?,
                    result: wire_result_document(result, limits)?,
                },
            )?)
        }
        c::OwnerHistoricalValue::Proposal(value) => {
            w::OwnerHistoricalValue::Proposal(w::OwnerHistoricalValueProposal::new(
                limits,
                w::OwnerHistoricalValueProposalParts { value: wire_owner_proposal_history(value, limits)? },
            )?)
        }
        c::OwnerHistoricalValue::PersonProposal { proposal, decision } => {
            w::OwnerHistoricalValue::PersonProposal(w::OwnerHistoricalValuePersonProposal::new(
                limits,
                w::OwnerHistoricalValuePersonProposalParts {
                    proposal: wire_owner_person_proposal(proposal, limits)?,
                    decision: (decision).as_ref().map(|item| wire_owner_proposal_decision(item, limits)).transpose()?,
                },
            )?)
        }
        c::OwnerHistoricalValue::Escalation { task, decision, hold } => {
            w::OwnerHistoricalValue::Escalation(w::OwnerHistoricalValueEscalation::new(
                limits,
                w::OwnerHistoricalValueEscalationParts {
                    task: wire_task(task, limits)?,
                    decision: wire_owner_escalation_decision_record(decision, limits)?,
                    hold: (hold).as_ref().map(|item| wire_owner_milestone_line(item, limits)).transpose()?,
                },
            )?)
        }
        c::OwnerHistoricalValue::Question { question, answer, ending } => {
            w::OwnerHistoricalValue::Question(w::OwnerHistoricalValueQuestion::new(
                limits,
                w::OwnerHistoricalValueQuestionParts {
                    question: wire_question(question, limits)?,
                    answer: (answer).as_ref().map(|item| wire_owner_activity_line(item, limits)).transpose()?,
                    ending: (ending).as_ref().map(|item| wire_result_document(item, limits)).transpose()?,
                },
            )?)
        }
        c::OwnerHistoricalValue::PersonTask { document, result } => {
            w::OwnerHistoricalValue::PersonTask(w::OwnerHistoricalValuePersonTask::new(
                limits,
                w::OwnerHistoricalValuePersonTaskParts {
                    document: wire_owner_person_task(document, limits)?,
                    result: wire_result_document(result, limits)?,
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_person_proposal(
    value: &c::OwnerPersonProposal,
    limits: &w::Limits,
) -> Result<w::OwnerPersonProposal, w::Problem> {
    w::OwnerPersonProposal::new(
        limits,
        w::OwnerPersonProposalParts {
            number: value.number,
            proposer: value.proposer,
            project: value.project,
            goal: wire_owner_person_proposal_goal(&value.goal, limits)?,
            state: wire_owner_person_proposal_state(&value.state, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_person_proposal_goal(
    value: &c::OwnerPersonProposalGoal,
    limits: &w::Limits,
) -> Result<w::OwnerPersonProposalGoal, w::Problem> {
    w::OwnerPersonProposalGoal::new(
        limits,
        w::OwnerPersonProposalGoalParts {
            number: value.number,
            project: value.project,
            executor: wire_owner_executor(&value.executor, limits)?,
            spec: wire_specification(&value.spec, limits)?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            authority: wire_owner_authority(&value.authority, limits)?,
            numbers: wire_owner_numbers(&value.numbers, limits)?,
            funder: wire_funder(&value.funder, limits)?,
            dependencies: list(value.dependencies.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            holdings: pieces(
                value.holdings.iter().map(|item| wire_owner_goal_holding(item, limits)),
                w::OwnerGoalHolding::measure,
                w::OwnerGoalHolding::encode,
            )?,
            wake: wire_owner_wake_policy(&value.wake, limits)?,
            tracked: value.tracked.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_goal_holding(
    value: &c::OwnerGoalHolding,
    limits: &w::Limits,
) -> Result<w::OwnerGoalHolding, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerGoalHolding::Write { connector, path, kind } => {
            w::OwnerGoalHolding::Write(w::OwnerGoalHoldingWrite::new(
                limits,
                w::OwnerGoalHoldingWriteParts {
                    connector: *(connector),
                    path: list((path).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
                    kind: *(kind),
                },
            )?)
        }
        c::OwnerGoalHolding::Slot { connector, path, kind } => w::OwnerGoalHolding::Slot(w::OwnerGoalHoldingSlot::new(
            limits,
            w::OwnerGoalHoldingSlotParts {
                connector: *(connector),
                path: list((path).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
                kind: *(kind),
            },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_person_proposal_state(
    value: &c::OwnerPersonProposalState,
    limits: &w::Limits,
) -> Result<w::OwnerPersonProposalState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPersonProposalState::Pending { since } => {
            w::OwnerPersonProposalState::Pending(w::OwnerPersonProposalStatePending::new(
                limits,
                w::OwnerPersonProposalStatePendingParts { since: (since).as_nanos() },
            )?)
        }
        c::OwnerPersonProposalState::Accepted { by } => {
            w::OwnerPersonProposalState::Accepted(w::OwnerPersonProposalStateAccepted::new(
                limits,
                w::OwnerPersonProposalStateAcceptedParts { by: wire_owner_requester(by, limits)? },
            )?)
        }
        c::OwnerPersonProposalState::Rejected { by, reason, message, at } => {
            w::OwnerPersonProposalState::Rejected(w::OwnerPersonProposalStateRejected::new(
                limits,
                w::OwnerPersonProposalStateRejectedParts {
                    by: wire_owner_requester(by, limits)?,
                    reason: bytes::copy_of((reason).bytes()),
                    message: *(message),
                    at: (at).as_nanos(),
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_proposal_decision(
    value: &c::OwnerProposalDecision,
    limits: &w::Limits,
) -> Result<w::OwnerProposalDecision, w::Problem> {
    w::OwnerProposalDecision::new(
        limits,
        w::OwnerProposalDecisionParts {
            project: value.project,
            proposer: wire_proposer(&value.proposer, limits)?,
            proposal: value.proposal,
            kind: wire_owner_message_proposal_kind(&value.kind, limits)?,
            by: value.by,
            at: value.at.as_nanos(),
            choice: wire_owner_proposal_choice(&value.choice, limits)?,
            created: value
                .created
                .as_ref()
                .map(|item| {
                    w::OwnerProposalDecisionCreated::new(
                        limits,
                        w::OwnerProposalDecisionCreatedParts { first: item.0, last: item.1 },
                    )
                })
                .transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_escalation_decision_record(
    value: &c::OwnerEscalationDecisionRecord,
    limits: &w::Limits,
) -> Result<w::OwnerEscalationDecisionRecord, w::Problem> {
    w::OwnerEscalationDecisionRecord::new(
        limits,
        w::OwnerEscalationDecisionRecordParts {
            project: value.project,
            requester: value.requester,
            task: value.task,
            revision: value.revision,
            by: value.by,
            at: value.at.as_nanos(),
            decision: wire_owner_escalation_decision(&value.decision, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_escalation_decision(
    value: &c::OwnerEscalationDecision,
    limits: &w::Limits,
) -> Result<w::OwnerEscalationDecision, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerEscalationDecision::Release => w::OwnerEscalationDecision::Release,
        c::OwnerEscalationDecision::Reject { reason } => {
            w::OwnerEscalationDecision::Reject(w::OwnerEscalationDecisionReject::new(
                limits,
                w::OwnerEscalationDecisionRejectParts { reason: bytes::copy_of((reason).bytes()) },
            )?)
        }
        c::OwnerEscalationDecision::Pass => w::OwnerEscalationDecision::Pass,
    })
}
