use crate::Bounds;
use jig_client_domain::contract::*;
use skein_lib::{List, Wall};

fn goal() -> PersonProposalGoal {
    let mut parameters = List::with_capacity(3);
    parameters.push(Parameter::Number { name: u32::MAX, value: u64::MAX }).expect("number fits");
    parameters.push(Parameter::Bytes { name: 0, value: super::text(UTF8_TEXTS[0]) }).expect("bytes fit");
    parameters.push(Parameter::Resource { name: 7, connector: u16::MAX, resource: u64::MAX }).expect("resource fits");
    let mut inputs = List::with_capacity(3);
    inputs.push(u64::MAX).expect("input fits");
    let mut holdings = List::with_capacity(3);
    for write in [true, false] {
        let mut path = List::with_capacity(3);
        path.push(Bytes::new(Box::from(b"a/b".as_slice()), 4096).unwrap()).expect("literal segment fits");
        path.push(Bytes::new(Box::from([255, 0]), 4096).unwrap()).expect("opaque segment fits");
        let holding = if write {
            GoalHolding::Write { connector: u16::MAX, path, kind: u16::MAX }
        } else {
            GoalHolding::Slot { connector: u16::MAX, path, kind: 0 }
        };
        holdings.push(holding).expect("holding fits");
    }
    PersonProposalGoal {
        number: u64::MAX,
        project: u32::MAX,
        executor: Executor::Agent { charter: u32::MAX },
        spec: Specification { asks: false, words: super::text(UTF8_TEXTS[0]), parameters, inputs },
        contract: TaskContractDocument::Report { words: u32::MAX },
        authority: super::authority().authority,
        numbers: Numbers { budget: 1, spent: 7, spent_below: 11, reserved: u64::MAX },
        funder: Funder::Pool { project: u32::MAX, person: u64::MAX, period: 0 },
        dependencies: List::with_capacity(3),
        holdings,
        wake: super::task().wake,
        tracked: Some(u32::MAX),
    }
}
fn person(state: PersonProposalState) -> PersonProposal {
    PersonProposal { number: u64::MAX, proposer: u64::MAX, project: u32::MAX, goal: goal(), state }
}
fn record(choice: ProposalChoice, created: Option<(u64, u64)>) -> ProposalDecision {
    ProposalDecision {
        project: u32::MAX,
        proposer: Proposer::Party(u64::MAX),
        proposal: u64::MAX,
        kind: MessageProposalKind::Batch,
        by: u64::MAX,
        at: Wall::EPOCH,
        choice,
        created,
    }
}
fn result() -> Result {
    Result {
        task: super::task().chip,
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText::complete(super::text(b"independent")))),
        at: None,
        details: None,
        acts: List::with_capacity(MAX_OFFERED_ACTS),
    }
}
fn question() -> Question {
    Question {
        task: super::task().chip,
        from: Some(Requester::Deployment { project: u32::MAX }),
        message: u64::MAX,
        words: DocumentText::complete(super::text(UTF8_TEXTS[0])),
        since: Wall::EPOCH,
        acts: List::with_capacity(0),
        ending: None,
    }
}
fn answer() -> ActivityLine {
    ActivityLine {
        task: 8,
        position: u64::MAX,
        at: Wall::from_nanos(u64::MAX),
        line: Activity::Words {
            message: 0,
            by: Requester::Task(u64::MAX),
            kind: MessageKind::Answer { question: u64::MAX },
            words: super::text(UTF8_TEXTS[1]),
        },
    }
}
fn envelope(key: ObjectKey, value: EndedValue) -> Ended {
    Ended { key, value, at: None, by: None }
}
fn person_value(state: PersonProposalState, decision: Option<ProposalDecision>) -> Ended {
    envelope(
        ObjectKey::Proposal(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }),
        EndedValue::PersonProposal { proposal: Box::new(person(state)), decision },
    )
}
fn bounds() -> Bounds {
    Bounds { rows: 3, segments: 3, text: 32, document_bytes: 1_000_000, ..Bounds::default() }
}
#[test]
fn ended_person_goal_keeps_raw_project_all_four_numbers_parameters_and_distinct_literal_holdings() {
    // These full-width receiving facts are intentionally separate from the
    // authentic producer's initial spent/reserved/dependency admission rules.
    let value = person_value(PersonProposalState::Pending { since: Wall::EPOCH }, None);
    let EndedValue::PersonProposal { proposal, .. } = &value.value else { panic!("person source") };
    assert_eq!(proposal.goal.project, u32::MAX);
    assert_eq!(proposal.goal.number, u64::MAX);
    assert_eq!(proposal.goal.numbers, Numbers { budget: 1, spent: 7, spent_below: 11, reserved: u64::MAX });
    assert_eq!(proposal.goal.spec.words.bytes(), UTF8_TEXTS[0]);
    assert_eq!(proposal.goal.spec.parameters.capacity(), 3);
    assert_eq!(proposal.goal.spec.parameters.get(0), Some(&Parameter::Number { name: u32::MAX, value: u64::MAX }));
    assert_eq!(
        proposal.goal.spec.parameters.get(1),
        Some(&Parameter::Bytes { name: 0, value: super::text(UTF8_TEXTS[0]) })
    );
    assert_eq!(
        proposal.goal.spec.parameters.get(2),
        Some(&Parameter::Resource { name: 7, connector: u16::MAX, resource: u64::MAX })
    );
    assert_eq!(proposal.goal.spec.inputs.as_slice(), &[u64::MAX]);
    assert_eq!(proposal.goal.dependencies.capacity(), 3);
    assert!(proposal.goal.dependencies.is_empty());
    let GoalHolding::Write { connector, path, kind } = proposal.goal.holdings.get(0).expect("write") else {
        panic!("write kind preserved")
    };
    assert_eq!((*connector, *kind), (u16::MAX, u16::MAX));
    assert_eq!(path.capacity(), 3);
    assert_eq!(path.get(0).expect("literal").bytes(), b"a/b");
    assert_eq!(path.get(1).expect("opaque").bytes(), &[255, 0]);
    let GoalHolding::Slot { connector, path, kind } = proposal.goal.holdings.get(1).expect("slot") else {
        panic!("slot kind preserved")
    };
    assert_eq!((*connector, *kind), (u16::MAX, 0));
    assert_eq!(path.capacity(), 3);
    assert_eq!(path.get(0).expect("literal").bytes(), b"a/b");
    assert!(value.matches(value.key));
    assert!(!value.matches(ObjectKey::Proposal(ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX })));
}
#[test]
fn ended_person_states_keep_independent_decisions_choices_created_pairs_and_outer_absence() {
    for choice in [
        ProposalChoice::Accepted,
        ProposalChoice::Rejected,
        ProposalChoice::Passed,
        ProposalChoice::Withdrawn,
        ProposalChoice::Stale,
    ] {
        for created in [None, Some((0, 0)), Some((0, u64::MAX)), Some((u64::MAX, u64::MAX))] {
            let decision = record(choice, created);
            let states = [
                PersonProposalState::Pending { since: Wall::EPOCH },
                PersonProposalState::Accepted { by: Requester::Person(u64::MAX) },
                PersonProposalState::Rejected {
                    by: Requester::Person(1),
                    reason: super::text(UTF8_TEXTS[1]),
                    message: u64::MAX,
                    at: Wall::EPOCH,
                },
            ];
            for state in states {
                let pending = matches!(state, PersonProposalState::Pending { .. });
                let original = person_value(state, Some(decision));
                for at in [None, Some(Wall::EPOCH), Some(Wall::from_nanos(u64::MAX))] {
                    for by in [
                        None,
                        Some(Requester::Task(0)),
                        Some(Requester::Person(u64::MAX)),
                        Some(Requester::Deployment { project: u32::MAX }),
                    ] {
                        let value = Ended { at, by, ..original.clone() };
                        assert!(value.matches(value.key));
                        assert_eq!(value.is_terminal(), !pending, "decision presence does not replace supplied state");
                        let EndedValue::PersonProposal { decision: stored, .. } = &value.value else {
                            panic!("person source")
                        };
                        assert_eq!(*stored, Some(decision));
                        assert_eq!((value.at, value.by), (at, by));
                    }
                }
            }
        }
    }
    let accepted = person_value(PersonProposalState::Accepted { by: Requester::Person(1) }, None);
    assert!(!accepted.matches(accepted.key), "authentic accepted source without decision returns Gone");
    let mut wrong =
        person_value(PersonProposalState::Pending { since: Wall::EPOCH }, Some(record(ProposalChoice::Passed, None)));
    let EndedValue::PersonProposal { decision: Some(decision), .. } = &mut wrong.value else { panic!("record") };
    decision.proposer = Proposer::Task(u64::MAX);
    assert!(!wrong.matches(wrong.key));
}

