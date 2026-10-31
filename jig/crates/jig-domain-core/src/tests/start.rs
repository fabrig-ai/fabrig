//! Start selection and phase discipline (jig/docs/design/domain/engine.md, 6).
mod inbox;
mod rows;
use super::{Core, Env, Queue, Time, Wall, config, limits, tasks};
use crate::start::{Action, Failure, Phase, SweepInput};
use crate::{Key, Request, Write, people};

struct Entrance {
    name: &'static str,
    input: fn(&mut Core, &Env<crate::Limits>),
}

fn call_key() -> crate::CallKey {
    crate::CallKey { task: 1, attempt: 1, completion: 1, position: 0 }
}

fn proof() -> crate::RunProof {
    crate::RunProof {
        task: 1,
        attempt: 1,
        conversation: 1,
        credentials: Box::new([]),
        accounts: Box::new([]),
        placement: None,
        account_failure: None,
        account_terminal: false,
        window: None,
        crossed: false,
        host: crate::fleet::HostKind::Engine,
        offered: None,
        outstanding: Box::new([]),
        turn: None,
        terminal: None,
    }
}

fn transcript() -> crate::Transcript {
    crate::Transcript {
        window: None,
        closed: false,
        loaded: false,
        previous_attempt: 1,
        last: None,
        from: 1,
        bytes: 0,
        kept: 0,
        texts: Queue::uncapped(),
        text_bytes: 0,
        text_kept: 0,
        turns: Queue::uncapped(),
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table audits every public operational entrance")]
fn every_operational_entrance_consumes_selection() {
    use skein_lib::Token;
    let cases = [
        Entrance {
            name: "reserve_connector_call",
            input: |core, _| {
                assert!(core.reserve_connector_call(call_key()));
            },
        },
        Entrance {
            name: "reserve_connector_subscription",
            input: |core, _| {
                assert!(core.reserve_connector_subscription(Token::new(1), call_key(), 1));
            },
        },
        Entrance {
            name: "remember_unpriced_terminal",
            input: |core, _| {
                core.proofs.insert(1, proof()).expect("proof room");
                core.remember_unpriced_terminal(Token::new(1), Token::new(1), tasks::End::Parked);
            },
        },
        Entrance { name: "record_call", input: |core, _| core.record_call(call_key(), crate::CallPart::Controlled) },
        Entrance {
            name: "restore_call",
            input: |core, _| {
                let _accepted = core.restore_call(call_key(), crate::CallPart::Controlled);
            },
        },
        Entrance {
            name: "retire_calls",
            input: |core, _| {
                let _retired = core.retire_calls(1, 1, 1, 1);
            },
        },
        Entrance {
            name: "restore_proof",
            input: |core, env| {
                let _accepted = core.restore_proof(proof(), env.limits.tasks.result_bytes);
            },
        },
        Entrance {
            name: "restore_task_row",
            input: |core, _| {
                let _accepted = core.restore_task_row(&tasks::Stored::Stub(tasks::Stub {
                    task: 1,
                    phase: tasks::Status::Done,
                    result: Token::new(1),
                }));
            },
        },
        Entrance {
            name: "restore_core",
            input: |core, env| {
                let _accepted =
                    core.restore_core(crate::CoreRecord::Deployment(core.counters.deployment()), &env.limits);
            },
        },
        Entrance {
            name: "restore_policy",
            input: |core, env| {
                let configured = config(41);
                let value = crate::policy::snapshot(&configured.seed_projects[0].policy.policy, &[], &[])
                    .expect("bounded policy");
                let _accepted = core.restore_policy(&env.limits, 1, Box::from("Project 1"), value);
            },
        },
        Entrance {
            name: "restore_resource_role",
            input: |core, env| {
                let name = tasks::Name { connector: 1, path: Box::new([Box::from(b"resource".as_slice())]) };
                let _accepted = core.restore_resource_role(env, 1, name, crate::connector::ResourceRole::Context);
            },
        },
        Entrance {
            name: "restore_inbox_row",
            input: |core, env| {
                let _accepted = core.restore_inbox_row(
                    &env.limits,
                    &crate::Record::Tasks(tasks::Stored::Stub(tasks::Stub {
                        task: 1,
                        phase: tasks::Status::Done,
                        result: Token::new(1),
                    })),
                );
            },
        },
        Entrance {
            name: "census",
            input: |core, env| {
                let _begin = core.start_begin(false, env.wall);
                finish_core_load(core, env, &mut Queue::uncapped());
                let _census = core.census(&env.limits);
            },
        },
        Entrance {
            name: "settings_restored",
            input: |core, env| {
                assert!(core.settings_restored(env));
            },
        },
        Entrance {
            name: "refuse_settings",
            input: |core, _| {
                let _refused = core.refuse_settings(crate::settings::Failure {
                    setting: crate::settings::Setting::Rules,
                    fault: crate::settings::Fault::Shape,
                });
            },
        },
        Entrance {
            name: "restart_begin",
            input: |core, _| {
                let _step = core.restart_begin(false);
            },
        },
        Entrance {
            name: "restart_done",
            input: |core, _| {
                let _step = core.restart_done(crate::RestartStep::LoadCore);
            },
        },
        Entrance {
            name: "restart_refuse",
            input: |core, _| {
                let _refused = core.restart_refuse();
            },
        },
        Entrance {
            name: "restart_checked",
            input: |core, _| {
                assert!(!core.restart_checked());
            },
        },
        Entrance {
            name: "restart_admit_claims",
            input: |core, _| {
                assert!(!core.restart_admit_claims());
            },
        },
        Entrance {
            name: "decide_named_call",
            input: |core, _| {
                assert!(!core.decide_named_call(call_key(), crate::CallPart::Controlled));
            },
        },
        Entrance {
            name: "person_message",
            input: |core, env| {
                let _refused = core.person_message(
                    &env.limits,
                    crate::PersonMessage {
                        request: Token::new(1),
                        person: 1,
                        project: 1,
                        task: 1,
                        question: None,
                        words: Box::new([]),
                        at: env.wall,
                    },
                );
            },
        },
        Entrance {
            name: "person_task",
            input: |core, _| {
                let _refused =
                    core.person_task(Token::new(1), 1, 1, None, people::Ask::TakePerson { project: 1, task: 1 });
            },
        },
        Entrance { name: "lost_channel", input: |core, env| core.lost_channel(env, Token::new(1)) },
        Entrance {
            name: "adopt",
            input: |core, env| {
                core.proofs.insert(1, proof()).expect("proof room");
                core.adopt(env, 1, 1, 0);
            },
        },
        Entrance {
            name: "goal_refused",
            input: |core, _| {
                let _route = core.goal_refused(Token::new(1), tasks::Refusal::Unknown);
            },
        },
        Entrance {
            name: "made",
            input: |core, _| {
                let _route = core.made(Token::new(u64::MAX), Box::new([]), false);
            },
        },
        Entrance {
            name: "person_proposed",
            input: |core, _| {
                core.goal_routes
                    .insert(Token::new(1), crate::GoalRoute::Offering { proposal: 1, to: 1, stubs: Box::new([]) })
                    .expect("route room");
                let _route = core.person_proposed(Token::new(1), 1);
            },
        },
        Entrance {
            name: "person_proposal_decided",
            input: |core, env| {
                core.goal_routes
                    .insert(Token::new(1), crate::GoalRoute::Deciding { proposer: 1, proposal: 1, by: 1 })
                    .expect("route room");
                let _route =
                    core.person_proposal_decided(Token::new(1), 1, 1, tasks::ProposalOutcome::Accepted, env.wall);
            },
        },
        Entrance {
            name: "task_proposal_decided_for_person",
            input: |core, env| {
                let _route = core.task_proposal_decided_for_person(
                    Token::new(1),
                    1,
                    1,
                    tasks::ProposalOutcome::Accepted,
                    env.wall,
                    false,
                );
            },
        },
        Entrance {
            name: "notice",
            input: |core, env| {
                let _notice = core.notice(1, 1, 1, tasks::NoticeState::Done, Box::new([]), env.wall);
            },
        },
        Entrance {
            name: "notice_timer",
            input: |core, env| {
                let _notice = core.notice_timer(1, 1, env.wall);
            },
        },
        Entrance {
            name: "sent",
            input: |core, env| {
                let request = Token::new(1);
                core.saying.insert(request, (1, None)).expect("message route room");
                let word = tasks::Word {
                    number: 1,
                    from: tasks::Party::Person(1),
                    kind: tasks::MessageKind::Words,
                    words: Box::new([]),
                    at: env.wall,
                    hits: 1,
                    eligible: false,
                };
                let _route = core.sent(request, 1, &word, env.limits.tasks.inbox_messages, env.limits.tasks.tasks);
            },
        },
        Entrance {
            name: "repair_seed",
            input: |core, _| {
                let _seed = core.repair_seed(1);
            },
        },
        Entrance {
            name: "connector_adopt_parties",
            input: |core, env| {
                let _seed = core.connector_adopt_parties(&env.limits.people, 1, 1, Box::new([]));
            },
        },
        Entrance {
            name: "connector_news",
            input: |core, env| {
                let _news = core.connector_news(1, 1, tasks::NewsClass::Dropped, Box::new([]), env.wall);
            },
        },
        Entrance { name: "begin_transcript", input: |core, _| core.begin_transcript(1, 1) },
        Entrance {
            name: "append_transcript",
            input: |core, env| {
                core.transcripts.insert(1, transcript()).expect("transcript room");
                let _accepted = core.append_transcript(
                    1,
                    crate::TurnRecord {
                        opens_window: true,
                        task: 1,
                        attempt: 1,
                        turn: 1,
                        spent: 0,
                        read: None,
                        at: env.wall,
                        transcript: Box::new([]),
                    },
                    None,
                );
            },
        },
        Entrance { name: "touch_party", input: |core, _| core.touch_party(1) },
        Entrance { name: "touch_settings", input: |core, _| core.touch_settings() },
        Entrance { name: "touch_deployment", input: |core, _| core.touch_deployment() },
        Entrance {
            name: "step",
            input: |core, env| {
                let _out = crate::step(core, env, crate::Event::Fill { bytes: 0 });
            },
        },
        Entrance {
            name: "want_accounts",
            input: |core, env| {
                let _out = crate::want_accounts(core, env);
            },
        },
        Entrance {
            name: "resume_fleet",
            input: |core, env| {
                let _out = crate::resume_fleet(core, env);
            },
        },
        Entrance {
            name: "fire",
            input: |core, env| {
                let _out = crate::fire(core, env, crate::Timer::View);
            },
        },
        Entrance {
            name: "finish_decision",
            input: |core, env| {
                let _out = crate::finish_decision(core, env);
            },
        },
        Entrance {
            name: "start_begin",
            input: |core, env| {
                step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
            },
        },
        Entrance {
            name: "start_done",
            input: |core, env| {
                let _begin = core.start_begin(false, env.wall);
                finish_core_load(core, env, &mut Queue::uncapped());
            },
        },
        Entrance {
            name: "start_sweep",
            input: |core, env| {
                let _begin = core.start_begin(false, env.wall);
                let mut out = Queue::uncapped();
                through_link(core, env, &mut out);
                step_phase(core.start_sweep(env, SweepInput::Finished, &mut out), Phase::Switch);
            },
        },
        Entrance {
            name: "start_resume",
            input: |core, env| {
                let _begin = core.start_begin(false, env.wall);
                let mut out = Queue::uncapped();
                through_link(core, env, &mut out);
                let _switch = core.start_sweep(env, SweepInput::Finished, &mut out);
                let _wires = core.start_done(env, Phase::Switch, &mut out);
                let Action::Commit(token) = core.start_done(env, Phase::Wires, &mut out) else {
                    panic!("Wires requires release")
                };
                step_phase(core.start_resume(env, token, &mut out), Phase::AdoptRuns);
            },
        },
        Entrance {
            name: "start_refuse",
            input: |core, env| {
                let _begin = core.start_begin(false, env.wall);
                let _refused = core.start_refuse(Failure::ProjectPolicy { project: 1 });
            },
        },
        Entrance {
            name: "start_missing",
            input: |core, env| {
                let _begin = core.start_begin(false, env.wall);
                core.start_missing(Key::Core(crate::CoreKey::RunProof(1)), Key::Tasks(tasks::Key::Live(1)));
            },
        },
    ];
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    for case in cases {
        let mut core = Core::new(config(41), &env.limits).expect("valid core configuration");
        (case.input)(&mut core, &env);
        assert!(core.start.consumed, "{} consumes even refused or empty input", case.name);
        let late = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| core.start_begin(false, env.wall)));
        assert!(late.is_err(), "{} prevents late selection", case.name);
    }
}

