//! Wire projections of owner ended facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
use super::{
    wire_activity_line, wire_authority, wire_executor, wire_funder, wire_hold_line, wire_message_proposal_kind,
    wire_numbers, wire_object_key, wire_person_task, wire_proposal_choice, wire_proposal_history, wire_proposer,
    wire_question, wire_requester, wire_result_document, wire_specification, wire_task, wire_task_contract_document,
    wire_wake_policy,
};
pub(in crate::codec) fn wire_ended(value: &c::Ended, limits: &w::Limits) -> Result<w::Ended, w::Problem> {
    Ok(w::Ended {
        key: wire_object_key(&value.key),
        value: wire_ended_value(&value.value, limits)?,
        at: value.at.as_ref().map(|item| (item).as_nanos()),
        by: value.by.as_ref().map(wire_requester),
    })
}

pub(in crate::codec) fn wire_ended_value(
    value: &c::EndedValue,
    limits: &w::Limits,
) -> Result<w::EndedValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EndedValue::Task { document, result } => w::EndedValue::Task(w::EndedValueTask {
            document: wire_task(document, limits)?,
            result: wire_result_document(result, limits)?,
        }),
        c::EndedValue::Proposal(value) => {
            w::EndedValue::Proposal(w::EndedValueProposal { value: wire_proposal_history(value, limits)? })
        }
        c::EndedValue::PersonProposal { proposal, decision } => {
            w::EndedValue::PersonProposal(w::EndedValuePersonProposal {
                proposal: wire_person_proposal(proposal, limits)?,
                decision: (decision).as_ref().map(wire_proposal_decision),
            })
        }
        c::EndedValue::Escalation { task, decision, hold } => w::EndedValue::Escalation(w::EndedValueEscalation {
            task: wire_task(task, limits)?,
            decision: wire_escalation_decision_record(decision),
            hold: (hold).as_ref().map(|item| wire_hold_line(item, limits)).transpose()?,
        }),
        c::EndedValue::Question { question, answer, ending } => w::EndedValue::Question(w::EndedValueQuestion {
            question: wire_question(question, limits)?,
            answer: (answer).as_ref().map(wire_activity_line),
            ending: (ending).as_ref().map(|item| wire_result_document(item, limits)).transpose()?,
        }),
        c::EndedValue::PersonTask { document, result } => w::EndedValue::PersonTask(w::EndedValuePersonTask {
            document: wire_person_task(document, limits)?,
            result: wire_result_document(result, limits)?,
        }),
    })
}

pub(in crate::codec) fn wire_person_proposal(
    value: &c::PersonProposal,
    limits: &w::Limits,
) -> Result<w::PersonProposal, w::Problem> {
    Ok(w::PersonProposal {
        number: value.number,
        proposer: value.proposer,
        project: value.project,
        goal: wire_person_proposal_goal(&value.goal, limits)?,
        state: wire_person_proposal_state(&value.state),
    })
}

pub(in crate::codec) fn wire_person_proposal_goal(
    value: &c::PersonProposalGoal,
    limits: &w::Limits,
) -> Result<w::PersonProposalGoal, w::Problem> {
    Ok(w::PersonProposalGoal {
        number: value.number,
        project: value.project,
        executor: wire_executor(&value.executor),
        spec: wire_specification(&value.spec),
        contract: wire_task_contract_document(&value.contract, limits)?,
        authority: wire_authority(&value.authority, limits)?,
        numbers: wire_numbers(&value.numbers),
        funder: wire_funder(&value.funder),
        dependencies: list(value.dependencies.iter().copied()),
        holdings: pieces(value.holdings.iter().map(wire_goal_holding), limits)?,
        wake: wire_wake_policy(&value.wake),
        tracked: value.tracked,
    })
}

pub(in crate::codec) fn wire_goal_holding(value: &c::GoalHolding) -> w::GoalHolding {
    match value {
        c::GoalHolding::Write { connector, path, kind } => w::GoalHolding::Write(w::GoalHoldingWrite {
            connector: *(connector),
            path: list((path).iter().map(|item| bytes::copy_of((item).bytes()))),
            kind: *(kind),
        }),
        c::GoalHolding::Slot { connector, path, kind } => w::GoalHolding::Slot(w::GoalHoldingSlot {
            connector: *(connector),
            path: list((path).iter().map(|item| bytes::copy_of((item).bytes()))),
            kind: *(kind),
        }),
    }
}

pub(in crate::codec) fn wire_person_proposal_state(value: &c::PersonProposalState) -> w::PersonProposalState {
    match value {
        c::PersonProposalState::Pending { since } => {
            w::PersonProposalState::Pending(w::PersonProposalStatePending { since: (since).as_nanos() })
        }
        c::PersonProposalState::Accepted { by } => {
            w::PersonProposalState::Accepted(w::PersonProposalStateAccepted { by: wire_requester(by) })
        }
        c::PersonProposalState::Rejected { by, reason, message, at } => {
            w::PersonProposalState::Rejected(w::PersonProposalStateRejected {
                by: wire_requester(by),
                reason: bytes::copy_of((reason).bytes()),
                message: *(message),
                at: (at).as_nanos(),
            })
        }
    }
}

pub(in crate::codec) fn wire_proposal_decision(value: &c::ProposalDecision) -> w::ProposalDecision {
    w::ProposalDecision {
        project: value.project,
        proposer: wire_proposer(&value.proposer),
        proposal: value.proposal,
        kind: wire_message_proposal_kind(&value.kind),
        by: value.by,
        at: value.at.as_nanos(),
        choice: wire_proposal_choice(&value.choice),
        created: value.created.as_ref().map(|item| w::ProposalDecisionCreated { first: item.0, last: item.1 }),
    }
}

pub(in crate::codec) fn wire_escalation_decision_record(
    value: &c::EscalationDecisionRecord,
) -> w::EscalationDecisionRecord {
    w::EscalationDecisionRecord {
        project: value.project,
        requester: value.requester,
        task: value.task,
        revision: value.revision,
        by: value.by,
        at: value.at.as_nanos(),
        decision: wire_escalation_decision(&value.decision),
    }
}

pub(in crate::codec) fn wire_escalation_decision(value: &c::EscalationDecision) -> w::EscalationDecision {
    match value {
        c::EscalationDecision::Release => w::EscalationDecision::Release,
        c::EscalationDecision::Reject { reason } => {
            w::EscalationDecision::Reject(w::EscalationDecisionReject { reason: bytes::copy_of((reason).bytes()) })
        }
        c::EscalationDecision::Pass => w::EscalationDecision::Pass,
    }
}
