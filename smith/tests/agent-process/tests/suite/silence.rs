//! The host can keep both pipe ends open and still cease making progress.

use skein_fake_llm_domain::api;
use skein_io::kernel::Exit;
use skein_lib::{Duration, List, Reader, Time};
use smith_agent_process_world::{TraceCase, World, charter, wire::edit_charter};
use smith_agent_world::fixture;
use smith_channel::{Record, RunResult, kind};
use smith_events::{AnswerClass, Event};

const SILENT: &str = "smith: the run could not answer: the host was silent past its deadline";
const WAKE: &[u8] = b"Now answer.";

fn after_verdict(world: &mut World) -> (Time, Option<u16>) {
    assert!(world.settle(), "the silent host's agent exits without outside help");
    assert_eq!(world.exit(), Some(Exit::Code(1)));
    assert!(world.answer().is_none());
    assert!(world.writing_end_at_exit());
    let (progress, verdict, write_aborted) = world.silence_observations();
    let exit = world.exited_at().expect("process exit");
    let link = world.link();
    let bound = smith_agent_profile::orphan_exit(&link).expect("derived orphan exit");
    assert!(verdict >= progress.saturating_add(link.no_progress), "full silence interval after last progress");
    assert!((verdict..=verdict.saturating_add(bound)).contains(&exit));
    let verdict_ms = verdict.saturating_since(Time::ZERO).as_nanos() / 1_000_000;
    assert!(
        world
            .records()
            .iter()
            .all(|record| { !matches!(record.event, Event::ResponseStarted(_)) || record.t_ms < verdict_ms }),
        "no completion starts after the verdict"
    );
    assert!(matches!(world.records().last().expect("session trace").event, Event::SessionEnded(_)));
    let stderr = String::from_utf8(world.errors()).expect("operator line");
    assert_eq!(stderr.lines().last(), Some(SILENT));
    assert!(!world.signalled());
    world.assert_agent_clean();
    eprintln!(
        "last progress {progress:?}; verdict {verdict:?}; exit {exit:?}; no_progress {:?}; orphan_exit {bound:?}",
        link.no_progress
    );
    (verdict, write_aborted)
}

fn call(name: &[u8], arguments: &[u8]) -> api::Line {
    api::Line::Call { name: name.into(), arguments: arguments.into() }
}

fn finishes(text: &str) -> api::Turn {
    let report = serde_json::json!({"form":"report","text":text,"fields":{}}).to_string();
    fixture::calls(Box::new([call(b"finish", report.as_bytes())]), 1)
}

fn open_waiter(seed: u64) -> (World, Duration) {
    let base = charter();
    let interval = World::new(seed, &base).link().no_progress;
    let delay = interval.checked_add(Duration::from_nanos(interval.as_nanos() / 2)).expect("past one interval");
    let charter = edit_charter(&base, |parts| {
        parts.mode = smith_charter::Mode::Open;
        smith_agent_process_world::wire::budget(parts, 8, 20);
        parts.budget.time = interval.checked_add(interval).expect("time for the wait and the answer");
        parts.budget.reserve =
            smith_charter::Reserve { turns: 1, time: Duration::from_nanos(interval.as_nanos() / 16) };
        parts.waiting = parts.budget.time;
        parts.tools.wait = true;
    });
    let mut world = World::new(seed, &charter);
    world.observe_facts(TraceCase::Accepted, 32);
    world.initial_message(17, b"request", b"Wait for my next message.");
    (world, delay)
}

#[test]
fn a_host_that_stops_taking_frames_is_lost_while_the_run_waits() {
    let (mut world, _) = open_waiter(31);
    world.stop_reading_after(kind::ADMITTED);
    world.ask(true, false, false);
    world.script(Box::new([fixture::calls(
        Box::new([api::Line::Text { text: vec![b'x'; 600].into_boxed_slice() }, call(b"wait", b"{}")]),
        1,
    )]));
    let (_, write_aborted) = after_verdict(&mut world);
    assert!(write_aborted.is_some(), "queued output was cut, not finished");
    assert_eq!(world.peer_queries().len(), 1);
    assert!(world.records().iter().any(|record| {
        matches!(&record.event, Event::RunCompleted(end) if end.failure.as_ref().is_some_and(|failure| failure.class == AnswerClass::Cancelled))
    }), "the run was still live when the host stopped making progress");
}

#[test]
fn a_host_that_reads_but_never_keeps_the_only_turn_is_lost() {
    let base = charter();
    let interval = World::new(32, &base).link().no_progress;
    let charter = edit_charter(&base, |parts| {
        smith_agent_process_world::wire::budget(parts, 8, 20);
        parts.budget.time = interval.checked_add(interval).expect("time for the host verdict");
    });
    let mut world = World::new(32, &charter);
    world.observe_facts(TraceCase::Accepted, 32);
    world.acknowledgement_window(1);
    world.withhold_host_replies();
    world.script(Box::new([
        fixture::says(Box::new([api::Line::Text { text: Box::from(*b"Still working.") }]), 1),
        finishes("done"),
    ]));
    let (_, write_aborted) = after_verdict(&mut world);
    assert_eq!(write_aborted, None, "the output pipe was drained, not the turn window");
    assert_eq!(world.observed().iter().filter(|frame| frame.kind == kind::TURN).count(), 1);
    assert_eq!(world.peer_queries().len(), 1, "there was no room to start the next turn");
}