#[test]
fn queries_drains_and_configuration_leave_selection_open() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    core.person_answers();
    assert_eq!(core.effect_deadline(), None);
    assert_eq!(core.start_phase(), None);
    assert_eq!(core.start_failure(), None);
    let _due = core.take_due();
    core.drain_facts(&limits);
    core.reclaim();
    step_phase(core.start_begin(false, Wall::EPOCH), Phase::LoadCore);
}

#[test]
fn refused_start_cannot_be_selected_again() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _first = core.start_begin(false, Wall::EPOCH);
    let _refused = core.start_refuse(Failure::ProjectPolicy { project: 1 });
    let repeated = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| core.start_begin(true, Wall::EPOCH)));
    let _panic = repeated.expect_err("refused start cannot be selected again");
}

#[test]
fn start_and_restart_selections_cannot_mix() {
    for completion in [false, true] {
        let limits = limits();
        let mut core = Core::new(config(41), &limits).expect("valid core configuration");
        let _begin = core.start_begin(false, Wall::EPOCH);
        let mixed = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
            if completion { core.restart_done(crate::RestartStep::LoadCore) } else { core.restart_begin(false) }
        }));
        let _panic = mixed.expect_err("start and restart selections cannot mix");
    }
}

#[test]
fn actual_stored_call_capacity_keeps_its_key() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _begin = core.start_begin(false, Wall::EPOCH);
    let capacity = limits.call_records;
    for task in 1..=u64::from(capacity) {
        assert!(core.restore_call(
            crate::CallKey { task, attempt: 1, completion: 1, position: 0 },
            crate::CallPart::Controlled
        ));
    }
    let key = crate::CallKey { task: u64::from(capacity) + 1, attempt: 1, completion: 1, position: 0 };
    assert!(!core.restore_call(key, crate::CallPart::Controlled));
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet {
            set: crate::start::LiveSet::Calls,
            key: Key::Core(crate::CoreKey::Call(key)),
            held: u64::from(capacity) + 1,
            limit: u64::from(capacity),
        })
    );
}

