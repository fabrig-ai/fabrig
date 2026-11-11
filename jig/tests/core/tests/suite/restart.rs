use jig_core_world::world::Setup;
use jig_domain_link::{Form, Grant, GrantTable, Hello, Hosting, Phase, Scheme};
const FIRST_GOAL: u8 = 151;
const CHAT: u8 = 152;
const TURN: u32 = 1;
const BEFORE: u64 = 153;
const AFTER: u64 = 154;
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use jig_test_system::Fault;
use skein_lib::{Duration, Token};

fn hello(world: &mut World, hosting: Box<[Hosting]>) {
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting,
        },
    })));
}

fn account(world: &mut World) {
    grant_account(world, 1, 1);
}

fn grant_account(world: &mut World, account: u32, generation: u64) {
    world.send(root::Event::Core(core::Event::Account(jig_domain_accounts::Event::Component(
        jig_domain_accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account,
            generation,
            valid: Duration::from_secs(60),
        },
    ))));
}

fn hosting(world: &World) -> Box<[Hosting]> {
    let assigned = world.assigned.as_ref().expect("a live chat");
    Box::new([Hosting { run: Token::new(assigned.task), attempt: Token::new(assigned.attempt), phase: Phase::Active }])
}

fn received(world: &World) -> Grant {
    world.received_grant().expect("host received a grant").3
}

fn applied_without_answer(world: &mut World) {
    world.hold_writes = true;
    world.call(1, 400);
    let (number, call) = world.pending_writes.remove(0);
    let _lost = world.systems[usize::from(number - 1)].answer(call, Fault::AfterApply);
    world.hold_writes = false;
}

#[test]
fn an_uncertain_effect_across_restarts_is_made_before_its_retry_deadline_and_once() {
    let mut world = World::new(Setup { seed: 113, ..Setup::default() });
    world.enable_tracing();
    applied_without_answer(&mut world);
    let (key, deadline) = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            root::Record::Connector { number: 1, record: connector::Record::Outbox(row) } => {
                Some((row.key, row.attempt.expect("attempt committed").deadline))
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("sent outbox record");
    world.restart(113, false);
    assert!(world.domain.ready());
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Connector { number: 1, record: connector::Record::Made { key: found, .. } } if *found==key)));
    assert!(
        world
            .store
            .rows
            .values()
            .any(|record| matches!(record, root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(_)))))
    );
    let applied = world.systems[0].observed();
    assert_eq!(applied.iter().filter(|row| row.applied).count(), 1);
    assert!(world.trace.iter().any(|line| line.contains("Look")));
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Core(core::Record::Core(core::CoreRecord::Call(row))) if matches!(row.part, core::CallPart::Effect { outcome: Some(core::effects::connector::OutboxOutcome::Made), .. }))));
    assert!(deadline > world.wall_time(), "lookup resolved long before the saved retry deadline");
    world.restart(113, false);
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
}

#[test]
fn nothing_is_decided_before_every_connector_has_settled_its_outbox() {
    let mut world = World::new(Setup { seed: 115, ..Setup::default() });
    applied_without_answer(&mut world);
    world.park_for_message(0);
    let mut seeded = Vec::new();
    for record in world.store.rows.values().cloned() {
        let mut record = record;
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = &mut record {
            row.phase = tasks::Phase::Active(tasks::Active::Due);
            seeded.push(record);
        }
    }
    world.seed(seeded.into_boxed_slice());
    world.assigned = None;
    world.hold_lookups = true;
    world.restart(115, false);
    assert!(!world.domain.ready());
    assert_eq!(world.pending_lookups.len(), 1);
    hello(&mut world, Box::new([]));
    account(&mut world);
    assert!(world.assigned.is_none(), "a due run cannot be assigned while recovery waits");
    world.finish_lookups();
    assert!(world.domain.ready());
    assert!(world.assigned.is_some(), "the due run starts after every connector settled");
}