#[test]
fn a_host_that_leaves_a_withdrawal_unanswered_is_lost() {
    let base = charter();
    let interval = World::new(33, &base).link().no_progress;
    let charter = edit_charter(&base, |parts| {
        smith_agent_process_world::wire::budget(parts, 8, 20);
        parts.budget.time = interval.checked_add(interval).expect("time for the withdrawal and verdict");
        parts.tools.host = List::with_capacity(1);
        parts
            .tools
            .host
            .push(smith_charter::HostTool {
                reserve: false,
                name: Box::from(*b"lookup"),
                description: Box::from(*b"Look up"),
                input: Box::from(*br#"{"type":"object","properties":{}}"#),
                effect: smith_charter::Effect::Read,
                deadline: Duration::from_secs(1),
            })
            .expect("host tool");
    });
    let mut world = World::new(33, &charter);
    world.observe_facts(TraceCase::Accepted, 32);
    world.withhold_host_replies();
    world.script(Box::new([fixture::calls(Box::new([call(b"lookup", b"{}")]), 1)]));
    let (_, write_aborted) = after_verdict(&mut world);
    assert_eq!(write_aborted, None, "the withdrawal crossed the channel");
    let names: Vec<_> = world
        .observed()
        .iter()
        .filter_map(|frame| {
            let mut reader = Reader::new(&frame.body);
            match frame.kind {
                kind::CALL => {
                    Some(smith_channel::Call::decode(&smith_channel::CEILINGS, &mut reader).expect("call").name)
                }
                kind::WITHDRAW => Some(
                    smith_channel::Withdraw::decode(&smith_channel::CEILINGS, &mut reader).expect("withdrawal").name,
                ),
                _ => None,
            }
        })
        .collect();
    assert_eq!(names.len(), 2, "one call followed by one withdrawal");
    assert_eq!(names[0], names[1]);
    assert_eq!(world.peer_queries().len(), 1);
}

#[test]
fn a_queued_answer_not_taken_by_the_host_does_not_count_as_sent() {
    let charter = edit_charter(&charter(), |parts| parts.contract.report.as_mut().expect("report").max = 1024);
    let mut world = World::new(34, &charter);
    world.observe_facts(TraceCase::Accepted, 32);
    world.stop_reading_after(kind::TURN);
    world.script(Box::new([finishes(&"a".repeat(768))]));
    let (verdict, write_aborted) = after_verdict(&mut world);
    assert_eq!(write_aborted, Some(kind::ANSWER), "the undelivered answer's write was aborted");
    let end = world
        .records()
        .iter()
        .find(|record| matches!(record.event, Event::RunCompleted(_)))
        .expect("run answered locally");
    assert!(Duration::from_millis(end.t_ms).as_nanos() < verdict.saturating_since(Time::ZERO).as_nanos());
    let Event::RunCompleted(answer) = &end.event else { unreachable!("selected the answer") };
    assert!(answer.failure.is_none(), "the local answer succeeded but did not cross the channel");
}

#[test]
fn an_idle_host_that_owes_nothing_can_wake_the_run_later() {
    let (mut world, delay) = open_waiter(35);
    world.script(Box::new([
        fixture::calls(Box::new([call(b"wait", b"{}")]), 1),
        fixture::says(Box::new([api::Line::Text { text: Box::from(*b"Done.") }]), 1),
    ]));
    world.wake_waiting_after(delay);
    assert!(world.settle());
    let (waiting, wake) = world.wake_observations();
    assert!(wake > waiting.saturating_add(world.link().no_progress));
    assert!(world.observed().iter().any(|frame| frame.kind == kind::WAITING));
    let answer = world.answer().expect("the host's next message woke the run");
    assert_eq!(answer.last_read, Some(18));
    assert!(matches!(answer.result, RunResult::Parked), "the open run parks after its reply: {answer:?}");
    assert_eq!(world.exit(), Some(Exit::Code(0)));
    assert_eq!(world.peer_queries().len(), 2);
    for (query, expected) in world.peer_queries().into_iter().zip([b"Wait for my next message.".as_slice(), WAKE]) {
        assert!(
            query.messages.iter().flat_map(|message| &message.parts).any(|part| {
                matches!(part, api::Part::Text { text } if skein_lib::bytes::find(text, expected).is_some())
            }),
            "the run read each message into its next completion"
        );
    }
    world.assert_agent_clean();
    eprintln!("idle wake {wake:?}; exit {:?}; no verdict", world.exited_at().expect("process exit"));
}
