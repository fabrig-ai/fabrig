use crate::tests::*;

const PREPARING: u64 = 1;
const DEPENDENCY: u64 = 12;
const INPUT: u64 = 13;
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

#[test]
fn two_preparations_share_one_load_slot_without_failing() {
    use crate::engine::{self, Engine, In, Out, Peer};
    const SECOND: u64 = 2;
    let (mut core, mut limits) = preparing();
    limits.load_slots = 1;
    core.contexts.get_mut(&PREPARING).unwrap().spec.inputs = Box::new([]);
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(SECOND)), task: SECOND });
    let mut context = core.tasks.run_context(SECOND).unwrap();
    context.dependencies = Box::new([INPUT]);
    core.contexts.insert(SECOND, context).unwrap();
    core.begin_transcript(SECOND, 0, None);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut engine = Engine::from_core(core);
    let mut out = Queue::uncapped();
    let window = fixture_window(engine.core_mut(), PREPARING, (1, 1));
    engine::step(
        &mut engine,
        &env,
        In::Store(engine_store::In::Loaded {
            load: window,
            result: engine_store::Answer::Rows(engine_store::Rows::Turns { turns: Box::new([]), more: false }),
        }),
        &mut out,
    );
    let Some(Out::Held(Peer::Store(engine_store::Out::Load { load: first, range }))) = out.pop() else {
        panic!("first result load")
    };
    assert_eq!(range, engine_store::Range::Record(Key::Tasks(tasks::Key::Ended(DEPENDENCY))));
    assert!(out.is_empty());
    let window = fixture_window(engine.core_mut(), SECOND, (1, 1));
    engine::step(
        &mut engine,
        &env,
        In::Store(engine_store::In::Loaded {
            load: window,
            result: engine_store::Answer::Rows(engine_store::Rows::Turns { turns: Box::new([]), more: false }),
        }),
        &mut out,
    );
    assert!(out.is_empty(), "no slot is free for a new load or failure");
    engine::step(&mut engine, &env, In::Store(ended_answer(first, Some(ended(DEPENDENCY)), false)), &mut out);
    let mut second = None;
    while let Some(output) = out.pop() {
        match output {
            Out::Held(Peer::Store(engine_store::Out::Load { load, range })) => {
                assert_eq!(range, engine_store::Range::Record(Key::Tasks(tasks::Key::Ended(INPUT))));
                assert!(second.replace(load).is_none());
            }
            Out::Core(Request::Now(now)) => assert_no_failure(&now),
            Out::Core(Request::Held(_) | Request::Write(_) | Request::Ask { .. } | Request::ProjectMade { .. }) => {}
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
            && let Held::NotesLoad { .. } = held.as_ref()
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
