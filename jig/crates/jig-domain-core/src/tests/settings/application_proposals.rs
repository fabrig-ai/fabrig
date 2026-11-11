//! Proposal admission for application settings (jig/docs/design/domain/engine/settings.md, 6).
#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "unit tests inspect selected boundary outputs"
)]
use crate::settings::{self as s, Kind, KindScope};
use crate::tests::*;

// A proposal, its extent, and an acceptance's revision and application state.
const SNAPSHOT_ROWS: u32 = 4;

fn application_core() -> (Core, Limits) {
    let (owner, mut limits) = core();
    limits.people.projects = 2;
    limits.people.requests = 16;
    let mut configured = config(41);
    configured.kinds = Box::new([kind(1, 2), kind(2, 2), kind(3, 3)]);
    configured.connectors = Box::new([2, 3]);
    let mut core = seeded_core(configured, &limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    for record in [
        people::Stored::Person { number: 1, identity: owner.people.identity(1).unwrap().clone() },
        people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
        people::Stored::Roles {
            project: 0,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut Queue::uncapped());
    }
    finish_people(&mut core.people, &env);
    let _message = fresh(&mut core.counters, Family::Message).unwrap();
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    (core, limits)
}
fn kind(number: u32, connector: u16) -> Kind {
    Kind { number, scope: KindScope::Deployment, connector, value_bytes: 8, per_scope: 4, charters: 0 }
}
fn item(kind: u32, number: u32, edit: s::Edit) -> s::Item {
    s::Item { setting: s::Setting::Application { kind, scope: 0, number }, edit }
}
fn set(kind: u32, number: u32) -> s::Item {
    item(kind, number, s::Edit::Set(s::Value::Application(b"value".as_slice().into())))
}

/// The scripted root answers every judge within the same decision.
fn send(core: &mut Core, limits: &Limits, event: Event, answers: Box<[Box<[connector::Judged]>]>) -> Box<[Request]> {
    let mut events = Queue::uncapped();
    events.push(event);
    let mut script = answers.into_iter();
    let mut output = List::with_capacity(512);
    while let Some(event) = events.pop() {
        let mut out = step(core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }, event);
        while let Some(request) = out.pop() {
            if let Request::Ask { ask: Ask::Effect(connector::Ask::JudgeSettings { owner, .. }), .. } = &request {
                events.push(Event::EffectConnector(connector::Event::SettingsJudged {
                    owner: *owner,
                    judged: script.next().expect("scripted judge answer"),
                }));
            }
            output.push(request).unwrap();
        }
    }
    assert!(script.next().is_none(), "every scripted answer was used");
    core.reclaim();
    output.into_boxed()
}
fn propose(core: &mut Core, limits: &Limits, items: Box<[s::Item]>) -> Box<[Request]> {
    send(
        core,
        limits,
        Event::ProposeSettings {
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: 10,
            key: [1; 16],
            digest: [1; 32],
            change: Box::new(s::Proposed { against: 1, reason: b"review".as_slice().into(), items }),
        },
        Box::default(),
    )
}
fn ask(core: &mut Core, limits: &Limits, ask: people::Ask, answers: Box<[Box<[connector::Judged]>]>) -> Box<[Request]> {
    send(
        core,
        limits,
        Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(92)),
            sign_in: 10,
            key: [2; 16],
            models: Box::default(),
            ask,
        }),
        answers,
    )
}
fn outcome(output: &[Request]) -> people::Outcome {
    for request in output {
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = held.as_ref()
        {
            return outcome.clone();
        }
    }
    panic!("keyed outcome: {output:?}")
}
fn proposed(output: &[Request]) -> u32 {
    let people::Outcome::Proposed { proposal, to } = outcome(output) else { panic!("proposal: {output:?}") };
    assert_eq!(to, people::Role::Owner.number());
    u32::try_from(proposal).unwrap()
}
fn decision(proposal: u32, decision: people::ProposalDecision) -> people::Ask {
    let running = match decision {
        people::ProposalDecision::Accept => Some(people::Running::Stays),
        people::ProposalDecision::Reject { .. } | people::ProposalDecision::Pass => None,
    };
    people::Ask::DecideProposal {
        project: 0,
        proposer: Proposer::Party(1),
        proposal: u64::from(proposal),
        decision,
        running,
    }
}
fn drops(output: &[Request]) -> Box<[(u16, u64)]> {
    let mut drops = List::with_capacity(8);
    for request in output {
        if let Request::Ask { connector, ask: Ask::Effect(connector::Ask::DropProposalSettings { proposal }) } = request
        {
            drops.push((*connector, *proposal)).unwrap();
        }
    }
    drops.into_boxed()
}
fn assert_no_application_writes(output: &[Request]) {
    for request in output {
        assert!(
            match request {
                Request::Write(Write::Save(Record::Core(CoreRecord::Settings(
                    s::Record::Application { .. } | s::Record::Revision { .. },
                )))) => false,
                _ => true,
            },
            "{request:?}"
        );
    }
}

