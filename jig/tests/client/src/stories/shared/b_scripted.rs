//! Seeded project responses use the contract carried by the request (jig/docs/design/client/domain/testing.md, 4).
use crate::Scripted;
use jig_client_domain::contract as c;
use jig_client_scripted_engine::{Change, Rule};
pub fn propose(front: &mut Scripted, party: u64, ask: &c::Ask) {
    let c::Ask::Propose { project, action: c::ProposeAction::Batch { tasks }, reason } = ask else { return };
    let proposed = tasks.iter().enumerate().map(|(index, task)| {
        assert!(task.spec.parameters.is_empty() && task.spec.inputs.is_empty(), "plain goal specifications");
        let c::NewContract::Report { words } = task.contract else { panic!("goal report contract") };
        c::ProposedTask {
            number: u64::try_from(index).expect("batch fits") + 1,
            deadline: task.deadline,
            spec: c::SpecificationDocument {
                specification: c::DocumentSpecification {
                    words: c::DocumentText::complete(task.spec.words.clone()),
                    parameters: crate::test::list([]),
                    inputs: crate::test::list([]),
                    asks: task.spec.asks,
                },
                cut: false,
            },
            executor: task.executor,
            contract: c::TaskContractDocument::Report { words },
            authority: c::OwnerAuthorityDocument { authority: task.authority.clone(), cut: false },
            budget: task.authority.spend,
            funder: c::Funder::Pool { project: *project, person: party, period: 1 },
            dependencies: crate::test::list([]),
            resources: crate::test::list([]),
            wake: crate::stories::task().wake,
            priority: task.tracked,
            cut: false,
        }
    });
    let mut proposal = crate::stories::proposal();
    proposal.project = *project;
    proposal.key = c::ProposalKey { proposer: c::Proposer::Party(party), number: 1 };
    proposal.reason = c::DocumentText::complete(reason.clone());
    proposal.action = c::OwnerProposalAction::Batch(crate::test::list(proposed));
    proposal.waiting.holder =
        c::OwnerHistoryHolder::Policy { project: *project, kind: c::OwnerMessageProposalKind::Batch };
    proposal.funding.clear();
    proposal.acts.clear();
    front
        .engine
        .script(Rule {
            party: Some(party),
            ask: ask.clone(),
            answer: c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Proposed { proposal: 1, to: 1 })),
            changes: vec![Change::Object {
                project: *project,
                key: c::ObjectKey::Proposal(proposal.key),
                value: c::ReadValue::OwnerProposal(Box::new(proposal)),
            }],
        })
        .expect("seeded project proposal fits");
}
pub fn cancel(front: &mut Scripted, party: u64, wall: skein_lib::Wall, ask: &c::Ask) {
    let operations = if let c::Ask::Cancel { project, task, reason, grace } = ask {
        crate::test::list([c::Operation::Cancel {
            project: *project,
            task: *task,
            reason: reason.clone(),
            grace: *grace,
        }])
    } else if let c::Ask::Compound { operations, .. } = ask {
        if !operations
            .iter()
            .all(|operation| matches!(operation, c::Operation::Cancel { .. } | c::Operation::Move { .. }))
        {
            return;
        }
        operations.clone()
    } else {
        return;
    };
    let mut changes = Vec::new();
    for operation in &operations {
        let (project, number) = match operation {
            c::Operation::Cancel { project, task, .. } | c::Operation::Move { project, task, .. } => (*project, *task),
            c::Operation::Message { .. }
            | c::Operation::Amend { .. }
            | c::Operation::Release { .. }
            | c::Operation::Stop { .. } => unreachable!("cancellation operations"),
        };
        let Some(c::ReadValue::Task(value)) = front.engine.truth().objects.get(&c::ObjectKey::Task(number)) else {
            return;
        };
        let mut value = *value.clone();
        match operation {
            c::Operation::Cancel { reason, .. } => {
                let ending = c::OwnerChipEnding::Cancelled {
                    reason: c::DocumentText::complete(reason.clone()),
                    result: None,
                    by: c::OwnerRequester::Person(party),
                    at: wall,
                };
                value.result = Some(c::TaskResult::Ending(Box::new(ending.clone())));
                value.chip.phase = c::Phase::Ended;
                value.chip.details.phase = c::OwnerChipPhase::Ended;
                value.chip.details.details.ending = Some(ending);
                value.chip.details.details.ended_at = Some(wall);
                value.delegates.clear();
                value.chip.details.details.live_delegates = 0;
                value.acts.clear();
            }
            c::Operation::Move { .. } => value.requester = c::OwnerRequester::Person(party),
            c::Operation::Message { .. }
            | c::Operation::Amend { .. }
            | c::Operation::Release { .. }
            | c::Operation::Stop { .. } => unreachable!("cancellation operations"),
        }
        value.changed = wall;
        value.last_activity = wall;
        changes.push(Change::Object {
            project,
            key: c::ObjectKey::Task(number),
            value: c::ReadValue::Task(Box::new(value)),
        });
    }
    let outcome = if let c::Ask::Cancel { task, .. } = ask {
        c::OwnerOutcome::Cancelled { task: *task }
    } else if let c::Ask::Compound { project, .. } = ask {
        c::OwnerOutcome::Compound { project: *project, operations: operations.len(), first_message: 0 }
    } else {
        unreachable!("cancellation requests")
    };
    front
        .engine
        .script(Rule {
            party: Some(party),
            ask: ask.clone(),
            answer: c::Answer::Done(c::Outcome::Owner(outcome)),
            changes,
        })
        .expect("seeded conversation cancellation fits");
}