#[test]
#[should_panic]
#[expect(
    clippy::should_panic_without_expect,
    reason = "the selection invariant is checked without matching panic text"
)]
fn empty_finishing_is_an_input_before_selection() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _outputs = crate::finish_decision(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits });
    let _late = core.start_begin(false, Wall::EPOCH);
}

#[test]
#[should_panic]
#[expect(
    clippy::should_panic_without_expect,
    reason = "the selection invariant is checked without matching panic text"
)]
fn runtime_resume_cannot_cross_wires_early() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _begin = core.start_begin(false, Wall::EPOCH);
    let _outputs = crate::resume_fleet(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits });
}

#[test]
fn actual_account_outputs_wait_for_wires_release() {
    let limits = limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _begin = core.start_begin(false, env.wall);
    super::restore_key_account(&mut core, &limits, 1, 2, 1);
    let mut child = Queue::uncapped();
    crate::accounts::step(
        &mut core.accounts,
        &Env { now: env.now, wall: env.wall, limits: limits.accounts },
        crate::accounts::Event::Restore { record: crate::accounts::Stored::Next { number: 3 } },
        &mut child,
    );
    assert!(child.is_empty());
    let mut out = Queue::uncapped();
    through_link(&mut core, &env, &mut out);
    assert!(out.is_empty());
    assert!(!core.start.held.is_empty());
    step_phase(core.start_sweep(&env, SweepInput::Finished, &mut out), Phase::Switch);
    step_phase(core.start_done(&env, Phase::Switch, &mut out), Phase::Wires);
    let Action::Commit(token) = core.start_done(&env, Phase::Wires, &mut out) else {
        panic!("Wires waits for release")
    };
    assert!(out.is_empty());
    step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
    let mut signing = false;
    let mut component = false;
    for _ in 0..out.len() {
        let request = out.pop().expect("released account output");
        if let Request::Now(now) = request {
            #[expect(clippy::wildcard_enum_match_arm, reason = "the fixture asserts the two account startup outputs")]
            match *now {
                crate::Now::Account(crate::accounts::Request::Signing(crate::accounts::Signing::Restore {
                    source,
                    ..
                })) => {
                    assert_eq!(source, 2);
                    signing = true;
                }
                crate::Now::Account(crate::accounts::Request::Component(crate::accounts::ToComponent::Want {
                    account,
                })) => {
                    assert_eq!(account, 2);
                    component = true;
                }
                _ => {}
            }
        }
    }
    assert!(signing && component);
    assert!(core.start.held.is_empty());
}

