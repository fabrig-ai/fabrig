//! Pending owner decisions and retained winners (protocol/client.md, 5).
use jig_client_domain::contract::{
    Ask, ChipPhase, DecideOn, Decision, DocumentText, ObjectKey, OwnerChipPhase, OwnerEscalationChoice,
    OwnerEscalationDecision, OwnerEscalationDecisionRecord, OwnerHistorical, OwnerHistoricalValue, OwnerOutcome,
    OwnerProposalAction, OwnerProposalChoice, OwnerProposalHistory, OwnerProposalHistoryState, OwnerRequester,
    OwnerRetainedAction, OwnerRetainedActionSettings, Phase, Proposer, ReadValue, RunStage, Runtime,
};
use skein_lib::Wall;

pub(super) fn repeated(value: &ReadValue, ask: &Ask) -> Option<OwnerOutcome> {
    let ReadValue::Historical(value) = value else { return None };
    match (&value.value, ask) {
        (
            OwnerHistoricalValue::Proposal(proposal),
            Ask::Decide { on: DecideOn::Proposal { proposal: key, .. }, .. },
        ) => {
            let choice = match &proposal.state {
                OwnerProposalHistoryState::Accepted { .. } => OwnerProposalChoice::Accepted,
                OwnerProposalHistoryState::Rejected { .. } => OwnerProposalChoice::Rejected,
                OwnerProposalHistoryState::Withdrawn => OwnerProposalChoice::Withdrawn,
                OwnerProposalHistoryState::Pending { .. } => return None,
            };
            let OwnerRequester::Person(by) = value.by? else { return None };
            Some(OwnerOutcome::ProposalDecided { proposal: *key, by, at: value.at?, choice, repeated: true })
        }
        (
            OwnerHistoricalValue::Escalation { decision, .. },
            Ask::Decide { on: DecideOn::Escalation { task, revision, .. }, .. },
        ) if *task == decision.task && revision.value == decision.revision => Some(OwnerOutcome::EscalationDecided {
            task: *task,
            revision: *revision,
            by: decision.by,
            at: decision.at,
            choice: match decision.decision {
                OwnerEscalationDecision::Release => OwnerEscalationChoice::Released,
                OwnerEscalationDecision::Reject { .. } => OwnerEscalationChoice::Rejected,
                OwnerEscalationDecision::Pass => OwnerEscalationChoice::Passed,
            },
        }),
        _ => None,
    }
}

pub(super) fn decided(
    value: ReadValue,
    ask: &Ask,
    by: u64,
    at: Wall,
    how: &Decision,
) -> Option<(ReadValue, OwnerOutcome)> {
    match value {
        ReadValue::OwnerProposal(value) => proposal_decided(*value, by, at, how),
        ReadValue::OwnerEscalation(value) => escalation_decided(*value, ask, by, at, how),
        ReadValue::Task(value) => task_released(value, ask),
        ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::OwnerProject(_)
        | ReadValue::Historical(_)
        | ReadValue::Me(_)
        | ReadValue::Turn(_)
        | ReadValue::Question(_)
        | ReadValue::OwnerPersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::OwnerEnded(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::OwnerInbox(_)
        | ReadValue::OwnerSpend(_)
        | ReadValue::Notes(_)
        | ReadValue::OwnerDeployment(_) => None,
    }
}

fn retained(action: OwnerProposalAction) -> Option<OwnerRetainedAction> {
    Some(match action {
        OwnerProposalAction::Batch(tasks) => OwnerRetainedAction::Batch(tasks),
        OwnerProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            OwnerRetainedAction::Effect { part, authority, attempt, completion, position, purpose }
        }
        OwnerProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            OwnerRetainedAction::ProcedureEffect { part, authority, step, purpose }
        }
        OwnerProposalAction::Release { task, .. } => OwnerRetainedAction::Release { task: task.number() },
        OwnerProposalAction::Widen { task, after, .. } => {
            OwnerRetainedAction::Widen { task: task.number(), authority: after }
        }
        OwnerProposalAction::Settings(value) => OwnerRetainedAction::Settings(OwnerRetainedActionSettings {
            against: value.against,
            items: value.items,
            made: value.made,
        }),
        OwnerProposalAction::Amend { .. } => return None,
    })
}

