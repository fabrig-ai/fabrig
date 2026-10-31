//! Completion-fed capture and input steps; policy time is supplied, with no kernel or child.

use super::*;
use crate::agent::{Close, Cue, Person, Script, Seen, Trigger};

use crate::agent::smith::recorded_events as events;

// The capture boundary is the eight-byte prefix asserted by the runner counterpart.
const CAPTURE_BYTES: usize = 8;
// The supplied observer pins its encoded-line ceiling at 65,536 bytes.
const TRACE_BYTES: usize = 65536;
const TERMINAL: Fd = Fd::new(10);
const PIPE: Fd = Fd::new(11);
const ANSWER_AT: Time = Time::from_nanos(159_000_000);
const HARD_AT: Time = Time::from_nanos(160_000_000);
const TEARDOWN: Duration = Duration::from_millis(180);
const GRACE: Duration = Duration::from_millis(20);
const TEARDOWN_AT: Time = Time::from_nanos(339_000_000);
const KILL_AT: Time = Time::from_nanos(359_000_000);

fn policy() -> Policy {
    Policy {
        deadline: Some(HARD_AT),
        teardown: TEARDOWN,
        grace: GRACE,
        answer_seen: false,
        ladder: Ladder::Watching,
        settled: false,
    }
}

fn raw_output() -> Output {
    let person = Person::new(Script { cues: Vec::new(), close: Close::AfterPrompt }).expect("empty person");
    Output::new(TERMINAL, PIPE, person, CAPTURE_BYTES, Capture::Raw)
}

fn read(output: &mut Output, bytes: &[u8]) {
    output.completions.push(Complete {
        op: Token::new(0),
        kind: Op::PipeRead { fd: PIPE, buf: bytes.into() },
        result: Ok(Done::Count(u32::try_from(bytes.len()).expect("fixture bytes"))),
    });
    output.iterate(Time::ZERO, Wall::EPOCH);
}

#[test]
fn stdout_at_its_bound_retains_every_byte_without_fault() {
    let mut output = raw_output();
    read(&mut output, b"1234");
    read(&mut output, b"5678");
    assert_eq!(output.bytes, b"12345678");
    assert_eq!(output.error, None);
}

#[test]
fn stdout_one_past_its_bound_keeps_its_prefix_and_fault() {
    let mut output = raw_output();
    read(&mut output, b"123456789");
    read(&mut output, b"discarded");
    assert_eq!(output.bytes, b"12345678");
    assert_eq!(output.error.as_deref(), Some("stdout exceeds the attempt log bound"));
}

#[test]
fn settled_stderr_overflow_still_reports_a_capture_failure() {
    let mut output = raw_output();
    output.tree = Tree::Settled;
    output.stderr_overflow(true);
    output.stderr_overflow(false);
    let mut policy = policy();
    assert_eq!(policy.advance(HARD_AT, None, true, output.error.is_some()), None);
    assert!(policy.settled);
    assert_eq!(output.error.as_deref(), Some("stderr exceeds the attempt log bound"));
}

#[test]
fn complete_stderr_capture_does_not_report_overflow() {
    let mut output = raw_output();
    output.stderr_overflow(false);
    assert_eq!(output.error, None);
}

#[test]
fn late_answer_receives_teardown_beyond_the_hard_deadline() {
    let mut policy = policy();
    assert_eq!(policy.advance(ANSWER_AT, Some(ANSWER_AT), false, false), None);
    assert_eq!(policy.deadline, Some(TEARDOWN_AT));
    assert_eq!(policy.advance(HARD_AT, Some(ANSWER_AT), false, false), None);
    assert_eq!(policy.advance(Time::from_nanos(338_999_999), Some(ANSWER_AT), true, false), None);
    assert!(policy.settled);
    assert_eq!(policy.ladder, Ladder::Watching);
    assert_eq!(policy.deadline, None);
}

#[test]
fn teardown_expiry_terminates_then_kills_after_full_grace() {
    let mut policy = policy();
    policy.advance(ANSWER_AT, Some(ANSWER_AT), false, false);
    assert_eq!(policy.advance(TEARDOWN_AT, Some(ANSWER_AT), false, false), Some(Signal::Terminate));
    assert_eq!(policy.deadline, Some(KILL_AT));
    assert_eq!(policy.advance(Time::from_nanos(358_999_999), Some(ANSWER_AT), false, false), None);
    assert_eq!(policy.advance(KILL_AT, Some(ANSWER_AT), false, false), Some(Signal::Kill));
    assert_eq!(policy.advance(KILL_AT, Some(ANSWER_AT), false, false), None);
}

#[test]
fn unanswered_program_terminates_exactly_at_its_hard_deadline() {
    let mut policy = policy();
    assert_eq!(policy.advance(ANSWER_AT, None, false, false), None);
    assert_eq!(policy.advance(HARD_AT, None, false, false), Some(Signal::Terminate));
}

fn terminal_output(cues: Vec<Cue>, close: Close, capture: Capture) -> Output {
    let person = Person::new(Script { cues, close }).expect("script");
    Output::new(TERMINAL, TERMINAL, person, TRACE_BYTES, capture)
}

fn start() -> Cue {
    Cue { when: Trigger::Start, text: "prompt\n".into() }
}

fn cue(tool: &str, text: &str) -> Cue {
    Cue { when: Trigger::ToolCall { tool: tool.into() }, text: text.into() }
}

