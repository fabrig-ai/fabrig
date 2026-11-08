//! A run's preparation and its answer at awkward moments: a store read that
//! lands after the preparation it served was dropped, and an answer whose
//! parts exceed their bounds.
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, ReplyTo, Time, Token, Wall};
use temper_engine_domain as engine;
use temper_engine_domain::{CoreRange, Range};
use temper_engine_domain_world::direct::{Driver, say_ask};
use temper_engine_domain_world::protocol;
use temper_engine_domain_world::{store::Store, walking};

const CHANNEL: Token = Token::new(7); // The direct world's worker.
const PROJECT: u32 = 1; // The walking configuration's project.
const CHAT_KEYS: [u8; 3] = [31, 32, 33]; // Three parked chats exceed the two shipped load slots.
const CHAT_SPEND: u64 = 100; // The walking policy's maximum run spend; three shares fit its period.

fn restart_runtime(driver: &Driver) -> bool {
    driver.root.ready() || matches!(driver.root.start_phase(), Some(
        jig_domain_core::start::Phase::AdoptRuns
        | jig_domain_core::start::Phase::ReadAfresh { .. }
        | jig_domain_core::start::Phase::SettleOutbox { .. }
        | jig_domain_core::start::Phase::Open))
}

/// Keep the final two restore loads retired through the opening pass. A load
/// slot is occupied until reclaim even when its archive waiter has ended.
fn restart_preparations(driver: &mut Driver) {
    const STEPS: u32 = 200; // The direct fixture's settlement horizon.
    const RELEASE_CYCLES: u32 = 22; // temper-service's engine::release bound.
    let mut opening = false;
    for _ in 0..STEPS {
        let runtime = restart_runtime(driver);
        if let Some(index) = driver.events.iter().position(|event| runtime || !matches!(event,
            engine::Event::Credential { .. } | engine::Event::Worker(engine::WorkerInput::Link { .. }))) {
            let event = driver.events.remove(index).expect("eligible restart input");
            if let engine::Event::Store(engine::StoreInput::Loaded { owner, next, .. }) = &event
                && driver.loads.iter().any(|(issued, range)| *issued == *owner && *range == Range::Core(CoreRange::Sweep)) {
                // Look ahead in the fake store to identify the final pair of pages.
                opening = next.is_none() || driver.store.page(&Range::Core(CoreRange::Sweep), next.as_ref(), driver.env.limits.loads.rows).1.is_none();
            }
            if let Err(event) = engine::try_step(&mut driver.root, &driver.env, event) {
                driver.events.push_front(event);
            }
        }
        if !driver.store.pending.is_empty() {
            let number = driver.store.apply();
            engine::try_step(&mut driver.root, &driver.env, engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 })).expect("store terminal admitted");
        }
        let mut empty = 0;
        for _ in 0..RELEASE_CYCLES {
            temper_engine_domain_world::poll(&mut driver.root, &driver.env, &mut driver.out);
            empty = if driver.out.is_empty() { empty + 1 } else { 0 };
            driver.collect();
            for _ in 0..driver.events.len() {
                let runtime = restart_runtime(driver);
                let Some(index) = driver.events.iter().position(|event| !matches!(event, engine::Event::Store(_) | engine::Event::Restart { .. })
                    && (runtime || !matches!(event, engine::Event::Credential { .. } | engine::Event::Worker(engine::WorkerInput::Link { .. })))) else {
                    break;
                };
                let event = driver.events.remove(index).expect("released child input");
                engine::try_step(&mut driver.root, &driver.env, event).expect("released child input admitted");
            }
            if empty == 2 {
                break;
            }
        }
        if !opening || (driver.root.ready() && driver.store.pending.is_empty()) {
            driver.root.reclaim();
            opening = false;
        }
    }
    assert!(driver.events.is_empty() && driver.store.pending.is_empty(), "restart's immediate work drained");
}

fn no_transcript_failure(driver: &mut Driver) {
    while let Some(fact) = driver.root.pop_fact() {
        if let engine::Fact::BriefFailed { task, counts, reason } = fact {
            assert_ne!(reason.as_ref(), b"transcript: its window could not be loaded",
                "task {task}: {counts}: {}", String::from_utf8_lossy(&reason));
        }
    }
}

