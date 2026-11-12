//! Charter values are checked before reaching the core
//! (temper/docs/design/domain/agent/tools.md).
use super::resume_fixture;
use crate::{Delivery, Domain, Event, Limits, PartyInput, PartyRequest, Request};
use alloc::boxed::Box;
use jig_domain_core::{
    self as core,
    settings::{self as settings, Edit, Item, Setting, Value},
};
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Token, Wall};

// A synthetic host part isolates the admission boundary from smith's measured table.
const HOST: crate::room::HostPart = crate::room::HostPart {
    bound: 3_048,
    fixed: 0,
    declared: 0,
    settings: 0,
    asking: 0,
    asking_settings: 0,
    inspect: 0,
    modify: 0,
    shell: 0,
    agents: 0,
};
const CHARTER: u32 = 1;
const REPLY: Token = Token::new(9);
// 3,048 - (999 instruction bytes + two line-ending bytes) = 2,047.
const SHORT_INSTRUCTIONS: u32 = 999;
const WORDS: &[u8] = b"charter 1 leaves its brief 2,047 bytes of its host part's 3,048; it needs at least 2,048";

fn root(host: Option<crate::room::HostPart>) -> (Domain, Env<Limits>) {
    let mut config = resume_fixture::config(7);
    config.host_part = host;
    resume_fixture::root_with_config(config)
}
fn short_charter() -> crate::Charter {
    let mut charter = resume_fixture::charter();
    charter.policy.instructions = super::bytes(SHORT_INSTRUCTIONS);
    charter
}
fn parts(bytes: u32) -> Box<[core::CharterPart]> {
    Box::new([core::CharterPart { on: jig_domain_tasks::Reasons::FIRST, instructions: super::bytes(bytes) }])
}

fn overlay(instructions: u32, part: Option<u32>) -> settings::Overlay {
    let mut overlay = settings::Overlay::empty();
    overlay.instructions = super::bytes(instructions);
    overlay.parts = part.map_or_else(|| Box::new([]) as Box<[_]>, parts);
    overlay.shell = Some(false);
    overlay
}

fn overlay_items(overlay: settings::Overlay) -> Box<[Item]> {
    Box::new([Item { setting: Setting::Charter(CHARTER), edit: Edit::Set(Value::Overlay(Box::new(overlay))) }])
}

fn with_items(input: PartyInput, items: Box<[Item]>) -> PartyInput {
    match input {
        PartyInput::ChangeSettings { reply_to, sign_in, key, digest, kinds, mut change } => {
            change.items = items;
            PartyInput::ChangeSettings { reply_to, sign_in, key, digest, kinds, change }
        }
        PartyInput::ProposeSettings { reply_to, sign_in, key, digest, kinds, mut change } => {
            change.items = items;
            PartyInput::ProposeSettings { reply_to, sign_in, key, digest, kinds, change }
        }
        _ => unreachable!("a settings entrance"),
    }
}

fn items() -> Box<[Item]> {
    Box::new([
        Item { setting: Setting::Rules, edit: Edit::Reset },
        Item { setting: Setting::Charter(CHARTER), edit: Edit::Set(Value::Charter(Box::new(short_charter()))) },
    ])
}
fn change() -> PartyInput {
    PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(REPLY),
        sign_in: 1,
        key: [1; 16],
        digest: [2; 32],
        kinds: Box::new([]),
        change: Box::new(settings::Change { against: 1, items: items(), running: settings::Running::Stays }),
    }
}
fn proposal() -> PartyInput {
    PartyInput::ProposeSettings {
        reply_to: ReplyTo::new(REPLY),
        sign_in: 1,
        key: [1; 16],
        digest: [2; 32],
        kinds: Box::new([]),
        change: Box::new(settings::Proposed { against: 1, items: items(), reason: b"a charter".as_slice().into() }),
    }
}
fn refused(input: PartyInput) -> people::Refusal {
    let (mut root, env) = root(Some(HOST));
    crate::try_step(&mut root, &env, Event::Party(input)).unwrap();
    let mut out = Queue::uncapped();
    crate::release(&mut root, &env, &mut out);
    crate::release(&mut root, &env, &mut out);
    let Some(Request::Party(PartyRequest::Deliver(Delivery::WebReply { to, reply: people::Reply::Refused(why) }))) =
        out.pop()
    else {
        panic!("charter refused at the settings entrance");
    };
    assert_eq!(to.into_token(), REPLY);
    assert!(out.is_empty());
    why
}
fn judged() -> people::Refusal {
    people::Refusal::Judged { at: 1, setting: Setting::Charter(CHARTER), why: WORDS.into() }
}