#[test]
fn ended_question_answer_and_full_ending_presence_are_independent_from_outer_sender_and_clock() {
    for answer_present in [false, true] {
        for ending_present in [false, true] {
            let value = envelope(
                ObjectKey::Question { task: 8, message: u64::MAX },
                EndedValue::Question {
                    question: question(),
                    answer: answer_present.then(answer),
                    ending: ending_present.then(|| Box::new(result())),
                },
            );
            assert!(value.matches(value.key));
            assert_eq!(value.is_terminal(), answer_present || ending_present);
            assert_eq!((value.at, value.by), (None, None));
        }
    }
    let mut value = envelope(
        ObjectKey::Question { task: 8, message: u64::MAX },
        EndedValue::Question { question: question(), answer: Some(answer()), ending: None },
    );
    let EndedValue::Question { answer: Some(answer), .. } = &mut value.value else { panic!("answer") };
    answer.line = Activity::Words {
        message: 0,
        by: Requester::Person(1),
        kind: MessageKind::Answer { question: 7 },
        words: super::text(b""),
    };
    assert!(!value.matches(value.key));
}

#[test]
fn ended_task_proposal_states_preserve_raw_maker_and_do_not_use_outer_context_as_terminal_evidence() {
    let states = [
        ProposalHistoryState::Pending { holder: HistoryHolder::Task(u64::MAX), since: Wall::EPOCH },
        ProposalHistoryState::Accepted { by: Requester::Deployment { project: u32::MAX } },
        ProposalHistoryState::Rejected {
            by: Requester::Task(0),
            reason: DocumentText { bytes: super::text(UTF8_TEXTS[1]), cut: true },
        },
        ProposalHistoryState::Withdrawn,
    ];
    for state in states {
        let pending = matches!(state, ProposalHistoryState::Pending { .. });
        let value = Ended {
            key: ObjectKey::Proposal(ProposalKey { proposer: Proposer::Task(u64::MAX), number: 0 }),
            value: EndedValue::Proposal(ProposalHistory {
                action: RetainedAction::Release { task: 0 },
                number: 0,
                proposer: u64::MAX,
                as_holder: false,
                state,
            }),
            at: Some(Wall::EPOCH),
            by: Some(Requester::Person(u64::MAX)),
        };
        assert!(value.matches(value.key));
        assert_eq!(value.is_terminal(), !pending);
        assert!(!value.matches(ObjectKey::Proposal(ProposalKey { proposer: Proposer::Party(u64::MAX), number: 0 })));
        assert!(crate::codec::read(&ReadValue::Ended(Box::new(value.clone())), &bounds()).is_ok());
    }
}