#[test]
#[should_panic(expected = "checked application value")]
fn application_proposal_value_mismatch_breaks_the_checked_promise() {
    let (core, _) = application_core();
    let charter = core.current.charter(1).unwrap().clone();
    let mut proposal = s::Proposal {
        key: None,
        against: 1,
        items: Box::new([item(1, 1, s::Edit::Set(s::Value::Charter(Box::new(charter))))]),
        applications: Box::default(),
    };
    s::application::keep_proposal(&core, &mut proposal, 1, &mut Queue::uncapped());
}

#[test]
fn application_only_proposal_keeps_its_sized_set() {
    let (mut core, limits) = application_core();
    let output =
        propose(&mut core, &limits, Box::new([set(1, 1), item(2, 2, s::Edit::Removed), item(3, 3, s::Edit::Reset)]));
    let proposal = proposed(&output);
    let set = core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap();
    let key = set.key.unwrap();
    assert_ne!(key, u64::from(proposal));
    assert!(set.items.is_empty());
    assert_eq!(
        set.applications.as_ref(),
        &[
            s::application::Recorded { kind: 1, scope: 0, number: 1, edit: s::application::SizedEdit::Set { size: 5 } },
            s::application::Recorded { kind: 2, scope: 0, number: 2, edit: s::application::SizedEdit::Remove },
            s::application::Recorded { kind: 3, scope: 0, number: 3, edit: s::application::SizedEdit::Reset },
        ]
    );
    let mut groups = List::with_capacity(2);
    for request in &output {
        if let Request::Ask {
            connector,
            ask: Ask::Effect(connector::Ask::KeepProposalSettings { proposal: number, items }),
        } = request
        {
            assert_eq!(*number, key);
            groups.push((*connector, items.clone())).unwrap();
        }
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(s::Record::Proposal { value, .. })))) =
            request
        {
            assert_eq!(value, set);
        }
    }
    assert_eq!(
        groups.into_boxed().as_ref(),
        &[
            (
                2,
                Box::from(
                    [
                        s::application::Item {
                            kind: 1,
                            scope: 0,
                            number: 1,
                            edit: s::application::Edit::Set(b"value".as_slice().into())
                        },
                        s::application::Item { kind: 2, scope: 0, number: 2, edit: s::application::Edit::Remove },
                    ]
                    .as_slice()
                )
            ),
            (
                3,
                Box::from(
                    [s::application::Item { kind: 3, scope: 0, number: 3, edit: s::application::Edit::Reset }]
                        .as_slice()
                )
            ),
        ]
    );
    assert!(core.current.applications.is_empty());
    assert!(drops(&output).is_empty());
    assert_no_application_writes(&output);
}

#[test]
fn a_partys_settings_proposal_neither_tells_nor_counts_in_task_arrivals() {
    let (mut core, limits) = application_core();
    let output = propose(&mut core, &limits, Box::new([set(1, 1)]));
    let _proposal = proposed(&output);
    for request in &output {
        if let Request::Held(held) = request
            && let Held::NeedArrived { .. } = held.as_ref()
        {
            panic!("a party's settings proposal does not tell arrivals");
        }
    }
    assert_eq!(core.settings_needs(1), 1, "settings proposal waits outside the task census");
    let _roles = watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Member }]),
        }),
    );
    let output = watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::Waiting {
            task: 7,
            entries: Box::new([people::Entry {
                task: 7,
                project: 1,
                whom: people::Whom::Person(1),
                kind: people::EntryKind::Question { message: 11 },
                at: Wall::EPOCH,
            }]),
        }),
    );
    let [Request::Held(held)] = output.as_ref() else { panic!("one held arrival: {output:?}") };
    let Held::NeedArrived { party, project, task, needs } = held.as_ref() else { panic!("arrival: {held:?}") };
    assert_eq!((*party, *project, *task, *needs), (1, 1, 7, 1));
}

