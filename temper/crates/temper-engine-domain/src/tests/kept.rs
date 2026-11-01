//! Root handoffs for removal and reset (draft playground-reset.md, 4 and 6.2).
use super::resume_fixture::root;
use crate::route::{asks, demo, forge_route, held, kept};
use crate::{Decision, Delivery, Domain, Limits, Request, Work};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo};
use temper_engine_domain_forge as forge;

// Distinct removal, project, task and demo-account fixtures.
const FORGE: u16 = 3;
const PROJECT: u32 = 5;
const TASK: u64 = 7;
const ACCOUNT: u32 = 5;
const DEMO_ACCOUNT: u32 = 6;
// An initial send and one resend, across fresh roots for reset keys.
const SENDS: u32 = 2;
// The fixture's restart script has fewer than sixteen steps.
const RESTART_STEPS: u32 = 16;
// The task's diagnostic ceiling.
const NOTE_BYTES: usize = 256;

fn cancel() -> forge::KeptAct {
    forge::KeptAct { key: [19; 16], forge: FORGE, act: forge::KeptActKind::Cancel { project: PROJECT, task: TASK } }
}

fn outputs(domain: &mut Domain, env: &Env<Limits>, request: forge::Request) {
    let mut out = Queue::uncapped();
    out.push(request);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    forge_route::outputs(domain, env, &mut decision, &mut out);
    assert!(out.is_empty());
    assert!(decision.empty());
    crate::decision::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, true)
        .unwrap();
}

fn act(domain: &mut Domain) -> (ReplyTo, [u8; 16], people::Act) {
    let Some(Work::Core(core::Event::Act { reply_to, connector, key, act, carried })) = domain.work.pop() else {
        panic!("deployment act reaches the core");
    };
    assert_eq!(connector, domain.config.forge_connector);
    assert!(carried.is_none());
    (reply_to, key, *act)
}

fn deliveries(domain: &mut Domain, env: &Env<Limits>, decision: Decision) -> Queue<Delivery> {
    crate::decision::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, true)
        .unwrap();
    while let Some(output) = crate::decision::commit(&mut domain.journal, &env.limits.journal) {
        let crate::JournalOutput::Commit { number, .. } = output else { panic!("store commit") };
        crate::decision::committed(&mut domain.journal, number);
    }
    let mut out = Queue::uncapped();
    crate::decision::resume(&mut domain.journal, &mut out);
    let mut deliveries = Queue::uncapped();
    while let Some(output) = out.pop() {
        let crate::JournalOutput::Deliver(delivery) = output else { panic!("held delivery") };
        deliveries.push(delivery);
    }
    deliveries
}

#[test]
fn a_kept_cancel_reaches_the_core_as_the_deployments_act_with_its_key() {
    let (mut domain, env) = root();
    let mut prior = None;
    for _ in 0..SENDS {
        outputs(&mut domain, &env, forge::Request::SendKeptAct(cancel()));
        let (to, key, act) = act(&mut domain);
        let token = to.into_token();
        assert_ne!(prior, Some(token));
        prior = Some(token);
        assert_eq!(key, cancel().key);
        let people::Act::Cancel { project, task, reason } = act else { panic!("cancel") };
        assert_eq!((project, task, reason.as_ref()), (PROJECT, TASK, b"Forge removed".as_slice()));
    }
    assert!(domain.work.is_empty());
}

#[test]
fn group_and_secret_acts_go_to_serve() {
    let (mut domain, env) = root();
    outputs(
        &mut domain,
        &env,
        forge::Request::SendKeptAct(forge::KeptAct { act: forge::KeptActKind::RemoveGroup, ..cancel() }),
    );
    outputs(
        &mut domain,
        &env,
        forge::Request::SendKeptAct(forge::KeptAct { act: forge::KeptActKind::EraseSecrets, ..cancel() }),
    );
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::RemoveGroup { forge: FORGE }) = out.pop() else { panic!("remove group through the now door") };
    let Some(Request::EraseSecrets { forge: FORGE }) = out.pop() else { panic!("erase secrets through the now door") };
    assert!(out.is_empty());
    assert!(domain.work.is_empty());
    // Serve's answers use the ordinary Forge input.
    for answer in [forge::Event::GroupRemoved { forge: FORGE }, forge::Event::SecretsErased { forge: FORGE }] {
        crate::try_step(&mut domain, &env, crate::Event::Forge(Box::new(answer))).unwrap();
    }
    assert!(!domain.journal.stopped());
}

