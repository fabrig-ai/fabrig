use crate::engine::{self, Engine, In, Out, Peer};
use crate::tests::*;

const PREPARING: u64 = 1;
const DEPENDENCY: u64 = 12;
const INPUT: u64 = 13;
const SECOND: u64 = 2;
const PROJECT: u32 = 1;
const WINDOW: (u64, u32) = (1, 1);
const CHARTER: u32 = 1;
const TEXT_BYTES: u32 = 256; // Both fixture results and their added line fit whole.
const TEXT_LIMITS: BriefTextLimits = BriefTextLimits { parts: 1, read_bytes: TEXT_BYTES, brief_bytes: TEXT_BYTES };

fn preparing() -> (Core, Limits) {
    let (mut core, limits) = make_tree();
    send_task(
        &mut core,
        &limits,
        tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(PREPARING)), task: PREPARING },
    );
    let mut context = core.tasks.run_context(PREPARING).unwrap();
    context.dependencies = Box::new([DEPENDENCY]);
    context.spec.inputs = Box::new([INPUT]);
    core.contexts.insert(PREPARING, context).unwrap();
    core.begin_transcript(PREPARING, 0, None);
    (core, limits)
}

pub(in crate::tests) fn requested(out: &[Request], wanted: u64) -> Token {
    for request in out {
        if let Request::Held(held) = request
            && let Held::Store(engine_store::Out::Load {
                load: request,
                range: engine_store::Range::Record(Key::Tasks(tasks::Key::Ended(task))),
            }) = held.as_ref()
        {
            assert_eq!(*task, wanted);
            return *request;
        }
    }
    panic!("ended result requested");
}

fn start(core: &mut Core, limits: &Limits) -> Token {
    requested(&last_empty_window_page(core, limits, PREPARING), DEPENDENCY)
}

fn ended(number: u64) -> Box<tasks::TaskRecord> {
    Box::new(fixtures::ended_report(number, tasks::Party::Person(1)))
}

fn answer(core: &mut Core, limits: &Limits, request: Token, row: Option<Box<tasks::TaskRecord>>) -> Box<[Request]> {
    watch_store_outputs(core, limits, ended_answer(request, row, false))
}

fn shared_slot_fixture() -> (Core, Limits) {
    let (mut core, mut limits) = tree_core();
    limits.load_slots = 1;
    core.opened = true;
    let funder = tasks::Funder::Period { project: PROJECT, period: 1 };
    let first = new_row(&core, PREPARING, funder, None);
    let second = new_row(&core, SECOND, funder, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([first, second]));
    let mut first = core.tasks.task(PREPARING).unwrap().clone();
    first.dependencies = Box::new([DEPENDENCY]);
    first.attempt = WINDOW.0;
    let mut second = core.tasks.task(SECOND).unwrap().clone();
    second.spec.inputs = Box::new([INPUT]);
    let ledger = *core.tasks.funding(funder).unwrap();
    fixtures::restore(&mut core, &limits, Box::new([first, second]));
    send_task(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) });
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    let mut proof = RunProof::claimed(PREPARING, WINDOW.0, 1, None, fleet::HostKind::Worker);
    proof.window = Some(CurrentWindow { attempt: WINDOW.0, turn: WINDOW.1, closed: false });
    core.proofs.insert(PREPARING, proof).unwrap();
    runs::run_choice::host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    runs::run_choice::account(&mut core, &limits, accounts::Event::Want);
    runs::run_choice::granted(&mut core, &limits, 1);
    warm_shared_notes(&mut core, &limits);
    (core, limits)
}

fn warm_shared_notes(core: &mut Core, limits: &Limits) {
    // Keep the brief's later notes stage from competing for the ended-result slot.
    const SCOPES: u32 = 2;
    let mut scopes = List::with_capacity(SCOPES);
    scopes.push(notes::Scope::Project { project: PROJECT }).unwrap();
    scopes.push(notes::Scope::Deployment).unwrap();
    let mut out = watch_outputs(
        core,
        limits,
        Event::Notes(notes::Event::Index { owner: Token::new(700), scopes, most: limits.notes.lines }),
    );
    for _ in 0..SCOPES {
        let [Request::Held(held)] = out.as_ref() else { panic!("one cold notes scope") };
        let Held::Store(engine_store::Out::Load { load, range: engine_store::Range::Notes(_) }) = held.as_ref() else {
            panic!("notes load")
        };
        out = watch_store_outputs(core, limits, point_answer(*load, None, false));
    }
    let [Request::Now(now)] = out.as_ref() else { panic!("notes index complete") };
    let Now::NotesIndexed { .. } = now.as_ref() else { panic!("notes indexed") };
}

fn activate(engine: &mut Engine, env: &Env<Limits>, task: u64) -> Queue<Out> {
    let context = engine.core().tasks.run_context(task).unwrap();
    let mut outputs = Queue::uncapped();
    engine::step(engine, env, In::Core(Event::Activate { context, ready: true }), &mut outputs);
    let mut out = Queue::uncapped();
    while let Some(output) = outputs.pop() {
        if let Out::Write(_) = output {
            continue;
        }
        out.push(output);
    }
    out
}