#[test]
#[should_panic]
#[expect(
    clippy::should_panic_without_expect,
    reason = "the selection invariant is checked without matching panic text"
)]
fn a_loaded_header_prevents_late_selection() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let header = core.counters.deployment();
    match core.restore_core(crate::CoreRecord::Deployment(header), &limits) {
        crate::Restored::Deployment { commits } => assert_eq!(commits, header.commits),
        crate::Restored::Live | crate::Restored::Archive | crate::Restored::Rejected => {
            panic!("expected deployment header")
        }
    }
    let _late = core.start_begin(false, Wall::EPOCH);
}

fn step_phase(action: Action, phase: Phase) {
    match action {
        Action::Step(found) => assert_eq!(found, phase),
        Action::Commit(_) | Action::Idle | Action::Refused(_) => panic!("expected phase step"),
    }
}

fn through_link(core: &mut Core, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    finish_core_load(core, env, out);
    let connectors = core.connectors.clone();
    let mut phase = Phase::LoadInbox;
    for connector in connectors {
        let next = Phase::RestoreConnector { connector };
        step_phase(core.start_done(env, phase, out), next);
        phase = next;
    }
    step_phase(core.start_done(env, phase, out), Phase::Link);
    step_phase(core.start_done(env, Phase::Link, out), Phase::Sweep);
}

