//! Literal write exchanges from forge/forgejo-operations.md, section 1.
use super::{Answer, BRANCH, Error, HEAD, ISSUE, PULL};
use crate::api::{self, Write};
use crate::forgejo::tests::fixtures::Run;
pub(super) const COMPARISON: &[u8] = br#"{"total_commits":1,"commits":[],"files":[]}"#;
const COMMENT: &[u8] = br#"{"id":7,"user":{"id":2,"login":"writer"},"body":"b","created_at":"1970-01-01T00:00:00Z"}"#;
const REVIEW: &[u8] = br#"{"id":7,"user":{"id":2,"login":"writer"},"state":"APPROVED","commit_id":"0100000000000000000000000000000000000000","body":"b","submitted_at":"1970-01-01T00:00:00Z","official":true,"dismissed":false}"#;
const STATUS: &[u8] =
    br#"{"creator":{"id":2,"login":"writer"},"context":"ci","status":"success","description":"","target_url":"","updated_at":"1970-01-01T00:00:00Z"}"#;
const MERGED: &[u8] = br#"{"number":1,"state":"closed","head":{"repo_id":1,"ref":"work","sha":"0100000000000000000000000000000000000000"},"base":{"repo_id":1,"ref":"main","sha":"0300000000000000000000000000000000000000"},"merged":true,"merge_commit_sha":"0100000000000000000000000000000000000000","mergeable":true,"requested_reviewers":[]}"#;
pub(in super::super) const REVIEWERS: &[u8] = br#"{"number":1,"state":"open","head":{"repo_id":1,"ref":"work","sha":"0100000000000000000000000000000000000000"},"base":{"repo_id":1,"ref":"main","sha":"0300000000000000000000000000000000000000"},"merged":false,"merge_commit_sha":null,"mergeable":true,"requested_reviewers":[{"id":8,"login":"eight"}]}"#;
pub(in super::super) fn open_pull() -> Write {
    Write::OpenPull {
        title: Box::from(&b"t"[..]),
        body: Box::from(&b"b"[..]),
        head: Box::from(&b"work"[..]),
        base: Box::from(&b"main"[..]),
    }
}
pub(in super::super) fn create_issue() -> Write {
    Write::CreateIssue { key: Box::from(&b"k"[..]), title: Box::from(&b"t"[..]), body: Box::from(&b"b"[..]) }
}
pub(in super::super) fn begun(write: Write) -> Run {
    let mut run = Run::known(api::Op::Write(write));
    run.begin();
    run
}
pub(in super::super) struct WriteCase {
    pub name: &'static str,
    pub write: Write,
    pub replies: &'static [(u16, &'static [u8])],
    pub expected: Result<Answer, Error>,
    pub sends: &'static [&'static str],
    pub users: &'static [(u64, &'static [u8])],
    pub bodies: &'static [Option<&'static [u8]>],
}
// forge/forgejo.md, 1.4: Forgejo's state-edit refusal for a merged pull request.
const MERGED_REFUSAL: &[u8] = br#"{"message":"cannot change state of this pull request, it was already merged"}"#;
const CLOSED_ITEM: &[u8] = br#"{"number":7,"state":"closed","pull_request":{"merged":true},"user":{"id":2,"login":"writer"},"labels":[],"updated_at":"1970-01-01T00:00:00Z","title":"t","body":"b"}"#;
const OPEN_ITEM: &[u8] = br#"{"number":7,"state":"open","user":{"id":2,"login":"writer"},"labels":[],"updated_at":"1970-01-01T00:00:00Z","title":"t","body":"b"}"#;
const CLOSE_SENDS: &[&str] =
    &["PATCH /api/v1/repos/fixture/specimen/issues/7", "GET /api/v1/repos/fixture/specimen/issues/7"];

