//! Wire projections of owner ended facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
use super::{
    wire_activity_line, wire_authority, wire_executor, wire_funder, wire_message_proposal_kind, wire_milestone_line,
    wire_numbers, wire_object_key, wire_person_task, wire_proposal_choice, wire_proposal_history, wire_proposer,
    wire_question, wire_requester, wire_result_document, wire_specification, wire_task, wire_task_contract_document,
    wire_wake_policy,
};
pub(in crate::codec) fn wire_ended(value: &c::Ended, limits: &w::Limits) -> Result<w::Ended, w::Problem> {
    w::Ended::new(
        limits,
        w::EndedParts {
            key: wire_object_key(&value.key, limits)?,
            value: wire_ended_value(&value.value, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            by: value.by.as_ref().map(|item| wire_requester(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_ended_value(
    value: &c::EndedValue,
    limits: &w::Limits,
) -> Result<w::EndedValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EndedValue::Task { document, result } => w::EndedValue::Task(w::EndedValueTask::new(
            limits,
            w::EndedValueTaskParts {
                document: wire_task(document, limits)?,
                result: wire_result_document(result, limits)?,
            },
        )?),
        c::EndedValue::Proposal(value) => w::EndedValue::Proposal(w::EndedValueProposal::new(
            limits,
            w::EndedValueProposalParts { value: wire_proposal_history(value, limits)? },
        )?),
        c::EndedValue::PersonProposal { proposal, decision } => {
            w::EndedValue::PersonProposal(w::EndedValuePersonProposal::new(
                limits,
                w::EndedValuePersonProposalParts {
                    proposal: wire_person_proposal(proposal, limits)?,
                    decision: (decision).as_ref().map(|item| wire_proposal_decision(item, limits)).transpose()?,
                },
            )?)
        }
        c::EndedValue::Escalation { task, decision, hold } => w::EndedValue::Escalation(w::EndedValueEscalation::new(
            limits,
            w::EndedValueEscalationParts {
                task: wire_task(task, limits)?,
                decision: wire_escalation_decision_record(decision, limits)?,
                hold: (hold).as_ref().map(|item| wire_milestone_line(item, limits)).transpose()?,
            },
        )?),
        c::EndedValue::Question { question, answer, ending } => w::EndedValue::Question(w::EndedValueQuestion::new(
            limits,
            w::EndedValueQuestionParts {
                question: wire_question(question, limits)?,
                answer: (answer).as_ref().map(|item| wire_activity_line(item, limits)).transpose()?,
                ending: (ending).as_ref().map(|item| wire_result_document(item, limits)).transpose()?,
            },
        )?),
        c::EndedValue::PersonTask { document, result } => w::EndedValue::PersonTask(w::EndedValuePersonTask::new(
            limits,
            w::EndedValuePersonTaskParts {
                document: wire_person_task(document, limits)?,
                result: wire_result_document(result, limits)?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_person_proposal(
    value: &c::PersonProposal,
    limits: &w::Limits,
) -> Result<w::PersonProposal, w::Problem> {
    w::PersonProposal::new(
        limits,
        w::PersonProposalParts {
            number: value.number,
            proposer: value.proposer,
            project: value.project,
            goal: wire_person_proposal_goal(&value.goal, limits)?,
            state: wire_person_proposal_state(&value.state, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_person_proposal_goal(
    value: &c::PersonProposalGoal,
    limits: &w::Limits,
) -> Result<w::PersonProposalGoal, w::Problem> {
    w::PersonProposalGoal::new(
        limits,
        w::PersonProposalGoalParts {
            number: value.number,
            project: value.project,
            executor: wire_executor(&value.executor, limits)?,
            spec: wire_specification(&value.spec, limits)?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            authority: wire_authority(&value.authority, limits)?,
            numbers: wire_numbers(&value.numbers, limits)?,
            funder: wire_funder(&value.funder, limits)?,
            dependencies: list(value.dependencies.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            holdings: pieces(
                value.holdings.iter().map(|item| wire_goal_holding(item, limits)),
                w::GoalHolding::measure,
                w::GoalHolding::encode,
            )?,
            wake: wire_wake_policy(&value.wake, limits)?,
            tracked: value.tracked.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_goal_holding(
    value: &c::GoalHolding,
    limits: &w::Limits,
) -> Result<w::GoalHolding, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::GoalHolding::Write { connector, path, kind } => w::GoalHolding::Write(w::GoalHoldingWrite::new(
            limits,
            w::GoalHoldingWriteParts {
                connector: *(connector),
                path: list((path).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
                kind: *(kind),
            },
        )?),
        c::GoalHolding::Slot { connector, path, kind } => w::GoalHolding::Slot(w::GoalHoldingSlot::new(
            limits,
            w::GoalHoldingSlotParts {
                connector: *(connector),
                path: list((path).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
                kind: *(kind),
            },
        )?),
    })
}

pub(in crate::codec) fn wire_person_proposal_state(
    value: &c::PersonProposalState,
    limits: &w::Limits,
) -> Result<w::PersonProposalState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PersonProposalState::Pending { since } => {
            w::PersonProposalState::Pending(w::PersonProposalStatePending::new(
                limits,
                w::PersonProposalStatePendingParts { since: (since).as_nanos() },
            )?)
        }
        c::PersonProposalState::Accepted { by } => {
            w::PersonProposalState::Accepted(w::PersonProposalStateAccepted::new(
                limits,
                w::PersonProposalStateAcceptedParts { by: wire_requester(by, limits)? },
            )?)
        }
        c::PersonProposalState::Rejected { by, reason, message, at } => {
            w::PersonProposalState::Rejected(w::PersonProposalStateRejected::new(
                limits,
                w::PersonProposalStateRejectedParts {
                    by: wire_requester(by, limits)?,
                    reason: bytes::copy_of((reason).bytes()),
                    message: *(message),
                    at: (at).as_nanos(),
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_proposal_decision(
    value: &c::ProposalDecision,
    limits: &w::Limits,
) -> Result<w::ProposalDecision, w::Problem> {
    w::ProposalDecision::new(
        limits,
        w::ProposalDecisionParts {
            project: value.project,
            proposer: wire_proposer(&value.proposer, limits)?,
            proposal: value.proposal,
            kind: wire_message_proposal_kind(&value.kind, limits)?,
            by: value.by,
            at: value.at.as_nanos(),
            choice: wire_proposal_choice(&value.choice, limits)?,
            created: value
                .created
                .as_ref()
                .map(|item| {
                    w::ProposalDecisionCreated::new(
                        limits,
                        w::ProposalDecisionCreatedParts { first: item.0, last: item.1 },
                    )
                })
                .transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_escalation_decision_record(
    value: &c::EscalationDecisionRecord,
    limits: &w::Limits,
) -> Result<w::EscalationDecisionRecord, w::Problem> {
    w::EscalationDecisionRecord::new(
        limits,
        w::EscalationDecisionRecordParts {
            project: value.project,
            requester: value.requester,
            task: value.task,
            revision: value.revision,
            by: value.by,
            at: value.at.as_nanos(),
            decision: wire_escalation_decision(&value.decision, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_escalation_decision(
    value: &c::EscalationDecision,
    limits: &w::Limits,
) -> Result<w::EscalationDecision, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EscalationDecision::Release => w::EscalationDecision::Release,
        c::EscalationDecision::Reject { reason } => w::EscalationDecision::Reject(w::EscalationDecisionReject::new(
            limits,
            w::EscalationDecisionRejectParts { reason: bytes::copy_of((reason).bytes()) },
        )?),
        c::EscalationDecision::Pass => w::EscalationDecision::Pass,
    })
}
