use smith_domain_session::llm::{NotRun, Problem, Returned};
use smith_domain_tools::{
    BytesClamp, Exit, Fault, JobEnd, JobNumber, Mismatch, Outcome, Resolved, SizePart, TimeoutClamp,
};
use smith_session_world::bytes;
use smith_session_world::shapes::{Answer, Call, Class, Reached, Reference, Sample, Tool, answer};

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
