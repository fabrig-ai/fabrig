//! Whole-task erasure plans checked against durable rows and core writes.
use super::*;
use crate::erasure::{MOST_KEYS, proposal_keys, record_keys, row_keys};

fn ended(requester: tasks::Party) -> tasks::TaskRecord {
    let mut row = fixtures::ended_report(1, requester);
    row.label = 3;
    row.flags.listed = true;
    row
}

#[test]
fn a_persons_ended_conversation_plans_every_index_row() {
    let row = ended(tasks::Party::Person(2));
    let mut keys = List::with_capacity(MOST_KEYS);
    assert_eq!(record_keys(&row, &mut keys), Some(()));
    let at = row.ended_at.expect("ended time");
    assert_eq!(
        keys.as_slice(),
        &[
            Key::Tasks(tasks::Key::Result { party: 2, position: 7 }),
            Key::Tasks(tasks::Update { party: 2, at, task: 1, object: tasks::UpdateObject::Result }.key()),
            Key::Tasks(tasks::EndedByParty { party: 2, label: 3, ended_at: at, task: 1 }.key()),
            Key::Tasks(tasks::EndedByProject { project: row.project, label: 3, ended_at: at, task: 1 }.key()),
            Key::Tasks(tasks::Key::Ended(1)),
        ]
    );
}

#[test]
fn an_unlisted_task_plans_no_project_index() {
    let mut row = ended(tasks::Party::Person(2));
    row.flags.listed = false;
    let mut keys = List::with_capacity(MOST_KEYS);
    assert_eq!(record_keys(&row, &mut keys), Some(()));
    assert_eq!(keys.len(), 4);
    let index = tasks::EndedByProject {
        project: row.project,
        label: row.label,
        ended_at: row.ended_at.expect("ended time"),
        task: row.number,
    };
    assert!(!keys.as_slice().contains(&Key::Tasks(index.key())));
}

#[test]
fn a_delegate_plans_its_requesters_edges() {
    let mut row = ended(tasks::Party::Task(2));
    row.flags.listed = false;
    let mut keys = List::with_capacity(MOST_KEYS);
    assert_eq!(record_keys(&row, &mut keys), Some(()));
    assert_eq!(
        keys.as_slice(),
        &[
            Key::Tasks(tasks::Key::EndedDelegate { requester: 2, task: 1 }),
            Key::Tasks(tasks::Key::Delegate { requester: 2, task: 1 }),
            Key::Tasks(tasks::Key::Ended(1)),
        ]
    );
}

fn reply() -> tasks::ActivityLine {
    tasks::ActivityLine {
        task: 1,
        position: 3,
        at: Wall::from_nanos(42),
        line: tasks::Line::Turn { reply: 5, attempt: 7, turn: 1, read: None },
    }
}

#[test]
fn a_rows_own_key_is_planned() {
    let at = Wall::EPOCH;
    let rows = [
        Record::Tasks(tasks::Stored::Stub(tasks::Stub { task: 1, phase: tasks::Status::Done, result: Token::new(1) })),
        Record::Tasks(tasks::Stored::History(tasks::History {
            task: 1,
            revision: 2,
            by: tasks::Party::Person(2),
            change: tasks::Change::Amended,
            reason: Box::new([]),
            proposal: None,
            at,
            amendment: None,
        })),
        Record::Tasks(tasks::Stored::Milestone(tasks::Milestone {
            task: 1,
            position: 2,
            phase: tasks::Phase::Waiting,
            at,
        })),
        Record::Tasks(tasks::Stored::Activity(reply())),
        Record::Tasks(tasks::Stored::EndedDelegate(tasks::Edge { requester: 2, task: 1 })),
        Record::Tasks(tasks::Stored::Delegate(tasks::Edge { requester: 2, task: 1 })),
        Record::Core(CoreRecord::RunProof(RunProof::claimed(1, 7, 7, None, fleet::HostKind::Worker))),
        Record::Core(CoreRecord::Call(CallRecord {
            key: CallKey { task: 1, attempt: 7, completion: 1, position: 0 },
            part: CallPart::Revised { task: 1, revision: 2 },
            settled: None,
        })),
        Record::Core(CoreRecord::Terminal(TerminalRecord {
            task: 1,
            attempt: 7,
            cumulative: 0,
            end: tasks::End::Parked,
        })),
        Record::Core(CoreRecord::EscalationDecision(EscalationDecisionRecord {
            project: 1,
            requester: 2,
            task: 1,
            revision: 2,
            by: 2,
            at,
            decision: people::EscalationDecision::Release,
        })),
        Record::People(people::Stored::EndedRead { task: 1, party: 2, through: 5 }),
    ];
    for row in rows {
        let mut keys = List::with_capacity(MOST_KEYS);
        assert_eq!(row_keys(&row, None, &mut keys), Some(()));
        assert_eq!(keys.as_slice(), &[row.key()]);
    }
}