#[test]
fn application_proposal_withdrawn_by_a_revision_drops_each_connector_once() {
    let (mut core, limits) = application_core();
    let proposal = proposed(&propose(&mut core, &limits, Box::new([set(1, 1), set(2, 2), set(3, 3)])));
    let key = core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap().key.unwrap();
    let output = send(
        &mut core,
        &limits,
        Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(93)),
            sign_in: 10,
            key: [2; 16],
            digest: [2; 32],
            change: Box::new(s::Change { against: 1, items: Box::new([set(1, 4)]), running: s::Running::Stays }),
        },
        Box::new([Box::new([connector::Judged::Accept { size: 5, charters: Box::default() }])]),
    );
    assert_eq!(outcome(&output), people::Outcome::SettingsChanged { revision: 2 });
    assert_eq!(drops(&output).as_ref(), &[(2, key), (3, key)]);
    assert!(core.settings_proposals.is_empty());
}

#[test]
fn application_proposal_rejected_or_withdrawn_drops_each_connector_once() {
    for withdraw in [false, true] {
        let (mut core, limits) = application_core();
        let proposal = proposed(&propose(&mut core, &limits, Box::new([set(1, 1), set(2, 2), set(3, 3)])));
        let key = core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap().key.unwrap();
        let action = if withdraw {
            people::Ask::Withdraw { project: 0, proposal: u64::from(proposal) }
        } else {
            decision(proposal, people::ProposalDecision::Reject { reason: b"later".as_slice().into() })
        };
        let output = ask(&mut core, &limits, action, Box::default());
        assert_eq!(drops(&output).as_ref(), &[(2, key), (3, key)]);
        assert!(core.settings_proposals.is_empty());
        assert_no_application_writes(&output);
    }
}

#[test]
fn application_proposal_accepted_judges_and_keeps_the_connectors_values() {
    let (mut core, limits) = application_core();
    let proposal = proposed(&propose(&mut core, &limits, Box::new([set(1, 1)])));
    let key = core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap().key.unwrap();
    let output = ask(
        &mut core,
        &limits,
        decision(proposal, people::ProposalDecision::Accept),
        Box::new([Box::new([connector::Judged::Accept { size: 7, charters: Box::default() }])]),
    );
    assert!(match outcome(&output) {
        people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. } => true,
        _ => false,
    });
    let expected =
        [s::application::Item { kind: 1, scope: 0, number: 1, edit: s::application::Edit::Set(Box::default()) }];
    let mut judges = 0_u32;
    let mut keeps = 0_u32;
    for request in &output {
        match request {
            Request::Ask {
                connector: 2,
                ask: Ask::Effect(connector::Ask::JudgeSettings { revision, proposal: number, items, .. }),
            } => {
                judges += 1;
                assert_eq!((*revision, *number), (2, Some(key)));
                assert_eq!(items.as_ref(), &expected);
                assert_eq!(keeps, 0);
            }
            Request::Ask {
                connector: 2,
                ask: Ask::Effect(connector::Ask::KeepSettings { revision, proposal: number, items }),
            } => {
                keeps += 1;
                assert_eq!((*revision, *number), (2, Some(key)));
                assert_eq!(items.as_ref(), &expected);
                assert_eq!(judges, 1);
            }
            _ => {}
        }
    }
    assert_eq!((judges, keeps), (1, 1));
    assert_eq!(core.current.applications.get(&set(1, 1).setting).unwrap().size, 7, "the judge reads the kept value");
    assert_eq!(core.current.head.proposal, Some((Proposer::Party(1), proposal)));
    assert!(core.settings_proposals.is_empty());
    assert!(drops(&output).is_empty());
}

#[test]
fn application_proposal_refused_acceptance_stays_pending_without_a_drop_or_keep() {
    for answer in [
        connector::Judged::Refuse { why: b"not permitted".as_slice().into() },
        connector::Judged::Accept { size: 9, charters: Box::default() },
        connector::Judged::Same,
    ] {
        let (mut core, limits) = application_core();
        let proposal = proposed(&propose(&mut core, &limits, Box::new([set(1, 1)])));
        let before = core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap().clone();
        let output = ask(
            &mut core,
            &limits,
            decision(proposal, people::ProposalDecision::Accept),
            Box::new([Box::new([answer.clone()])]),
        );
        let expected = match answer {
            connector::Judged::Refuse { why } => people::Refusal::Judged { at: 0, setting: set(1, 1).setting, why },
            connector::Judged::Accept { .. } => people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::Settings { setting: set(1, 1).setting, fault: s::Fault::Shape, detail: None },
            },
            connector::Judged::Same => {
                people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged }
            }
        };
        assert_eq!(outcome(&output), people::Outcome::Refused(expected));
        assert_eq!(core.settings_proposals.get(&(Proposer::Party(1), proposal)), Some(&before));
        assert!(core.tasks.person_proposal(1, u64::from(proposal)).is_some());
        assert_eq!(core.current.newest(), 1);
        assert!(drops(&output).is_empty());
        for request in &output {
            assert!(match request {
                Request::Ask { ask: Ask::Effect(connector::Ask::KeepSettings { .. }), .. } => false,
                _ => true,
            });
        }
        assert_no_application_writes(&output);
    }
}

