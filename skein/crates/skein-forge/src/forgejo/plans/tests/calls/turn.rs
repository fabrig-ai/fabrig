//! Turn calls through request spelling, server documents and streamed decoding.
use super::*;
use crate::api::Excess;
use crate::api::{MergeHow, UpdateStyle, Write};
use crate::forgejo::{client, document as d, plans::plan::Bounds, request::Request, server};
use alloc::boxed::Box;
use skein_lib::List;
// Two full run documents and their listing envelope fit within this fixture's cap.
const TURN_DOCUMENT_BYTES: u32 = 1024;
fn limits() -> crate::forgejo::limits::Limits {
    let mut limits = fixtures::limits();
    limits.writer.length = TURN_DOCUMENT_BYTES;
    limits.json.length = TURN_DOCUMENT_BYTES;
    limits
}
fn known(op: Op) -> Run {
    Run::with_limits(limits(), BOUNDS, op)
}
fn wire_run() -> d::Run {
    let limits = limits();
    let decoded = fixtures::decode(
        client::decoder(&Request::Run { at: fixtures::repository(), run: 7 }, &limits).expect("run decoder"),
        RUN_DOC,
        &limits,
    );
    let Ok(d::Document::Run(run)) = decoded.document else {
        panic!("run document");
    };
    run
}
const RUN_DOC: &[u8] = br#"{"id":7,"index_in_repo":5,"workflow_id":"ci.yaml","commit_sha":"0100000000000000000000000000000000000000","event":"workflow_dispatch","trigger_actor":{"id":2},"status":"failure","html_url":"https://forge/fixture/specimen/actions/runs/5","created_at":"1970-01-01T00:00:03Z"}"#;
const JOBS_DOC: &[u8] = br#"[{"id":9,"run_id":7,"attempt":1,"name":"passed","status":"success"},{"id":10,"run_id":7,"attempt":3,"name":"failed","status":"failure"}]"#;
const TREE_DOC: &[u8] = br#"{"tree":[{"path":"a","mode":"100644","type":"blob","sha":"0100000000000000000000000000000000000000"},{"path":"b","mode":"040000","type":"tree","sha":"0300000000000000000000000000000000000000"}],"truncated":false}"#;
fn run_value() -> api::Run {
    api::Run {
        id: 7,
        number: 5,
        workflow: Box::from(*b"ci.yaml"),
        commit: HEAD,
        event: api::Event::Dispatch,
        actor: 2,
        state: api::RunState::Completed,
        conclusion: Some(api::Conclusion::Failure),
        url: Box::from(*b"https://forge/fixture/specimen/actions/runs/5"),
        created: Time::from_nanos(3_000_000_000),
        jobs: Box::new([]),
    }
}
fn round_request(step: Step) -> Request {
    let Step::Send(exchange) = step else {
        panic!("request expected");
    };
    let limits = limits();
    let encoded = client::encode(&exchange.request, &limits).expect("request fits");
    let decoded = match server::route(encoded.method, &encoded.target, &limits).expect("server route") {
        server::Route::Ready(request) => request,
        server::Route::Reference(_) => panic!("plans encode resolved commit ids"),
        server::Route::Body(_) => {
            fixtures::request(encoded.method, &encoded.target, encoded.body.as_deref().expect("request body"), &limits)
                .expect("server body")
        }
    };
    assert_eq!(decoded, exchange.request);
    decoded
}
fn server_answer(answer: &server::Answer) -> Box<[u8]> {
    server::encode(answer, &limits()).expect("server answer fits")
}
fn dispatch() -> Write {
    Write::Dispatch {
        workflow: Box::from(*b"ci.yaml"),
        branch: Box::from(*b"work"),
        inputs: Box::new([
            api::Input { name: Box::from(*b"key"), value: Box::from(*b"receipt") },
            api::Input { name: Box::from(*b"gate"), value: Box::from(*b"focused") },
        ]),
    }
}
#[test]
fn tree_pages_are_full_or_truncated_and_objects_round_trip() {
    let expected = Box::new([
        api::TreeEntry { name: Box::from(*b"a"), mode: 0o100_644, kind: api::TreeEntryKind::Blob, object: HEAD },
        api::TreeEntry { name: Box::from(*b"b"), mode: 0o040_000, kind: api::TreeEntryKind::Tree, object: BASE },
    ]);
    let mut plan = known(Op::Read(Read::Tree { object: HEAD, page: 2 }));
    round_request(plan.begin());
    assert_eq!(plan.answer(200, TREE_DOC), Step::Done(Ok(Answer::Tree { entries: expected, more: true })));
    plan.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/git/trees/0100000000000000000000000000000000000000?page=2&per_page=2",
    ]);
    for (truncated, more) in [(false, false), (true, true)] {
        let mut plan = known(Op::Read(Read::Tree { object: BASE, page: 1 }));
        round_request(plan.begin());
        let answer = server::Answer::Tree {
            entries: Box::new([d::TreeEntry {
                path: Box::from(*b"submodule"),
                mode: 0o160_000,
                kind: api::TreeEntryKind::Commit,
                sha: crate::forgejo::id::ObjectId { bytes: HEAD, format: crate::forgejo::id::ObjectFormat::Sha1 },
            }]),
            truncated,
        };
        let bytes = server_answer(&answer);
        assert_eq!(
            plan.answer(200, &bytes),
            Step::Done(Ok(Answer::Tree {
                entries: Box::new([api::TreeEntry {
                    name: Box::from(*b"submodule"),
                    mode: 0o160_000,
                    kind: api::TreeEntryKind::Commit,
                    object: HEAD
                }]),
                more
            }))
        );
    }
}
#[test]
fn runs_keep_dispatch_and_other_events_newest_first() {
    let limits = limits();
    let decoded = fixtures::decode(
        client::decoder(&Request::Run { at: fixtures::repository(), run: 7 }, &limits).expect("run decoder"),
        RUN_DOC,
        &limits,
    );
    let Ok(d::Document::Run(newer)) = decoded.document else {
        panic!("run document");
    };
    let mut older = newer.clone();
    older.id = 6;
    older.event = Box::from(*b"push");
    older.created_at = skein_lib::Wall::from_nanos(2_000_000_000);
    let mut expected_older = run_value();
    expected_older.id = 6;
    expected_older.event = api::Event::Other;
    expected_older.created = Time::from_nanos(2_000_000_000);
    let bytes = server_answer(&server::Answer::Runs { total_count: 2, runs: Box::new([newer, older]) });
    let mut plan = known(Op::Read(Read::Runs { workflow: Box::from(*b"ci.yaml"), commit: HEAD, page: 1 }));
    round_request(plan.begin());
    assert_eq!(
        plan.answer(200, &bytes),
        Step::Done(Ok(Answer::Runs { runs: Box::new([run_value(), expected_older]), more: true }))
    );
    plan.assert_sent(&["GET /api/v1/repos/fixture/specimen/actions/runs?workflow_id=ci.yaml&head_sha=0100000000000000000000000000000000000000&page=1&limit=2"]);
    let mut empty = known(Op::Read(Read::Runs { workflow: Box::from(*b"ci.yaml"), commit: HEAD, page: 2 }));
    round_request(empty.begin());
    assert_eq!(
        empty.answer(200, br#"{"total_count":2,"workflow_runs":[]}"#),
        Step::Done(Ok(Answer::Runs { runs: Box::new([]), more: false }))
    );
}
#[test]
fn run_reads_jobs_and_the_first_failed_step() {
    let mut plan = known(Op::Read(Read::Run { run: 7 }));
    round_request(plan.begin());
    round_request(plan.answer(200, &server_answer(&server::Answer::Run(wire_run()))));
    round_request(plan.answer(200, JOBS_DOC));
    let mut expected = run_value();
    expected.jobs = Box::new([
        api::RunJob {
            steps: 0,
            id: 9,
            name: Box::from(*b"passed"),
            attempt: 1,
            state: api::RunState::Completed,
            conclusion: Some(api::Conclusion::Success),
            step: None,
        },
        api::RunJob {
            steps: 2,
            id: 10,
            name: Box::from(*b"failed"),
            attempt: 3,
            state: api::RunState::Completed,
            conclusion: Some(api::Conclusion::Failure),
            step: Some(api::Step { number: 1, name: Box::from(*b"x") }),
        },
    ]);
    let bytes = server_answer(&server::Answer::Job {
        job: d::Job {
            id: 10,
            run_id: 7,
            attempt: 3,
            name: Box::from(*b"failed"),
            status: d::StepStatus::Failure,
            labels: Box::new([]),
            cut: false,
        },
        steps: Box::new([
            d::Step { number: 1, name: Box::from(*b"x"), status: d::StepStatus::Failure },
            d::Step { number: 2, name: Box::from(*b"cleanup"), status: d::StepStatus::Failure },
        ]),
    });
    assert_eq!(plan.answer(200, &bytes), Step::Done(Ok(Answer::Run(expected))));
    plan.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/actions/runs/7",
        "GET /api/v1/repos/fixture/specimen/actions/runs/7/jobs",
        "GET /api/v1/repos/fixture/specimen/actions/jobs/10",
    ]);
}
#[test]
fn dispatch_and_rerun_have_typed_empty_answers_and_refusals() {
    for (write, status, expected) in [
        (dispatch(), 204, Ok(Answer::Done)),
        (dispatch(), 404, Err(Error::Missing)),
        (dispatch(), 400, Err(Error::Refused)),
        (dispatch(), 403, Err(Error::Forbidden)),
        (Write::Rerun { attempt: ATTEMPT }, 204, Ok(Answer::Done)),
        (Write::Rerun { attempt: ATTEMPT }, 400, Err(Error::Refused)),
        (Write::Rerun { attempt: ATTEMPT }, 403, Err(Error::Forbidden)),
    ] {
        let mut plan = known(Op::Write(write));
        round_request(plan.begin());
        assert_eq!(plan.answer(status, b""), Step::Done(expected));
    }
    let mut plan = known(Op::Write(dispatch()));
    round_request(plan.begin());
    plan.assert_bodies(&[Some(br#"{"ref":"work","inputs":{"key":"receipt","gate":"focused"}}"#)]);
}
#[test]
fn merge_fast_forward_and_rebase_update_spell_their_style() {
    let mut plan = known(Op::Write(Write::Merge { number: 1, head: HEAD, how: MergeHow::FastForwardOnly }));
    round_request(plan.begin());
    plan.assert_bodies(&[Some(
        br#"{"Do":"fast-forward-only","head_commit_id":"0100000000000000000000000000000000000000"}"#,
    )]);
    for (status, body, expected) in [
        (200, b"".as_slice(), Ok(Answer::Done)),
        (409, br#"{"message":"merge failed because of conflict"}"#.as_slice(), Err(Error::Conflict)),
        (409, b"".as_slice(), Err(Error::Conflict)),
    ] {
        let mut plan = known(Op::Write(Write::Update { number: 1, style: UpdateStyle::Rebase }));
        round_request(plan.begin());
        plan.assert_sent(&["POST /api/v1/repos/fixture/specimen/pulls/1/update?style=rebase"]);
        assert_eq!(plan.answer(status, body), Step::Done(expected));
    }
}
#[test]
fn recorded_merge_names_its_commit_and_reads_back_or_is_refused() {
    let mut commit = [0; 32];
    commit.fill(0xc1);
    // The repository uses SHA-1: only its twenty bytes cross the wire.
    commit.get_mut(20..).expect("SHA-1 padding").fill(0);
    let write = Write::Merge { number: 7, head: HEAD, how: MergeHow::Recorded { commit } };
    let mut plan = known(Op::Write(write.clone()));
    round_request(plan.begin());
    plan.assert_bodies(&[Some(
        br#"{"Do":"manually-merged","MergeCommitID":"c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1"}"#,
    )]);
    round_request(plan.answer(200, b""));
    let merged = br#"{"number":7,"state":"closed","head":{"repo_id":1,"ref":"work","sha":"0100000000000000000000000000000000000000"},"base":{"repo_id":1,"ref":"main","sha":"c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1"},"merged":true,"merge_commit_sha":"c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1","mergeable":false,"requested_reviewers":[]}"#;
    assert_eq!(plan.answer(200, merged), Step::Done(Ok(Answer::Merged(commit))));
    plan.assert_sent(&[
        "POST /api/v1/repos/fixture/specimen/pulls/7/merge",
        "GET /api/v1/repos/fixture/specimen/pulls/7",
    ]);
    for (status, message) in [
        (409, b"Wrong commit ID".as_slice()),
        (405, b"manually-merged is not an allowed merge style for this repository"),
    ] {
        let mut plan = known(Op::Write(write.clone()));
        round_request(plan.begin());
        let body = server_answer(&server::Answer::Error(d::ErrorBody { message: d::Body::Kept(Box::from(message)) }));
        assert_eq!(plan.answer(status, &body), Step::Done(Err(Error::Refused)));
    }
}

#[test]
fn status_sends_description_and_link_only_when_present() {
    for (description, link, body) in [
        (
            b"gate passed".as_slice(),
            b"https://forge/run/7".as_slice(),
            br#"{"description":"gate passed","target_url":"https://forge/run/7","state":"success","context":"ci"}"#
                .as_slice(),
        ),
        (b"".as_slice(), b"".as_slice(), br#"{"state":"success","context":"ci"}"#.as_slice()),
    ] {
        let mut plan = known(Op::Write(Write::Status {
            commit: HEAD,
            context: Box::from(*b"ci"),
            check: api::Check::Passed,
            description: Box::from(description),
            link: Box::from(link),
        }));
        round_request(plan.begin());
        plan.assert_bodies(&[Some(body)]);
    }
}

#[test]
fn an_empty_tree_page_is_null_on_the_wire_and_an_empty_answer() {
    let answer = server::Answer::Tree { entries: Box::new([]), truncated: false };
    let bytes = server_answer(&answer);
    assert_eq!(&*bytes, br#"{"truncated":false,"tree":null}"#);
    for body in [bytes.as_ref(), br#"{"tree":[],"truncated":false}"#] {
        let mut plan = known(Op::Read(Read::Tree { object: HEAD, page: 3 }));
        round_request(plan.begin());
        assert_eq!(plan.answer(200, body), Step::Done(Ok(Answer::Tree { entries: Box::new([]), more: false })));
    }
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "step tests construct and inspect wire fixtures (testing-strategy.md, 4)"
)]
fn dispatch_trigger_event_takes_precedence_over_the_empty_payload_event() {
    for fields in [
        br#""event":"","trigger_event":"workflow_dispatch""#.as_slice(),
        br#""trigger_event":"workflow_dispatch","event":"""#.as_slice(),
    ] {
        let original = core::str::from_utf8(RUN_DOC).expect("ASCII fixture");
        let text =
            original.replace("\"event\":\"workflow_dispatch\"", core::str::from_utf8(fields).expect("ASCII fields"));

        let mut plan = known(Op::Read(Read::Runs { workflow: Box::from(*b"ci.yaml"), commit: HEAD, page: 1 }));
        round_request(plan.begin());
        let listing = [b"{\"total_count\":1,\"workflow_runs\":[".as_slice(), text.as_bytes(), b"]}"].concat();
        assert_eq!(
            plan.answer(200, &listing),
            Step::Done(Ok(Answer::Runs { runs: Box::new([run_value()]), more: false }))
        );
    }
    let bytes = server_answer(&server::Answer::Run(wire_run()));
    assert!(
        bytes
            .windows(b"\"event\":\"\",\"trigger_event\":\"workflow_dispatch\"".len())
            .any(|w| w == b"\"event\":\"\",\"trigger_event\":\"workflow_dispatch\"")
    );
}

#[test]
fn the_pinned_diverging_fast_forward_error_is_an_unknown_write_outcome() {
    let message = crate::forgejo::failure::DIVERGING_FAST_FORWARD;
    let body = server_answer(&server::Answer::Error(d::ErrorBody { message: d::Body::Kept(Box::from(message)) }));
    let mut plan = known(Op::Write(Write::Merge { number: 1, head: HEAD, how: MergeHow::FastForwardOnly }));
    round_request(plan.begin());
    assert_eq!(plan.answer(500, &body), Step::Done(Err(Error::Timeout)));
}

#[test]
fn runs_filter_workflows_locally_but_more_counts_the_providers_rows() {
    for (workflow, commit) in [(b"other.yaml".as_slice(), HEAD), (b"ci.yaml".as_slice(), BASE)] {
        let matching = wire_run();
        let mut unrelated = matching.clone();
        unrelated.id = 8;
        unrelated.workflow = Box::from(workflow);
        unrelated.commit_sha.bytes = commit;
        let body =
            server_answer(&server::Answer::Runs { total_count: 2, runs: Box::new([unrelated.clone(), matching]) });
        let mut plan = known(Op::Read(Read::Runs { workflow: Box::from(*b"ci.yaml"), commit: HEAD, page: 1 }));
        round_request(plan.begin());
        assert_eq!(plan.answer(200, &body), Step::Done(Ok(Answer::Runs { runs: Box::new([run_value()]), more: true })));
        let body =
            server_answer(&server::Answer::Runs { total_count: 2, runs: Box::new([unrelated.clone(), unrelated]) });
        let mut plan = known(Op::Read(Read::Runs { workflow: Box::from(*b"ci.yaml"), commit: HEAD, page: 2 }));
        round_request(plan.begin());
        assert_eq!(plan.answer(200, &body), Step::Done(Ok(Answer::Runs { runs: Box::new([]), more: true })));
    }
}

fn range_limits() -> crate::forgejo::limits::Limits {
    // A merge's two parents are the largest nested list in the range fixture.
    crate::forgejo::limits::Limits { inner: 2, body_bytes: 23, ..limits() }
}
const MERGE: api::Commit = {
    let mut id = [0; 32];
    id[0] = 2;
    id
};
fn range_values() -> [api::CommitMessage; 3] {
    [
        api::CommitMessage { id: BASE, parents: Box::new([]), message: Box::from(*b"first\n\nTrailer: first\n") },
        api::CommitMessage { id: HEAD, parents: Box::new([BASE]), message: Box::from(*b"second\n") },
        api::CommitMessage {
            id: MERGE,
            parents: Box::new([HEAD, BASE]),
            message: Box::from(*b"merge\n\nTrailer: merged\n"),
        },
    ]
}
fn range_document() -> server::Answer {
    // The fixture has three commits.
    const COMMITS: u32 = 3;
    let mut commits = List::with_capacity(COMMITS);
    for value in range_values().into_iter().rev() {
        commits.push(wire_commit(value)).expect("fixture commits");
    }
    server::Answer::Commits { commits: commits.into_boxed() }
}

const COMMITS_GET: &str = "GET /api/v1/repos/fixture/specimen/compare/0300000000000000000000000000000000000000...0100000000000000000000000000000000000000?files=false&verification=false";

fn read_commit_page(run: &mut Run, count: Option<&[u8]>, document: &[u8]) -> Step {
    let started = run.begin();
    if let Step::Done(_) = started {
        return started;
    }
    if let Some(count) = count {
        let counted = run.answer(200, count);
        if let Step::Done(_) = counted {
            return counted;
        }
    }
    run.answer(200, document)
}

#[test]
fn commits_reverse_before_paging_and_keep_merge_parents() {
    let [first, second, merge] = range_values();
    let document = server_answer(&range_document());
    for (page, count, requests, expected) in [
        (1, None, [COMMITS_GET].as_slice(), Answer::Commits { commits: Box::new([first, second]), more: true }),
        (
            2,
            Some(document.as_ref()),
            [COMMITS_GET; 2].as_slice(),
            Answer::Commits { commits: Box::new([merge]), more: false },
        ),
    ] {
        let mut run =
            Run::with_limits(range_limits(), BOUNDS, Op::Read(Read::Commits { before: BASE, after: HEAD, page }));
        assert_eq!(read_commit_page(&mut run, count, &document), Step::Done(Ok(expected)));
        run.assert_sent(requests);
    }
}

// The incident's 24 commits at eight per page, with a two-parent newest commit.
const RANGE_COMMITS: u32 = 24;
const RANGE_PAGE: u32 = 8;
const LONG_RANGE_BOUNDS: Bounds = Bounds { page_size: RANGE_PAGE, rows: RANGE_PAGE, ..BOUNDS };
// The 24 wire rows, each under 150 bytes except for the merge's extra parent.
const LONG_RANGE_DOCUMENT_BYTES: u32 = 4096;
const RANGE_MESSAGE: &[u8] = b"commit\n";
// Eight native commit slots, nine parents and eight whole messages.
const NEWEST_PAGE_BYTES: u32 = 8 * 64 + 9 * 32 + 8 * 7;

fn long_range_id(ordinal: u32) -> api::Commit {
    let mut id = [0; 32];
    *id.get_mut(1).expect("id has 32 bytes") = u8::try_from(ordinal).expect("fixture ordinal fits");
    id
}

fn long_range_value(ordinal: u32) -> api::CommitMessage {
    let parent = if ordinal == 1 { BASE } else { long_range_id(ordinal.saturating_sub(1)) };
    let parents: Box<[api::Commit]> =
        if ordinal == RANGE_COMMITS { Box::new([BASE, parent]) } else { Box::new([parent]) };
    api::CommitMessage { id: long_range_id(ordinal), parents, message: Box::from(RANGE_MESSAGE) }
}

fn long_range_limits() -> crate::forgejo::limits::Limits {
    let mut limits = range_limits();
    limits.writer.length = LONG_RANGE_DOCUMENT_BYTES;
    limits.json.length = LONG_RANGE_DOCUMENT_BYTES;
    limits
}

fn wire_id(bytes: api::Commit) -> crate::forgejo::id::ObjectId {
    crate::forgejo::id::ObjectId { bytes, format: crate::forgejo::id::ObjectFormat::Sha1 }
}

fn wire_commit(value: api::CommitMessage) -> d::CommitMessage {
    // The fixtures have at most two parents per commit.
    const PARENTS: u32 = 2;
    let mut parents = List::with_capacity(PARENTS);
    for id in &value.parents {
        parents.push(wire_id(*id)).expect("fixture parents");
    }
    d::CommitMessage { id: wire_id(value.id), parents: parents.into_boxed(), message: value.message }
}

fn long_range_document() -> Box<[u8]> {
    let mut commits = List::with_capacity(RANGE_COMMITS);
    for ordinal in (1..=RANGE_COMMITS).rev() {
        commits.push(wire_commit(long_range_value(ordinal))).expect("fixture commits");
    }
    server::encode(&server::Answer::Commits { commits: commits.into_boxed() }, &long_range_limits())
        .expect("long range fits")
}

fn long_range_run(page: u32, bounds: Bounds) -> Run {
    Run::with_limits(long_range_limits(), bounds, Op::Read(Read::Commits { before: BASE, after: HEAD, page }))
}

fn long_range_page(first: u32, last: u32, more: bool) -> Step {
    let mut commits = List::with_capacity(RANGE_PAGE);
    for ordinal in first..=last {
        commits.push(long_range_value(ordinal)).expect("fixture page");
    }
    Step::Done(Ok(Answer::Commits { commits: commits.into_boxed(), more }))
}

// The count probe needs no commit fields, even when the commits precede the count.
const LONG_RANGE_COUNT: &[u8] = br#"{"commits":[{"commit":{"message":"this message is longer than the decoder cap"}}],"total_commits":24,"files":[]}"#;

#[test]
fn a_long_commit_range_answers_every_page_oldest_first() {
    let document = long_range_document();
    for (page, count, requests, expected) in [
        (1, None, [COMMITS_GET].as_slice(), long_range_page(1, 8, true)),
        (2, Some(LONG_RANGE_COUNT), [COMMITS_GET; 2].as_slice(), long_range_page(9, 16, true)),
        (3, Some(LONG_RANGE_COUNT), [COMMITS_GET; 2].as_slice(), long_range_page(17, 24, false)),
    ] {
        let mut run = long_range_run(page, LONG_RANGE_BOUNDS);
        assert_eq!(read_commit_page(&mut run, count, &document), expected, "page {page}");
        run.assert_sent(requests);
    }
}

#[test]
fn a_later_commit_page_needs_only_its_own_bytes() {
    let document = long_range_document();
    let mut run = long_range_run(3, Bounds { answer_bytes: NEWEST_PAGE_BYTES, ..LONG_RANGE_BOUNDS });
    assert_eq!(read_commit_page(&mut run, Some(LONG_RANGE_COUNT), &document), long_range_page(17, 24, false));
}

#[test]
fn a_later_commit_page_refuses_its_own_excess() {
    let document = long_range_document();
    for (bounds, excess) in [
        (
            Bounds { answer_bytes: NEWEST_PAGE_BYTES - 1, ..LONG_RANGE_BOUNDS },
            Excess::at_least(api::Bound::AnswerBytes, u64::from(NEWEST_PAGE_BYTES - 1), u64::from(NEWEST_PAGE_BYTES)),
        ),
        (
            Bounds { rows: RANGE_PAGE - 1, ..LONG_RANGE_BOUNDS },
            Excess::at_least(api::Bound::Rows, u64::from(RANGE_PAGE - 1), u64::from(RANGE_PAGE)),
        ),
    ] {
        let mut run = long_range_run(3, bounds);
        assert_eq!(
            read_commit_page(&mut run, Some(LONG_RANGE_COUNT), &document),
            Step::Done(Err(Error::TooLarge(excess)))
        );
    }
}

#[test]
fn a_commit_page_past_the_range_ends_after_counting() {
    for page in [4, u32::MAX] {
        let mut run = long_range_run(page, LONG_RANGE_BOUNDS);
        assert_eq!(
            read_commit_page(&mut run, Some(LONG_RANGE_COUNT), b""),
            Step::Done(Ok(Answer::Commits { commits: Box::new([]), more: false })),
        );
        run.assert_sent(&[COMMITS_GET]);
    }
}

#[test]
fn the_two_commit_passes_must_agree_on_the_total() {
    // Both bodies report a count, but only the second's rows matter.
    const FIRST_COUNT: &[u8] = br#"{"total_commits":25,"commits":[],"files":[]}"#;
    let mut run = long_range_run(3, LONG_RANGE_BOUNDS);
    assert_eq!(
        read_commit_page(&mut run, Some(FIRST_COUNT), &long_range_document()),
        Step::Done(Err(Error::InvalidAnswer)),
    );
    run.assert_sent(&[COMMITS_GET; 2]);
}

#[test]
fn later_commit_pages_skip_both_sides_of_their_window() {
    // Only the middle wire row fits the decoder's message bound. The count follows the rows.
    const DOCUMENT: &[u8] = br#"{"commits":[{"sha":"0200000000000000000000000000000000000000","parents":[],"commit":{"message":"newer message past the body cap"}},{"sha":"0100000000000000000000000000000000000000","parents":[{"sha":"0300000000000000000000000000000000000000"}],"commit":{"message":"second\n"}},{"sha":"0300000000000000000000000000000000000000","parents":[],"commit":{"message":"older message past the body cap"}}],"total_commits":3,"files":[]}"#;
    let [_, middle, _] = range_values();
    let bounds = Bounds { page_size: 1, ..BOUNDS };
    let mut run =
        Run::with_limits(range_limits(), bounds, Op::Read(Read::Commits { before: BASE, after: HEAD, page: 2 }));
    assert_eq!(
        read_commit_page(&mut run, Some(DOCUMENT), DOCUMENT),
        Step::Done(Ok(Answer::Commits { commits: Box::new([middle]), more: true })),
    );
}
// Three native commit slots, three parent ids, and these whole messages.
const RANGE_BYTES: u32 = 192 + 96 + 22 + 7 + 23;
// Two native slots, second's parent, and first and second's whole messages.
const OLDEST_TWO_BYTES: u32 = 128 + 32 + 22 + 7;
// First's native slot and whole message, with no parents.
const OLDEST_BYTES: u32 = 64 + 22;
#[test]
fn a_range_past_the_answer_keeps_its_oldest_commits() {
    let oldest_page = || {
        let [first, second, _] = range_values();
        Step::Done(Ok(Answer::Commits { commits: Box::new([first, second]), more: true }))
    };
    skein_rows::check(
        [
            ("all_three_at_the_answer_bound", (Bounds { answer_bytes: RANGE_BYTES, ..BOUNDS }, 1, oldest_page())),
            (
                "one_byte_under_keeps_the_oldest_two",
                (Bounds { answer_bytes: RANGE_BYTES - 1, ..BOUNDS }, 1, oldest_page()),
            ),
            ("two_rows_keep_the_oldest_two", (Bounds { rows: 2, ..BOUNDS }, 1, oldest_page())),
            (
                "only_the_oldest_kept_holds_no_whole_page",
                (
                    Bounds { answer_bytes: OLDEST_TWO_BYTES - 1, ..BOUNDS },
                    1,
                    Step::Done(Err(Error::TooLarge(Excess::at_least(
                        api::Bound::AnswerBytes,
                        u64::from(OLDEST_TWO_BYTES - 1),
                        u64::from(OLDEST_TWO_BYTES),
                    )))),
                ),
            ),
            (
                "the_oldest_alone_past_the_answer",
                (
                    Bounds { answer_bytes: OLDEST_BYTES - 1, ..BOUNDS },
                    1,
                    Step::Done(Err(Error::TooLarge(Excess::at_least(
                        api::Bound::AnswerBytes,
                        u64::from(OLDEST_BYTES - 1),
                        u64::from(OLDEST_BYTES),
                    )))),
                ),
            ),
            (
                "page_two_fits_two_rows",
                (
                    Bounds { rows: 2, ..BOUNDS },
                    2,
                    Step::Done(Ok(Answer::Commits {
                        commits: Box::new([range_values().into_iter().last().expect("merge fixture")]),
                        more: false,
                    })),
                ),
            ),
        ],
        |(bounds, page, expected)| {
            let mut run =
                Run::with_limits(range_limits(), bounds, Op::Read(Read::Commits { before: BASE, after: HEAD, page }));
            let document = server_answer(&range_document());
            let count = (page > 1).then_some(document.as_ref());
            assert_eq!(read_commit_page(&mut run, count, &document), expected);
        },
    );
}