fn saved_update(core: &Core, row: &tasks::Stored, at: Wall) -> Key {
    let mut saved = Queue::uncapped();
    indexes::save(core, row, at, &mut saved);
    let Request::Write(Write::Save(Record::Tasks(tasks::Stored::Update(update)))) = saved.pop().expect("saved update")
    else {
        panic!("expected update");
    };
    assert!(saved.is_empty());
    Key::Tasks(update.key())
}

#[test]
fn an_activity_reply_plans_its_update() {
    let (mut core, limits) = tree_core();
    make_task(&mut core, &limits, 1);
    let stored = tasks::Stored::Activity(reply());
    let update = saved_update(&core, &stored, Wall::EPOCH);
    let mut keys = List::with_capacity(MOST_KEYS);
    let row = Record::Tasks(stored);
    assert_eq!(row_keys(&row, Some(tasks::Party::Person(1)), &mut keys), Some(()));
    assert_eq!(keys.as_slice(), &[row.key(), update]);
}

#[test]
fn a_reply_of_a_task_requested_by_a_task_plans_no_update() {
    let row = Record::Tasks(tasks::Stored::Activity(reply()));
    for requester in [Some(tasks::Party::Task(2)), None] {
        let mut keys = List::with_capacity(MOST_KEYS);
        assert_eq!(row_keys(&row, requester, &mut keys), Some(()));
        assert_eq!(keys.as_slice(), &[row.key()]);
    }
}

#[test]
fn a_notification_plans_its_recipients_update_without_a_requester() {
    let (core, _) = tree_core();
    let mut line = reply();
    line.line = tasks::Line::Notified {
        recipient: tasks::Party::Person(2),
        word: tasks::Word {
            number: 5,
            from: tasks::Party::Task(1),
            kind: tasks::MessageKind::Words,
            words: Box::new([]),
            at: line.at,
            hits: 1,
            eligible: true,
        },
    };
    let stored = tasks::Stored::Activity(line);
    let update = saved_update(&core, &stored, Wall::EPOCH);
    let row = Record::Tasks(stored);
    let mut keys = List::with_capacity(MOST_KEYS);
    assert_eq!(row_keys(&row, None, &mut keys), Some(()));
    assert_eq!(keys.as_slice(), &[row.key(), update]);
}

#[test]
fn a_decided_proposal_plans_its_decision_and_update() {
    let (core, _) = tree_core();
    let row = tasks::PersonProposal {
        number: 9,
        proposer: 2,
        project: 1,
        action: tasks::PersonAction::Amend {
            task: 1,
            revision: 2,
            amendment: Box::new(fixtures::unchanged(b"change")),
        },
        reason: Box::new([]),
        state: tasks::PersonProposalState::Accepted { by: tasks::Party::Person(3) },
    };
    let stored = tasks::Stored::DecidedProposal(Box::new(row.clone()));
    let at = Wall::from_nanos(42);
    let decision = proposals::decision_record(&stored, at).expect("accepted decision");
    let update = saved_update(&core, &stored, at);
    let own = Key::Tasks(tasks::Key::DecidedProposal(9));
    let evidence = Key::Core(CoreKey::ProposalDecision { proposer: Proposer::Party(2), proposal: 9 });
    let mut keys = List::with_capacity(MOST_KEYS);
    assert_eq!(proposal_keys(&row, Some(&decision), &mut keys), Some(()));
    assert_eq!(keys.as_slice(), &[own.clone(), evidence.clone(), update]);
    keys.clear();
    assert_eq!(proposal_keys(&row, None, &mut keys), Some(()));
    assert_eq!(keys.as_slice(), &[own.clone(), evidence]);
    keys.clear();
    assert_eq!(row_keys(&Record::Tasks(stored), None, &mut keys), Some(()));
    assert_eq!(keys.as_slice(), &[own]);
    for (decision, count) in [(Some(&decision), 3), (None, 2)] {
        for capacity in 0..count {
            let mut keys = List::with_capacity(capacity);
            assert_eq!(proposal_keys(&row, decision, &mut keys), None);
            assert_eq!(keys.len(), capacity);
        }
    }
}

#[test]
fn full_outputs_refuse_every_planner_without_panicking() {
    let row = ended(tasks::Party::Person(2));
    for capacity in 0..MOST_KEYS {
        let mut keys = List::with_capacity(capacity);
        assert_eq!(record_keys(&row, &mut keys), None);
        assert_eq!(keys.len(), capacity);
    }
    let row = Record::Tasks(tasks::Stored::Activity(reply()));
    for capacity in 0..2 {
        let mut keys = List::with_capacity(capacity);
        assert_eq!(row_keys(&row, Some(tasks::Party::Person(2)), &mut keys), None);
        assert_eq!(keys.len(), capacity);
    }
}

fn saves(core: &mut Core, limits: &Limits, event: Event, rows: &mut List<Record>, erased: &mut List<Key>) {
    for request in watch_outputs(core, limits, event) {
        match request {
            Request::Write(Write::Save(row)) => rows.push(row).expect("bounded fixture writes"),
            Request::Write(Write::Erase(key)) => erased.push(key).expect("bounded fixture erasures"),
            Request::Now(now) => {
                if let Now::AcceptedTurn { task, attempt, turn, accepted, .. } = *now {
                    saves(
                        core,
                        limits,
                        Event::AcceptedTurn {
                            opens_window: false,
                            task,
                            attempt,
                            turn,
                            accepted,
                            cumulative: 2,
                            read: None,
                            transcript: b"committed reply".as_slice().into(),
                        },
                        rows,
                        erased,
                    );
                }
            }
            Request::Write(Write::CopySettingsRevision(_) | Write::DropTranscript { .. })
            | Request::Ask { .. }
            | Request::Held(_)
            | Request::ProjectMade { .. }
            | Request::Decided => {}
        }
    }
}