#[test]
fn a_restart_after_a_turn_committed_resumes_from_it() {
    let mut world = World::new(Setup { seed: 116, ..Setup::default() });
    world.turn(1, 3, None, b"committed whole turn");
    let assignment = world.assigned.as_ref().expect("live worker");
    let hosted =
        Hosting { run: Token::new(assignment.task), attempt: Token::new(assignment.attempt), phase: Phase::Active };
    world.restart(116, false);
    hello(&mut world, Box::new([hosted]));
    world.park_for_message(3);
    account(&mut world);
    world.say(90);
    assert_eq!(
        world.assigned.as_ref().expect("resumed").transcript.as_ref(),
        [Box::<[u8]>::from(&b"committed whole turn"[..])]
    );
    assert_eq!(world.spent(), 3);
}

#[test]
fn an_engine_claim_is_settled_at_once_instead_of_waiting_for_a_worker() {
    let mut world = World::new(Setup { seed: 117, ..Setup::default() });
    world.turn(1, 2, None, b"engine turn");
    let mut seeded = Vec::new();
    for record in world.store.rows.values().cloned() {
        let mut record = record;
        if let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = &mut record {
            proof.host = fleet::HostKind::Engine;
            proof.placement = Some(Token::new(0));
            seeded.push(record);
        }
    }
    world.seed(seeded.into_boxed_slice());
    world.restart(117, false);
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) if row.tries.lost==1)));
    assert!(world.domain.ready());
}

#[test]
fn an_unfound_uncertain_entry_keeps_its_saved_retry_deadline_across_restarts() {
    let mut world = World::new(Setup { seed: 118, ..Setup::default() });
    world.hold_writes = true;
    world.call(1, 401);
    let attempt = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            root::Record::Connector { record: connector::Record::Outbox(row), .. } => row.attempt,
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable attempt");
    world.restart(118, false);
    assert!(world.domain.ready());
    assert!(world.systems[0].observed().is_empty(), "not found does not retry before the saved deadline");
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Connector { record: connector::Record::Outbox(row), .. } if row.attempt==Some(attempt))));
    world.restart(118, false);
    assert!(world.systems[0].observed().is_empty());
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Connector { record: connector::Record::Outbox(row), .. } if row.attempt==Some(attempt))));
}

#[test]
fn an_uncertain_effect_released_without_making_it_again_tells_its_executor() {
    use jig_domain_people as people;
    use skein_lib::ReplyTo;
    let mut world = World::new(Setup { seed: 119, ..Setup::default() });
    world.recovery = connector::Recovery::Unrecoverable;
    world.restart(119, false);
    let task = world.call_key(1).task;
    applied_without_answer(&mut world);
    world.park_for_message(0);
    world.restart(119, false);
    assert!(world.domain.ready());
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Connector { record: connector::Record::Outbox(row), .. } if row.phase==connector::EffectPhase::Held)));
    assert!(world.store.rows.values().any(|record| matches!(record, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) if matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Uncertain { .. }, .. }))));
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), 1);
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))]
    else {
        panic!("live held task")
    };
    let revision = row.revisions.hold;
    let sign_in = world.sign_in.expect("restored sign-in");
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(7001)),
        sign_in,
        key: [80; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    })));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority {
            proposable: None,
            lacking: people::Lack::Again { required: true }
        })))
    );
    let made = world.systems[0].observed().iter().filter(|row| row.applied).count();
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    let answers = world.people_answers.len();
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(7002)),
        sign_in,
        key: [81; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: Some(false) },
    })));
    assert_eq!(world.people_answers.len(), answers, "release answer waits for the outbox decision");
    world.release_commits();
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Released { task })));
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), made);
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))]
    else {
        panic!("live released task")
    };
    assert!(
        row.inbox.iter().any(|word| matches!(word.kind, tasks::MessageKind::EffectNotMade { .. })),
        "the resumed executor reads the not-made outcome"
    );
    assert!(!world.store.rows.values().any(
        |row| matches!(row, root::Record::Connector { record: connector::Record::Outbox(row), .. } if row.task == task)
    ));
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Call(row))) if row.key.task == task && matches!(row.part, core::CallPart::Effect { outcome: Some(core::effects::connector::OutboxOutcome::NotMade), .. }))));
}