fn park_turn(driver: &mut Driver, run: &engine::Assignment, read: Option<u64>, words: &[u8]) {
    driver.send(protocol::turn(CHANNEL, run.task, run.attempt, 1, 1, read, words));
    driver.pump();
    driver.answer(CHANNEL, run.task, run.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.pump();
}

fn parked_chats() -> (Driver, Vec<(engine::Assignment, u8)>) {
    // Take the shipped task/request capacities; leave the world's other bounds,
    // including root.loads.loads=2, unchanged.
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let own = temper_service::limits::profile(temper_service::limits::Profile::Small);
    let shipped = temper_service::limits::engine(&jig, &own).expect("shipped root limits");
    let mut limits = walking::limits();
    limits.tasks.tasks = shipped.tasks.tasks;
    limits.tasks.project_tasks = shipped.tasks.project_tasks;
    limits.people.inbox_tasks = shipped.people.inbox_tasks;
    limits.people.inbox_recipients = shipped.people.inbox_recipients;
    limits.people.requests = shipped.people.requests;
    assert_eq!(limits.loads.loads, 2);
    let mut driver = Driver::configured(Store::new(), [walking::config(91), walking::config(91)], &limits);
    driver.hello(CHANNEL);
    driver.settle();
    driver.sign_in();
    driver.settle();
    let mut parked = Vec::new();
    for key in CHAT_KEYS {
        let people::Ask::Create { project, mut batch } = driver.chat_ask(PROJECT, b"standing chat") else {
            panic!("chat template creates a task")
        };
        batch[0].flags.ended_by = people::EndedBy::Requester;
        batch[0].budget = people::Budget::Named;
        batch[0].authority.spend = CHAT_SPEND;
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1001)),
            sign_in: driver.session(),
            key: [key; 16],
            ask: people::Ask::Create { project, batch },
        }));
        driver.settle();
        let run = driver.last_assignment();
        park_turn(&mut driver, &run, None, &[key]);
        parked.push((run, key));
    }
    (driver, parked)
}

/// temper/docs/design/domain/root.md, 8 and 9.1: due windows wait for load room.
#[test]
fn tasks_due_together_past_the_load_slots_each_load_their_window() {
    let (mut driver, parked) = parked_chats();
    no_transcript_failure(&mut driver);
    // Commit all three wakes before answering any transcript read, then restart
    // the parked chats together with their wake messages durable.
    for (run, key) in &parked {
        driver.send(say_ask(driver.session(), run.task, key + 20));
    }
    while !driver.store.pending.is_empty() {
        let number = driver.store.apply();
        driver.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
    }
    for (run, _) in &parked {
        assert_ne!(driver.task(run.task).expect("woken chat").last_message, 0);
    }
    driver.reopen(walking::config(91));
    driver.loads.clear(); // A restarted root's load tokens belong to its own history.
    driver.hello(CHANNEL);
    restart_preparations(&mut driver);
    // The direct world's worker has one slot; return it after each resumed run.
    let mut resumed = Vec::new();
    for _ in &parked {
        no_transcript_failure(&mut driver);
        let next = driver.delivered.iter().find_map(|delivery| match delivery {
            engine::Delivery::Assigned { assignment, .. } if !resumed.contains(&assignment.task) => Some(assignment.clone()),
            _ => None,
        }).expect("every parked chat is assigned again");
        let (previous, key) = parked.iter().find(|(run, _)| run.task == next.task).expect("a parked chat resumed");
        assert!(next.attempt > previous.attempt);
        assert_eq!(next.transcript.as_ref(), &[Box::<[u8]>::from([*key])]);
        assert_eq!(driver.task(next.task).expect("resumed chat").tries, tasks::Tries::NONE, "room spends no try");
        resumed.push(next.task);
        let read = next.messages.last().expect("the wake message is offered").number;
        park_turn(&mut driver, &next, Some(read), b"resumed");
        // Past the walking fixture's capacity-refusal backoff, without a real sleep.
        driver.fire_at(Duration::from_secs(1));
        driver.pump();
    }
    no_transcript_failure(&mut driver);
    for (run, _) in &parked {
        let row = driver.task(run.task).expect("chat remains live");
        assert_eq!(row.tries, tasks::Tries::NONE, "room spends no try");
        assert!(resumed.contains(&run.task), "every chat resumed");
        assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Idle));
    }
}

