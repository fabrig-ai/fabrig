use std::collections::BTreeSet;

use smith_domain_session::llm::{Block, Decoded, Message, NotRun, Problem, Returned, Role, ToolCall, Usage};
use smith_domain_session::record::{Kind, Turn, VERSION};
use smith_domain_tools::{
    BytesClamp, Exit, Fault, JobEnd, JobNumber, Mismatch, Outcome, Resolved, SizePart, TimeoutClamp,
};
use smith_session_world::bytes;
use smith_session_world::shapes::{Answer, Call, Class, Reached, Reference, Sample, Tool, VOCABULARY, answer};

const JOB: JobNumber = JobNumber(1);

fn job(end: JobEnd) -> Outcome {
    Outcome::Job {
        job: JOB,
        head: Box::default(),
        tail: Box::default(),
        dropped: 0,
        output: BytesClamp::Within,
        wait: TimeoutClamp::Within,
        end,
    }
}

fn vocabulary_words<'a>(line: Option<&'a str>, field: &str) -> Vec<&'a str> {
    let (name, text) = line.and_then(|line| line.split_once(' ')).unwrap_or_else(|| panic!("missing {field} line"));
    assert_eq!(name, field);
    let words: Vec<_> = text.split_ascii_whitespace().collect();
    assert_eq!(words.join(" "), text, "spacing in {field}");
    let unique: BTreeSet<_> = words.iter().copied().collect();
    assert_eq!(unique.len(), words.len(), "duplicate word in {field}");
    words
}

#[test]
fn vocabulary_file_matches_the_kits_version_and_words() {
    let file = include_str!("../../../../testing/smith-shapes/vocabulary.txt");
    let mut lines = file.split_terminator('\n');
    assert_eq!(lines.next(), Some(VOCABULARY));
    let fields = [
        ("tool", Tool::ALL.iter().copied().map(Tool::word).collect::<Vec<_>>()),
        ("answer", Answer::ALL.iter().copied().map(Answer::word).collect()),
        ("reference", Reference::ALL.iter().copied().map(Reference::word).collect()),
        ("class", Class::ALL.iter().copied().map(Class::word).collect()),
    ];
    for (field, words) in fields {
        assert_eq!(vocabulary_words(lines.next(), field), words, "words in {field}");
    }
    for field in ["end", "begin", "wake"] {
        assert!(vocabulary_words(lines.next(), field).contains(&"unknown"), "missing unknown in {field}");
    }
    assert_eq!(lines.next(), None, "extra line after wake");
    assert!(file.ends_with('\n'), "missing final newline");
}

#[test]
fn returned_answers_use_the_vocabulary_words() {
    let own = |outcome| Some(Returned::Owned { outcome });
    let invalid = |problem| Some(Returned::Invalid { problem });
    let not_run = |why| Some(Returned::NotRun { why });
    let working = || Resolved::Working { path: bytes(b"notes") };
    let rows = [
        (own(Outcome::Removed), "ok"),
        (own(Outcome::Found { hits: Box::default(), more: 0, timed_out: true }), "search_timed_out"),
        (own(job(JobEnd::Running)), "running"),
        (own(job(JobEnd::Ended(Exit::Code { code: 0 }))), "exited"),
        (own(job(JobEnd::Ended(Exit::Code { code: 1 }))), "exit_nonzero"),
        (own(job(JobEnd::Ended(Exit::Signal { signal: 15 }))), "signalled"),
        (own(job(JobEnd::Ended(Exit::TimedOut))), "command_timed_out"),
        (own(job(JobEnd::Stopped)), "stopped"),
        (own(job(JobEnd::Cancelled)), "cancelled"),
        (
            own(Outcome::Interrupted { ran: None, head: Box::default(), last: Box::default(), dropped: 0 }),
            "interrupted",
        ),
        (own(Outcome::NotGranted), "not_granted"),
        (own(Outcome::TooLong), "too_long"),
        (own(Outcome::Outside { path: Resolved::Outside { path: bytes(b"notes") } }), "outside"),
        (own(Outcome::ReadOnly { path: working() }), "read_only"),
        (own(Outcome::Protected), "protected"),
        (own(Outcome::NotFound { path: working() }), "not_found"),
        (own(Outcome::NotFile), "not_file"),
        (own(Outcome::NotDirectory { path: working() }), "not_directory"),
        (own(Outcome::Linked), "linked"),
        (own(Outcome::NotEmpty), "not_empty"),
        (own(Outcome::NulByte), "nul_byte"),
        (own(Outcome::TooLarge { what: SizePart::File, size: 8, bound: 7 }), "too_large"),
        (own(Outcome::NotRead), "not_read"),
        (own(Outcome::Stale), "stale"),
        (own(Outcome::EditFailed { edit: 1, why: Mismatch::NoMatch }), "edit_no_match"),
        (
            own(Outcome::EditFailed { edit: 1, why: Mismatch::Ambiguous { count: 2, lines: Box::new([1, 2]) } }),
            "edit_ambiguous",
        ),
        (own(Outcome::EditFailed { edit: 1, why: Mismatch::Unchanged }), "edit_unchanged"),
        (own(Outcome::Failed { fault: Fault::Denied }), "failed"),
        (own(Outcome::TimedOut), "timed_out"),
        (own(Outcome::Busy), "busy"),
        (invalid(Problem::UnknownTool), "unknown_tool"),
        (invalid(Problem::NotAnObject), "malformed"),
        (invalid(Problem::TooLarge), "call_too_large"),
        (invalid(Problem::CutOff { bytes: 2 }), "cut_off"),
        (invalid(Problem::UnknownJob { job: JOB.0 }), "unknown_job"),
        (invalid(Problem::JobsFull { jobs: 4 }), "jobs_full"),
        (not_run(NotRun::OutsideChoice { choice: None }), "not_run_outside_choice"),
        (not_run(NotRun::AfterFailedWrite { call: 1 }), "not_run_after_failure"),
        (not_run(NotRun::Stopped), "not_run_stopped"),
        (Some(Returned::Withdrawn), "withdrawn"),
        (Some(Returned::Text { text: bytes(b"served"), error: false, replay: None }), "delegated"),
        (None, "unanswered"),
    ];
    for (returned, word) in rows {
        assert_eq!(answer(returned.as_ref()).word(), word, "{returned:?}");
    }
}