#[test]
fn every_start_reaches_switch_and_waits_for_wires() {
    let limits = limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    for pending in [false, true] {
        for connectors in [Box::<[u16]>::from([]), Box::from([1, 2])] {
            let mut configured = config(41);
            configured.connectors = connectors;
            configured.grants = Box::new([]);
            let mut core = Core::new(configured, &limits).expect("valid core configuration");
            let mut out = Queue::uncapped();
            step_phase(core.start_begin(pending, env.wall), Phase::LoadCore);
            through_link(&mut core, &env, &mut out);
            step_phase(core.start_sweep(&env, SweepInput::Finished, &mut out), Phase::Switch);
            step_phase(core.start_done(&env, Phase::Switch, &mut out), Phase::Wires);
            let Action::Commit(token) = core.start_done(&env, Phase::Wires, &mut out) else {
                panic!("Wires waits for release")
            };
            assert_eq!(core.start_phase(), Some(Phase::Wires));
            assert!(!core.people.ready());
            assert!(out.is_empty());
            step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
            assert!(core.people.ready());
        }
    }
}

#[test]
fn frozen_expiry_and_stale_cleanup_continuation() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let mut env = Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits };
    let mut out = Queue::uncapped();
    step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    let row = people::Stored::SignIn { number: 7, person: 99, expires: Wall::from_nanos(5) };
    let mut child = Queue::uncapped();
    people::step(
        &mut core.people,
        &Env { now: env.now, wall: env.wall, limits: limits.people },
        people::Event::Restore { record: row.clone() },
        &mut child,
    );
    assert!(child.is_empty());
    through_link(&mut core, &env, &mut out);
    env.wall = Wall::EPOCH;
    let Action::Commit(token) = core.start_sweep(&env, SweepInput::Record(row), &mut out) else {
        panic!("expired row commits")
    };
    let Some(Request::Write(Write::Erase(key))) = out.pop() else { panic!("expected expiry erase") };
    assert_eq!(key, Key::People(people::Key::SignIn(7)));
    assert!(out.is_empty());
    let stale = token.clone();
    step_phase(core.start_resume(&env, token, &mut out), Phase::Sweep);
    let rejected = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| core.start_resume(&env, stale, &mut out)));
    let _panic = rejected.expect_err("stale continuation must panic");
    assert!(out.is_empty());
    assert_eq!(core.start_phase(), Some(Phase::Sweep));
}

