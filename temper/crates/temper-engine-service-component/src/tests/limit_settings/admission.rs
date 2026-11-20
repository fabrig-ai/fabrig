//! Admission and live takes use the start's resolver (temper/docs/design/protocol/configuration/limit-settings.md, 4).
use super::{build, inputs, item, opening, reading};
use crate::limit_settings::{LimitSettings, admission, resolve_limits};
use crate::tests::fixtures;
use jig_domain_core::{self as core, settings as s};
use jig_domain_people as people;
use skein_lib::{Queue, ReplyTo, Token};
use smith_channel::Record;
use temper_engine_domain as root;
use temper_engine_domain_forge::settings as f;

const REPLY: Token = Token::new(301);
const TEMPER: s::Setting = s::Setting::Application { kind: 22, scope: 0, number: 1 };

fn state() -> LimitSettings {
    let inputs = inputs();
    let in_force = resolve_limits(&inputs, &[], &[]).unwrap();
    LimitSettings {
        inputs,
        jig: Box::new([]),
        temper: Box::new([]),
        jig_taken: Box::new([]),
        temper_taken: Box::new([]),
        in_force,
        revision: 1,
        held: false,
    }
}
pub(in crate::tests) fn jig(values: Box<[s::Override]>) -> s::Item {
    s::Item { setting: s::Setting::Limits, edit: s::Edit::Set(s::Value::Limits(values)) }
}
pub(in crate::tests) fn temper(values: Box<[s::Override]>) -> s::Item {
    s::Item {
        setting: TEMPER,
        edit: s::Edit::Set(s::Value::Application(temper_engine_smith::kinds::encode(&f::Value::Limits(values)))),
    }
}
pub(in crate::tests) fn party(items: Box<[s::Item]>, propose: bool, against: u32) -> root::Event {
    let event = if propose {
        core::Event::ProposeSettings {
            reply_to: ReplyTo::new(REPLY),
            sign_in: 1,
            key: [u8::try_from(against).unwrap(); 16],
            digest: [u8::try_from(against).unwrap(); 32],
            change: Box::new(s::Proposed { against, items, reason: b"room for work".as_slice().into() }),
        }
    } else {
        core::Event::ChangeSettings {
            reply_to: ReplyTo::new(REPLY),
            sign_in: 1,
            key: [u8::try_from(against).unwrap(); 16],
            digest: [u8::try_from(against).unwrap(); 32],
            change: Box::new(s::Change { against, items, running: s::Running::Stays }),
        }
    };
    root::Event::Party(crate::clients::translation::to_root(event).unwrap())
}
fn refused(state: &LimitSettings, items: Box<[s::Item]>, propose: bool) -> people::Refusal {
    let result = admission::party(Some(state), party(items, propose, state.revision)).unwrap_err();
    let jig_protocol_engine_clients::Event::Reply { to, reply: people::Reply::Refused(why) } = *result else {
        panic!("judged reply")
    };
    assert_eq!(to, ReplyTo::new(REPLY));
    why
}
fn judged(why: people::Refusal, at: u32, setting: s::Setting) -> Box<[u8]> {
    let people::Refusal::Judged { at: actual, setting: named, why } = why else { panic!("judged refusal: {why:?}") };
    assert_eq!((actual, named), (at, setting));
    why
}
fn stock_failure(state: &LimitSettings, jig: &[s::Override], temper: &[s::Override]) -> Box<[u8]> {
    temper_engine_service_startup::startup::lines(&resolve_limits(&state.inputs, jig, temper).unwrap_err())
}