#[test]
fn a_refused_claim_keeps_the_conversation_for_the_next_resume() {
    let mut world = World::new(Setup { seed: 120, ..Setup::default() });
    world.turn(1, 1, None, b"durable conversation");
    world.park_for_message(1);
    let assignment = world.assigned.as_ref().expect("first assignment");
    let task = assignment.task;
    let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
    let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(saved))) = &world.store.rows[&key] else {
        panic!("durable proof")
    };
    let limits = jig_test_profile::limits().core;
    let mut core = core::Core::new(jig_test_profile::config(120).core, &limits).expect("fixture core configuration");
    core.replaced.insert(task, saved.clone()).expect("one durable replacement");
    let mut pending = saved.clone();
    pending.attempt += 1;
    pending.conversation = pending.attempt;
    core.proofs.insert(task, pending.clone()).expect("one pending proof");
    core.claiming.insert(task, (pending.attempt, tasks::Reasons::ALL, None)).expect("one pending claim");
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let _decision = core::step(&mut core, &env, core::Event::ClaimRefused { task });
    assert_eq!(core.proofs.get(&task), Some(saved));
    assert!(core.replaced.is_empty());
    core.begin_transcript(task, saved.attempt, saved.window.map(|window| (window.attempt, window.turn)));
    assert_eq!(core.transcripts.get(&task).expect("next resume").from, saved.conversation);
}

#[test]
fn a_restart_restores_people_before_live_tasks_and_core_rows() {
    let mut world = World::new(Setup { seed: 120, ..Setup::default() });
    world.restart(120, false);
    assert_eq!(
        world.restore_ranges,
        [
            core::RestoreRange::Deployment,
            core::RestoreRange::Settings,
            core::RestoreRange::Accounts,
            core::RestoreRange::People,
            core::RestoreRange::Tasks,
            core::RestoreRange::Core,
        ]
    );
    assert!(world.domain.ready());
}

#[test]
fn a_limited_source_keeps_its_reset_across_a_restart() {
    let mut world = World::new(Setup { seed: 121, ..Setup::default() });
    account(&mut world);
    let (now, wall) = world.clocks();
    let reset = now.saturating_add(Duration::from_secs(300));
    world.send(root::Event::Core(core::Event::Account(jig_domain_accounts::Event::Limited {
        account: 1,
        reset: Some(reset),
    })));
    let limited = jig_domain_accounts::State::Limited {
        until: Some(jig_domain_accounts::Deadline {
            at: reset,
            wall: skein_lib::Wall::from_nanos(wall.as_nanos() + Duration::from_secs(300).as_nanos()),
        }),
    };
    assert_eq!(world.domain.accounts().source(1).expect("the configured source").state, limited);
    let row = root::Key::Core(core::Key::Accounts(jig_domain_accounts::Key::Source { account: 0, number: 1 }));
    assert!(world.store.rows.contains_key(&row), "the hold is committed with the decision that limits it");
    world.restart(121, false);
    assert!(world.domain.ready());
    let restored = world.domain.accounts();
    assert_eq!(restored.source(1).expect("the configured source").state, limited, "the same reset after the restart");
    assert_eq!(restored.restore_limit(), None);
}

fn restored_task(world: &World, task: u64) -> &tasks::TaskRecord {
    let row = &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))];
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) = row else { panic!("live task") };
    task
}