#[test]
#[should_panic]
#[expect(
    clippy::should_panic_without_expect,
    reason = "the selection invariant is checked without matching panic text"
)]
fn first_external_missing_link_stops_before_sweep() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    let _begin = core.start_begin(false, env.wall);
    let record = Key::Tasks(tasks::Key::Live(1));
    let target = Key::People(people::Key::Person(99));
    core.start_missing(record.clone(), target.clone());
    core.start_missing(Key::Tasks(tasks::Key::Live(2)), Key::People(people::Key::Person(98)));
    assert_eq!(core.start.missing, Some((record, target)));
    through_link(&mut core, &env, &mut out);
}

#[test]
fn begin_selects_before_the_people_lifecycle_input() {
    let limits = limits();
    for pending in [false, true] {
        let mut core = Core::new(config(41), &limits).expect("valid core configuration");
        step_phase(core.start_begin(pending, Wall::EPOCH), Phase::LoadCore);
        assert_eq!(core.start_phase(), Some(Phase::LoadCore));
        assert_eq!(core.start_failure(), None);
    }
}

#[test]
#[should_panic]
#[expect(
    clippy::should_panic_without_expect,
    reason = "the selection invariant is checked without matching panic text"
)]
fn begin_is_selected_once() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _first = core.start_begin(false, Wall::EPOCH);
    let _second = core.start_begin(false, Wall::EPOCH);
}

#[test]
fn first_failure_is_sticky() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _begin = core.start_begin(false, Wall::EPOCH);
    let first = Failure::ProjectPolicy { project: 1 };
    for failure in [first.clone(), Failure::ProjectPolicy { project: 2 }] {
        match core.start_refuse(failure) {
            Action::Refused(found) => assert_eq!(found, first),
            Action::Step(_) | Action::Commit(_) | Action::Idle => panic!("expected first refusal"),
        }
    }
    assert_eq!(core.start_failure(), Some(first));
}

#[test]
#[should_panic]
#[expect(
    clippy::should_panic_without_expect,
    reason = "the selection invariant is checked without matching panic text"
)]
fn direct_resource_input_prevents_late_start_selection() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let name = tasks::Name { connector: core.connectors[0], path: Box::new([Box::from(b"resource".as_slice())]) };
    let mut out = Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
        tasks::Event::Resource { name: name.clone(), hold: tasks::HoldKind::Shared },
        &mut out,
    );
    assert!(out.is_empty());
    assert_eq!(core.tasks.resource_hold(&name), Some(tasks::HoldKind::Shared));
    core.tasks.start_restore();
}

fn finish_core_load(core: &mut Core, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    match core.start_done(env, Phase::LoadCore, out) {
        Action::Commit(token) => {
            while let Some(request) = out.pop() {
                match request {
                    Request::Write(_) => {}
                    Request::Now(_)
                    | Request::Ask { .. }
                    | Request::Held(_)
                    | Request::ProjectMade { .. }
                    | Request::Decided => panic!("seed commit contains only writes"),
                }
            }
            step_phase(core.start_resume(env, token, out), Phase::LoadInbox);
        }
        action @ (Action::Step(_) | Action::Idle | Action::Refused(_)) => step_phase(action, Phase::LoadInbox),
    }
}
