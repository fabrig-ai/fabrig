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
    driver.root.ready()
        || matches!(
            driver.root.start_phase(),
            Some(
                jig_domain_core::start::Phase::AdoptRuns
                    | jig_domain_core::start::Phase::ReadAfresh { .. }
                    | jig_domain_core::start::Phase::SettleOutbox { .. }
                    | jig_domain_core::start::Phase::Open
            )
        )
}

fn release_preparations(driver: &mut Driver) {
    const RELEASE_CYCLES: u32 = 22; // temper-engine-service's engine::release bound.
    let mut empty = 0;
    for _ in 0..RELEASE_CYCLES {
        temper_engine_domain_world::poll(&mut driver.root, &driver.env, &mut driver.out);
        empty = if driver.out.is_empty() { empty + 1 } else { 0 };
        driver.collect();
        for _ in 0..driver.events.len() {
            let runtime = restart_runtime(driver);
            let Some(index) = driver.events.iter().position(|event| {
                !matches!(event, engine::Event::Store(_) | engine::Event::Restart { .. })
                    && (runtime
                        || !matches!(
                            event,
                            engine::Event::Credential { .. } | engine::Event::Worker(engine::WorkerInput::Link { .. })
                        ))
            }) else {
                break;
            };
            let event = driver.events.remove(index).expect("released child input");
            engine::try_step(&mut driver.root, &driver.env, event).expect("released child input admitted");
        }
        if empty == 2 {
            break;
        }
    }
}

/// Keep the final two restore loads retired through the opening pass. A load
/// slot is occupied until reclaim even when its archive waiter has ended.
fn restart_preparations(driver: &mut Driver) {
    const STEPS: u32 = 200; // The direct fixture's settlement horizon.
    let mut opening = false;
    for _ in 0..STEPS {
        let runtime = restart_runtime(driver);
        if let Some(index) = driver.events.iter().position(|event| {
            runtime
                || !matches!(
                    event,
                    engine::Event::Credential { .. } | engine::Event::Worker(engine::WorkerInput::Link { .. })
                )
        }) {
            let event = driver.events.remove(index).expect("eligible restart input");
            if let engine::Event::Store(engine::StoreInput::Loaded { owner, next, .. }) = &event
                && driver
                    .loads
                    .iter()
                    .any(|(issued, range)| *issued == *owner && *range == Range::Core(CoreRange::Sweep))
            {
                // Look ahead in the fake store to identify the final pair of pages.
                opening = next.is_none()
                    || driver
                        .store
                        .page(&Range::Core(CoreRange::Sweep), next.as_ref(), driver.env.limits.loads.rows)
                        .1
                        .is_none();
            }
            if let Err(event) = engine::try_step(&mut driver.root, &driver.env, event) {
                driver.events.push_front(event);
            }
        }
        if !driver.store.pending.is_empty() {
            let number = driver.store.apply();
            engine::try_step(
                &mut driver.root,
                &driver.env,
                engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }),
            )
            .expect("store terminal admitted");
        }
        release_preparations(driver);
        if !opening || (driver.root.ready() && driver.store.pending.is_empty()) {
            driver.root.reclaim();
            opening = false;
            if driver.root.ready() {
                // After the opening pass, reclaim between inputs and their
                // continuations as the direct world does for runtime work.
                driver.pump();
                return;
            }
        }
    }
    assert!(driver.events.is_empty() && driver.store.pending.is_empty(), "restart's immediate work drained");
}

fn no_transcript_failure(driver: &mut Driver) {
    while let Some(fact) = driver.root.pop_fact() {
        if let engine::Fact::BriefFailed { task, counts, reason } = fact {
            assert_ne!(
                reason.as_ref(),
                b"transcript: its window could not be loaded",
                "task {task}: {counts}: {}",
                String::from_utf8_lossy(&reason)
            );
        }
    }
}