#[test]
fn a_restart_with_workers_in_process_settles_their_claims_at_once() {
    struct Row {
        name: &'static str,
        seed: u64,
        stop: bool,
        in_process: bool,
        interrupted: usize,
        lost: u32,
        refusals: u32,
        due: bool,
    }
    for row in [
        Row {
            name: "crashed",
            seed: 121,
            stop: false,
            in_process: true,
            interrupted: 0,
            lost: 1,
            refusals: 0,
            due: false,
        },
        Row {
            name: "stopped, worker elsewhere",
            seed: 127,
            stop: true,
            in_process: false,
            interrupted: 0,
            lost: 0,
            refusals: 2,
            due: false,
        },
        Row {
            name: "stopped",
            seed: 126,
            stop: true,
            in_process: true,
            interrupted: 1,
            lost: 0,
            refusals: 2,
            due: true,
        },
    ] {
        let mut world = resumable_worker(row.seed, row.in_process);
        world.turn(1, 2, None, b"worker turn");
        let (task, attempt) = (world.task(), world.attempt());
        let saved = restored_task(&world, task).saved.clone();
        let turn_key = root::Key::Core(core::Key::Core(core::CoreKey::Turn { task, attempt, turn: 1 }));
        let turn = world.store.rows[&turn_key].clone();
        if row.stop {
            stop_with_late_words(&mut world, row.interrupted, row.name);
        }
        let before = world.wall_time().as_nanos();
        world.assigned = None;
        world.restart_with(worker_configuration(row.seed, row.in_process));
        assert!(world.domain.ready(), "{}", row.name);
        let restored = restored_task(&world, task);
        assert_eq!(restored.tries, tasks::Tries { lost: row.lost, ..tasks::Tries::NONE }, "{}", row.name);
        assert_eq!(restored.refusals, row.refusals, "{}", row.name);
        assert_eq!(restored.saved, saved, "{}", row.name);
        assert_eq!(world.store.rows.get(&turn_key), Some(&turn), "{}: committed turn kept", row.name);
        assert_eq!(restored.phase == tasks::Phase::Active(tasks::Active::Due), row.due, "{}", row.name);
        if row.interrupted > 0 {
            assert!(
                !world.store.rows.values().any(|record| matches!(record,
                root::Record::Core(core::Record::Tasks(tasks::Stored::Writer(slot)))
                if slot.writer == tasks::Writer::Run { task, attempt })),
                "{}: writer released",
                row.name
            );
        }
        if !row.in_process {
            hello(
                &mut world,
                Box::new([Hosting { run: Token::new(task), attempt: Token::new(attempt), phase: Phase::Active }]),
            );
            assert_eq!(restored_task(&world, task).phase, tasks::Phase::Active(tasks::Active::Running { attempt }));
        }
        assert!(world.wall_time().as_nanos() - before < Duration::from_secs(5).as_nanos(), "{}", row.name);
    }
}

fn worker_configuration(seed: u64, in_process: bool) -> root::Config {
    let mut config = jig_core_world::world::fixture(seed, false).0;
    config.core.workers_in_process = in_process;
    config
}

fn resumable_worker(seed: u64, in_process: bool) -> World {
    let mut world =
        World::new(Setup { seed, configuration: Some(worker_configuration(seed, in_process)), ..Setup::default() });
    world.park_for_message(0);
    let task = world.task();
    let mut kept = restored_task(&world, task).clone();
    kept.saved = Box::new([tasks::SavedResource { connector: 1, path: Box::new([Box::from(*b"work")]) }]);
    kept.refusals = 2;
    world.seed(Box::new([root::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(kept))))]));
    world.restart_with(worker_configuration(seed, in_process));
    hello(&mut world, Box::new([]));
    account(&mut world);
    world.say(93);
    world
}

fn stop_with_late_words(world: &mut World, expected: usize, name: &str) {
    let (task, attempt) = (world.task(), world.attempt());
    let relay = world.host_call(b"read-at-stop", b"read", b"{}", false);
    let answers = world.host_answers.len();
    let decisions = world.commits.len();
    let cancellations = world.cancellations.len();
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    world.stop();
    assert_eq!(world.cancellations.len(), cancellations, "{name}: stop tells the host nothing");
    assert_eq!(world.commits.len(), decisions + expected, "{name}: one stop decision");
    let drained_before_commit = world.stop_deliveries.iter().any(|held| matches!(held, core::Held::Drained));
    world.release_commits();
    assert_eq!(interrupted(world, task, attempt), expected, "{name}: terminal at stop");
    assert_eq!(drained_before_commit, expected == 0, "{name}: drain waits for terminal");
    assert_eq!(world.stop_deliveries.iter().filter(|held| matches!(held, core::Held::Drained)).count(), 1, "{name}");
    let commits = world.store.applied;
    world.turn(2, 3, None, b"late turn");
    world.end(tasks::End::Parked, 3);
    world.send(root::Event::CallAnswered {
        to: skein_lib::ReplyTo::new(relay),
        call: core::SettledCall {
            serial: 0,
            tool: b"read".as_slice().into(),
            answer: core::SettledAnswer::Host { error: false, body: b"late result".as_slice().into() },
        },
    });
    assert_eq!(world.host_answers.len(), answers, "{name}: interrupted relay answer dropped");
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: Token::new(7) })));
    assert_eq!(world.store.applied, commits, "{name}: late host words commit nothing");
}