#[test]
fn reset_acts_carry_keys_derived_from_their_numbers() {
    // Independently calculated SHA-256 prefixes, including an eight-byte number.
    const PROJECT_KEY: [u8; 16] = [236, 215, 170, 57, 151, 155, 15, 137, 105, 94, 224, 253, 108, 144, 157, 134];
    const ACCOUNT_KEY: [u8; 16] = [117, 62, 218, 17, 233, 174, 196, 188, 5, 44, 24, 140, 156, 1, 213, 170];
    for (reset, expected, expected_act) in [
        (
            forge::ResetAct::RemoveProject { project: PROJECT },
            PROJECT_KEY,
            people::Act::RemoveProject { project: PROJECT },
        ),
        (
            forge::ResetAct::RemoveAccount { account: ACCOUNT },
            ACCOUNT_KEY,
            people::Act::RemoveAccount { account: ACCOUNT },
        ),
    ] {
        for _ in 0..SENDS {
            let (mut domain, env) = root();
            outputs(&mut domain, &env, forge::Request::ResetAct(reset));
            let (_, key, act) = act(&mut domain);
            assert_eq!(key, expected);
            assert_eq!(act, expected_act);
        }
    }
}

fn answered(reset: Option<forge::ResetAct>, outcome: people::Outcome, keep: bool, why: Option<&[u8]>) {
    let (mut domain, env) = root();
    let request = match reset {
        Some(act) => forge::Request::ResetAct(act),
        None => forge::Request::SendKeptAct(cancel()),
    };
    outputs(&mut domain, &env, request);
    let (to, _, _) = act(&mut domain);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    held::held(&mut domain, &env, &mut decision, core::Held::Acted { to, outcome });
    let mut out = deliveries(&mut domain, &env, decision);
    let Some(Delivery::DemoActed { to, outcome }) = out.pop() else { panic!("durable act answer") };
    assert!(out.is_empty());
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    kept::acted(&mut domain, &env, &mut decision, to, outcome);
    match (domain.work.pop(), reset) {
        (Some(Work::Forge(forge::Event::KeptActAnswered { key, keep: sent })), None) => {
            assert_eq!(key, cancel().key);
            assert_eq!(sent, keep);
        }
        (Some(Work::Forge(forge::Event::ResetActed { act, keep: sent })), Some(reset)) => {
            assert_eq!(act, reset);
            assert_eq!(sent, keep);
        }
        other => panic!("answer reaches its connector row: {other:?}"),
    }
    assert!(domain.work.is_empty());
    let mut out = deliveries(&mut domain, &env, decision);
    if let Some(why) = why {
        let Some(Delivery::DemoNote { line }) = out.pop() else { panic!("diagnostic") };
        let prefix = if reset.is_some() { b"playground reset: ".as_slice() } else { b"forge removal: ".as_slice() };
        assert!(line.starts_with(prefix));
        let mut found = false;
        for part in line.windows(why.len()) {
            found = found || part == why;
        }
        assert!(found);
        assert!(line.len() <= NOTE_BYTES);
    }
    assert!(out.is_empty());
}

#[test]
fn again_no_grant_and_no_owner_keep_the_act_and_name_why() {
    for (reset, grant) in [
        (None, b"cancel (engine.acts)".as_slice()),
        (Some(forge::ResetAct::RemoveProject { project: PROJECT }), b"remove_project (engine.acts)".as_slice()),
        (Some(forge::ResetAct::RemoveAccount { account: ACCOUNT }), b"remove_account (engine.acts)".as_slice()),
    ] {
        for again in [
            people::Again::Busy,
            people::Again::NotReady,
            people::Again::Full { held: 1, room_at: skein_lib::Wall::EPOCH },
        ] {
            answered(reset, people::Outcome::Again(again), true, None);
        }
        answered(reset, people::Outcome::Refused(people::Refusal::NotOffered), true, Some(grant));
        answered(
            reset,
            people::Outcome::Refused(people::Refusal::Role { held: None }),
            true,
            Some(b"no first owner configured"),
        );
    }
}

