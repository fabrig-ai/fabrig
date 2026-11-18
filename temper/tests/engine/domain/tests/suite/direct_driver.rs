//! Direct draining keeps the fixed-step driver's observations and injected clocks.
use skein_lib::{Token, Wall};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, fixtures, protocol};

fn observation(driver: &Driver) -> String {
    format!(
        "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}",
        driver.store.rows,
        driver.delivered,
        driver.events,
        driver.store.pending,
        driver.root.next_deadline(&driver.env),
        driver.env.now,
        driver.env.wall,
    )
}

#[test]
fn a_store_fork_keeps_transcript_bytes_after_its_source_drops_them() {
    use jig_domain_core::{CoreKey, CoreRecord, Key, Record};
    let (mut driver, run) = Driver::chat();
    driver.send(protocol::turn(Token::new(7), run.task, run.attempt, 1, 1, None, b"kept transcript"));
    driver.pump();
    let mut source = driver.take_store();
    let mut copy = source.fork();
    let range = engine::Range::Core(engine::CoreRange::Turns { task: run.task, attempt: run.attempt });
    let before = source.page_with_texts(&range, None, 2);
    assert!(!before.0.is_empty(), "the source has a committed turn");
    let before = format!("{before:?}");
    assert_eq!(format!("{:?}", copy.page_with_texts(&range, None, 2)), before);
    let mut header = source.header();
    header.commits += 1;
    source.submit(
        header.commits,
        Box::new([
            engine::Write::DropTranscript { task: run.task },
            engine::Write::Save(engine::Record::Core(Record::Core(CoreRecord::Deployment(header)))),
        ]),
        0,
    );
    source.apply();
    assert!(matches!(source.page_terminal(&range, None, 2), engine::StoreInput::Unloaded { .. }));
    assert_eq!(
        format!("{:?}", copy.page_with_texts(&range, None, 2)),
        before,
        "copied transcript logs and maps are independent"
    );
    let original = source.header();
    let mut header = copy.header();
    header.messages += 1;
    copy.seed(Box::new([engine::Record::Core(Record::Core(CoreRecord::Deployment(header)))]), Box::new([]));
    assert_eq!(source.header(), original, "a fork's later commit cannot change its source");
    let page = copy.page(&engine::Range::Core(engine::CoreRange::Deployment), None, 1);
    assert_eq!(page.0[0].key(), engine::Key::Core(Key::Core(CoreKey::Deployment)));
    assert_eq!(copy.header().messages, header.messages);
    source.crash(0);
    copy.crash(0);
    assert_eq!(
        format!("{:?}", copy.page_with_texts(&range, None, 2)),
        before,
        "both copies cold-open their durable backing"
    );
}

#[test]
fn draining_keeps_turns_calls_and_due_timers_at_the_same_instant() {
    let (mut drained, run) = Driver::chat();
    let (mut stepped, other) = Driver::chat();
    assert_eq!((run.task, run.attempt), (other.task, other.attempt));
    for driver in [&mut drained, &mut stepped] {
        driver.send(protocol::turn(Token::new(7), run.task, run.attempt, 1, 1, None, b"durable words"));
    }
    drained.pump();
    for _ in 0..200 {
        stepped.advance(true);
    }
    assert_eq!(observation(&drained), observation(&stepped));
    let call = 42;
    fixtures::tool_call(&mut drained, &run, call, engine::Tool::Delegate { batch: Box::new([]) });
    stepped.call(
        Token::new(7),
        run.task,
        run.attempt,
        call,
        engine::Call { completion: 1, position: 42, tool: engine::Tool::Delegate { batch: Box::new([]) } },
    );
    for _ in 0..60 {
        stepped.advance(true);
    }
    assert!(drained.answered(call).is_some(), "the call's durable answer drains before returning");
    assert_eq!(observation(&drained), observation(&stepped));

    let due = drained.root.next_deadline(&drained.env).expect("the live run has an alarm");
    for driver in [&mut drained, &mut stepped] {
        driver.env.now = due;
        driver.env.wall = Wall::from_nanos(due.as_nanos());
    }
    drained.pump();
    for _ in 0..200 {
        stepped.advance(true);
    }
    assert_eq!(drained.env.now, due, "pump never advances the story's clock");
    assert_eq!(observation(&drained), observation(&stepped), "a due alarm still waits for an explicit fire");
}