#[test]
fn the_plan_matches_what_the_core_saved() {
    let (mut core, limits) = tree_core();
    let mut new = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    new.flags.conversation = true;
    new.flags.listed = true;
    let mut rows = List::with_capacity(128);
    let mut erased = List::with_capacity(128);
    let to = ReplyTo::new(Token::new(1));
    saves(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(u64::MAX)),
            creator: tasks::Party::Person(1),
            batch: Box::new([new]),
        }),
        &mut rows,
        &mut erased,
    );
    assert!(core.tasks.task(1).is_some());
    saves(&mut core, &limits, Event::Tasks(tasks::Event::Prepare { reply_to: to, task: 1 }), &mut rows, &mut erased);
    core.proofs.insert(1, RunProof::claimed(1, 7, 7, None, fleet::HostKind::Worker)).expect("proof room");
    core.claiming.insert(1, 7).expect("claim room");
    for event in [
        Event::ClaimPrepared { task: 1, attempt: 7, budget: 10, writes: Box::new([]) },
        Event::Tasks(tasks::Event::Started { task: 1, attempt: 7 }),
        Event::TurnPayload {
            refused: Box::new([]),
            run: Token::new(1),
            attempt: Token::new(7),
            turn: 1,
            body: Token::new(11),
            read: None,
            cumulative: 2,
        },
    ] {
        saves(&mut core, &limits, event, &mut rows, &mut erased);
    }
    saves(
        &mut core,
        &limits,
        Event::AnswerPayload {
            read: None,
            refused: Box::new([]),
            run: Token::new(1),
            attempt: Token::new(7),
            payload: Token::new(13),
            cumulative: 3,
            end: tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
                cancel_delegates: false,
            },
            saved: None,
            invalid_saved: false,
        },
        &mut rows,
        &mut erased,
    );
    for connector in core.connectors.clone() {
        saves(
            &mut core,
            &limits,
            Event::EffectConnector(connector::Event::Closed { task: 1, connector }),
            &mut rows,
            &mut erased,
        );
    }
    saves(&mut core, &limits, Event::Tasks(tasks::Event::Settled { task: 1 }), &mut rows, &mut erased);
    let mut record = None;
    for row in &rows {
        if let Record::Tasks(tasks::Stored::Ended(row)) = row {
            record = Some(row.as_ref());
        }
    }
    let record = record.expect("core ended the task");
    assert_eq!(record.last_reply, 1);
    let mut plan = List::with_capacity(256);
    assert_eq!(record_keys(record, &mut plan), Some(()));
    let mut replies = 0_u32;
    let mut indexed = 0_u32;
    for row in &rows {
        match row {
            Record::Tasks(tasks::Stored::Activity(line)) => {
                if let tasks::Line::Turn { .. } = line.line {
                    replies = replies.checked_add(1).expect("bounded replies");
                }
                assert_eq!(row_keys(row, Some(record.requester), &mut plan), Some(()));
            }
            Record::Tasks(
                tasks::Stored::History(_)
                | tasks::Stored::Milestone(_)
                | tasks::Stored::Stub(_)
                | tasks::Stored::EndedDelegate(_)
                | tasks::Stored::Delegate(_),
            )
            | Record::Core(
                CoreRecord::RunProof(_)
                | CoreRecord::Call(_)
                | CoreRecord::Terminal(_)
                | CoreRecord::EscalationDecision(_),
            )
            | Record::People(people::Stored::EndedRead { .. }) => {
                assert_eq!(row_keys(row, Some(record.requester), &mut plan), Some(()));
            }
            Record::Core(_) | Record::Tasks(_) | Record::People(_) | Record::Notes(_) | Record::Accounts(_) => {}
        }
    }
    assert_eq!(replies, 1);
    for row in &rows {
        match row {
            // Live rows and proofs already erased by ending are absent from the store.
            Record::Tasks(tasks::Stored::Live(_)) => assert!(erased.as_slice().contains(&row.key())),
            // Funding and deployment survive; turns are erased with the transcript file.
            Record::Tasks(tasks::Stored::Ledger(_)) | Record::Core(CoreRecord::Deployment(_) | CoreRecord::Turn(_)) => {
            }
            Record::Core(_) | Record::Tasks(_) | Record::People(_) | Record::Notes(_) | Record::Accounts(_) => {
                assert!(plan.as_slice().contains(&row.key()), "unplanned saved key: {:?}", row.key());
                indexed = indexed.checked_add(1).expect("bounded saved rows");
            }
        }
    }
    assert!(indexed > 5, "checks ended indexes and activity rows");
}