#[test]
fn refused_closes_read_the_item_once_and_keep_the_refusal_unless_closed() {
    use crate::forgejo::plans::{Failure, plan::Step};
    for (name, status, body, read_status, read_body, expected) in [
        ("merged", 412, MERGED_REFUSAL, 200, CLOSED_ITEM, Ok(Answer::Done)),
        ("open", 412, MERGED_REFUSAL, 200, OPEN_ITEM, Err(Error::Refused)),
        ("missing read", 412, MERGED_REFUSAL, 404, &b"{}"[..], Err(Error::Refused)),
        ("failed read", 412, MERGED_REFUSAL, 500, &b"{}"[..], Err(Error::Refused)),
        ("undecodable read", 412, MERGED_REFUSAL, 200, &b"{"[..], Err(Error::Refused)),
        ("unknown message", 422, &br#"{"message":"unknown"}"#[..], 200, CLOSED_ITEM, Ok(Answer::Done)),
        ("other refused status", 400, &b"{}"[..], 200, CLOSED_ITEM, Ok(Answer::Done)),
    ] {
        let mut run = begun(Write::Close { number: 7 });
        assert_eq!(
            run.answer(status, body),
            Step::send(crate::forgejo::request::Request::Issue {
                at: crate::forgejo::tests::fixtures::repository(),
                number: 7,
            }),
            "{name}"
        );
        assert_eq!(run.answer(read_status, read_body), Step::Done(expected), "{name}");
        run.assert_sent(CLOSE_SENDS);
        run.assert_bodies(&[Some(br#"{"state":"closed"}"#), None]);
    }
    for failure in [Failure::Unseen, Failure::Seen] {
        let mut run = begun(Write::Close { number: 7 });
        run.answer(412, MERGED_REFUSAL);
        assert_eq!(run.fail(failure), Step::Done(Err(Error::Refused)));
        run.assert_sent(CLOSE_SENDS);
    }
}

#[test]
fn closes_read_nothing_after_other_classifications_and_reopen_never_reads() {
    use crate::forgejo::plans::{Fields, plan::Step};
    use crate::forgejo::tests::fixtures;
    use skein_lib::Duration;
    for (status, body, expected) in [
        (201, ISSUE, Ok(Answer::Done)),
        (404, &b"{}"[..], Err(Error::Missing)),
        (500, &b"{}"[..], Err(Error::Timeout)),
        (200, &b"{"[..], Err(Error::Timeout)),
        (401, &b"{}"[..], Err(Error::Forbidden)),
        (403, &b"{}"[..], Err(Error::Forbidden)),
        (429, &b"{}"[..], Err(Error::RateLimited { after: Duration::from_secs(5) })),
    ] {
        let mut run = begun(Write::Close { number: 7 });
        let fields = Fields { retry_after: Some(Duration::from_secs(5)), ..fixtures::fields() };
        assert_eq!(run.answer_with(status, body, fields), Step::Done(expected), "{status}");
        run.assert_sent(&[CLOSE_SENDS[0]]);
    }
    let mut run = begun(Write::Reopen { number: 7 });
    assert_eq!(run.answer(412, MERGED_REFUSAL), Step::Done(Err(Error::Refused)));
    run.assert_sent(&[CLOSE_SENDS[0]]);
}

pub(in super::super) fn cases() -> [[WriteCase; 5]; 3] {
    [creating(), editing(), branches()]
}
fn creating() -> [WriteCase; 5] {
    [
        WriteCase {
            name: "create issue",
            write: create_issue(),
            replies: &[(201, ISSUE)],
            expected: Ok(Answer::Created(1)),
            sends: &["POST /api/v1/repos/fixture/specimen/issues"],
            users: &[],
            bodies: &[Some(br#"{"title":"t","body":"<!-- kit:key 6b -->\nb"}"#)],
        },
        WriteCase {
            name: "open pull",
            write: open_pull(),
            replies: &[(200, BRANCH), (200, BASE_BRANCH), (200, COMPARISON), (201, PULL)],
            expected: Ok(Answer::Created(1)),
            sends: &OPEN_SENDS,
            users: &[],
            bodies: &[None, None, None, Some(br#"{"head":"work","base":"main","title":"t","body":"b"}"#)],
        },
        WriteCase {
            name: "post",
            write: Write::Post { number: 1, key: Box::from(&b"k"[..]), body: Box::from(&b"b"[..]) },
            replies: &[(201, COMMENT)],
            expected: Ok(Answer::Commented(7)),
            sends: &["POST /api/v1/repos/fixture/specimen/issues/1/comments"],
            users: &[],
            bodies: &[Some(br#"{"body":"<!-- kit:key 6b -->\nb"}"#)],
        },
        WriteCase {
            name: "review",
            write: Write::Review {
                number: 1,
                key: Box::from(&b"k"[..]),
                verdict: api::Verdict::Approve,
                body: Box::from(&b"b"[..]),
            },
            replies: &[(200, REVIEW)],
            expected: Ok(Answer::Reviewed(7)),
            sends: &["POST /api/v1/repos/fixture/specimen/pulls/1/reviews"],
            users: &[],
            bodies: &[Some(br#"{"event":"APPROVED","body":"<!-- kit:key 6b -->\nb"}"#)],
        },
        WriteCase {
            name: "edit",
            write: Write::Edit { number: 1, title: Some(Box::from(&b"t"[..])), body: Some(Box::from(&b"b"[..])) },
            replies: &[(201, ISSUE)],
            expected: Ok(Answer::Done),
            sends: &["PATCH /api/v1/repos/fixture/specimen/issues/1"],
            users: &[],
            bodies: &[Some(br#"{"title":"t","body":"b"}"#)],
        },
    ]
}
fn editing() -> [WriteCase; 5] {
    [
        WriteCase {
            name: "set reviewers",
            write: Write::SetReviewers { number: 1, reviewers: Box::from([7]) },
            replies: &[(200, PULL), (201, b"")],
            expected: Ok(Answer::Done),
            sends: &[
                "GET /api/v1/repos/fixture/specimen/pulls/1",
                "POST /api/v1/repos/fixture/specimen/pulls/1/requested_reviewers",
            ],
            users: &[(7, b"seven")],
            bodies: &[None, Some(br#"{"reviewers":["seven"]}"#)],
        },
        WriteCase {
            name: "close",
            write: Write::Close { number: 1 },
            replies: &[(201, ISSUE)],
            expected: Ok(Answer::Done),
            sends: &["PATCH /api/v1/repos/fixture/specimen/issues/1"],
            users: &[],
            bodies: &[Some(br#"{"state":"closed"}"#)],
        },
        WriteCase {
            name: "reopen",
            write: Write::Reopen { number: 1 },
            replies: &[(201, ISSUE)],
            expected: Ok(Answer::Done),
            sends: &["PATCH /api/v1/repos/fixture/specimen/issues/1"],
            users: &[],
            bodies: &[Some(br#"{"state":"open"}"#)],
        },
        WriteCase {
            name: "merge",
            write: Write::Merge { number: 1, head: HEAD, how: api::MergeHow::Style },
            replies: &[(200, b""), (200, MERGED)],
            expected: Ok(Answer::Merged(HEAD)),
            sends: &["POST /api/v1/repos/fixture/specimen/pulls/1/merge", "GET /api/v1/repos/fixture/specimen/pulls/1"],
            users: &[],
            bodies: &[Some(br#"{"Do":"squash","head_commit_id":"0100000000000000000000000000000000000000"}"#), None],
        },
        WriteCase {
            name: "update",
            write: Write::Update { number: 1, style: api::UpdateStyle::Merge },
            replies: &[(200, b"")],
            expected: Ok(Answer::Done),
            sends: &["POST /api/v1/repos/fixture/specimen/pulls/1/update?style=merge"],
            users: &[],
            bodies: &[None],
        },
    ]
}
fn branches() -> [WriteCase; 5] {
    [
        WriteCase {
            name: "status",
            write: Write::Status {
                commit: HEAD,
                context: Box::from(&b"ci"[..]),
                check: api::Check::Passed,
                description: Box::new([]),
                link: Box::new([]),
            },
            replies: &[(201, STATUS)],
            expected: Ok(Answer::Done),
            sends: &["POST /api/v1/repos/fixture/specimen/statuses/0100000000000000000000000000000000000000"],
            users: &[],
            bodies: &[Some(br#"{"state":"success","context":"ci"}"#)],
        },
        WriteCase {
            name: "create branch",
            write: Write::CreateBranch { branch: Box::from(&b"work"[..]), commit: HEAD },
            replies: &[(201, BRANCH)],
            expected: Ok(Answer::Branch(api::BranchCreation::Created)),
            sends: &["POST /api/v1/repos/fixture/specimen/branches"],
            users: &[],
            bodies: &[Some(br#"{"new_branch_name":"work","old_ref_name":"0100000000000000000000000000000000000000"}"#)],
        },
        WriteCase {
            name: "existing branch",
            write: Write::CreateBranch { branch: Box::from(&b"work"[..]), commit: HEAD },
            replies: &[(409, b"{}")],
            expected: Ok(Answer::Branch(api::BranchCreation::Exists)),
            sends: &["POST /api/v1/repos/fixture/specimen/branches"],
            users: &[],
            bodies: &[Some(br#"{"new_branch_name":"work","old_ref_name":"0100000000000000000000000000000000000000"}"#)],
        },
        WriteCase {
            name: "delete branch",
            write: Write::DeleteBranch { branch: Box::from(&b"work"[..]) },
            replies: &[(204, b"")],
            expected: Ok(Answer::Done),
            sends: &["DELETE /api/v1/repos/fixture/specimen/branches/work"],
            users: &[],
            bodies: &[None],
        },
        WriteCase {
            name: "nothing to merge",
            write: open_pull(),
            replies: &[(200, BRANCH), (200, BASE_BRANCH), (200, crate::forgejo::tests::fixtures::EMPTY_COMPARE)],
            expected: Err(Error::NothingToMerge),
            sends: &OPEN_SENDS[..COMPARE_REQUESTS],
            users: &[],
            bodies: &[None, None, None],
        },
    ]
}
pub(super) const BASE_BRANCH: &[u8] = br#"{"name":"main","commit":{"id":"0300000000000000000000000000000000000000"},"effective_branch_protection_name":""}"#;

use super::{Asked, Drop};
use skein_lib::Duration;
type FailureCase = (Asked, u16, &'static [u8], Option<Duration>, Error, Drop);
pub(super) fn failures() -> [FailureCase; 47] {
    use Asked::{BeforeWrite, DeleteBranch, Job, Leaf, Merge, OpenPull, Read, Update, Write};
    use Drop::{Nothing, Path, Row};
    [
        (Read, 302, b"", None, Error::Unavailable, Path),
        (Write, 302, b"", None, Error::Unavailable, Path),
        (Write, 401, b"", None, Error::Forbidden, Row),
        (Write, 403, b"", None, Error::Forbidden, Row),
        (Read, 401, b"", None, Error::Forbidden, Nothing),
        (Leaf, 404, b"", None, Error::Missing, Nothing),
        (Leaf, 403, b"", None, Error::Forbidden, Nothing),
        (Read, 403, b"", None, Error::Forbidden, Nothing),
        (Write, 423, b"", None, Error::Forbidden, Row),
        (Read, 404, b"", None, Error::Missing, Path),
        (Job, 404, b"", None, Error::MissingJob, Nothing),
        (Write, 404, b"", None, Error::Missing, Path),
        (Write, 413, b"", None, Error::Full, Nothing),
        (OpenPull, 409, b"", None, Error::Exists, Nothing),
        (Read, 500, b"", None, Error::Timeout, Nothing),
        (Write, 503, b"", None, Error::Timeout, Nothing),
        (Read, 400, b"", None, Error::InvalidAnswer, Nothing),
        (Write, 400, b"", None, Error::Refused, Nothing),
        (DeleteBranch, 403, b"", None, Error::Protected, Nothing),
        (BeforeWrite, 500, b"", None, Error::Unavailable, Nothing),
        (BeforeWrite, 400, b"", None, Error::Unavailable, Nothing),
        (Read, 422, b"", None, Error::InvalidAnswer, Nothing),
        (Write, 429, b"", None, Error::RateLimited { after: Duration::ZERO }, Nothing),
        (Write, 429, b"", Some(Duration::from_secs(5)), Error::RateLimited { after: Duration::from_secs(5) }, Nothing),
        (BeforeWrite, 404, b"", None, Error::Missing, Path),
        (BeforeWrite, 401, b"", None, Error::Forbidden, Nothing),
        (Merge, 405, b"", None, Error::Closed, Nothing),
        (Merge, 405, b"!", None, Error::Refused, Nothing),
        (Merge, 400, b"", None, Error::Refused, Nothing),
        (Merge, 405, b"Please try again later", None, Error::Conflict, Nothing),
        (Merge, 405, b"Please try again later!", None, Error::Refused, Nothing),
        (Merge, 400, b"Please try again later", None, Error::Refused, Nothing),
        (Merge, 405, b"not allowed to merge [reason: Does not have enough approvals]", None, Error::Protected, Nothing),
        (Merge, 405, b"not allowed to merge [reason: Does not have enough approvals]!", None, Error::Refused, Nothing),
        (Merge, 400, b"not allowed to merge [reason: Does not have enough approvals]", None, Error::Refused, Nothing),
        (Merge, 405, b"squash is not an allowed merge style for this repository", None, Error::Refused, Row),
        (Merge, 405, b"squash is not an allowed merge style for this repository!", None, Error::Refused, Nothing),
        (Merge, 400, b"squash is not an allowed merge style for this repository", None, Error::Refused, Nothing),
        (Merge, 409, b"head out of date", None, Error::Stale, Nothing),
        (Merge, 409, b"head out of date!", None, Error::Refused, Nothing),
        (Merge, 400, b"head out of date", None, Error::Refused, Nothing),
        (Update, 409, b"merge failed because of conflict", None, Error::Conflict, Nothing),
        (Update, 409, b"merge failed because of conflict!", None, Error::Refused, Nothing),
        (Update, 400, b"merge failed because of conflict", None, Error::Refused, Nothing),
        (Write, 422, b"[Title]: Required", None, Error::Empty, Nothing),
        (Write, 422, b"[Title]: Required!", None, Error::Refused, Nothing),
        (Write, 400, b"[Title]: Required", None, Error::Refused, Nothing),
    ]
}

// The three pre-POST stages in forge/forgejo-operations.md 1, counting the request at which the call fails.
const HEAD_REQUESTS: usize = 1;
const BASE_REQUESTS: usize = 2;
const COMPARE_REQUESTS: usize = 3;
pub(in super::super) const OPEN_SENDS: [&str; 4] = [
    "GET /api/v1/repos/fixture/specimen/branches/work",
    "GET /api/v1/repos/fixture/specimen/branches/main",
    "GET /api/v1/repos/fixture/specimen/compare/0300000000000000000000000000000000000000...0100000000000000000000000000000000000000?files=false&verification=false",
    "POST /api/v1/repos/fixture/specimen/pulls",
];

pub(in super::super) type OpenCase = (&'static [(u16, &'static [u8])], &'static [&'static str]);
pub(in super::super) fn open_failures() -> [OpenCase; 3] {
    [
        (&[], &OPEN_SENDS[..HEAD_REQUESTS]),
        (&[(200, BRANCH)], &OPEN_SENDS[..BASE_REQUESTS]),
        (&[(200, BRANCH), (200, BASE_BRANCH)], &OPEN_SENDS[..COMPARE_REQUESTS]),
    ]
}