#[test]
fn a_partys_change_counts_every_part() {
    // Instructions alone leave 4,000 bytes; the part and its blank line leave 998.
    let host = crate::room::HostPart { bound: 5_001, ..HOST };
    let mut charter = short_charter();
    let item = |charter| Box::new([Item {
        setting: Setting::Charter(CHARTER), edit: Edit::Set(Value::Charter(Box::new(charter))),
    }]) as Box<[_]>;
    let (mut root, env) = owner_root();
    root.host_part = Some(host);
    charter.parts = parts(3_000);
    assert_eq!(
        settled(&mut root, &env, with_items(change(), item(charter.clone()))),
        people::Reply::Refused(people::Refusal::Judged {
            at: 0, setting: Setting::Charter(CHARTER),
            why: b"charter 1 leaves its brief 998 bytes of its host part's 5,001; it needs at least 2,048".as_slice().into(),
        })
    );
    charter.parts = Box::new([]);
    assert_eq!(settled(&mut root, &env, with_items(change(), item(charter))),
        people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 }));
}

#[test]
fn party_settings_judge_overlays_over_the_default() {
    let default_bytes = u32::try_from(resume_fixture::charter().policy.instructions.len()).unwrap();
    let added = SHORT_INSTRUCTIONS - default_bytes - 2;
    for propose in [false, true] {
        for (overlay, short) in [
            (overlay(added, None), true),
            (overlay(0, Some(added)), true),
            (overlay(added - 1, None), false),
            (overlay(0, Some(added - 1)), false),
            (overlay(0, None), false),
        ] {
            let (mut root, env) = owner_root();
            root.host_part = Some(HOST);
            let input = with_items(if propose { proposal() } else { change() }, overlay_items(overlay));
            let reply = settled(&mut root, &env, input);
            if short {
                assert_eq!(reply, people::Reply::Refused(people::Refusal::Judged {
                    at: 0, setting: Setting::Charter(CHARTER), why: WORDS.into(),
                }));
            } else if propose {
                assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Proposed { .. })), "{reply:?}");
            } else {
                assert_eq!(reply, people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 }));
            }
        }
    }
}

#[test]
fn a_stored_overlay_is_judged_over_the_new_default_after_restore() {
    let (mut root, env) = super::resume::at_phase(false, core::start::Phase::LoadCore);
    root.host_part = Some(HOST);
    let default = resume_fixture::charter();
    let added = SHORT_INSTRUCTIONS - u32::try_from(default.policy.instructions.len()).unwrap() - 2;
    let overlay = overlay(0, Some(added));
    let mut old_default = default.clone();
    old_default.policy.instructions = Box::new([]);
    let old = settings::overlaid(&old_default, &overlay);
    // Under the old default the overlay left more than the brief floor.
    assert!(HOST.room(&old.policy) >= crate::room::BRIEF_ROOM);
    crate::restart::restore_page_row(&mut root, &env, crate::Record::Core(core::Record::Core(
        core::CoreRecord::Settings(settings::Record::Changed {
            setting: Setting::Charter(CHARTER),
            changed: settings::Changed { value: Some(Value::Overlay(Box::new(overlay))), revision: 1 },
        })
    )));
    // A persisted head makes this a restored settings set, not the first start.
    crate::restart::restore_page_row(&mut root, &env, crate::Record::Core(core::Record::Core(
        core::CoreRecord::Settings(settings::Record::Revision {
            history: false,
            revision: settings::Revision {
                head: settings::Head { number: 1, against: 0, at: Wall::EPOCH, by: settings::Decider::FirstStart, proposal: None, moved: false },
                items: Box::new([]), applications: Box::new([]),
            },
        })
    )));
    root.work.push(crate::Work::StartDone(core::start::Phase::LoadCore));
    let mut decision = crate::Decision::decide(&mut root.journal, &env.limits.journal);
    crate::route::route_into(&mut root, &env, &mut decision);
    assert_eq!(root.start_stop(), Some(crate::StartStop::Room(crate::room::Short {
        charter: CHARTER, room: 2_047, bound: HOST.bound,
    })));
    assert!(!root.ready());
}