fn proposal_decided(
    value: jig_client_domain::contract::OwnerProposal,
    by: u64,
    at: Wall,
    how: &Decision,
) -> Option<(ReadValue, OwnerOutcome)> {
    let choice = match how {
        Decision::Accepted => OwnerProposalChoice::Accepted,
        Decision::Rejected { .. } => OwnerProposalChoice::Rejected,
        Decision::PassedUp | Decision::Released { .. } | Decision::LeftHeld { .. } => return None,
    };
    let state = match how {
        Decision::Accepted => OwnerProposalHistoryState::Accepted { by: OwnerRequester::Person(by) },
        Decision::Rejected { reason } => OwnerProposalHistoryState::Rejected {
            by: OwnerRequester::Person(by),
            reason: DocumentText::complete(reason.clone()),
        },
        Decision::PassedUp | Decision::Released { .. } | Decision::LeftHeld { .. } => return None,
    };
    let action = retained(value.action)?;
    let proposer = match value.key.proposer {
        Proposer::Task(task) | Proposer::Party(task) => task,
    };
    let outcome = OwnerOutcome::ProposalDecided { proposal: value.key, by, at, choice, repeated: false };
    Some((
        ReadValue::Historical(Box::new(OwnerHistorical {
            key: ObjectKey::Proposal(value.key),
            value: OwnerHistoricalValue::Proposal(OwnerProposalHistory {
                action,
                number: value.key.number,
                proposer,
                as_holder: value.as_holder,
                state,
            }),
            at: Some(at),
            by: Some(OwnerRequester::Person(by)),
        })),
        outcome,
    ))
}

fn escalation_decided(
    value: jig_client_domain::contract::OwnerEscalation,
    ask: &Ask,
    by: u64,
    at: Wall,
    how: &Decision,
) -> Option<(ReadValue, OwnerOutcome)> {
    let Ask::Decide { on: DecideOn::Escalation { task, revision, .. }, project } = ask else { return None };
    let (decision, choice) = match how {
        Decision::Accepted | Decision::Released { .. } => {
            (OwnerEscalationDecision::Release, OwnerEscalationChoice::Released)
        }
        Decision::Rejected { reason } => {
            (OwnerEscalationDecision::Reject { reason: reason.clone() }, OwnerEscalationChoice::Rejected)
        }
        Decision::PassedUp | Decision::LeftHeld { .. } => return None,
    };
    let record = OwnerEscalationDecisionRecord {
        project: *project,
        requester: by,
        task: *task,
        revision: revision.value,
        by,
        at,
        decision,
    };
    Some((
        ReadValue::Historical(Box::new(OwnerHistorical {
            key: ObjectKey::Escalation(*task),
            value: OwnerHistoricalValue::Escalation { task: value.task, decision: record, hold: None },
            at: Some(at),
            by: Some(OwnerRequester::Person(by)),
        })),
        OwnerOutcome::EscalationDecided { task: *task, revision: *revision, by, at, choice },
    ))
}

fn task_released(mut value: Box<jig_client_domain::contract::Task>, ask: &Ask) -> Option<(ReadValue, OwnerOutcome)> {
    let Ask::Release { task, .. } = ask else { return None };
    value.hold = None;
    if let Some(context) = &mut value.context {
        context.runtime = Runtime::Starting;
    }
    value.chip.phase = Phase::Due;
    let ChipPhase { phase, details } = value.chip.details.as_mut();
    *phase = OwnerChipPhase::Active(RunStage::Due);
    details.hold = None;
    Some((ReadValue::Task(value), OwnerOutcome::Released { task: *task }))
}