/// Step until the root has asked for `task`'s transcript, leaving the
/// store's answer queued; the answer's owner.
fn transcript_in_flight(driver: &mut Driver, task: u64) -> Token {
    for _ in 0..50 {
        driver.advance(true);
        let asked = driver.loads.iter().find_map(|(owner, range)| match range {
            Range::Core(CoreRange::TaskTranscript { task: read, .. }) if *read == task => Some(*owner),
            Range::Core(_) | Range::Census(_) | Range::Forge => None,
        });
        if let Some(owner) = asked {
            return owner;
        }
    }
    panic!("the parked chat's preparation read its transcript")
}

// A message can end a preparation while its store read is in flight; the
// read's answer then belongs to none (jig's `examples/ops/domain/src/restart.rs`).
#[test]
fn a_transcript_read_landing_after_a_message_ended_its_preparation_is_dropped() {
    let (mut driver, first) = Driver::open_chat();
    driver.send(protocol::turn(Token::new(7), first.task, first.attempt, 1, 1, None, b"first"));
    driver.settle();
    driver.answer(Token::new(7), first.task, first.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.send(say_ask(driver.session(), first.task, 50));
    transcript_in_flight(&mut driver, first.task);
    driver.send(say_ask(driver.session(), first.task, 51));
    driver.settle();
    // The message ended that preparation; the task backs off and prepares again.
    driver.env.now = Time::from_nanos(Duration::from_secs(1).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(1).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    driver.settle();
    let next = driver.last_assignment();
    assert_eq!((next.task, next.attempt), (first.task, 2));
    assert_eq!(next.transcript.len(), 1);
    assert_eq!(next.messages.iter().map(|message| message.words.as_ref()).collect::<Vec<_>>(), [[50], [51]]);
}

// Parts beyond their bounds make the answer invalid; the run fails as invalid.
#[test]
fn an_answer_pushing_more_heads_than_a_task_may_hold_fails_its_run_as_invalid() {
    let (mut driver, run) = Driver::chat();
    let pushed = (0..5).map(|tag| temper_engine_domain_forge::Pushed { tag, commit: [7; 32] }).collect();
    let parts = engine::Parts { saved: None, pushed, preparation: None };
    let end = tasks::End::Finished {
        result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
        cancel_delegates: false,
    };
    driver.answer(Token::new(7), run.task, run.attempt, 1, end, parts);
    driver.settle();
    let task = driver.task(run.task).expect("the chat lives on");
    assert_eq!(task.tries.invalid, 1);
}

fn failed_preparation(run: &engine::Assignment, preparation: engine::Preparation) -> engine::Event {
    let ending = host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: None }),
        detail: Box::new([]),
        work: host::Work { left: None, saved: None },
    };
    let answer = host::RunAnswer { read: None, refused: Box::new([]), turns: 0, spent: 1, ending };
    let parts = engine::Parts { saved: None, pushed: Box::new([]), preparation: Some(preparation) };
    let up = host::LinkUp::Answer { run: Token::new(run.task), attempt: Token::new(run.attempt), answer };
    protocol::up(Token::new(7), up, parts)
}

// A permanent preparation failure names its repository beside the answer;
// the root reports it as a fact of the attempt (domain/root.md, 7.4).
#[test]
fn a_permanent_preparation_failure_is_reported_as_a_fact_of_its_attempt() {
    let (mut driver, run) = Driver::chat();
    let preparation = engine::Preparation::Missing { tag: 3, missing: engine::Missing::Branch };
    driver.send(failed_preparation(&run, preparation));
    driver.settle();
    let fact = engine::Fact::PreparationFailed { task: run.task, attempt: run.attempt, preparation };
    assert_eq!(driver.root.pop_fact(), Some(fact));
    assert_eq!(driver.root.pop_fact(), None);
}

#[test]
fn an_answer_without_a_preparation_reports_no_fact() {
    let (mut driver, run) = Driver::chat();
    driver.answer(Token::new(7), run.task, run.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    assert_eq!(driver.root.pop_fact(), None);
}

#[test]
fn a_failed_preparation_retries_by_its_answers_class() {
    let (mut driver, run) = Driver::chat();
    let preparation = engine::Preparation::Refused { tag: 3 };
    driver.send(failed_preparation(&run, preparation));
    driver.settle();
    assert_eq!(driver.task(run.task).expect("the chat lives on").tries.permanent, 1);
}