#[test]
fn two_preparations_share_one_load_slot_without_failing() {
    let (core, limits) = shared_slot_fixture();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut engine = Engine::from_core(core);
    let mut out = activate(&mut engine, &env, PREPARING);
    let Some(Out::Held(Peer::Store(engine_store::Out::Load {
        load: window,
        range: engine_store::Range::Window { task: PREPARING, from: WINDOW },
    }))) = out.pop()
    else {
        panic!("first preparation's transcript window");
    };
    assert!(out.is_empty());
    let turn = TurnRecord {
        task: PREPARING,
        attempt: WINDOW.0,
        turn: WINDOW.1,
        spent: 0,
        read: None,
        opens_window: true,
        at: Wall::EPOCH,
        transcript: Box::new([]),
    };
    engine::step(
        &mut engine,
        &env,
        In::Store(engine_store::In::Loaded {
            load: window,
            result: engine_store::Answer::Rows(engine_store::Rows::Turns {
                turns: Box::new([(turn, None)]),
                more: false,
            }),
        }),
        &mut out,
    );
    let Some(Out::Held(Peer::Store(engine_store::Out::Load { load: first, range }))) = out.pop() else {
        panic!("first result load")
    };
    assert_eq!(range, engine_store::Range::Record(Key::Tasks(tasks::Key::Ended(DEPENDENCY))));
    assert!(out.is_empty());
    // A fresh run has no window to open, so its result can queue behind the first.
    let mut out = activate(&mut engine, &env, SECOND);
    assert!(out.is_empty(), "no slot is free for a new load or failure");
    assert_both_preparing(&engine);
    engine::step(&mut engine, &env, In::Store(ended_answer(first, Some(ended(DEPENDENCY)), false)), &mut out);
    let mut second = None;
    while let Some(output) = out.pop() {
        match output {
            Out::Held(Peer::Store(engine_store::Out::Load { load, range })) => {
                assert_eq!(range, engine_store::Range::Record(Key::Tasks(tasks::Key::Ended(INPUT))));
                assert!(second.replace(load).is_none());
            }
            Out::Core(Request::Now(now)) => assert_no_failure(&now),
            Out::Write(_)
            | Out::Core(Request::Held(_) | Request::Ask { .. } | Request::ProjectMade { .. })
            | Out::Ask(..)
            | Out::Held(Peer::Connector(..)) => {}
            Out::Core(Request::Write(_)) => panic!("writes use the store port"),
            Out::Now(_)
            | Out::Held(Peer::Application(_) | Peer::Resume(_) | Peer::Store(engine_store::Out::Switch)) => {
                panic!("no start is driven")
            }
        }
    }
    engine::step(
        &mut engine,
        &env,
        In::Store(ended_answer(second.expect("second result load"), Some(ended(INPUT)), false)),
        &mut out,
    );
    while let Some(output) = out.pop() {
        if let Out::Core(Request::Now(now)) = output {
            assert_no_failure(&now);
        }
    }
    assert_both_preparing(&engine);
}

fn assert_both_preparing(engine: &Engine) {
    for task in [PREPARING, SECOND] {
        assert_eq!(engine.core().tasks.task(task).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    }
}

fn assert_no_failure(now: &Now) {
    if let Now::BriefFailed { task, failure } = now {
        panic!("task {task}: {failure:?}");
    }
}

fn assert_empty(out: &[Request]) {
    assert!(out.is_empty());
}

#[test]
fn dependencies_and_inputs_load_in_order_before_the_notes() {
    let (mut core, limits) = preparing();
    let request = start(&mut core, &limits);
    assert_eq!(core.note_flights(), 0);
    assert!(core.dependency_results.is_empty());
    let out = answer(&mut core, &limits, request, Some(ended(DEPENDENCY)));
    let next = requested(&out, INPUT);
    assert_ne!(next, request);
    assert_eq!(core.note_flights(), 0);
    assert!(core.dependency_results.is_empty());
    let out = answer(&mut core, &limits, next, Some(ended(INPUT)));
    assert_eq!(core.result_read_flights(), 0);
    assert_eq!(
        read_brief_part(&core, TEXT_LIMITS, PREPARING, BriefPart::Dependencies).unwrap().as_ref(),
        b"task 12: report\ndone\n\ntask 13: report\ndone\n\n",
    );
    let mut notes_loaded = false;
    for request in &out {
        if let Request::Held(held) = request
            && let Held::Store(engine_store::Out::Load { range: engine_store::Range::Notes(_), .. }) = held.as_ref()
        {
            notes_loaded = true;
        }
    }
    assert!(notes_loaded);
}

#[test]
fn abandoning_preparation_drops_its_read_and_ignores_late_answers() {
    let (mut core, limits) = preparing();
    let request = start(&mut core, &limits);
    watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::Hold { task: PREPARING, why: tasks::Hold::Budget }));
    assert_eq!(core.result_read_flights(), 0);
    let phase = core.tasks.task(PREPARING).unwrap().phase.clone();
    let out = answer(&mut core, &limits, request, None);
    assert_empty(&out);
    assert_eq!(core.tasks.task(PREPARING).unwrap().phase, phase);
    assert_eq!(core.result_read_flights(), 0);
    assert!(core.dependency_results.is_empty());
    assert!(core.contexts.is_empty());
    assert_eq!(core.note_flights(), 0);
}