fn interrupted(world: &World, task: u64, attempt: u64) -> usize {
    world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(terminal)))
        if terminal.task == task && terminal.attempt == attempt && terminal.end == tasks::End::Recheck)
        })
        .count()
}

#[test]
fn a_restart_with_workers_in_process_assigns_without_a_grace() {
    let mut world = World::new(Setup { seed: 122, ..Setup::default() });
    let (task, attempt) = (world.task(), world.attempt());
    let before = world.wall_time().as_nanos();
    let mut config = jig_core_world::world::fixture(122, false).0;
    config.core.workers_in_process = true;
    world.assigned = None;
    world.restart_with(config);
    assert_eq!(restored_task(&world, task).refusals, 1, "an unworked claim was not started");
    assert_eq!(restored_task(&world, task).tries.lost, 0);
    hello(&mut world, Box::new([]));
    account(&mut world);
    world.advance(Duration::from_millis(100));
    let assigned = world.assigned.as_ref().expect("next attempt passes only its retry backoff");
    assert_eq!(assigned.task, task);
    assert!(assigned.attempt > attempt);
    assert!(world.wall_time().as_nanos() - before < Duration::from_secs(5).as_nanos());
    // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
    world.broken();
}

#[test]
fn a_restart_with_workers_elsewhere_still_waits_for_them() {
    let mut world = World::new(Setup { seed: 123, ..Setup::default() });
    world.turn(1, 2, None, b"remote turn");
    let (task, attempt) = (world.task(), world.attempt());
    world.assigned = None;
    world.restart(123, false);
    account(&mut world);
    world.advance(Duration::from_millis(100));
    assert!(world.assigned.is_none());
    assert_eq!(restored_task(&world, task).tries.lost, 0);
    hello(
        &mut world,
        Box::new([Hosting { run: Token::new(task), attempt: Token::new(attempt), phase: Phase::Active }]),
    );
    assert_eq!(restored_task(&world, task).attempt, attempt);
    assert_eq!(restored_task(&world, task).phase, tasks::Phase::Active(tasks::Active::Running { attempt }));
    assert_eq!(restored_task(&world, task).tries.lost, 0);
}

#[test]
fn a_contact_cancellation_after_restart_resumes_without_spending_a_try() {
    use jig_domain_host as host;
    let mut world = World::new(Setup { seed: 126, ..Setup::default() });
    world.window_turn(1, 2, None, b"remote window before restart");
    let (task, attempt) = (world.task(), world.attempt());
    let tries = restored_task(&world, task).tries;
    world.restart(126, false);
    hello(
        &mut world,
        Box::new([Hosting { run: Token::new(task), attempt: Token::new(attempt), phase: Phase::Active }]),
    );
    account(&mut world);
    world.send(root::Event::Link {
        channel: Token::new(7),
        link: root::link::Event::Up(host::LinkUp::Answer {
            run: Token::new(task),
            attempt: Token::new(attempt),
            answer: host::RunAnswer {
                read: None,
                refused: Box::new([]),
                turns: 1,
                spent: 3,
                ending: host::Ending::Failed {
                    failure: host::Failure::Cancelled(host::Reason::Contact),
                    detail: Box::new([]),
                    work: host::Work { left: None, saved: None },
                },
            },
        }),
    });
    assert_eq!(restored_task(&world, task).tries, tries);
    let resumed = world.assigned.as_ref().expect("immediate resumed assignment");
    assert!(resumed.attempt > attempt);
    assert_eq!(resumed.conversation, attempt);
    assert_eq!(resumed.transcript.as_ref(), [Box::<[u8]>::from(&b"remote window before restart"[..])]);
    assert_eq!(restored_task(&world, task).numbers.spent, 3);
    assert_eq!(interrupted(&world, task, attempt), 1);
}