#[test]
fn a_partys_change_refuses_the_short_charter_item() {
    assert_eq!(refused(change()), judged());
}

#[test]
fn a_partys_proposal_refuses_the_short_charter_item() {
    assert_eq!(refused(proposal()), judged());
}

#[test]
fn a_reset_carries_its_previously_admitted_value() {
    let (mut root, env) = owner_root();
    let _changed = settled(&mut root, &env, change());
    // Even the default leaves zero bytes under this new host bound.
    root.host_part = Some(crate::room::HostPart { bound: 0, ..HOST });
    let reply = settled(
        &mut root,
        &env,
        PartyInput::ChangeSettings {
            reply_to: ReplyTo::new(REPLY),
            sign_in: 1,
            key: [3; 16],
            digest: [4; 32],
            kinds: Box::new([]),
            change: Box::new(settings::Change {
                against: 2,
                items: Box::new([Item { setting: Setting::Charter(CHARTER), edit: Edit::Reset }]),
                running: settings::Running::Stays,
            }),
        },
    );
    assert_eq!(reply, people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 3 }));
    assert_eq!(root.core.current.charter(CHARTER), Some(&resume_fixture::charter()));
}

#[test]
fn a_root_without_an_agent_checks_no_room() {
    let (mut root, env) = owner_root();
    assert_eq!(
        settled(&mut root, &env, change()),
        people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })
    );
    assert_eq!(
        root.core.current.charter(CHARTER).unwrap().policy.instructions.len(),
        usize::try_from(SHORT_INSTRUCTIONS).unwrap()
    );
}

fn changed_row() -> crate::Record {
    crate::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Changed {
        setting: Setting::Charter(CHARTER),
        changed: settings::Changed { value: Some(Value::Charter(Box::new(short_charter()))), revision: 1 },
    })))
}

#[test]
fn a_stored_short_charter_refuses_before_the_next_start_phase() {
    let (mut root, env) = super::resume::at_phase(false, core::start::Phase::LoadCore);
    root.host_part = Some(HOST);
    crate::restart::restore_page_row(&mut root, &env, changed_row());
    root.work.push(crate::Work::StartDone(core::start::Phase::LoadCore));
    let mut decision = crate::Decision::decide(&mut root.journal, &env.limits.journal);
    crate::route::route_into(&mut root, &env, &mut decision);
    let mut out = Queue::uncapped();
    crate::release(&mut root, &env, &mut out);
    assert_eq!(
        root.start_stop(),
        Some(crate::StartStop::Room(crate::room::Short { charter: CHARTER, room: 2_047, bound: 3_048 }))
    );
    assert_eq!(root.start_phase(), Some(core::start::Phase::LoadCore));
    assert!(!root.ready());
    assert_eq!(out.pop(), Some(Request::Stop));
    crate::release(&mut root, &env, &mut out);
    assert!(out.is_empty(), "the retained refusal stops once");
}

#[test]
fn stored_charters_without_an_agent_do_not_check_room() {
    let (mut root, env) = super::resume::at_phase(false, core::start::Phase::LoadCore);
    crate::restart::restore_page_row(&mut root, &env, changed_row());
    root.work.push(crate::Work::StartDone(core::start::Phase::LoadCore));
    let mut decision = crate::Decision::decide(&mut root.journal, &env.limits.journal);
    crate::route::route_into(&mut root, &env, &mut decision);
    assert_eq!(root.start_stop(), None);
    assert_ne!(root.start_phase(), Some(core::start::Phase::LoadCore));
}

