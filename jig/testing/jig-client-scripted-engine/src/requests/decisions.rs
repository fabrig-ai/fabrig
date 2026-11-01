//! Pending owner decisions and retained winners (protocol/client.md, 5).
use jig_client_domain::contract::{
    Ask, ChipPhase, ChipPhaseFacts, DecideOn, Decision, DocumentOutcome, DocumentText, Ended, EndedValue,
    EscalationChoice, EscalationDecision, EscalationDecisionRecord, ObjectKey, Phase, ProposalAction, ProposalChoice,
    ProposalHistory, ProposalHistoryState, Proposer, ReadValue, Requester, RetainedAction, RetainedActionSettings,
    RunStage, Runtime,
};
use skein_lib::Wall;

pub(super) fn repeated(value: &ReadValue, ask: &Ask) -> Option<DocumentOutcome> {
    let ReadValue::Ended(value) = value else { return None };
    match (&value.value, ask) {
        (EndedValue::Proposal(proposal), Ask::Decide { on: DecideOn::Proposal { proposal: key, .. }, .. }) => {
            let choice = match &proposal.state {
                ProposalHistoryState::Accepted { .. } => ProposalChoice::Accepted,
                ProposalHistoryState::Rejected { .. } => ProposalChoice::Rejected,
                ProposalHistoryState::Withdrawn => ProposalChoice::Withdrawn,
                ProposalHistoryState::Pending { .. } => return None,
            };
            let Requester::Person(by) = value.by? else { return None };
            Some(DocumentOutcome::ProposalDecided { proposal: *key, by, at: value.at?, choice, repeated: true })
        }
        (
            EndedValue::Escalation { decision, .. },
            Ask::Decide { on: DecideOn::Escalation { task, revision, .. }, .. },
        ) if *task == decision.task && revision.value == decision.revision => {
            Some(DocumentOutcome::EscalationDecided {
                task: *task,
                revision: *revision,
                by: decision.by,
                at: decision.at,
                choice: match decision.decision {
                    EscalationDecision::Release => EscalationChoice::Released,
                    EscalationDecision::Reject { .. } => EscalationChoice::Rejected,
                    EscalationDecision::Pass => EscalationChoice::Passed,
                },
            })
        }
        _ => None,
    }
}

pub(super) fn decided(
    value: ReadValue,
    ask: &Ask,
    by: u64,
    at: Wall,
    how: &Decision,
) -> Option<(ReadValue, DocumentOutcome)> {
    match value {
        ReadValue::Proposal(value) => proposal_decided(*value, by, at, how),
        ReadValue::Escalation(value) => escalation_decided(*value, ask, by, at, how),
        ReadValue::Task(value) => task_released(value, ask),
        ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Ended(_)
        | ReadValue::Me(_)
        | ReadValue::Turn(_)
        | ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}

fn retained(action: ProposalAction) -> Option<RetainedAction> {
    Some(match action {
        ProposalAction::Batch(tasks) => RetainedAction::Batch(tasks),
        ProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            RetainedAction::Effect { part, authority, attempt, completion, position, purpose }
        }
        ProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            RetainedAction::ProcedureEffect { part, authority, step, purpose }
        }
        ProposalAction::Release { task, .. } => RetainedAction::Release { task: task.number() },
        ProposalAction::Widen { task, after, .. } => RetainedAction::Widen { task: task.number(), authority: after },
        ProposalAction::Settings(value) => RetainedAction::Settings(RetainedActionSettings {
            against: value.against,
            items: value.items,
            made: value.made,
        }),
        ProposalAction::Amend { .. } => return None,
    })
}

fn proposal_decided(
    value: jig_client_domain::contract::Proposal,
    by: u64,
    at: Wall,
    how: &Decision,
) -> Option<(ReadValue, DocumentOutcome)> {
    let choice = match how {
        Decision::Accepted => ProposalChoice::Accepted,
        Decision::Rejected { .. } => ProposalChoice::Rejected,
        Decision::PassedUp | Decision::Released { .. } | Decision::LeftHeld { .. } => return None,
    };
    let state = match how {
        Decision::Accepted => ProposalHistoryState::Accepted { by: Requester::Person(by) },
        Decision::Rejected { reason } => {
            ProposalHistoryState::Rejected { by: Requester::Person(by), reason: DocumentText::complete(reason.clone()) }
        }
        Decision::PassedUp | Decision::Released { .. } | Decision::LeftHeld { .. } => return None,
    };
    let action = retained(value.action)?;
    let proposer = match value.key.proposer {
        Proposer::Task(task) | Proposer::Party(task) => task,
    };
    let outcome = DocumentOutcome::ProposalDecided { proposal: value.key, by, at, choice, repeated: false };
    Some((
        ReadValue::Ended(Box::new(Ended {
            key: ObjectKey::Proposal(value.key),
            value: EndedValue::Proposal(ProposalHistory {
                action,
                number: value.key.number,
                proposer,
                as_holder: value.as_holder,
                state,
            }),
            at: Some(at),
            by: Some(Requester::Person(by)),
        })),
        outcome,
    ))
}

fn escalation_decided(
    value: jig_client_domain::contract::Escalation,
    ask: &Ask,
    by: u64,
    at: Wall,
    how: &Decision,
) -> Option<(ReadValue, DocumentOutcome)> {
    let Ask::Decide { on: DecideOn::Escalation { task, revision, .. }, project } = ask else { return None };
    let (decision, choice) = match how {
        Decision::Accepted | Decision::Released { .. } => (EscalationDecision::Release, EscalationChoice::Released),
        Decision::Rejected { reason } => {
            (EscalationDecision::Reject { reason: reason.clone() }, EscalationChoice::Rejected)
        }
        Decision::PassedUp | Decision::LeftHeld { .. } => return None,
    };
    let record = EscalationDecisionRecord {
        project: *project,
        requester: by,
        task: *task,
        revision: revision.value,
        by,
        at,
        decision,
    };
    Some((
        ReadValue::Ended(Box::new(Ended {
            key: ObjectKey::Escalation(*task),
            value: EndedValue::Escalation { task: value.task, decision: record, hold: None },
            at: Some(at),
            by: Some(Requester::Person(by)),
        })),
        DocumentOutcome::EscalationDecided { task: *task, revision: *revision, by, at, choice },
    ))
}

fn task_released(mut value: Box<jig_client_domain::contract::Task>, ask: &Ask) -> Option<(ReadValue, DocumentOutcome)> {
    let Ask::Release { task, .. } = ask else { return None };
    value.hold = None;
    if let Some(context) = &mut value.context {
        context.runtime = Runtime::Starting;
    }
    value.chip.phase = Phase::Due;
    let ChipPhaseFacts { phase, details } = value.chip.details.as_mut();
    *phase = ChipPhase::Active(RunStage::Due);
    details.hold = None;
    Some((ReadValue::Task(value), DocumentOutcome::Released { task: *task }))
}