#[test]
fn ended_unknown_and_compound_answers_are_done() {
    for reset in [
        None,
        Some(forge::ResetAct::RemoveProject { project: PROJECT }),
        Some(forge::ResetAct::RemoveAccount { account: ACCOUNT }),
    ] {
        answered(reset, people::Outcome::Refused(people::Refusal::Unknown), false, None);
        answered(reset, people::Outcome::Refused(people::Refusal::KeyConflict), false, Some(b"KeyConflict"));
        answered(
            reset,
            people::Outcome::Refused(people::Refusal::Role { held: Some(people::Role::Member) }),
            false,
            Some(b"Role"),
        );
    }
    answered(None, people::Outcome::Refused(people::Refusal::Ended { how: people::Ending::Done }), false, None);
    answered(None, people::Outcome::Compound { project: PROJECT, operations: 1, first_message: 0 }, false, None);
    answered(
        None,
        people::Outcome::CompoundRefused { at: 0, why: people::Refusal::KeyConflict },
        false,
        Some(b"KeyConflict"),
    );
    answered(
        Some(forge::ResetAct::RemoveProject { project: PROJECT }),
        people::Outcome::ProjectRemoved { project: PROJECT },
        false,
        None,
    );
    answered(
        Some(forge::ResetAct::RemoveAccount { account: ACCOUNT }),
        people::Outcome::AccountRemoved { confirmed: true, failed_hosts: Box::new([]) },
        false,
        None,
    );
}

#[test]
fn the_start_asks_for_kept_acts_after_the_core_opens() {
    let limits = super::resume_fixture::limits();
    let mut domain = Domain::new(super::resume_fixture::config(7), &limits).unwrap();
    let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut request = domain.core.restart_begin(false);
    for _ in 0..RESTART_STEPS {
        let core::RestartRequest::Step(step) = request else { panic!("restart step") };
        assert!(!domain.ready());
        assert!(domain.work.is_empty());
        if step == core::RestartStep::Open {
            let mut decision = Decision::decide(&mut domain.journal, &limits.journal);
            crate::restart::restart_request(&mut domain, &env, &mut decision, request);
            assert!(domain.ready());
            let Some(Work::Forge(forge::Event::SendDemoActs)) = domain.work.pop() else { panic!("demo acts first") };
            let Some(Work::Forge(forge::Event::SendKeptActs)) = domain.work.pop() else { panic!("kept acts next") };
            assert!(domain.work.is_empty());
            return;
        }
        if step == core::RestartStep::LoadInbox {
            let _requests = core::want_accounts(&mut domain.core, &crate::limits::environment_core(&env));
        }
        if step == core::RestartStep::Switch {
            super::resume_fixture::check_switch(&mut domain, &env);
        }
        request = domain.core.restart_done(step);
    }
    panic!("restart did not open");
}

#[test]
fn a_release_reaches_the_connector_and_its_answer_the_core() {
    let (mut domain, env) = root();
    let connector = domain.config.forge_connector;
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    asks::ask(&mut domain, &env, &mut decision, connector, core::Ask::ReleaseProject { project: PROJECT });
    let Some(Work::Forge(forge::Event::ReleaseProject { project: PROJECT })) = domain.work.pop() else {
        panic!("connector release")
    };
    assert!(domain.work.is_empty());
    assert!(decision.empty());
    crate::decision::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, true)
        .unwrap();
    outputs(&mut domain, &env, forge::Request::ProjectReleased { project: PROJECT });
    let Some(Work::Core(core::Event::ProjectReleased { project: PROJECT, connector: sent })) = domain.work.pop() else {
        panic!("core release acknowledgement")
    };
    assert_eq!(sent, connector);
    assert!(domain.work.is_empty());
}

#[test]
fn interleaved_demo_kept_and_reset_answers_find_their_own_maps() {
    let (mut domain, env) = root();
    demo::send_end_act(
        &mut domain,
        forge::DemoAct { key: [21; 16], act: forge::DemoActKind::RemoveAccount { account: DEMO_ACCOUNT } },
    );
    let (demo, _, _) = act(&mut domain);
    outputs(&mut domain, &env, forge::Request::SendKeptAct(cancel()));
    let (kept, _, _) = act(&mut domain);
    outputs(&mut domain, &env, forge::Request::ResetAct(forge::ResetAct::RemoveProject { project: PROJECT }));
    let (reset, _, _) = act(&mut domain);
    let tokens = [reset.into_token(), demo.into_token(), kept.into_token()];
    assert_ne!(tokens[0], tokens[1]);
    assert_ne!(tokens[1], tokens[2]);
    assert_ne!(tokens[0], tokens[2]);
    for token in tokens {
        let mut out = Queue::uncapped();
        crate::domain::step_released(
            &mut domain,
            &env,
            crate::Released(Delivery::DemoActed {
                to: ReplyTo::new(token),
                outcome: people::Outcome::Refused(people::Refusal::Unknown),
            }),
            &mut out,
        );
        assert!(!domain.journal.stopped());
        assert!(out.is_empty());
    }
    assert!(domain.demo_act_replies.is_empty());
    assert_eq!(domain.playground(), domain.forge.playground().copied());
}