fn run_root() -> (Domain, Env<Limits>, u64) {
    let (source, _, task) = super::live_task(None, jig_domain_tasks::Executor::Agent { charter: CHARTER });
    let mut row = source.core.tasks.task(task).unwrap().clone();
    row.authority.tools = core::RunPolicy::EVERY_FAMILY;
    let mut config = resume_fixture::config(7);
    config.host_part = Some(HOST);
    config.core.defaults.rules.ceiling.tools.0 |= core::ToolFamilies::JIG.settings;
    config.core.seed_projects[0].policy.policy.ceiling.tools.0 |= core::ToolFamilies::JIG.settings;
    let (mut root, env) = resume_fixture::root_with_records(
        config,
        &resume_fixture::limits(),
        &[
            crate::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Ledger(
                *source.core.tasks.funding(row.funder).unwrap(),
            ))),
            crate::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(row)))),
        ],
    );
    root.core
        .proofs
        .insert(
            task,
            core::RunProof {
                task,
                attempt: 1,
                conversation: 1,
                credentials: Box::new([]),
                accounts: Box::new([]),
                placement: Some(Token::new(7)),
                account_failure: None,
                account_terminal: false,
                window: None,
                crossed: false,
                host: jig_domain_fleet::HostKind::Worker,
                offered: None,
                outstanding: Box::new([]),
                turn: None,
                terminal: None,
            },
        )
        .unwrap();
    (root, env, task)
}

#[test]
fn a_runs_proposal_judges_overlays_over_the_default() {
    let default_bytes = u32::try_from(resume_fixture::charter().policy.instructions.len()).unwrap();
    let added = SHORT_INSTRUCTIONS - default_bytes - 2;
    for (overlay, short) in [
        (overlay(added, None), true),
        (overlay(0, Some(added)), true),
        (overlay(added - 1, None), false),
        (overlay(0, Some(added - 1)), false),
        (overlay(0, None), false),
    ] {
        use crate::route::host_route::{Flight, FlightRender};
        let (mut root, env, task) = run_root();
        let key = crate::CallKey { task, attempt: 1, completion: 1, position: 0 };
        root.call_renders.insert(REPLY, Flight {
            task, attempt: 1, key: Some(key), note_decoding: false, job: None, page: None,
            render: FlightRender::Plain, name: b"proposal".as_slice().into(), tool: b"propose_settings".as_slice().into(),
        }).unwrap();
        let mut decision = crate::Decision::decide(&mut root.journal, &env.limits.journal);
        crate::route::relay_payload(&mut root, &env, &mut decision, ReplyTo::new(REPLY), key, crate::Call {
            completion: 1, position: 0,
            tool: crate::Tool::ProposeSettings {
                more: false, against: 1, items: overlay_items(overlay), reason: b"an overlay".as_slice().into(), kinds: Box::new([]),
            },
        });
        if short {
            let Some(crate::Work::Fleet(jig_domain_fleet::Event::Relayed { to, answer })) = root.work.pop() else {
                panic!("the short overlay is refused at the run's entrance");
            };
            assert_eq!(to.into_token(), REPLY);
            let Some(crate::Payload::SettledCall(call)) = root.hub_link.take_payload(answer) else {
                panic!("a settled host answer");
            };
            assert_eq!(call.answer, core::SettledAnswer::Host { error: true, body: WORDS.into() });
        } else {
            assert!(matches!(root.work.pop(), Some(crate::Work::Core(core::Event::NamedAction {
                action: core::NamedAction::ProposeSettings { .. }, ..
            }))), "the fitting overlay reaches jig");
        }
        assert!(root.work.is_empty());
    }
}