fn park_turn(driver: &mut Driver, run: &engine::Assignment, read: Option<u64>, words: &[u8]) {
    driver.send(protocol::turn(CHANNEL, run.task, run.attempt, 1, 1, read, words));
    driver.pump();
    driver.answer(CHANNEL, run.task, run.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.pump();
}

fn park_turns(driver: &mut Driver, run: &engine::Assignment, turns: usize, key: u8) {
    for turn in 1..=turns {
        let number = u32::try_from(turn).expect("short window");
        driver.send(protocol::turn(
            CHANNEL,
            run.task,
            run.attempt,
            number,
            u64::from(number),
            None,
            &[key, u8::try_from(turn).expect("short window")],
        ));
        driver.pump();
    }
    driver.answer(
        CHANNEL,
        run.task,
        run.attempt,
        u64::try_from(turns).expect("short window"),
        tasks::End::Parked,
        engine::Parts::none(),
    );
    driver.pump();
}

fn parked_chats(window_len: usize) -> (Driver, Vec<(engine::Assignment, u8)>) {
    parked_windows(&[window_len; CHAT_KEYS.len()])
}

fn parked_windows(lengths: &[usize]) -> (Driver, Vec<(engine::Assignment, u8)>) {
    // Take the shipped task/request capacities; leave the world's other bounds,
    // including root.loads.loads=2, unchanged.
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let own =
        temper_engine_service_vocabulary::limits::profile(temper_engine_service_vocabulary::limits::Profile::Small);
    let shipped = temper_engine_service_vocabulary::limits::engine(&jig, &own).expect("shipped root limits");
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
    for (key, turns) in CHAT_KEYS.into_iter().zip(lengths) {
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
        park_turns(&mut driver, &run, *turns, key);
        parked.push((run, key));
    }
    (driver, parked)
}

/// temper/docs/design/domain/root.md, 7: a large window turn starts fresh,
/// with its place kept in the tail, without spending a preparation try.
#[test]
fn a_window_with_a_large_turn_starts_fresh_from_the_tail_that_fits() {
    let shipped =
        temper_engine_service_vocabulary::limits::profile(temper_engine_service_vocabulary::limits::Profile::Standard);
    let mut limits = walking::limits();
    limits.loads = shipped.root.loads;
    limits.journal = shipped.root.journal;
    let room = Store::with_limits(&limits).transcript_page_bytes().min(limits.loads.reply_bytes);
    let row = limits.loads.transcript_bytes;
    let large = room.checked_mul(2).expect("two pages fit u32");
    eprintln!(
        "bounds: row={row}, page={}, store room={room}, journal={}",
        limits.loads.bytes, limits.journal.transcript_bytes
    );
    let mut failures = Vec::new();
    for (name, bytes, around, cut) in [
        ("inside row", row - 1, false, false),
        ("past row", row + 1, false, true),
        ("past store room", large, false, true),
        ("pages around cut", large, true, true),
    ] {
        let result = std::panic::catch_unwind(|| large_window_row(&limits, name, bytes, around, cut));
        if let Err(error) = result {
            let reason = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap_or("non-text panic");
            eprintln!("{name}: {reason}");
            failures.push(name);
        } else {
            eprintln!("{name}: passed");
        }
    }
    assert!(failures.is_empty(), "large-window rows failed: {failures:?}");
}

fn large_window_row(limits: &engine::Limits, name: &str, bytes: u32, around: bool, cut: bool) {
    let mut driver = Driver::configured(Store::with_limits(limits), [walking::config(91), walking::config(91)], limits);
    driver.hello(CHANNEL);
    driver.settle();
    driver.sign_in();
    driver.settle();
    let people::Ask::Create { project, mut batch } = driver.chat_ask(PROJECT, b"standing chat") else {
        panic!("chat template creates a task")
    };
    batch[0].flags.ended_by = people::EndedBy::Requester;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(1001)),
        sign_in: driver.session(),
        key: [31; 16],
        ask: people::Ask::Create { project, batch },
    }));
    driver.settle();
    let first = driver.last_assignment();
    let large = vec![b'x'; usize::try_from(bytes).expect("turn fits usize")];
    let turns: Vec<&[u8]> = if around {
        vec![b"before one\n", b"before two\n", b"before three\n", &large, b"after\n"]
    } else {
        vec![&large]
    };
    for (index, text) in turns.iter().enumerate() {
        let number = u32::try_from(index + 1).expect("five turns");
        driver.send(protocol::turn(CHANNEL, first.task, first.attempt, number, u64::from(number), None, text));
        driver.pump();
    }
    driver.answer(
        CHANNEL,
        first.task,
        first.attempt,
        u64::try_from(turns.len()).expect("five turns"),
        tasks::End::Parked,
        engine::Parts::none(),
    );
    driver.pump();
    assert_eq!(driver.task(first.task).expect("parked chat").phase, tasks::Phase::Active(tasks::Active::Idle));
    driver.send(say_ask(driver.session(), first.task, 60));
    driver.pump();
    while let Some(fact) = driver.root.pop_fact() {
        if let engine::Fact::BriefFailed { counts, reason, .. } = fact {
            panic!("{name}: BriefFailed counts={counts}: {}", String::from_utf8_lossy(&reason));
        }
    }
    assert!(!driver.stopped && driver.root.ready(), "{name}: root remains ready");
    assert_eq!(driver.task(first.task).expect("woken chat").tries, tasks::Tries::NONE, "{name}: no try spent");
    let next = driver.last_assignment();
    assert_eq!((next.task, next.attempt), (first.task, first.attempt + 1), "{name}: assigned at once");
    if cut {
        assert!(!next.run.policy.resume, "{name}: starts fresh");
        assert!(next.transcript.is_empty(), "{name}: no resumed body");
        let tail = next
            .sections
            .iter()
            .find_map(|section| {
                if section.kind == engine::BriefKind::Core(jig_domain_brief::Core::TranscriptTail) {
                    let engine::BriefBody::Text(text) = &section.body else { panic!("tail is text") };
                    Some(text.as_ref())
                } else {
                    None
                }
            })
            .expect("fresh brief carries a transcript tail");
        let expected = if around {
            b"before one\nbefore two\nbefore three\n[a turn too large to load]\nafter\n".as_slice()
        } else {
            b"[a turn too large to load]\n".as_slice()
        };
        assert_eq!(tail, expected, "{name}: cut keeps its place among short turns");
    } else if next.run.policy.resume {
        assert_eq!(next.transcript.as_ref(), [large.into_boxed_slice()], "{name}: the whole turn resumes");
    } else {
        assert!(next.transcript.is_empty(), "{name}: a fresh run resumes no body");
        let tail = next
            .sections
            .iter()
            .find_map(|section| {
                if section.kind == engine::BriefKind::Core(jig_domain_brief::Core::TranscriptTail) {
                    let engine::BriefBody::Text(text) = &section.body else { panic!("tail is text") };
                    Some(text.as_ref())
                } else {
                    None
                }
            })
            .expect("whole oversized window has a tail");
        assert!(tail.windows(64).any(|text| text == &large[..64]), "{name}: the loaded whole turn supplies its tail");
    }
}