#[test]
fn parties_changes_and_proposals_use_the_starts_words_and_item_position() {
    let state = state();
    for propose in [false, true] {
        for temper_kind in [false, true] {
            let values: Box<[s::Override]> = if temper_kind {
                Box::new([item(b"forge.entries", 0)])
            } else {
                Box::new([item(b"people.inbox_tasks", 1)])
            };
            let expected =
                if temper_kind { stock_failure(&state, &[], &values) } else { stock_failure(&state, &values, &[]) };
            let bad = if temper_kind { temper(values) } else { jig(values) };
            let setting = bad.setting;
            let why =
                refused(&state, Box::new([s::Item { setting: s::Setting::Rules, edit: s::Edit::Reset }, bad]), propose);
            assert_eq!(judged(why, 1, setting), expected);
            assert!(
                String::from_utf8_lossy(&expected).contains(if temper_kind {
                    "forge.entries (setting temper-limits):"
                } else {
                    "people.inbox_tasks (setting limits):"
                }),
                "{}",
                String::from_utf8_lossy(&expected)
            );
        }
    }
}

#[test]
fn names_are_checked_at_entry_even_when_the_start_would_leave_them_out() {
    let state = state();
    for (temper_kind, name, expected) in [
        (false, b"tasks.gone".as_slice(), b"tasks.gone is no limit of this release".as_slice()),
        (true, b"forge.gone", b"forge.gone is no limit of this release"),
        (false, b"forge.entries", b"forge.entries is a limit of temper-limits, not limits"),
        (true, b"tasks.tasks", b"tasks.tasks is a limit of limits, not temper-limits"),
    ] {
        let bad = if temper_kind { temper(Box::new([item(name, 1)])) } else { jig(Box::new([item(name, 1)])) };
        let setting = bad.setting;
        assert_eq!(judged(refused(&state, Box::new([bad]), false), 0, setting).as_ref(), expected);
    }
}

fn call(input: &[u8]) -> root::HostRequest {
    let name = smith_channel::CallName { activation: 1, completion: 2, position: 1 };
    let name = name.encode(&smith_channel::CEILINGS).expect("fixed call name");
    root::HostRequest::Decode {
        to: ReplyTo::new(REPLY),
        task: 1,
        attempt: 1,
        call: jig_domain_fleet::Call {
            contract: 1,
            name,
            tool: b"propose_settings".as_slice().into(),
            input: input.into(),
            writes: true,
            deadline: skein_lib::Duration::from_secs(1),
        },
    }
}
fn decode(service: &mut crate::Service, input: &[u8]) -> root::Event {
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.events.is_empty());
    crate::route::request(engine, root::Request::Worker(root::WorkerRequest::Host(Box::new(call(input)))));
    engine.events.pop().unwrap()
}
fn standalone() -> crate::Service {
    let mut service = fixtures::start_with_store(&[(1, 100)], None);
    fixtures::restart(&mut service);
    service.engine.as_mut().unwrap().limits_in_force = Some(Box::new(state()));
    service
}