#[test]
fn application_proposal_set_restores_its_sized_edits() {
    let (mut core, limits) = application_core();
    let output =
        propose(&mut core, &limits, Box::new([set(1, 1), item(2, 2, s::Edit::Removed), item(3, 3, s::Edit::Reset)]));
    let proposal = proposed(&output);
    let mut rows = Map::with_capacity(SNAPSHOT_ROWS);
    remember(&mut rows, &output);
    let mut restarted = restart(&core, &limits, &rows);
    let next = offer(&mut restarted, Proposer::Task(2), 7);
    assert_eq!(keeps(&next).as_ref(), &[(2, 2)]);
    assert_eq!(
        restarted.settings_proposals.get(&(Proposer::Party(1), proposal)),
        core.settings_proposals.get(&(Proposer::Party(1), proposal))
    );
}

#[test]
fn empty_or_malformed_application_proposals_keep_nothing() {
    for items in [
        Box::<[s::Item]>::default(),
        Box::from([set(99, 1)].as_slice()),
        Box::from([set(1, 0)].as_slice()),
        Box::from([set(1, 1), set(1, 1)].as_slice()),
        Box::from([item(1, 1, s::Edit::Set(s::Value::Application(b"123456789".as_slice().into())))].as_slice()),
        Box::from([set(1, 1), set(1, 2), set(1, 3), set(1, 4), set(1, 5)].as_slice()),
    ] {
        let (mut core, limits) = application_core();
        let output = propose(&mut core, &limits, items);
        assert!(
            match outcome(&output) {
                people::Outcome::Refused(people::Refusal::NotWellFormed { .. }) => true,
                _ => false,
            },
            "{output:?}"
        );
        assert!(core.settings_proposals.is_empty());
        assert_no_application_writes(&output);
        for request in &output {
            assert!(
                match request {
                    Request::Ask { ask: Ask::Effect(_), .. } => false,
                    _ => true,
                },
                "no connector handoff for invalid set"
            );
        }
    }
}

#[test]
fn accepted_proposal_finishes_connectors_whose_items_are_all_unchanged() {
    let (mut core, limits) = application_core();
    let proposal = proposed(&propose(&mut core, &limits, Box::new([set(1, 1), set(3, 1)])));
    let key = core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap().key.unwrap();
    let output = ask(
        &mut core,
        &limits,
        decision(proposal, people::ProposalDecision::Accept),
        Box::new([
            Box::from([connector::Judged::Same].as_slice()),
            Box::from([connector::Judged::Accept { size: 5, charters: Box::default() }].as_slice()),
        ]),
    );
    assert_eq!(core.current.newest(), 2);
    assert!(core.settings_proposals.is_empty());
    assert!(drops(&output).is_empty());
    let mut connectors = List::with_capacity(2);
    for request in &output {
        if let Request::Ask {
            connector,
            ask: Ask::Effect(connector::Ask::KeepSettings { proposal: number, items, .. }),
        } = request
        {
            assert_eq!(*number, Some(key));
            if *connector == 2 {
                assert!(items.is_empty(), "unchanged items make no changes");
            } else {
                assert_eq!(items.len(), 1);
            }
            connectors.push(*connector).unwrap();
        }
    }
    assert_eq!(connectors.as_slice(), &[2, 3], "every connector must finish its kept proposal");
    assert!(!core.current.applications.contains_key(&set(1, 1).setting));
}