#[test]
fn completion_shapes_preserve_call_order_and_attach_references() {
    let mut reached = Reached::new();
    reached.completion(&[
        Call { tool: Tool::Job, answer: Answer::UnknownJob, references: vec![(Reference::Job, Class::Consumed)] },
        Call { tool: Tool::Write, answer: Answer::NotRunAfterFailure, references: Vec::new() },
        Call { tool: Tool::Read, answer: Answer::Ok, references: Vec::new() },
    ]);
    assert!(reached.contains(
        "job:unknown_job:job=consumed,write:not_run_after_failure,read:ok end=unknown begin=unknown wake=unknown"
    ));
}

#[test]
fn a_message_to_an_unissued_task_has_a_reference_refusal_shape() {
    assert_eq!(VOCABULARY, "smith/3");
    let mut reached = Reached::new();
    reached.completion(&[Call {
        tool: Tool::Message,
        answer: Answer::NoReference,
        references: vec![(Reference::Task, Class::Unissued)],
    }]);
    assert!(reached.contains("message:no_reference:task=unissued end=unknown begin=unknown wake=unknown"));
}

#[test]
fn a_completion_without_calls_has_a_text_shape() {
    let mut reached = Reached::new();
    reached.completion(&[]);
    assert!(reached.contains("text end=unknown begin=unknown wake=unknown"));
}

#[test]
fn a_sample_counts_answers_and_classes_by_case() {
    let mut sample = Sample::new("sample");
    let mut repeated = Reached::new();
    let call =
        [Call { tool: Tool::Job, answer: Answer::UnknownJob, references: vec![(Reference::Job, Class::Unissued)] }];
    repeated.completion(&call);
    repeated.completion(&call);
    sample.ran(&repeated).expect("valid shapes");
    sample.ran(&Reached::new()).expect("an empty case");
    assert_eq!(
        (sample.cases(), sample.answered(Answer::UnknownJob), sample.classed(Reference::Job, Class::Unissued)),
        (2, 1, 1)
    );
}

fn turn(sequence: u32, messages: Vec<Message>) -> Turn {
    Turn {
        version: VERSION,
        kind: Kind::Ordinary,
        endpoint: None,
        model: bytes(b"fixture"),
        effort: None,
        dialect: 0,
        sequence,
        usage: Usage::NONE,
        spent: 0,
        jobs: 0,
        messages: messages.into_boxed_slice(),
    }
}

fn recorded_call(id: &[u8], name: &[u8], input: &[u8]) -> Block {
    Block::ToolCall(ToolCall {
        id: bytes(id),
        name: bytes(name),
        input: bytes(input),
        call: Decoded::Historical,
        replay: None,
    })
}

#[test]
fn recorded_turns_join_results_by_id_and_keep_a_withheld_wait_unanswered() {
    let turns = [
        turn(
            1,
            vec![
                Message {
                    role: Role::Assistant,
                    content: Box::new([
                        recorded_call(b"r", b"read", br#"{"path":"README.md"}"#),
                        recorded_call(b"j", b"job", br#"{"job":1}"#),
                        recorded_call(b"w", b"wait", b"{}"),
                    ]),
                },
                Message {
                    role: Role::User,
                    content: Box::new([Block::ToolResult {
                        id: bytes(b"j"),
                        result: Returned::Invalid { problem: Problem::UnknownJob { job: JOB.0 } },
                    }]),
                },
            ],
        ),
        turn(
            2,
            vec![Message {
                role: Role::User,
                content: Box::new([Block::ToolResult {
                    id: bytes(b"r"),
                    result: Returned::Owned {
                        outcome: Outcome::Read {
                            content: bytes(b"hello\n"),
                            first: 1,
                            lines: 1,
                            total: 1,
                            next: smith_domain_tools::Next::End,
                            cut: smith_domain_tools::Cut::Whole,
                            window: BytesClamp::Within,
                        },
                    },
                }]),
            }],
        ),
    ];
    let mut reached = Reached::new();
    reached.turns(&turns);
    assert!(reached.contains(
        "read:ok:path=unknown,job:unknown_job:job=unknown,delegated:unanswered end=unknown begin=unknown wake=unknown"
    ));
    assert!(!reached.contains("text end=unknown begin=unknown wake=unknown"));
}