/// temper/docs/design/domain/root.md, 7 and 8.1: due windows wait for load room.
#[test]
fn tasks_due_together_past_the_load_slots_each_load_their_window() {
    for window_len in [1, 3] {
        eprintln!("restart window length {window_len}");
        let (mut driver, parked) = parked_chats(window_len);
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
            let next = driver
                .delivered
                .iter()
                .find_map(|delivery| match delivery {
                    engine::Delivery::Assigned { assignment, .. } if !resumed.contains(&assignment.task) => {
                        Some(assignment.clone())
                    }
                    _ => None,
                })
                .expect("every parked chat is assigned again");
            let (previous, key) = parked.iter().find(|(run, _)| run.task == next.task).expect("a parked chat resumed");
            assert!(next.attempt > previous.attempt);
            let expected: Vec<Box<[u8]>> = (1..=window_len)
                .map(|turn| Box::<[u8]>::from([*key, u8::try_from(turn).expect("short window")]))
                .collect();
            assert_eq!(next.transcript.as_ref(), expected.as_slice(), "window length {window_len}");
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
}

/// temper/docs/design/domain/root.md, 7: later pages keep the first page's slot.
#[test]
fn a_window_of_any_length_loads_while_another_load_holds_a_slot() {
    for window_len in [2, 3, 7] {
        eprintln!("held-slot window length {window_len}");
        let (mut driver, parked) = parked_windows(&[window_len, 1]);
        let (first, key) = &parked[0];
        let second = &parked[1].0;
        assert_eq!((driver.env.limits.loads.loads, driver.env.limits.loads.rows), (2, 2));
        driver.send(say_ask(driver.session(), second.task, 60));
        let held = transcript_in_flight(&mut driver, second.task);
        let index = driver
            .events
            .iter()
            .position(|event| {
                matches!(event,
                    engine::Event::Store(engine::StoreInput::Loaded { owner, .. }) if *owner == held
                )
            })
            .expect("the second chat's store answer is queued");
        let answer = driver.events.remove(index).expect("held answer");
        driver.send(say_ask(driver.session(), first.task, 61));
        driver.pump();
        no_transcript_failure(&mut driver);
        let next = driver.last_assignment();
        assert_eq!(next.task, first.task, "window length {window_len}");
        assert!(next.attempt > first.attempt);
        let expected: Vec<Box<[u8]>> =
            (1..=window_len).map(|turn| Box::<[u8]>::from([*key, u8::try_from(turn).expect("short window")])).collect();
        assert_eq!(next.transcript.as_ref(), expected.as_slice(), "window length {window_len}");
        assert_eq!(driver.task(first.task).expect("resumed chat").tries, tasks::Tries::NONE);
        driver.send(answer);
        driver.pump();
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
    for window_len in [1, 3] {
        let (mut driver, first) = Driver::open_chat();
        park_turns(&mut driver, &first, window_len, b'x');
        driver.send(say_ask(driver.session(), first.task, 50));
        let owner = transcript_in_flight(&mut driver, first.task);
        let index = driver
            .events
            .iter()
            .position(|event| {
                matches!(event,
                    engine::Event::Store(engine::StoreInput::Loaded { owner: found, .. }) if *found == owner
                )
            })
            .expect("first page is in flight");
        let answer = driver.events.remove(index).expect("queued first page");
        if window_len > 1 {
            assert!(
                matches!(&answer, engine::Event::Store(engine::StoreInput::Loaded { next: Some(_), .. })),
                "the stale page would continue the window"
            );
        }
        driver.send(say_ask(driver.session(), first.task, 51));
        driver.send(answer);
        driver.settle();
        assert!(driver.root.quiescent(), "root quiescence includes Loads::quiescent after reclaim");
        assert_eq!(
            driver
                .loads
                .iter()
                .filter(|(_, range)| matches!(range,
                    Range::Core(CoreRange::TaskTranscript { task, .. }) if *task == first.task
                ))
                .count(),
            1,
            "a stale first page issues no continuation"
        );
        // The message ended that preparation; the task backs off and prepares again.
        driver.env.now = Time::from_nanos(Duration::from_secs(1).as_nanos());
        driver.env.wall = Wall::from_nanos(Duration::from_secs(1).as_nanos());
        engine::fire(&mut driver.root, &driver.env);
        driver.settle();
        let next = driver.last_assignment();
        assert_eq!((next.task, next.attempt), (first.task, 2));
        let expected: Vec<Box<[u8]>> =
            (1..=window_len).map(|turn| Box::<[u8]>::from([b'x', u8::try_from(turn).expect("short window")])).collect();
        assert_eq!(next.transcript.as_ref(), expected.as_slice());
        assert_eq!(next.messages.iter().map(|message| message.words.as_ref()).collect::<Vec<_>>(), [[50], [51]]);
        assert_eq!(driver.task(first.task).expect("resumed chat").tries, tasks::Tries::NONE);
        no_transcript_failure(&mut driver);
    }
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
// the root reports it as a fact of the attempt (domain/root.md, 6.4).
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