#[test]
fn a_restart_keeps_the_window_a_turn_opened() {
    let mut world = World::new(Setup { seed: 125, ..Setup::default() });
    world.turn(1, 1, None, b"earlier window");
    world.window_turn(2, 2, None, b"current window");
    let assignment = world.assigned.as_ref().expect("live worker");
    let (task, attempt) = (assignment.task, assignment.attempt);
    let hosted = Hosting { run: Token::new(task), attempt: Token::new(attempt), phase: Phase::Active };
    world.restart(125, false);
    hello(&mut world, Box::new([hosted]));
    world.park_for_message(2);
    account(&mut world);
    world.say(92);
    let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
    let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = &world.store.rows[&key] else {
        panic!("restored conversation proof")
    };
    assert_eq!(proof.window, Some(core::CurrentWindow { attempt, turn: 2, closed: false }));
}

#[test]
fn a_restart_between_a_window_turn_and_its_resume_sends_the_window() {
    let mut world = World::new(Setup { seed: 124, ..Setup::default() });
    world.turn(1, 1, None, b"earlier window");
    world.window_turn(2, 2, None, b"window to resume");
    let assignment = world.assigned.as_ref().expect("live worker");
    let hosted =
        Hosting { run: Token::new(assignment.task), attempt: Token::new(assignment.attempt), phase: Phase::Active };
    world.restart(124, false);
    hello(&mut world, Box::new([hosted]));
    world.park_for_message(2);
    account(&mut world);
    world.say(91);
    assert_eq!(
        world.assigned.as_ref().expect("resumed window").transcript.as_ref(),
        [Box::<[u8]>::from(&b"window to resume"[..])]
    );
}

