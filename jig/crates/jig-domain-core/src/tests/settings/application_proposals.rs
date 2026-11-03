//! Proposal admission for application settings (jig/docs/design/domain/engine.md, 17.6).
#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "unit tests inspect selected boundary outputs"
)]
use crate::settings::{self as s, Kind, KindScope};
use crate::tests::*;

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
fn application_only_proposal_keeps_its_sized_set() {
    let (mut core, limits) = application_core();
    let output =
        propose(&mut core, &limits, Box::new([set(1, 1), item(2, 2, s::Edit::Removed), item(3, 3, s::Edit::Reset)]));
    let proposal = proposed(&output);
    let set = core.settings_proposals.get(&(Proposer::Party(1), proposal)).unwrap();
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
            assert_eq!(*number, u64::from(proposal));
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
fn application_proposal_withdrawn_by_a_revision_drops_each_connector_once() {
    let (mut core, limits) = application_core();
    let proposal = proposed(&propose(&mut core, &limits, Box::new([set(1, 1), set(2, 2), set(3, 3)])));
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
    assert_eq!(drops(&output).as_ref(), &[(2, u64::from(proposal)), (3, u64::from(proposal))]);
    assert!(core.settings_proposals.is_empty());
}

#[test]
fn application_proposal_rejected_or_withdrawn_drops_each_connector_once() {
    for withdraw in [false, true] {
        let (mut core, limits) = application_core();
        let proposal = proposed(&propose(&mut core, &limits, Box::new([set(1, 1), set(2, 2), set(3, 3)])));
        let action = if withdraw {
            people::Ask::Withdraw { project: 0, proposal: u64::from(proposal) }
        } else {
            decision(proposal, people::ProposalDecision::Reject { reason: b"later".as_slice().into() })
        };
        let output = ask(&mut core, &limits, action, Box::default());
        assert_eq!(drops(&output).as_ref(), &[(2, u64::from(proposal)), (3, u64::from(proposal))]);
        assert!(core.settings_proposals.is_empty());
        assert_no_application_writes(&output);
    }
}

#[test]
fn application_proposal_accepted_judges_and_keeps_the_connectors_values() {
    let (mut core, limits) = application_core();
    let proposal = proposed(&propose(&mut core, &limits, Box::new([set(1, 1)])));
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
                assert_eq!((*revision, *number), (2, Some(u64::from(proposal))));
                assert_eq!(items.as_ref(), &expected);
                assert_eq!(keeps, 0);
            }
            Request::Ask {
                connector: 2,
                ask: Ask::Effect(connector::Ask::KeepSettings { revision, proposal: number, items }),
            } => {
                keeps += 1;
                assert_eq!((*revision, *number), (2, Some(u64::from(proposal))));
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
                why: people::Malformed::Settings { setting: set(1, 1).setting, fault: s::Fault::Shape },
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
    let mut configured = config(41);
    configured.kinds = core.kinds.clone();
    configured.connectors = Box::new([2, 3]);
    let mut restarted = Core::new(configured, &limits).unwrap();
    let _begin = restarted.start_begin(false, Wall::EPOCH);
    for request in output {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(s::Record::Proposal {
            proposer,
            proposal: number,
            value,
        })))) = request
        {
            assert_eq!(
                restarted.restore_core(
                    CoreRecord::Settings(s::Record::Proposal { proposer, proposal: number, value }),
                    &limits
                ),
                Restored::Live
            );
        }
    }
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
            assert_eq!(*number, Some(u64::from(proposal)));
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