#[test]
fn run_parts_are_checked_separately_before_the_root_holds_them() {
    let mut service = standalone();
    for kind in ["limits", "temper-limits"] {
        for more in [false, true] {
            let (name, value) = if kind == "limits" { ("people.inbox_tasks", 1) } else { ("forge.entries", 0) };
            let input = serde_json::to_vec(&serde_json::json!({"against":1_u32,"reason":"room","more":more,"items":[{"setting":{"kind":kind},"set":[{"name":name,"value":value,"reason":"room"}]}]})).unwrap();
            let event = decode(&mut service, &input);
            let root::Event::Worker(root::WorkerInput::RenderedHostCall {
                to,
                answer: core::SettledAnswer::Host { error, body },
            }) = event
            else {
                panic!("tool refusal: {event:?}")
            };
            assert_eq!(to, ReplyTo::new(REPLY));
            assert!(error);
            let state = service.engine.as_ref().unwrap().limits_in_force().unwrap();
            let overrides = [item(name.as_bytes(), value)];
            let expected = if kind == "limits" {
                stock_failure(state, &overrides, &[])
            } else {
                stock_failure(state, &[], &overrides)
            };
            assert_eq!(body, expected);
        }
    }
    let event = decode(&mut service, br#"{"against":1,"reason":"room","more":true,"items":[{"setting":{"kind":"limits"},"set":[{"name":"tasks.tasks","value":1024,"reason":"room"}]}]}"#);
    let root::Event::Worker(root::WorkerInput::DecodedHostCall { .. }) = event else {
        panic!("valid part stays decoded: {event:?}")
    };
    // A subsequent part is judged on its own, not merged with this unsaved one.
    let event = decode(&mut service, br#"{"against":1,"reason":"room","more":false,"items":[{"setting":{"kind":"limits"},"set":[{"name":"people.inbox_tasks","value":1536,"reason":"room"}]}]}"#);
    let root::Event::Worker(root::WorkerInput::DecodedHostCall { .. }) = event else {
        panic!("valid part stays decoded: {event:?}")
    };
}

#[test]
fn non_limit_requests_bypass_resolution_and_limit_items_replace_the_whole_list() {
    let mut state = state();
    state.jig = Box::new([item(b"tasks.tasks", 1024)]);
    let replacement = [jig(Box::new([item(b"people.inbox_tasks", 1536)]))];
    assert!(state.check(&replacement, &[]).is_none(), "no merge by name");
    let kept = [jig(Box::new([item(b"people.inbox_tasks", 1536), item(b"tasks.tasks", 1024)]))];
    assert!(state.check(&kept, &[]).is_some());
    let reset = [s::Item { setting: s::Setting::Limits, edit: s::Edit::Reset }];
    assert!(state.check(&reset, &[]).is_none());
    // A held start can retain invalid settings; unrelated requests still do not resolve them.
    state.temper = Box::new([item(b"forge.entries", 0)]);
    state.held = true;
    let event = party(Box::new([s::Item { setting: s::Setting::Rules, edit: s::Edit::Reset }]), false, 1);
    admission::party(Some(&state), event).unwrap();
    let acceptance = root::Event::Party(root::PartyInput::Ask {
        reply_to: ReplyTo::new(REPLY),
        sign_in: 1,
        key: [1; 16],
        models: Box::new([]),
        ask: people::Ask::DecideProposal {
            project: 0,
            proposer: people::Proposer::Party(1),
            proposal: 1,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    });
    assert!(admission::party(Some(&state), acceptance).is_ok(), "proposal acceptance has no second check");
}

fn owned() -> Box<crate::Service> {
    owned_with_held(false)
}

pub(in crate::tests) fn owned_with_held(held: bool) -> Box<crate::Service> {
    let mut input = inputs();
    let mut document: serde_json::Value = serde_json::from_slice(&input.document).unwrap();
    document["engine"]["owners"] = serde_json::json!([{"provider":1_u32,"subject":"owner-a"}]);
    input.document = serde_json::to_vec(&document).unwrap().into_boxed_slice();
    let mut store = opening(&input, None);
    let held = if held { Some((&[][..], &[][..])) } else { None };
    let state = store.resolve_settings(input, held).unwrap();
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    assert_eq!(
        service.engine.as_ref().unwrap().limits_in_force().unwrap().revision,
        1,
        "the restored revision is known before a party input"
    );
    service.engine.as_mut().unwrap().step(root::Event::Party(root::PartyInput::SignedIn {
        reply_to: ReplyTo::new(REPLY),
        held: None,
        until: None,
        identity: people::Identity {
            key: people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    service
}

#[test]
fn a_hundred_goals_is_accepted_for_restart_but_a_hundred_and_one_is_refused() {
    let mut service = owned();
    let engine = service.engine.as_mut().unwrap();
    let before = engine.limits_in_force().unwrap().in_force.root;
    let change = party(Box::new([temper(Box::new([item(b"live_goals", 100)]))]), false, 1);
    let change = admission::party(engine.limits_in_force(), change).expect("100 is admitted at entry");
    engine.step(change);
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    let state = service.engine.as_ref().unwrap().limits_in_force().unwrap();
    assert_eq!(state.revision, 2);
    assert_eq!(state.temper.as_ref(), &[item(b"live_goals", 100)]);
    assert_eq!(state.in_force.temper.live_goals, 40, "a raise waits for the next start");
    assert_eq!(state.in_force.root, before);
    let input = service.engine.as_mut().unwrap().limits_in_force.take().unwrap().inputs;
    let mut store = opening(&input, Some(super::kept(service)));
    let state = store.resolve_settings(input, None).unwrap();
    assert_eq!(state.in_force.temper.live_goals, 100);
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    let state = service.engine.as_ref().unwrap().limits_in_force().unwrap();
    assert_eq!(state.in_force.temper.live_goals, 100);
    for propose in [false, true] {
        let why = refused(state, Box::new([temper(Box::new([item(b"live_goals", 101)]))]), propose);
        assert_eq!(judged(why, 0, TEMPER).as_ref(), b"live_goals (setting temper-limits): 101, past its bound of 100, the most live goals the store is sized for; put limits-held in the data directory to start at the stock\n");
    }
}

#[test]
fn durable_words_update_the_service_lists_but_not_its_running_figures() {
    let mut service = owned();
    let engine = service.engine.as_mut().unwrap();
    assert_eq!(engine.limits_in_force().unwrap().revision, 1);
    let before = engine.limits_in_force().unwrap().in_force.jig.core.tasks.tasks;
    let change = party(Box::new([jig(Box::new([item(b"tasks.tasks", 1024)]))]), false, 1);
    engine.step(change);
    assert_eq!(engine.limits_in_force().unwrap().revision, 1, "not durable yet");
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    let engine = service.engine.as_mut().unwrap();
    let state = engine.limits_in_force().unwrap();
    assert_eq!(state.revision, 2);
    assert_eq!(state.jig.as_ref(), &[item(b"tasks.tasks", 1024)]);
    assert!(state.temper.is_empty());
    assert_eq!(state.in_force.jig.core.tasks.tasks, before, "taking a raise at once is separate");
    let why = refused(
        state,
        Box::new([jig(Box::new([item(b"people.inbox_tasks", 1536), item(b"tasks.tasks", 1024)]))]),
        false,
    );
    assert!(
        String::from_utf8_lossy(&judged(why, 0, s::Setting::Limits)).contains("people.inbox_tasks (setting limits):")
    );
    admission::party(Some(state), party(Box::new([jig(Box::new([item(b"people.inbox_tasks", 1536)]))]), false, 2))
        .unwrap();
    // Removing the entry check would let this invalid revision reach the root.
    let head = engine.domain.settings_revision();
    engine.step(party(Box::new([temper(Box::new([item(b"forge.entries", 0)]))]), false, 2));
    assert_eq!(engine.domain.settings_revision(), head);
    assert_eq!(engine.domain.commits_in_flight(), 0);
}

fn lowered_start() -> Box<crate::Service> {
    let values: Box<[s::Override]> = Box::new([
        item(b"forge.client.entries", 8),
        item(b"forge.entries", 8),
        item(b"forge.holds", 2),
        item(b"forge.landing_members", 2),
        item(b"forge.landings", 1),
        item(b"forge.tasks", 2),
        item(b"transcript_memory", 1024),
    ]);
    restart_with_temper_limits(values)
}

fn restart_with_temper_limits(values: Box<[s::Override]>) -> Box<crate::Service> {
    let mut service = owned();
    service.engine.as_mut().unwrap().step(party(Box::new([temper(values)]), false, 1));
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    let input = service.engine.as_mut().unwrap().limits_in_force.take().unwrap().inputs;
    let mut store = opening(&input, Some(super::kept(service)));
    let state = store.resolve_settings(input, None).unwrap();
    let mut service = build(store, state);
    fixtures::restart(&mut service);
    service
}

#[test]
fn durable_raises_reach_the_running_environment_and_in_force_together() {
    let mut service = lowered_start();
    let engine = service.engine.as_mut().unwrap();
    let before = engine.env.limits;
    let values: Box<[s::Override]> = Box::new([
        item(b"forge.client.entries", 16),
        item(b"forge.entries", 16),
        item(b"forge.holds", 4),
        item(b"forge.landing_members", 2),
        item(b"forge.landings", 1),
        item(b"forge.tasks", 4),
        item(b"transcript_memory", 2048),
    ]);
    engine.step(party(Box::new([temper(values.clone())]), false, 2));
    assert_eq!(engine.env.limits, before, "acceptance alone cannot raise a running limit");
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    let engine = service.engine.as_ref().unwrap();
    let state = engine.limits_in_force().unwrap();
    assert_eq!(state.revision, 3);
    assert_eq!(state.temper, values);
    assert_eq!(state.temper_taken, values);
    assert_eq!(engine.env.limits, state.in_force.root);
    assert_eq!(engine.env.limits.forge.entries, 16);
    assert_eq!(engine.env.limits.forge.client.entries, 16);
    assert_eq!(engine.env.limits.forge.holds, 4);
    assert_eq!(engine.env.limits.forge.tasks, 4);
    assert_eq!(engine.env.limits.transcript_memory, 2048);
}

#[test]
fn lowerings_and_mixed_revisions_wait_whole_for_the_next_start() {
    for mixed in [false, true] {
        let mut service = lowered_start();
        let engine = service.engine.as_mut().unwrap();
        let before = engine.env.limits;
        let issues = engine.limits_in_force().unwrap().in_force.temper.forge.issues;
        let mut values = vec![item(b"forge.client.entries", 8)];
        values.push(item(b"forge.entries", if mixed { 16 } else { 4 }));
        values.push(item(b"forge.holds", 2));
        if mixed {
            values.push(item(b"forge.issues", u64::from(issues) + 1));
        }
        values.push(item(b"forge.landing_members", 2));
        values.push(item(b"forge.landings", 1));
        values.push(item(b"forge.tasks", 2));
        values.push(item(b"transcript_memory", 1024));
        if mixed {
            values[0] = item(b"forge.client.entries", 16);
        }
        engine.step(party(Box::new([temper(values.clone().into_boxed_slice())]), false, 2));
        fixtures::settle_root(&mut service, &mut Queue::uncapped());
        let engine = service.engine.as_ref().unwrap();
        let state = engine.limits_in_force().unwrap();
        assert_eq!(state.revision, 3);
        assert_eq!(engine.env.limits, before);
        assert_eq!(state.in_force.root, before);
        let next = resolve_limits(&state.inputs, &state.jig, &state.temper).unwrap();
        assert_eq!(next.root.forge.entries, if mixed { 16 } else { 4 });
        assert_eq!(next.root.forge.issues, if mixed { issues + 1 } else { issues });
    }
}

const REMOTE: &[u8] = b"checkout.remote_timeout_ms";

#[test]
fn durable_live_timeout_raises_and_lowers_whole() {
    let mut service = owned();
    let engine = service.engine.as_ref().unwrap();
    let before = engine.env.limits;
    assert_eq!(engine.limits_in_force().unwrap().in_force.temper.checkout.remote_timeout.as_millis(), 60_000);
    for (against, timeout) in [(1, 600_000), (2, 30_000)] {
        let values: Box<[_]> = Box::new([item(REMOTE, timeout)]);
        service.engine.as_mut().unwrap().step(party(Box::new([temper(values.clone())]), false, against));
        fixtures::settle_root(&mut service, &mut Queue::uncapped());
        let engine = service.engine.as_ref().unwrap();
        let state = engine.limits_in_force().unwrap();
        assert_eq!(state.revision, against + 1);
        assert_eq!(
            state.in_force.temper.checkout.remote_timeout.as_millis(),
            timeout,
            "the live timeout applies in either direction"
        );
        assert_eq!(state.temper_taken, values);
        assert_eq!(engine.env.limits, before);
        let row = reading::read_named(state, REMOTE);
        assert_eq!(row.in_force, Some(timeout));
        assert!(!row.waiting);
    }
}

#[test]
fn mixed_revisions_take_only_the_live_timeout() {
    let mut service = owned();
    let before = service.engine.as_ref().unwrap().env.limits;
    for (against, timeout) in [(1, 600_000), (2, 30_000)] {
        let taken = [item(REMOTE, timeout)];
        let values = Box::new([taken[0].clone(), item(b"live_goals", 60)]);
        service.engine.as_mut().unwrap().step(party(Box::new([temper(values)]), false, against));
        fixtures::settle_root(&mut service, &mut Queue::uncapped());
        let engine = service.engine.as_ref().unwrap();
        let state = engine.limits_in_force().unwrap();
        assert_eq!(state.revision, against + 1);
        assert_eq!(state.temper_taken.as_ref(), &taken, "the taken lists replace only the live override");
        let checked = resolve_limits(&state.inputs, &[], &taken).unwrap();
        assert_eq!(state.in_force.temper, checked.temper);
        assert_eq!(state.in_force.root, before);
        assert_eq!(engine.env.limits, before);
        let row = reading::read_named(state, REMOTE);
        assert_eq!(row.in_force, Some(timeout));
        assert!(!row.waiting);
        let goals = reading::read_named(state, b"live_goals");
        assert_eq!(goals.setting.as_ref().unwrap().value, 60);
        assert_eq!(goals.in_force, Some(40));
        assert!(goals.waiting);
    }
}

#[test]
fn an_invalid_live_mix_takes_nothing() {
    // The start holds checkout.message_bytes at least as high as the carrier payload.
    let high = item(b"tool_payload", 64_000);
    let mut service = restart_with_temper_limits(Box::new([high.clone()]));
    let engine = service.engine.as_mut().unwrap();
    let against = engine.domain.settings_revision();
    let state = engine.limits_in_force().unwrap();
    let before = state.in_force.temper;
    let stock = resolve_limits(&state.inputs, &[], &[]).unwrap();
    let message = item(b"checkout.message_bytes", u64::from(stock.temper.checkout.message_bytes));
    assert_eq!(state.temper_taken.as_ref(), ::core::slice::from_ref(&high));
    assert!(message.value < high.value);
    let mix = [message.clone(), high.clone()];
    let refusal = resolve_limits(&state.inputs, &[], &mix).unwrap_err();
    let line = temper_engine_service_startup::startup::lines(&refusal);
    let line = String::from_utf8_lossy(&line);
    assert!(line.contains("checkout.message_bytes") && line.contains("the source's derivation"), "{line}");
    let values: Box<[_]> = Box::new([message, item(REMOTE, 600_000)]);
    assert!(resolve_limits(&state.inputs, &[], &values).is_ok(), "the revision passes as a whole");
    engine.step(party(Box::new([temper(values.clone())]), false, against));
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    let state = service.engine.as_ref().unwrap().limits_in_force().unwrap();
    assert_eq!(state.revision, against + 1);
    assert_eq!(state.temper, values);
    assert_eq!(state.in_force.temper, before, "neither live figure is taken");
    assert_eq!(state.temper_taken.as_ref(), &[high]);
}

#[test]
fn limits_held_never_takes_a_raise() {
    let mut service = lowered_start();
    let engine = service.engine.as_mut().unwrap();
    let before = engine.env.limits;
    let state = engine.limits_in_force_mut().unwrap();
    state.held = true;
    let mut values = state.temper.to_vec();
    values.insert(0, item(REMOTE, 600_000));
    for value in &mut values {
        if value.name.as_ref() == b"transcript_memory" {
            value.value *= 2;
        }
    }
    engine.limits_committed(3, Box::new([]), values.into_boxed_slice());
    assert_eq!(engine.env.limits, before);
    let state = engine.limits_in_force().unwrap();
    assert_eq!(state.in_force.root, before);
    assert_eq!(state.in_force.temper.checkout.remote_timeout.as_millis(), 60_000);
}

#[test]
fn the_other_kind_comes_from_durable_settings_not_the_running_figures() {
    let mut service = owned();
    let change = party(Box::new([jig(Box::new([item(b"authority.projects", 12)]))]), false, 1);
    let engine = service.engine.as_mut().unwrap();
    let change = admission::party(engine.limits_in_force(), change).expect("valid first list");
    engine.step(change);
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    let engine = service.engine.as_mut().unwrap();
    let state = engine.limits_in_force().unwrap();
    assert_eq!(state.revision, 2);
    assert_eq!(state.in_force.jig.core.authority.projects, 8);
    assert!(resolve_limits(&state.inputs, &[], &[item(b"forge.judge_projects", 8)]).is_ok(), "valid over stock alone");
    let why =
        judged(refused(state, Box::new([temper(Box::new([item(b"forge.judge_projects", 8)]))]), false), 0, TEMPER);
    assert_eq!(why, stock_failure(state, &state.jig, &[item(b"forge.judge_projects", 8)]));
    engine.step(party(Box::new([temper(Box::new([item(b"forge.judge_projects", 12)]))]), false, 2));
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    let state = service.engine.as_ref().unwrap().limits_in_force().unwrap();
    assert_eq!(state.revision, 3);
    assert_eq!(state.jig.as_ref(), &[item(b"authority.projects", 12)]);
    assert_eq!(state.temper.as_ref(), &[item(b"forge.judge_projects", 12)]);
    service.engine.as_mut().unwrap().step(party(
        Box::new([s::Item { setting: s::Setting::Limits, edit: s::Edit::Reset }]),
        false,
        3,
    ));
    fixtures::settle_root(&mut service, &mut Queue::uncapped());
    let state = service.engine.as_ref().unwrap().limits_in_force().unwrap();
    assert_eq!(state.revision, 4);
    assert!(state.jig.is_empty());
    assert_eq!(state.temper.as_ref(), &[item(b"forge.judge_projects", 12)]);
    let why = judged(
        refused(state, Box::new([jig(Box::new([item(b"authority.projects", 16)]))]), false),
        0,
        s::Setting::Limits,
    );
    assert_eq!(why, stock_failure(state, &[item(b"authority.projects", 16)], &state.temper));
    let together = party(
        Box::new([
            jig(Box::new([item(b"authority.projects", 16)])),
            temper(Box::new([item(b"forge.judge_projects", 16)])),
        ]),
        false,
        4,
    );
    let before = format!("{together:?}");
    let admitted = admission::party(Some(state), together).expect("both kinds resolved together");
    assert_eq!(format!("{admitted:?}"), before, "the admitted request is unchanged");
}

#[test]
fn agents_edits_are_bounded_by_the_carrier_payload_fixed_at_start() {
    let state = state();
    let bound = state.in_force.payload;
    let setting = s::Setting::Application { kind: 21, scope: 0, number: 1 };
    for payload in [bound, bound + 1] {
        let agents = super::agents_with_payload(payload);
        let value = f::Value::AgentFigures(agents.clone());
        let item =
            s::Item { setting, edit: s::Edit::Set(s::Value::Application(temper_engine_smith::kinds::encode(&value))) };
        let kinds = [root::KindValue {
            kind: 21,
            scope: 0,
            number: 1,
            value: f::SizedValue { size: temper_engine_smith::kinds::size(&value), value },
        }];
        let items = [s::Item { setting: s::Setting::Rules, edit: s::Edit::Reset }, item];
        let refusal = state.check(&items, &kinds);
        if payload == bound {
            assert!(refusal.is_none(), "the current carrier bound is allowed");
        } else {
            let why = judged(refusal.expect("a larger call needs larger carriers"), 1, setting);
            let why = ::core::str::from_utf8(&why).unwrap();
            assert!(why.contains("agents.declared.tool_payload"));
            assert!(why.contains("raise tool_payload in temper-limits and restart first"));
        }
        let f::Value::AgentFigures(unchanged) = &kinds[0].value.value else { panic!("agents") };
        assert_eq!(unchanged, &agents, "admission never rewrites the agents profile");
    }
}