fn census_world(seed: u64) -> World {
    let (configuration, mut limits) = jig_core_world::world::fixture(seed, false);
    limits.core.people.inbox_entries = 8;
    limits.core.tasks.inbox_messages = 8;
    limits.core.tasks.run_inbox = 8;
    limits.core.documents.rows = 2;
    let room = limits.core.routes;
    let connectors = connector::OUTPUTS_BOUND * 2;
    limits.journal.commits = 2;
    limits.watermark = root::default_watermark(&limits.journal);
    limits.journal.writes = room + connectors;
    let mut world = World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() });
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    world
}
fn finish_census_task(world: &mut World, cumulative: u64) {
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"census result".as_slice().into() },
            cancel_delegates: false,
        },
        cumulative,
    );
}
fn census_ended_key(task: u64) -> root::Key {
    root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))
}
fn census_result(world: &World, task: u64) -> &tasks::TaskRecord {
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) = &world.store.rows[&census_ended_key(task)]
    else {
        panic!("ended census result")
    };
    row
}
fn census_goal(world: &mut World, key: u8) -> u64 {
    crate::read_support::request(world, key, jig_core_world::stories::offers::goal(100));
    let task = world.assigned.as_ref().expect("goal assigned").task;
    finish_census_task(world, 0);
    assert!(census_result(world, task).tracked.is_some());
    task
}
fn census_mark_results(world: &mut World, through: u64, key: u8) {
    crate::read_support::request(
        world,
        key,
        jig_domain_people::Ask::MarkSeen { seen: jig_domain_people::Seen::Results { through } },
    );
}
#[test]
fn a_restart_reads_no_ended_task_whose_result_was_read() {
    let mut world = census_world(790);
    let chat = world.assigned.as_ref().expect("initial chat").task;
    world.turn(1, 1, None, b"initial reply");
    finish_census_task(&mut world, 1);
    crate::read_support::request(
        &mut world,
        99,
        jig_domain_people::Ask::MarkSeen { seen: jig_domain_people::Seen::Conversation { task: chat, through: 1 } },
    );
    let through = census_result(&world, chat).result_position;
    census_mark_results(&mut world, through, 100);
    let first = census_goal(&mut world, 101);
    let _second = census_goal(&mut world, 102);
    let _third = census_goal(&mut world, 103);
    let through = census_result(&world, first).result_position;
    census_mark_results(&mut world, through, 104);
    world.restart(790, false);
    assert_eq!(world.census_ended, 3);
    assert!(world.store.rows.contains_key(&census_ended_key(first)));
}
fn census_updates(world: &mut World, reply: u64) -> core::documents::Count {
    jig_core_world::reads::ask(world, reply, core::reads::Read::Frame);
    let core::reads::Answer::Document(document) = jig_core_world::reads::latest(world) else { panic!("frame") };
    let core::documents::Document::Frame(frame) = &**document else { panic!("frame document") };
    frame.updates
}
fn census_unread_results(world: &mut World) -> Vec<u64> {
    let mut from = None;
    let mut tasks = Vec::new();
    for page in 0..4 {
        jig_core_world::reads::ask(
            world,
            810 + page,
            core::reads::Read::Inbox {
                group: core::reads::Group::Results,
                filter: core::reads::Filter { project: Some(1), goal: None },
                from: from.take(),
            },
        );
        let core::reads::Answer::Page(rows) = jig_core_world::reads::latest(world) else { panic!("results page") };
        for row in &rows.rows {
            let core::reads::PageRow::Inbox(entry) = row else { panic!("result row") };
            let core::documents::TaskLink::Retained { chip, .. } = &entry.task else { panic!("retained task") };
            tasks.push(chip.number);
        }
        from.clone_from(&rows.next);
        if from.is_none() {
            break;
        }
    }
    tasks
}
#[test]
fn a_restart_restores_unread_results_and_chat_totals() {
    let mut world = census_world(791);
    let mut chats = Vec::new();
    for number in 0..3 {
        if number != 0 {
            let request = jig_core_world::fixtures::chat(&world, 1, b"next census chat".as_slice().into());
            crate::read_support::request(&mut world, 110 + number, request);
        }
        chats.push(world.assigned.as_ref().expect("chat").task);
        world.turn(1, 1, None, b"unread reply");
        finish_census_task(&mut world, 1);
    }
    assert_eq!(census_updates(&mut world, 800), core::documents::Count::Known(6));
    world.restart(791, false);
    assert_eq!(census_updates(&mut world, 801), core::documents::Count::Known(6));
    chats.reverse();
    assert_eq!(census_unread_results(&mut world), chats);
}

fn both_kinds_engine_fixture(seed: u64) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_core_world::world::fixture(seed, false);
    config.core.defaults.charters[0].hosts = fleet::Kinds::Both;
    limits.core.fleet.engine_slots = 2;
    limits.core.fleet.slots = 1;
    (config, limits)
}

#[test]
fn a_restart_settles_an_engine_claim_of_a_both_kinds_charter_at_once() {
    let (config, limits) = both_kinds_engine_fixture(125);
    let mut world = World::new(Setup { seed: 125, configuration: Some(config), limits, ..Setup::default() });
    let assignment = world.assigned.as_ref().expect("engine claim");
    let task = assignment.task;
    let attempt = assignment.attempt;
    world.send(root::Event::Turn {
        channel: Token::new(0),
        task,
        attempt,
        turn: 1,
        cumulative: 2,
        read: None,
        opens_window: true,
        refused: Box::new([]),
        transcript: b"engine window".as_slice().into(),
    });
    world.restart_with(both_kinds_engine_fixture(125).0);
    assert!(world.domain.ready(), "engine claim settles before any worker hello");
    assert!(world.store.rows.values().any(|record| matches!(record,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))
        if row.number == task && row.tries.lost == 1)));
}

#[test]
fn a_new_generation_reaches_the_live_chat() {
    let mut world = World::new(Setup { seed: 161, ..Setup::default() });
    grant_account(&mut world, 1, 2);
    let grant = received(&world);
    assert_eq!((grant.form.table(), grant.account, grant.generation), (GrantTable::Accounts, 1, 2));
}