// The tasks child saves a proposal under its proposer's number, which can coincide.
fn offer(core: &mut Core, proposer: Proposer, number: u32) -> Box<[Request]> {
    let by = match proposer {
        Proposer::Party(by) | Proposer::Task(by) => by,
    };
    core.fly(flight::Flight::SettingsOffering {
        request: Token::new(94),
        by,
        number,
        value: s::Proposal {
            key: None,
            against: core.current.newest(),
            items: Box::new([set(1, 1)]),
            applications: Box::default(),
        },
    });
    let mut out = Queue::uncapped();
    let row = match proposer {
        Proposer::Party(_) => tasks::Stored::PersonProposal(Box::new(tasks::PersonProposal {
            number: u64::from(number),
            proposer: by,
            project: 0,
            action: tasks::PersonAction::Settings { against: core.current.newest(), items: Box::default() },
            reason: Box::default(),
            state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
        })),
        Proposer::Task(_) => {
            let mut history = tasks::History {
                task: by,
                revision: 1,
                by: tasks::Party::Person(1),
                change: tasks::Change::Proposed,
                reason: Box::default(),
                proposal: None,
                at: Wall::EPOCH,
                amendment: None,
            };
            history.proposal = Some(Box::new(tasks::Proposal {
                number: u64::from(number),
                proposer: by,
                project: 0,
                action: tasks::ProposalAction::Settings {
                    projects: Box::default(),
                    against: core.current.newest(),
                    items: Box::default(),
                },
                reason: Box::default(),
                as_holder: false,
                state: tasks::ProposalState::Pending {
                    holder: tasks::ProposalHolder::Policy { project: 0, kind: tasks::ProposalKind::Settings },
                    since: Wall::EPOCH,
                },
            }));
            tasks::Stored::History(history)
        }
    };
    crate::routing::settings::dispatch::proposal_saved(core, &row, &mut out);
    let mut output = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        output.push(request).unwrap();
    }
    output.into_boxed()
}
fn keeps(output: &[Request]) -> Box<[(u16, u64)]> {
    let mut keys = List::with_capacity(u32::try_from(output.len()).unwrap());
    for request in output {
        if let Request::Ask { connector, ask: Ask::Effect(connector::Ask::KeepProposalSettings { proposal, .. }) } =
            request
        {
            keys.push((*connector, *proposal)).unwrap();
        }
    }
    keys.into_boxed()
}
fn remember(rows: &mut Map<s::Key, s::Record>, output: &[Request]) {
    for request in output {
        match request {
            Request::Write(Write::Save(Record::Core(CoreRecord::Settings(row)))) => {
                rows.insert(row.key(), row.clone()).unwrap();
            }
            Request::Write(Write::Erase(Key::Core(CoreKey::Settings(key)))) => {
                rows.remove(key);
            }
            _ => {}
        }
    }
}
fn restart(core: &Core, limits: &Limits, rows: &Map<s::Key, s::Record>) -> Core {
    let mut configured = config(41);
    configured.kinds = core.kinds.clone();
    configured.connectors = Box::new([2, 3]);
    let mut restarted = Core::new(configured, limits).unwrap();
    let _begin = restarted.start_begin(false, Wall::EPOCH);
    for (&setting, default) in &core.current.defaults {
        assert!(restarted.restore_setting(s::Record::Default { setting, default: default.clone() }, limits));
    }
    assert!(restarted.restore_setting(
        s::Record::Revision {
            history: false,
            revision: s::Revision {
                head: core.current.head.clone(),
                items: Box::default(),
                applications: Box::default()
            },
        },
        limits
    ));
    for (_, row) in rows {
        match row {
            s::Record::Proposal { .. } | s::Record::Extent { .. } => {
                assert!(restarted.restore_setting(row.clone(), limits));
            }
            _ => {}
        }
    }
    assert!(restarted.finish_restored_settings(&Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }));
    restarted
}
#[test]
fn two_proposers_with_one_number_reach_distinct_connector_keys() {
    let (mut core, limits) = application_core();
    let first = offer(&mut core, Proposer::Party(1), 7);
    let second = offer(&mut core, Proposer::Task(1), 7);
    assert_eq!(keeps(&first).as_ref(), &[(2, 1)]);
    assert_eq!(keeps(&second).as_ref(), &[(2, 2)]);
    let mut rows = Map::with_capacity(SNAPSHOT_ROWS);
    remember(&mut rows, &first);
    remember(&mut rows, &second);
    let mut restarted = restart(&core, &limits, &rows);
    assert_eq!(restarted.settings_proposals.get(&(Proposer::Party(1), 7)).unwrap().key, Some(1));
    assert_eq!(restarted.settings_proposals.get(&(Proposer::Task(1), 7)).unwrap().key, Some(2));
    assert_eq!(keeps(&offer(&mut restarted, Proposer::Party(2), 7)).as_ref(), &[(2, 3)]);
}
fn pending_application() -> (Core, Limits, u32, Map<s::Key, s::Record>) {
    let (mut core, limits) = application_core();
    let output = propose(&mut core, &limits, Box::new([set(1, 1)]));
    let proposal = proposed(&output);
    let mut rows = Map::with_capacity(SNAPSHOT_ROWS);
    remember(&mut rows, &output);
    (core, limits, proposal, rows)
}
#[test]
fn restart_keeps_the_counter_after_every_proposal_was_decided() {
    let (mut core, limits, proposal, mut rows) = pending_application();
    let output = ask(
        &mut core,
        &limits,
        decision(proposal, people::ProposalDecision::Reject { reason: Box::default() }),
        Box::default(),
    );
    remember(&mut rows, &output);
    assert!(core.settings_proposals.is_empty());
    let mut restarted = restart(&core, &limits, &rows);
    assert!(restarted.settings_proposals.is_empty());
    assert_eq!(keeps(&offer(&mut restarted, Proposer::Party(1), 7)).as_ref(), &[(2, 2)]);
}
#[test]
fn jig_only_proposals_draw_no_key_and_send_no_connector_ask() {
    for keys in [1, u64::MAX] {
        let (mut core, limits) = application_core();
        core.current.keys = keys;
        let mut charter = core.current.charter(1).unwrap().clone();
        charter.policy.instructions = b"changed".as_slice().into();
        let output = propose(
            &mut core,
            &limits,
            Box::new([s::Item {
                setting: s::Setting::Charter(1),
                edit: s::Edit::Set(s::Value::Charter(Box::new(charter))),
            }]),
        );
        let proposal = proposed(&output);
        assert_eq!(core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap().key, None);
        assert_eq!(core.current.keys, keys);
        for request in output {
            assert!(match request {
                Request::Ask { ask: Ask::Effect(_), .. } => false,
                _ => true,
            });
        }
    }
}
#[test]
fn proposal_key_overflow_is_refused_before_retaining_the_set() {
    let (mut core, limits) = application_core();
    core.current.keys = u64::MAX;
    let output = propose(&mut core, &limits, Box::new([set(1, 1)]));
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Arithmetic })
    );
    assert_eq!(core.current.keys, u64::MAX);
    assert!(core.settings_proposals.is_empty());
    assert!(keeps(&output).is_empty());
}
#[test]
#[expect(
    clippy::disallowed_types,
    reason = "the panic payload is a String whose text must name the broken proposal link"
)]
fn restart_refuses_a_proposal_key_at_or_past_the_counter() {
    for key in [2, 3] {
        let (mut core, limits) = application_core();
        let output = propose(&mut core, &limits, Box::new([set(1, 1)]));
        let mut rows = Map::with_capacity(SNAPSHOT_ROWS);
        remember(&mut rows, &output);
        let row = rows.get_mut(&s::Key::Proposal { proposer: Proposer::Party(1), proposal: 2 }).unwrap();
        let s::Record::Proposal { value, .. } = row else { panic!("proposal row") };
        value.key = Some(key);
        let panic = std::panic::catch_unwind(|| restart(&core, &limits, &rows)).unwrap_err();
        let message = panic.downcast_ref::<String>().unwrap();
        assert!(message.contains("Proposal { proposer: Party(1), proposal: 2 }"), "{message}");
    }
}

#[test]
fn the_last_drawable_proposal_key_advances_without_wrapping() {
    let (mut core, limits) = application_core();
    core.current.keys = u64::MAX - 1;
    let output = propose(&mut core, &limits, Box::new([set(1, 1)]));
    let proposal = proposed(&output);
    assert_eq!(core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap().key, Some(u64::MAX - 1));
    assert_eq!(core.current.keys, u64::MAX);
    assert_eq!(keeps(&output).as_ref(), &[(2, u64::MAX - 1)]);
}
#[test]
fn accepting_the_last_proposal_preserves_the_counter_in_its_revision() {
    let (mut core, limits, proposal, mut rows) = pending_application();
    let output = ask(
        &mut core,
        &limits,
        decision(proposal, people::ProposalDecision::Accept),
        Box::new([Box::new([connector::Judged::Accept { size: 7, charters: Box::default() }])]),
    );
    remember(&mut rows, &output);
    assert!(core.settings_proposals.is_empty());
    let mut restarted = restart(&core, &limits, &rows);
    assert_eq!(keeps(&offer(&mut restarted, Proposer::Party(1), 7)).as_ref(), &[(2, 2)]);
}
