//! Completion-fed capture and input steps; policy time is supplied, with no kernel or child.

use super::*;
use crate::agent::{Close, Cue, Person, Script, Seen, Trigger};

use crate::fixture::events;

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

#[test]
fn containment_requests_the_walk_only_on_linux() {
    for (linux, force_walk, expected) in [
        (true, true, Containment::Walk),
        (true, false, Containment::Best),
        (false, true, Containment::Best),
        (false, false, Containment::Best),
    ] {
        assert_eq!(containment(force_walk, linux), expected, "linux={linux}, force_walk={force_walk}");
    }
}

fn policy() -> Policy {
    Policy {
        finding: None,
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
    assert_eq!(writes(&mut output).len(), 0);
    output.person.see(&Seen::ToolCall { tool: "finish".into() });
    acknowledge(&mut output, b"done\n");
    output.person.see(&Seen::ToolCall { tool: "finish".into() });
    acknowledge(&mut output, b"\x04");
    assert_eq!(writes(&mut output).len(), 0);
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
    assert_eq!(writes(&mut output).len(), 0);
    output.person.see(&Seen::ToolCall { tool: "read".into() });
    acknowledge(&mut output, b"middle\n");
    output.person.see(&Seen::ToolCall { tool: "read".into() });
    assert_eq!(writes(&mut output).len(), 0);
    output.person.see(&Seen::ToolCall { tool: "finish".into() });
    acknowledge(&mut output, b"last\n");
    acknowledge(&mut output, b"\x04");
    assert_eq!(writes(&mut output).len(), 0);
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

// Supply completions only for operations Output submitted. The fixture owns
// time and the keeper evidence, not the policy or the receipt's observations.
struct Steps {
    output: Output,
    policy: Policy,
    pending: Vec<Submit>,
    stream: Fd,
}

const TRACE: Fd = Fd::new(12);
const EXIT_AT: Time = Time::from_nanos(159_500_000);
const RUNNER_STDOUT: usize = 16_384;

impl Steps {
    fn new(capture: Capture, prompt: &[u8], limit: usize, trace: Option<Fd>) -> Self {
        let stream = if trace.is_some() { TERMINAL } else { PIPE };
        let script = prompt_script(prompt.into(), &capture).expect("prompt script");
        let person = Person::new(script).expect("scripted person");
        let mut output = Output::new(TERMINAL, stream, person, limit, capture);
        output.trace = trace;
        let mut steps = Self { output, policy: policy(), pending: Vec::new(), stream };
        steps.iterate(Time::ZERO);
        steps
    }

    fn iterate(&mut self, now: Time) {
        self.output.iterate(now, Wall::EPOCH);
        self.pending.extend(std::iter::from_fn(|| self.output.submissions.pop()));
    }

    fn take(&mut self, matching: impl Fn(&Op) -> bool) -> Submit {
        let index = self.pending.iter().position(|submit| matching(&submit.kind)).expect("submitted operation");
        self.pending.remove(index)
    }

    fn complete(&mut self, submit: Submit, done: Done, now: Time) {
        self.output.completions.push(Complete { op: submit.op, kind: submit.kind, result: Ok(done) });
        self.iterate(now);
    }

    fn write(&mut self, bytes: &[u8], now: Time) {
        let submit = self.take(|op| matches!(op, Op::PipeWrite { fd, .. } if *fd == TERMINAL));
        let Op::PipeWrite { bytes: written, from, .. } = &submit.kind else { unreachable!("input write") };
        assert_eq!(&written[usize::try_from(*from).expect("offset")..], bytes);
        self.complete(submit, Done::Count(u32::try_from(bytes.len()).expect("small write")), now);
    }

    fn close(&mut self, descriptor: Fd, now: Time) {
        let submit = self.take(|op| matches!(op, Op::Close { fd } if *fd == descriptor));
        self.complete(submit, Done::Nothing, now);
    }

    fn read(&mut self, trace: bool, bytes: &[u8], now: Time) {
        let stream = self.stream;
        let mut submit = self.take(|op| match op {
            Op::Read { fd, .. } => trace && *fd == TRACE,
            Op::PipeRead { fd, .. } => !trace && *fd == stream,
            _ => false,
        });
        let (Op::Read { buf, .. } | Op::PipeRead { buf, .. }) = &mut submit.kind else {
            unreachable!("submitted read")
        };
        for (slot, byte) in buf[..bytes.len()].iter_mut().zip(bytes) {
            *slot = *byte;
        }
        self.complete(submit, Done::Count(u32::try_from(bytes.len()).expect("chunk fits")), now);
    }

    fn observe(&mut self, now: Time, settled: bool) -> Option<Signal> {
        self.policy.observe(now, &mut self.output, settled, false)
    }

    fn finish(mut self, now: Time, settled: Settlement) -> ProcessResult {
        self.read(false, b"", now);
        assert_eq!(self.observe(now, true), None);
        assert!(self.policy.passed());
        self.iterate(now);
        if self.output.trace.is_some() {
            self.read(true, b"", now);
            self.close(TRACE, now);
        }
        self.close(self.stream, now);
        assert_eq!(self.pending.len(), 0, "uncompleted submissions: {:?}", self.pending);
        assert!(self.output.is_empty());
        receipt(self.output, Now { now: Time::ZERO, wall: Wall::EPOCH }, now, settled)
    }
}

fn settlement(exit: skein_io::kernel::Exit, cleanup: Cleanup) -> Settlement {
    let term = cleanup != Cleanup::None;
    let killed = cleanup == Cleanup::Kill;
    let pids = |sent| if sent { vec![42] } else { Vec::new() };
    Settlement {
        exit,
        stderr: Vec::new(),
        stderr_overflowed: false,
        counts: Counts {
            method: skein_world::end_to_end::Method::Group,
            user: Duration::from_millis(1),
            system: Duration::ZERO,
            peak_rss_bytes: 42,
            peak_scope: skein_world::end_to_end::PeakScope::LargestProcess { baseline_bytes: 0 },
        },
        tree: TreeStatus {
            live_pids: Vec::new(),
            live_at_leader_exit: Some(Vec::new()),
            interrupted_pids: Vec::new(),
            stopped_pids: Vec::new(),
            continued_pids: Vec::new(),
            terminated_pids: pids(term),
            killed_pids: pids(killed),
            forced_cleanup: killed,
        },
    }
}

fn clean_exit() -> Settlement {
    settlement(skein_io::kernel::Exit::Code(0), Cleanup::None)
}

#[test]
fn a_prepared_adapter_observes_the_answer_through_the_common_supervisor() {
    use crate::agent::{Observing, PreparedAgent, codex::PreparedCodex};
    let prepared = PreparedAgent::Codex(PreparedCodex {
        overrides: Vec::new(),
        credential: "/fixture/codex/auth.json".into(),
        expires: Wall::EPOCH,
        borrowed: None,
        home: "/fixture/codex".into(),
        final_message: "/fixture/final.txt".into(),
        model: crate::ModelChoice { model: Some("gpt-6-luna".into()), effort: Some("low".into()) },
        program: "/fixture/codex/bin/codex".into(),
        workspace: "/fixture/workspace".into(),
        teardown: TEARDOWN,
    });
    let capture = Capture::Agent(Box::new(prepared.observe().expect("prepared observer")));
    let prompt = b"offline replay\n";
    let mut steps = Steps::new(capture, prompt, TRACE_BYTES, None);
    steps.write(prompt, Time::ZERO);
    steps.close(TERMINAL, Time::ZERO);
    let recording = include_bytes!("../../tests/recorded/codex/completed.jsonl");
    steps.read(false, recording, ANSWER_AT);
    assert_eq!(steps.observe(ANSWER_AT, false), None);
    assert_eq!(steps.policy.deadline, Some(TEARDOWN_AT));
    let settled = clean_exit();
    let counts = settled.counts;
    let result = steps.finish(EXIT_AT, settled);
    assert_eq!(result.error, None);
    let Some(Observing::Codex(observing)) = result.observing.as_deref() else { panic!("prepared observer retained") };
    assert_eq!(
        observing.prepared.as_ref().expect("paired preparation").final_message,
        std::path::Path::new("/fixture/final.txt")
    );
    assert_eq!(observing.observer.classify(result.exit, result.forced).end, crate::End::Completed);
    assert_eq!(result.answered, Some(ANSWER_AT));
    assert!(result.answered.is_some_and(|answer| answer >= result.start && answer <= result.end));
    assert_eq!(result.counts, counts);
    assert_eq!(result.tree.live_pids, [0u32; 0]);
}

#[test]
fn a_program_without_an_answer_ends_at_the_hard_deadline() {
    let hard_at = Time::from_nanos(180_000_000);
    let kill_at = hard_at.checked_add(GRACE).expect("fixture grace");
    let before_kill = Time::from_nanos(199_999_999);
    for (cleanup, signal, forced) in
        [(Cleanup::Terminate, 15, crate::Forced::Terminated), (Cleanup::Kill, 9, crate::Forced::Killed)]
    {
        let capture = Capture::Agent(Box::new(crate::agent::Observing::replay()));
        let prompt = b"answer\n";
        let mut steps = Steps::new(capture, prompt, CAPTURE_BYTES, None);
        steps.policy.deadline = Some(hard_at);
        steps.write(prompt, Time::ZERO);
        steps.close(TERMINAL, Time::ZERO);
        assert_eq!(steps.observe(Time::from_nanos(179_999_999), false), None);
        assert_eq!(steps.observe(hard_at, false), Some(Signal::Terminate));
        assert_eq!(steps.observe(before_kill, false), None);
        let end = if cleanup == Cleanup::Kill {
            assert_eq!(steps.observe(kill_at, false), Some(Signal::Kill));
            kill_at
        } else {
            before_kill
        };
        let result = steps.finish(end, settlement(skein_io::kernel::Exit::Signal(signal), cleanup));
        let observing = result.observing.expect("observer retained");
        assert_eq!(observing.classify(result.exit, result.forced).end, crate::End::Timeout);
        assert_eq!(result.forced, forced);
        assert_eq!(result.exit, crate::Exit::Signal(i32::try_from(signal).expect("small signal")));
        assert_eq!(result.answered, None);
        assert_eq!(result.tree.live_pids, [0u32; 0]);
    }
}

#[test]
fn an_overflowing_stdout_retains_its_prefix_and_settles_the_tree() {
    let maximum = RUNNER_STDOUT;
    let mut steps = Steps::new(Capture::Raw, b"", maximum, None);
    steps.close(TERMINAL, Time::ZERO);
    let bytes: Vec<_> = b"output\n".iter().copied().cycle().take(maximum + 1).collect();
    for chunk in bytes.chunks(4096) {
        steps.read(false, chunk, Time::ZERO);
    }
    assert_eq!(steps.observe(Time::ZERO, false), Some(Signal::Terminate));
    assert_eq!(steps.observe(Time::from_nanos(19_999_999), false), None);
    let end = Time::from_nanos(20_000_000);
    assert_eq!(steps.observe(end, false), Some(Signal::Kill));
    let result = steps.finish(end, settlement(skein_io::kernel::Exit::Signal(9), Cleanup::Kill));
    assert_eq!(result.error.as_deref(), Some("stdout exceeds the attempt log bound"));
    assert_eq!(result.stdout, bytes[..maximum]);
    assert_eq!(result.forced, crate::Forced::Killed);
    assert_eq!(result.answered, None);
    assert_eq!(result.tree.live_pids, [0u32; 0]);
    assert!(result.end < HARD_AT, "overflow, not the hard deadline, began teardown");
}

#[test]
fn stderr_at_its_bound_is_complete_and_a_settled_overflow_still_fails() {
    for (bytes, error) in
        [(b"12345678".as_slice(), None), (b"123456789".as_slice(), Some("stderr exceeds the attempt log bound"))]
    {
        let mut capture = skein_world::end_to_end::StderrCapture::new(CAPTURE_BYTES);
        capture.push(bytes, Expectation::Supervise);
        // The keeper only reports this capture at settlement, too late to signal the tree.
        let settled =
            Settlement { stderr: capture.bytes().to_vec(), stderr_overflowed: capture.overflowed(), ..clean_exit() };
        let mut steps = Steps::new(Capture::Raw, b"", CAPTURE_BYTES, None);
        steps.close(TERMINAL, Time::ZERO);
        let result = steps.finish(EXIT_AT, settled);
        assert_eq!(result.error.as_deref(), error);
        assert_eq!(result.stderr, b"12345678");
        assert_eq!(result.tree.live_pids, [0u32; 0]);
    }
}

fn chat_capture(encoded_line: usize) -> Capture {
    use crate::agent::{
        PreparedAgent,
        smith::{FaceSince, Prepared, Receiving, events::Shape},
    };
    let prepared = PreparedAgent::Smith(Prepared {
        face: Shape::Chat,
        settings: "/fixture/settings.json".into(),
        trace: Some("/fixture/trace.jsonl".into()),
        registry: FaceSince {
            commit: "30259d89b88834ed3edaaa08b3f320d2252bbb9a".into(),
            shape: Shape::Chat,
            receiving: Receiving {
                string: 524_288,
                content: 4096,
                items: 32,
                encoded_line: u32::try_from(encoded_line).expect("fixture ceiling"),
            },
        },
        rendered: serde_json::json!({}),
        program: "/fixture/smith".into(),
        state: "/fixture/state".into(),
        serial: None,
        teardown: TEARDOWN,
    });
    Capture::Agent(Box::new(prepared.observe().expect("checked writer bounds")))
}

#[test]
fn the_selected_event_face_observes_the_answer_and_drains_its_native_boundaries() {
    use crate::{End, Forced, Measure, SpendBasis, agent::Observing};
    let records = events::bytes(&events::balanced());
    let ceiling = records.split_inclusive(|byte| *byte == b'\n').map(<[u8]>::len).max().expect("record lines");
    let mut steps = Steps::new(chat_capture(ceiling), b"task\n", 16384, Some(TRACE));
    steps.write(b"task\n", Time::ZERO);
    assert!(!steps.pending.iter().any(|submit| matches!(submit.kind, Op::PipeWrite { .. } | Op::Close { .. })));
    for chunk in records.chunks(4096) {
        steps.read(true, chunk, ANSWER_AT);
    }
    assert_eq!(steps.observe(ANSWER_AT, false), None);
    steps.write(b"\x04", ANSWER_AT);
    let result = steps.finish(EXIT_AT, clean_exit());
    assert_eq!(result.forced, Forced::No);
    assert_eq!(result.trace, records);
    assert_eq!(result.answered, Some(ANSWER_AT));
    assert_eq!(result.error, None);
    assert_eq!(result.tree.live_pids, [0u32; 0]);
    let Some(Observing::Smith(native)) = result.observing.as_deref() else { panic!("selected native face retained") };
    assert_eq!(native.observer.classify(result.exit, result.forced).end, End::Completed);
    assert_eq!(native.counts().responses, Measure::Observed { value: 1 });
    let health = native.health(result.exit, result.forced).expect("typed passive evidence");
    for check in &health.passive_checks {
        if check.check == "teardown"
            || (check.check.starts_with("headroom/")
                && !matches!(check.check.as_str(), "headroom/prompt-tokens" | "headroom/spend"))
        {
            assert!(matches!(check.passed, Measure::Unavailable { .. }), "{}", check.check);
        } else {
            assert_eq!(check.passed, Measure::Observed { value: true }, "{}", check.check);
        }
    }
    assert_eq!(health.headroom.len(), 7);
    let spend = native.spend(&native.tokens().expect("exact response usage")).expect("reported notional spend");
    assert_eq!(spend.basis, SpendBasis::Notional);
    assert_eq!(spend.usd, Measure::Observed { value: 0.000_003 });
}

#[test]
fn an_incomplete_file_record_faults_after_observed_tree_settlement_and_retains_its_prefix() {
    let partial = b"{\"type\":";
    let mut steps = Steps::new(chat_capture(16384), b"", 16384, Some(TRACE));
    steps.read(true, partial, Time::ZERO);
    steps.read(true, b"", Time::ZERO);
    assert_eq!(steps.output.tree, Tree::Running);
    assert_eq!(steps.output.error, None, "the writer may still finish the line");
    assert_eq!(steps.output.next_mechanism_deadline(), Some(Time::from_nanos(10_000_000)));
    let result = steps.finish(Time::from_nanos(1_000_000), clean_exit());
    assert!(result.error.as_ref().is_some_and(|error| error.contains("partial record")));
    assert_eq!(result.trace, partial);
    assert_eq!(result.tree.live_pids, [0u32; 0]);
}

pub(crate) fn check_receipt(bounds: Bounds, code: u8, stdout: &[u8], stderr: &[u8]) -> ProcessResult {
    let mut steps = Steps::new(Capture::Raw, b"", bounds.stdout, None);
    steps.close(TERMINAL, Time::ZERO);
    for chunk in stdout.chunks(4096) {
        steps.read(false, chunk, ANSWER_AT);
    }
    let mut capture = skein_world::end_to_end::StderrCapture::new(bounds.stderr);
    capture.push(stderr, Expectation::Supervise);
    let settled = Settlement {
        stderr: capture.bytes().to_vec(),
        stderr_overflowed: capture.overflowed(),
        ..settlement(skein_io::kernel::Exit::Code(code), Cleanup::None)
    };
    steps.finish(EXIT_AT, settled)
}

pub(crate) fn exec_prompt_receipt(capture: Capture, prompt: &[u8]) -> ProcessResult {
    assert!(trace_file(&capture).expect("trace selection").is_none());
    let mut steps = Steps::new(capture, prompt, TRACE_BYTES, None);
    assert!(!steps.pending.iter().any(|submit| matches!(submit.kind, Op::Close { .. })));
    steps.write(prompt, Time::ZERO);
    steps.close(TERMINAL, Time::ZERO);
    steps.finish(EXIT_AT, clean_exit())
}