fn writes(output: &mut Output) -> Vec<Submit> {
    output.write_input();
    std::iter::from_fn(|| output.submissions.pop()).collect()
}

fn acknowledge(output: &mut Output, expected: &[u8]) {
    let mut writes = writes(output);
    assert_eq!(writes.len(), 1, "one outstanding input write");
    let submit = writes.remove(0);
    let Op::PipeWrite { ref bytes, from, .. } = submit.kind else { panic!("input write") };
    assert_eq!(&bytes[usize::try_from(from).expect("offset")..], expected);
    output.completions.push(Complete {
        op: submit.op,
        kind: submit.kind,
        result: Ok(Done::Count(u32::try_from(expected.len()).expect("bounded line"))),
    });
    output.iterate(Time::ZERO, Wall::EPOCH);
    // The step may schedule the next input and the terminal read; retain only input effects.
    let submissions: Vec<_> = std::iter::from_fn(|| output.submissions.pop()).collect();
    for submit in submissions {
        if matches!(submit.kind, Op::PipeWrite { .. }) {
            output.submissions.push(submit);
        }
    }
}

#[test]
fn matching_tool_cue_releases_one_line_and_then_eof() {
    let mut output = terminal_output(vec![start(), cue("finish", "done\n")], Close::AfterLastCue, Capture::Raw);
    acknowledge(&mut output, b"prompt\n");
    output.person.see(&Seen::ToolCall { tool: "read".into() });
    assert!(writes(&mut output).is_empty());
    output.person.see(&Seen::ToolCall { tool: "finish".into() });
    acknowledge(&mut output, b"done\n");
    output.person.see(&Seen::ToolCall { tool: "finish".into() });
    acknowledge(&mut output, b"\x04");
    assert!(writes(&mut output).is_empty());
}

#[test]
fn later_line_waits_for_named_cue_before_terminal_eof() {
    let mut output = terminal_output(
        vec![start(), cue("read", "middle\n"), cue("finish", "last\n")],
        Close::AfterLastCue,
        Capture::Raw,
    );
    acknowledge(&mut output, b"prompt\n");
    output.person.see(&Seen::ToolCall { tool: "finish".into() });
    assert!(writes(&mut output).is_empty());
    output.person.see(&Seen::ToolCall { tool: "read".into() });
    acknowledge(&mut output, b"middle\n");
    output.person.see(&Seen::ToolCall { tool: "read".into() });
    assert!(writes(&mut output).is_empty());
    output.person.see(&Seen::ToolCall { tool: "finish".into() });
    acknowledge(&mut output, b"last\n");
    acknowledge(&mut output, b"\x04");
    assert!(writes(&mut output).is_empty());
}

fn failed_capture() -> (Capture, Vec<u8>) {
    let mut records = events::balanced();
    let failed = events::record(include_str!("../../../crates/smith-events/golden/v4/answer_class_budget.jsonl"));
    let smith_events::Event::RunCompleted(failed) = failed.event else { panic!("run terminal") };
    let mut edited = 0;
    let mut modes = 0;
    for record in &mut records {
        if let smith_events::Event::SessionStarted(value) = &mut record.event {
            value.mode = smith_events::Mode::Chat;
            modes += 1;
        }
        if let smith_events::Event::RunCompleted(value) = &mut record.event {
            value.status = smith_events::Status::Failed;
            value.result = None;
            value.failure.clone_from(&failed.failure);
            value.failure.as_mut().expect("budget").which = Some(Box::from(&b"time"[..]));
            edited += 1;
        }
    }
    assert_eq!(edited, 1, "one main terminal changed");
    assert_eq!(modes, 1, "one session mode changed");
    let mut prepared = crate::agent::smith::recorded_program().prepared;
    prepared.face = crate::agent::smith::events::Shape::Chat;
    prepared.registry.shape = prepared.face;
    prepared.rendered = serde_json::json!({});
    let observing = crate::agent::smith::Observing::new(prepared).expect("fresh observer");
    (Capture::Agent(Box::new(crate::agent::Observing::Smith(observing))), events::bytes(&records))
}

#[test]
fn failed_main_closes_input_and_retains_its_budget_end() {
    let (capture, records) = failed_capture();
    let mut output = terminal_output(vec![start()], Close::AfterAnswer, capture);
    acknowledge(&mut output, b"prompt\n");
    output.trace_done(&records, Ok(Done::Count(u32::try_from(records.len()).expect("trace fixture"))), ANSWER_AT);
    acknowledge(&mut output, b"\x04");
    assert_eq!(output.answered, Some(ANSWER_AT));
    let Capture::Agent(observing) = output.capture else { panic!("paired observer") };
    assert_eq!(
        observing.classify(crate::Exit::Code(0), crate::Forced::No).end,
        crate::End::Budget { which: crate::BudgetLimit::Time }
    );
}

#[test]
fn capture_fault_starts_termination_before_the_hard_deadline() {
    let mut output = raw_output();
    output.stderr_overflow(true);
    let mut policy = policy();
    assert_eq!(policy.advance(Time::ZERO, None, false, output.error.is_some()), Some(Signal::Terminate));
    assert_eq!(policy.deadline, Some(Time::from_nanos(20_000_000)));
    assert_eq!(policy.advance(Time::from_nanos(19_999_999), None, false, true), None);
    assert_eq!(policy.advance(Time::from_nanos(20_000_000), None, false, true), Some(Signal::Kill));
}
