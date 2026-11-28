//! Compare the client's wire classification with the shapes world's domain classification.

use super::{
    CEILINGS, List, call, call_at, decode, encode, exited, job, limits, message, owned, result, smith, source,
};
use smith_domain_session::llm;
use smith_protocol_translate::Endpoints;

const ID: &[u8] = b"id";
const PLACE: u32 = 2; // An ordinary turn following the opening.
#[expect(clippy::disallowed_macros, reason = "the test compares words against the published vocabulary in the tree")]
const VOCABULARY: &str = include_str!("../../../../../smith/testing/smith-shapes/vocabulary.txt");

#[test]
#[expect(clippy::disallowed_methods, reason = "the test selects the vocabulary line and joins its case tables")]
fn stored_answers_agree_with_the_shapes_world_and_vocabulary() {
    let answer_words = VOCABULARY.lines().find_map(|line| line.strip_prefix("answer ")).expect("answer vocabulary");
    let endpoints = Endpoints::new(List::with_capacity(0));
    let cases = file_outcomes().into_iter().chain(command_outcomes()).map(owned);
    for returned in cases.chain(invalid_answers()).chain(session_answers()) {
        check_answer(Some(returned), &endpoints, answer_words);
    }
    check_answer(None, &endpoints, answer_words);
}

#[expect(clippy::disallowed_methods, reason = "the test follows the optional result and searches the vocabulary")]
fn check_answer(returned: Option<smith::Returned>, endpoints: &Endpoints, answer_words: &str) {
    let assistant = message(smith::Role::Assistant, &[call(ID)]);
    let mut turn = match &returned {
        Some(returned) => source(&[assistant, message(smith::Role::User, &[result(ID, returned.clone())])]),
        None => source(&[assistant]),
    };
    turn.place = PLACE;
    let bytes = encode(&turn);
    let domain = smith_protocol_translate::decode_turn(&bytes, PLACE, &CEILINGS, endpoints)
        .unwrap_or_else(|why| panic!("smith did not read {returned:?}: {why:?}"));
    let domain_result = domain.messages.get(1).map(|message| {
        let Some(llm::Block::ToolResult { id, result }) = message.content.first() else { panic!("stored result") };
        assert_eq!(id.as_ref(), ID);
        result
    });
    let expected = domain_result.map(|result| smith_shapes::answer(Some(result)).word());
    let decoded = decode(&limits(), &bytes).expect("client reads the same turn");
    let decoded_word = call_at(&decoded, 0, 0).answer;
    assert_eq!(decoded_word, expected.map(str::as_bytes), "{returned:?}");
    if let Some(word) = decoded_word {
        assert!(answer_words.split_ascii_whitespace().any(|allowed| allowed.as_bytes() == word), "{returned:?}");
    }
}

fn file_outcomes() -> [smith::Outcome; 28] {
    use smith::Outcome;
    [
        Outcome::Read(smith::Read {
            content: Box::new([]),
            first: 1,
            lines: 0,
            total: 0,
            next: smith::Next::End,
            cut: smith::ReadCut::Whole,
            window: smith::BytesClamp::Within,
        }),
        Outcome::Listed(smith::Listed { entries: List::with_capacity(0), more: 0 }),
        Outcome::Found(smith::Found { hits: Box::new([]), more: 0, timed_out: false }),
        Outcome::Found(smith::Found { hits: Box::new([]), more: 0, timed_out: true }),
        Outcome::Written(smith::Written { created: true }),
        Outcome::Edited(smith::Edited { replaced: 1 }),
        Outcome::Removed,
        Outcome::NotGranted,
        Outcome::TooLong,
        Outcome::Outside(super::resolved()),
        Outcome::ReadOnly(super::resolved()),
        Outcome::Protected,
        Outcome::NotFound(super::resolved()),
        Outcome::NotFile,
        Outcome::NotDirectory(super::resolved()),
        Outcome::Linked,
        Outcome::NotEmpty,
        Outcome::NulByte,
        Outcome::TooLarge(smith::TooLarge { what: smith::SizePart::File, size: 2, bound: 1 }),
        Outcome::NotRead,
        Outcome::Stale,
        super::edit_failed(smith::Mismatch::NoMatch),
        super::edit_failed(smith::Mismatch::Ambiguous(smith::Ambiguous { count: 2, lines: List::with_capacity(0) })),
        super::edit_failed(smith::Mismatch::Unchanged),
        super::failed(smith::Fault::Other),
        Outcome::TimedOut,
        Outcome::Busy,
        Outcome::Cancelled,
    ]
}

fn command_outcomes() -> [smith::Outcome; 13] {
    use smith::{Exit, JobEnd};
    let zero = Exit::Code(smith::ExitCode { code: 0 });
    let nonzero = Exit::Code(smith::ExitCode { code: 3 });
    let signal = Exit::Signal(smith::ExitSignal { signal: 9 });
    [
        smith::Outcome::Running(smith::CommandRunning {
            job: 1,
            head: Box::new([]),
            last: Box::new([]),
            dropped: 0,
            timeout: smith::TimeoutClamp::Within,
            output: smith::BytesClamp::Within,
            after: smith::TimeoutClamp::Within,
        }),
        smith::Outcome::Interrupted(smith::Interrupted {
            ran_ms: None,
            head: Box::new([]),
            last: Box::new([]),
            dropped: 0,
        }),
        exited(zero.clone()),
        exited(nonzero.clone()),
        exited(signal.clone()),
        exited(Exit::TimedOut),
        job(JobEnd::Running),
        job(JobEnd::Stopped),
        job(JobEnd::Cancelled),
        job(JobEnd::Ended(smith::JobExit { exit: zero })),
        job(JobEnd::Ended(smith::JobExit { exit: nonzero })),
        job(JobEnd::Ended(smith::JobExit { exit: signal })),
        job(JobEnd::Ended(smith::JobExit { exit: Exit::TimedOut })),
    ]
}

fn invalid_answers() -> [smith::Returned; 11] {
    use smith::CallProblem;
    let field = smith::FieldProblem { field: Box::from(b"path".as_slice()) };
    [
        CallProblem::UnknownTool,
        CallProblem::NotAnObject,
        CallProblem::Missing(field.clone()),
        CallProblem::WrongType(field.clone()),
        CallProblem::BadValue(field),
        CallProblem::TooLarge,
        CallProblem::Oversize(smith::OversizeProblem { bytes: 9, bound: 8 }),
        CallProblem::CutOff(smith::CutProblem { bytes: 17 }),
        CallProblem::Nesting,
        CallProblem::JobsFull(smith::JobsFullProblem { jobs: 3 }),
        CallProblem::UnknownJob(smith::UnknownJobProblem { job: 7 }),
    ]
    .map(|problem| smith::Returned::Invalid(smith::Invalid { problem }))
}

fn session_answers() -> [smith::Returned; 6] {
    use smith::{NotRunWhy, Returned};
    let text = b"Tool refused: not granted".as_slice();
    [
        Returned::NotRun(smith::NotRun { why: NotRunWhy::Stopped }),
        Returned::NotRun(smith::NotRun { why: NotRunWhy::OutsideChoice(smith::OutsideChoice { only: None }) }),
        Returned::NotRun(smith::NotRun { why: NotRunWhy::AfterFailedWrite(smith::AfterFailedWrite { call: 1 }) }),
        Returned::Said(smith::Said { text: text.into(), error: false }),
        Returned::Said(smith::Said { text: text.into(), error: true }),
        Returned::Withdrawn,
    ]
}