#[test]
fn a_runs_short_proposal_is_answered_and_its_next_call_is_decided() {
    use crate::route::host_route::{Flight, FlightRender};
    let (mut root, env, task) = run_root();
    let key = crate::CallKey { task, attempt: 1, completion: 1, position: 0 };
    root.call_renders
        .insert(
            REPLY,
            Flight {
                task,
                attempt: 1,
                key: Some(key),
                note_decoding: false,
                job: None,
                page: None,
                render: FlightRender::Plain,
                name: b"proposal".as_slice().into(),
                tool: b"propose_settings".as_slice().into(),
            },
        )
        .unwrap();
    let mut decision = crate::Decision::decide(&mut root.journal, &env.limits.journal);
    crate::route::relay_payload(
        &mut root,
        &env,
        &mut decision,
        ReplyTo::new(REPLY),
        key,
        crate::Call {
            completion: 1,
            position: 0,
            tool: crate::Tool::ProposeSettings {
                more: false,
                against: 1,
                items: items(),
                reason: b"a charter".as_slice().into(),
                kinds: Box::new([]),
            },
        },
    );
    let Some(crate::Work::Fleet(jig_domain_fleet::Event::Relayed { to, answer })) = root.work.pop() else {
        panic!("the host error goes straight back to the run");
    };
    assert_eq!(to.into_token(), REPLY);
    let Some(crate::Payload::SettledCall(call)) = root.hub_link.take_payload(answer) else {
        panic!("a settled host answer");
    };
    assert_eq!(call.answer, core::SettledAnswer::Host { error: true, body: WORDS.into() });
    assert!(!root.core.call_parts.contains_key(&key));
    assert_eq!(root.core.current.newest(), 1);
    let next = crate::CallKey { completion: 2, ..key };
    crate::route::relay_payload(
        &mut root,
        &env,
        &mut decision,
        ReplyTo::new(REPLY),
        next,
        crate::Call { completion: 2, position: 0, tool: crate::Tool::Unavailable },
    );
    crate::route::route_into(&mut root, &env, &mut decision);
    assert_eq!(root.core.call_parts.get(&next), Some(&core::CallPart::Unavailable));
    assert!(root.core.current_proof(task, 1));
    assert!(root.core.tasks.task(task).is_some());
}

fn owner_root() -> (Domain, Env<Limits>) {
    let mut config = resume_fixture::config(7);
    config.core.owners[0].project = people::DEPLOYMENT;
    let records = [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: config.core.owners[0].identity.clone(),
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::SignIn { number: 1, person: 1, expires: Wall::from_nanos(100) },
        people::Stored::Roles {
            project: people::DEPLOYMENT,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ]
    .map(core::Record::People)
    .map(crate::Record::Core);
    resume_fixture::root_with_records(config, &resume_fixture::limits(), &records)
}

// Release the root's decisions against a durable store; the party is the only outside reader.
#[expect(clippy::wildcard_enum_match_arm, reason = "this fixture only commits settings and receives party replies")]
fn settled(root: &mut Domain, env: &Env<Limits>, input: PartyInput) -> people::Reply {
    // Each change needs fewer than 64 journal and child continuation handoffs.
    const HANDOFFS: u32 = 64;
    crate::try_step(root, env, Event::Party(input)).unwrap();
    let mut out = Queue::uncapped();
    for _ in 0..HANDOFFS {
        crate::release(root, env, &mut out);
        if let Some(request) = out.pop() {
            match request {
                Request::Store(crate::StoreRequest::Commit { number, .. }) => {
                    crate::try_step(root, env, Event::Store(crate::StoreInput::Committed { number, fill: 0 })).unwrap();
                }
                Request::Party(PartyRequest::Deliver(Delivery::WebReply { reply, .. })) => return reply,
                other => panic!("unexpected settings output: {other:?}"),
            }
        }
    }
    panic!("settings reply did not settle");
}

#[test]
fn an_accepted_stored_proposal_keeps_its_admitted_charter() {
    let (mut root, env) = owner_root();
    let people::Reply::Outcome(people::Outcome::Proposed { proposal: number, .. }) =
        settled(&mut root, &env, proposal())
    else {
        panic!("the proposal is stored");
    };
    root.host_part = Some(HOST);
    let reply = settled(
        &mut root,
        &env,
        PartyInput::Ask {
            reply_to: ReplyTo::new(REPLY),
            sign_in: 1,
            key: [3; 16],
            models: Box::new([]),
            ask: people::Ask::DecideProposal {
                project: people::DEPLOYMENT,
                proposer: people::Proposer::Party(1),
                proposal: number,
                decision: people::ProposalDecision::Accept,
                running: Some(people::Running::Stays),
            },
        },
    );
    let people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. }) =
        reply
    else {
        panic!("the stored proposal is accepted: {reply:?}");
    };
    assert_eq!(
        root.core.current.charter(CHARTER).unwrap().policy.instructions.len(),
        usize::try_from(SHORT_INSTRUCTIONS).unwrap()
    );
}