#[test]
fn ended_escalation_decisions_keep_opaque_rejection_and_independent_optional_hold() {
    for decision in [
        EscalationDecision::Release,
        EscalationDecision::Pass,
        EscalationDecision::Reject { reason: super::text(b"") },
        EscalationDecision::Reject { reason: super::text(UTF8_TEXTS[0]) },
    ] {
        for hold in [None, Some(MilestoneLine { task: 8, position: 0, phase: MilestonePhase::Waiting, at: None })] {
            let record = EscalationDecisionRecord {
                project: 1,
                requester: u64::MAX,
                task: 8,
                revision: u64::MAX,
                by: 0,
                at: Wall::from_nanos(u64::MAX),
                decision: decision.clone(),
            };
            let value = Ended {
                key: ObjectKey::Escalation(8),
                value: EndedValue::Escalation {
                    task: Box::new(super::task()),
                    decision: record.clone(),
                    hold: hold.clone(),
                },
                at: None,
                by: None,
            };
            assert!(value.matches(value.key));
            assert!(value.is_terminal());
            assert_eq!((value.at, value.by), (None, None));
            let EndedValue::Escalation { decision: stored, hold: stored_hold, .. } = &value.value else {
                panic!("escalation")
            };
            assert_eq!(stored, &record);
            assert_eq!(stored_hold, &hold);
            assert!(crate::codec::read(&ReadValue::Ended(Box::new(value.clone())), &bounds()).is_ok());
            let mut wrong = value.clone();
            let EndedValue::Escalation { decision, .. } = &mut wrong.value else { panic!("decision") };
            decision.task = 9;
            assert!(!wrong.matches(wrong.key));
        }
    }
}

const UTF8_TEXTS: [&[u8]; 3] = [b"~\0/", b"~\0", &[b'~'; 32]];