#[test]
fn a_restart_sends_an_adopted_chat_its_current_grant() {
    let mut world = World::new(Setup { seed: 162, ..Setup::default() });
    let live = hosting(&world);
    world.restart(162, false);
    grant_account(&mut world, 1, 7);
    hello(&mut world, live);
    let grant = received(&world);
    assert_eq!((grant.form.table(), grant.account, grant.generation), (GrantTable::Accounts, 1, 7));
    assert!(world.grant_follows_assignment());
}

#[test]
fn a_generation_renewed_while_the_assignment_is_held_reaches_the_attempt_after_it() {
    const SECOND_ANALYST: u32 = 2;
    let mut world = jig_core_world::fixtures::waiting_on_source(163, SECOND_ANALYST);
    assert!(world.assigned.is_none());
    world.renew_on_assignment(SECOND_ANALYST, 2, Form::Credential(Scheme::Bearer));
    grant_account(&mut world, SECOND_ANALYST, 1);
    assert_eq!(world.assigned.as_ref().expect("assigned after commit").grants[0].generation, 1);
    let grant = received(&world);
    assert_eq!((grant.account, grant.generation), (SECOND_ANALYST, 2));
    assert!(world.grant_follows_assignment());
}

#[test]
fn a_generation_renewed_before_a_waiting_chat_is_placed_reaches_its_host() {
    let mut world = World::new(Setup { seed: 165, ..Setup::default() });
    world.park_for_message(0);
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: Token::new(7) })));
    world.say(99);
    grant_account(&mut world, 1, 2);
    hello(&mut world, Box::new([]));
    assert_eq!(world.assigned.as_ref().expect("resumed assignment").grants[0].generation, 2);
    assert_eq!(received(&world).generation, 2);
    assert!(world.grant_follows_assignment());
}

#[test]
fn a_restart_keeps_an_unread_goal_older_than_an_unread_conversation() {
    let mut world = census_world(839);
    finish_census_task(&mut world, 0);
    let goal = census_goal(&mut world, FIRST_GOAL);
    let request = jig_core_world::fixtures::chat(&world, 1, b"newer chat".as_slice().into());
    crate::read_support::request(&mut world, CHAT, request);
    let chat = world.assigned.as_ref().expect("chat").task;
    world.turn(TURN, 1, None, b"reply");
    finish_census_task(&mut world, 1);
    let updates = census_updates(&mut world, BEFORE);
    world.restart(839, false);
    let results = census_unread_results(&mut world);
    assert!(results.contains(&goal));
    assert_eq!(results[0], chat);
    assert_eq!(census_updates(&mut world, AFTER), updates);
}

#[test]
fn a_durable_placed_stop_restores_with_its_task_and_releases_after_restart() {
    use jig_core_world::fixtures::{ask_as, live_row};
    use jig_domain_people as people;
    let mut world = World::new(Setup { seed: 2210, ..Setup::default() });
    let (task, attempt) = world.assignments[0];
    let sign_in = world.sign_in.expect("signed in requester");
    ask_as(&mut world, sign_in, 71, people::Ask::Stop { project: 1, task });
    world.park_for_message(0);
    let hold = live_row(&world, task).placed.expect("placed stop");
    let revision = live_row(&world, task).revisions.hold;
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::PlacedHold(hold)));
    let durable = world.store.rows[&key].clone();
    let assignments = world.assignments.len();
    world.restart(2210, false);
    assert_eq!(world.domain.start_failure(), None);
    assert!(world.domain.ready());
    assert_eq!(world.assignments.len(), assignments);
    assert_eq!(world.store.rows.get(&key), Some(&durable));
    assert_eq!(live_row(&world, task).revisions.hold, revision);
    hello(&mut world, Box::new([]));
    account(&mut world);
    ask_as(&mut world, sign_in, 72, people::Ask::Release { project: 1, task, revision, again: None });
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Released { task })));
    assert!(!world.store.rows.contains_key(&key));
    assert_eq!(live_row(&world, task).placed, None);
    assert!(world.assignments.iter().any(|(number, next)| *number == task && *next > attempt));
}