#[test]
fn a_reply_for_a_changed_preparation_is_silently_dropped() {
    let (mut core, limits) = preparing();
    let request = start(&mut core, &limits);
    core.contexts.get_mut(&PREPARING).unwrap().last_message = 1;
    let out = answer(&mut core, &limits, request, None);
    assert_empty(&out);
    assert_eq!(core.tasks.task(PREPARING).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert_eq!(core.result_read_flights(), 0);
    assert!(core.dependency_results.is_empty());
    assert_eq!(core.note_flights(), 0);
}

#[test]
fn unavailable_or_rejected_results_back_off_without_spending_attempts() {
    let mut wrong_project = ended(DEPENDENCY);
    wrong_project.project = 2;
    let mut live = ended(DEPENDENCY);
    live.phase = tasks::Phase::Active(tasks::Active::Due);
    for (label, row, busy) in [
        ("missing", None, false),
        ("busy", Some(ended(DEPENDENCY)), true),
        ("wrong task", Some(ended(INPUT)), false),
        ("wrong project", Some(wrong_project), false),
        ("live task", Some(live), false),
    ] {
        let (mut core, limits) = preparing();
        let request = start(&mut core, &limits);
        let out = watch_store_outputs(&mut core, &limits, ended_answer(request, row, busy));
        let row = core.tasks.task(PREPARING).unwrap();
        let tasks::Phase::Active(tasks::Active::BackingOff { .. }) = row.phase else {
            panic!("{label}: preparation did not back off");
        };
        assert_eq!(row.attempt, 0, "{label}");
        assert_eq!(row.tries, tasks::Tries { transient: 1, ..tasks::Tries::NONE }, "{label}");
        assert_eq!(
            row.retry_notice,
            Some(tasks::RetryNotice::Failed {
                class: tasks::Class::Transient,
                count: 1,
                allowance: limits.tasks.retries.transient.retries,
                reason: Some(b"results: a dependency's result could not be read".as_slice().into()),
            }),
            "{label}"
        );
        assert!((core.result_read_flights() == 0), "{label}");
        assert!(core.contexts.is_empty(), "{label}");
        assert!(core.dependency_results.is_empty(), "{label}");
        assert!((core.note_flights() == 0), "{label}");
        let mut failures = 0_u32;
        for request in &out {
            if let Request::Now(now) = request
                && let Now::BriefFailed { task, failure } = now.as_ref()
            {
                assert_eq!(*task, PREPARING);
                assert_eq!(
                    *failure,
                    jig_domain_brief::GatherFailure {
                        fate: jig_domain_brief::GatherFate::Transient,
                        section: b"results",
                        words: Box::from(b"a dependency's result could not be read".as_slice()),
                    }
                );
                failures = failures.checked_add(1).unwrap();
            }
        }
        assert_eq!(failures, 1, "{label}: root hears the classified failure once");
    }
}

#[test]
fn added_dependency_lines_name_the_time_party_and_outcome() {
    let (mut core, _) = make_tree();
    for (party, kind, expected) in [
        (
            tasks::Party::Task(4),
            tasks::ResultKind::Report,
            b"waited for task 12, added 99 by task 4: done\ntask 12: report\nresult\n\n".as_slice(),
        ),
        (
            tasks::Party::Person(5),
            tasks::ResultKind::Failed,
            b"waited for task 12, added 99 by person 5: failed\ntask 12: failed\nresult\n\n".as_slice(),
        ),
        (
            tasks::Party::Deployment { project: 6 },
            tasks::ResultKind::Cancelled,
            b"waited for task 12, added 99 by deployment for project 6: cancelled\ntask 12: cancelled\nresult\n\n"
                .as_slice(),
        ),
    ] {
        let mut context = settings::charters::context(PREPARING, CHARTER);
        context.added = Box::new([tasks::Added { task: DEPENDENCY, at: Wall::from_nanos(99), by: party }]);
        core.contexts.insert(PREPARING, context).unwrap();
        core.dependency_results
            .insert(
                PREPARING,
                Box::new([RetainedResult { task: DEPENDENCY, kind, words: b"result".as_slice().into() }]),
            )
            .unwrap();
        assert_eq!(read_brief_part(&core, TEXT_LIMITS, PREPARING, BriefPart::Dependencies).unwrap().as_ref(), expected);
    }
}
