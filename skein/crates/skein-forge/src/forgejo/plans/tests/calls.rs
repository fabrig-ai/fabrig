//! Canned exchanges for every call, without a connected component.
use crate::api::{self, Answer, Error, Op, Read};
use crate::forgejo::plans::plan::{Range, Step};
use crate::forgejo::tests::fixtures::{
    self, ATTEMPT, BASE, BOUNDS, BRANCH, EMPTY_COMPARE, EMPTY_STATUS, HEAD, JOB, PAGE_SIZE, PULL, RAW_BYTES, REPO,
    REPOSITORY, ROWS, RUN, Run,
};
use skein_lib::Time;
const ISSUE: &[u8] = br#"{"number":1,"state":"open","user":{"id":2,"login":"writer"},"labels":[],"updated_at":"1970-01-01T00:00:00Z","title":"t","body":"b"}"#;
fn summary() -> api::Summary {
    api::Summary {
        number: 1,
        kind: api::Kind::Issue,
        state: api::State::Open,
        author: 2,
        labels: Box::new([]),
        title: Box::from(&b"t"[..]),
        body: Box::from(&b"b"[..]),
        key: None,
        updated: Time::ZERO,
    }
}
fn pull() -> api::Pull {
    api::Pull {
        queued: None,
        dequeued: None,
        size: None,
        number: 1,
        state: api::State::Open,
        head: Box::from(&b"work"[..]),
        base: Box::from(&b"main"[..]),
        commit: HEAD,
        base_commit: Some(BASE),
        merged: None,
        mergeable: true,
        reviewers: Box::new([]),
        ci: api::Ci::None,
    }
}
pub(super) struct ReadCase {
    pub(super) name: &'static str,
    pub(super) known: bool,
    pub(super) read: Read,
    pub(super) replies: &'static [(u16, &'static [u8])],
    pub(super) expected: Result<Answer, Error>,
    sends: &'static [&'static str],
    range: Option<Range>,
    kept: bool,
    login: Option<&'static [u8]>,
}
pub(super) fn read_cases() -> [Box<[ReadCase]>; 4] {
    [find_cases(), Box::new(review_cases()), Box::new(content_cases()), Box::new(repository_cases())]
}
fn find_cases() -> Box<[ReadCase]> {
    Box::new([
        ReadCase {
            name: "locate", known: false,
            read: Read::Locate { owner: Box::from(&b"fixture"[..]), name: Box::from(&b"specimen"[..]) },
            replies: &[(200, REPOSITORY)],
            expected: Ok(Answer::Located { repository: 1 }),
            sends: &["GET /api/v1/repos/fixture/specimen"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "items", known: false,
            read: Read::Items { single: false, since: Time::ZERO, page: 1, kind: None },
            replies: &[(200, REPOSITORY), (200, b"[]")],
            expected: Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: Time::ZERO }),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/issues?state=all&since=1970-01-01T00%3A00%3A00Z&sort=leastupdate&page=1&limit=2",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "item", known: false,
            read: Read::Item { number: 1, after: 0 , since: Time::ZERO },
            replies: &[(200, REPOSITORY), (200, ISSUE), (200, b"[]")],
            expected: Ok(Answer::Item { item: summary(), comments: Box::new([]), more: false }),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/issues/1",
                "GET /api/v1/repos/fixture/specimen/issues/1/comments",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "item comments since, including an edited older id", known: false,
            read: Read::Item { number: 1, after: 1, since: Time::from_nanos(1_000_000_000) },
            replies: &[(200, REPOSITORY), (200, ISSUE), (200, br#"[{"id":1,"user":{"id":2,"login":"writer"},"created_at":"1970-01-01T00:00:00Z","updated_at":"1970-01-01T00:00:02Z","body":"edited"},{"id":2,"user":{"id":2,"login":"writer"},"created_at":"1970-01-01T00:00:02Z","updated_at":"1970-01-01T00:00:02Z","body":"new"}]"#)],
            expected: Ok(Answer::Item { item: summary(), comments: Box::new([api::Comment {
                id: 2, author: 2, created: Time::from_nanos(2_000_000_000),
                body: Box::from(*b"new"), key: None, provenance: api::Provenance::Original,
                revision: super::super::reading::revision(&crate::forgejo::document::Body::Kept(Box::from(*b"new")), Some(skein_lib::Wall::from_nanos(2_000_000_000))),
            }]), more: false }),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/issues/1",
                "GET /api/v1/repos/fixture/specimen/issues/1/comments?since=1970-01-01T00%3A00%3A01Z",
            ],
            range: None, kept: true, login: None,
        },
        ReadCase {
            name: "pull without ci",
            known: true,
            read: Read::Pull { number: 7, ci: false },
            replies: &[(200, br#"{"number":7,"state":"open","head":{"repo_id":1,"ref":"work","sha":"0100000000000000000000000000000000000000"},"base":{"repo_id":1,"ref":"main","sha":"0300000000000000000000000000000000000000"},"merged":false,"merge_commit_sha":null,"merge_base":"0300000000000000000000000000000000000000","mergeable":true,"requested_reviewers":[]}"#)],
            expected: Ok(Answer::Pull(api::Pull { number: 7, ..pull() })),
            sends: &["GET /api/v1/repos/fixture/specimen/pulls/7"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "pull", known: false,
            read: Read::Pull { number: 1, ci: true },
            replies: &[(200, REPOSITORY), (200, PULL), (200, EMPTY_STATUS)],
            expected: Ok(Answer::Pull(pull())),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/pulls/1",
                "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=1&limit=2",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "pull for", known: false,
            read: Read::PullFor { head: Box::from(&b"work"[..]), base: Box::from(&b"main"[..]) },
            replies: &[(200, REPOSITORY), (200, PULL)],
            expected: Ok(Answer::Pull(pull())),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/pulls/main/work",
            ],
            range: None,
            kept: true,
            login: None,
        },
    ])
}
fn review_cases() -> [ReadCase; 5] {
    [
        ReadCase {
            name: "reviews",
            known: false,
            read: Read::Reviews { number: 1, page: 1, skip: 0 },
            replies: &[(200, REPOSITORY), (200, b"[]")],
            expected: Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
            sends: &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/pulls/1/reviews?page=1&limit=2"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "statuses",
            known: false,
            read: Read::Statuses { commit: HEAD, page: 1 },
            replies: &[(200, REPOSITORY), (200, EMPTY_STATUS)],
            expected: Ok(Answer::Statuses { ci: api::Ci::None, statuses: Box::new([]), more: false }),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=1&limit=2",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "remarks",
            known: false,
            read: Read::Remarks { number: 1, review: 7, page: 1, skip: 0 },
            replies: &[(200, REPOSITORY), (200, b"[]")],
            expected: Ok(Answer::Remarks { remarks: Box::new([]), more: false, cut: false }),
            sends: &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/pulls/1/reviews/7/comments"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "branch",
            known: false,
            read: Read::Branch { branch: Box::from(&b"work"[..]) },
            replies: &[(200, REPOSITORY), (200, BRANCH)],
            expected: Ok(Answer::Commit(HEAD)),
            sends: &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/branches/work"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "branches",
            known: false,
            read: Read::Branches { prefix: Box::from(&b"m"[..]) },
            replies: &[(200, REPOSITORY), (200, b"[]")],
            expected: Ok(Answer::Branches(Box::new([]))),
            sends: &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/git/refs/heads/m"],
            range: None,
            kept: true,
            login: None,
        },
    ]
}
#[expect(clippy::too_many_lines, reason = "each canned content exchange states its request and bounded answer")]
fn content_cases() -> [ReadCase; 6] {
    [
        ReadCase {
            name: "pull files",
            known: false,
            read: Read::PullFiles { number: 1, head: HEAD, page: 1, skip: 0, sides: true },
            replies: &[(200, REPOSITORY), (200, PULL), (200, b"[]")],
            expected: Ok(Answer::PullFiles { head: HEAD, files: Box::new([]), more: false, cut: false }),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/pulls/1",
                "GET /api/v1/repos/fixture/specimen/pulls/1/files?page=1&limit=2",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "pull paths",
            known: true,
            read: Read::PullFiles { number: 1, head: HEAD, page: 1, skip: 0, sides: false },
            replies: &[
                (200, PULL),
                (200, br#"[{"filename":"a","status":"modified"},{"filename":"b","status":"added"}]"#),
            ],
            expected: Ok(Answer::PullFiles {
                head: HEAD,
                files: Box::new([
                    api::File { path: Box::from(*b"a"), before: None, after: None },
                    api::File { path: Box::from(*b"b"), before: None, after: None },
                ]),
                more: true,
                cut: false,
            }),
            sends: &[
                "GET /api/v1/repos/fixture/specimen/pulls/1",
                "GET /api/v1/repos/fixture/specimen/pulls/1/files?page=1&limit=2",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "compare",
            known: false,
            read: Read::Compare { before: BASE, after: HEAD },
            replies: &[(200, REPOSITORY), (200, LARGE_COMPARE), (200, REVERSE_COMPARE)],
            expected: Ok(Answer::Compare {
                before: BASE,
                after: HEAD,
                ahead: 22,
                behind: 3,
                changed: 12,
                files: Box::new([
                    Box::from(*b"a"),
                    Box::from(*b"b"),
                    Box::from(*b"c"),
                    Box::from(*b"d"),
                    Box::from(*b"e"),
                    Box::from(*b"f"),
                    Box::from(*b"g"),
                    Box::from(*b"h"),
                ]),
            }),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/compare/0300000000000000000000000000000000000000...0100000000000000000000000000000000000000?verification=false",
                "GET /api/v1/repos/fixture/specimen/compare/0100000000000000000000000000000000000000...0300000000000000000000000000000000000000?files=false&verification=false",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "checks",
            known: false,
            read: Read::Checks { commit: HEAD },
            replies: &[(200, REPOSITORY), (200, EMPTY_STATUS)],
            expected: Ok(Answer::Checks(Box::new([]))),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/commits/0100000000000000000000000000000000000000/status?page=1&limit=2",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "job",
            known: false,
            read: Read::Job { part: api::LogPart::End, attempt: ATTEMPT, max_bytes: RAW_BYTES },
            replies: &[(200, REPOSITORY), (200, RUN), (200, JOB), (200, b"abc")],
            expected: Ok(Answer::Job {
                attempt: ATTEMPT,
                step: Some(api::Step { number: 1, name: Box::from(&b"x"[..]) }),
                log: Box::from(&b"abc"[..]),
                truncated: false,
            }),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/actions/runs/7",
                "GET /api/v1/repos/fixture/specimen/actions/jobs/10",
                "GET /api/v1/repos/fixture/specimen/actions/jobs/10/logs?attempt=3&step=1",
            ],
            range: Some(Range::Tail(RAW_BYTES)),
            kept: true,
            login: None,
        },
        ReadCase {
            name: "file",
            known: false,
            read: Read::File { head: HEAD, path: Box::from(&b"x"[..]), max_bytes: RAW_BYTES },
            replies: &[(200, REPOSITORY), (200, b"abc")],
            expected: Ok(Answer::File {
                head: HEAD,
                path: Box::from(&b"x"[..]),
                bytes: Box::from(&b"abc"[..]),
                truncated: false,
            }),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/raw/x?ref=0100000000000000000000000000000000000000",
            ],
            range: Some(Range::Head(RAW_BYTES + 1)),
            kept: true,
            login: None,
        },
    ]
}
fn repository_cases() -> [ReadCase; 7] {
    [
        ReadCase {
            name: "protection",
            known: false,
            read: Read::Protection { branch: Box::from(&b"work"[..]) },
            replies: &[(200, REPOSITORY), (200, BRANCH), (200, b"[]")],
            expected: Ok(Answer::Protection(None)),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/repos/fixture/specimen/branches/work",
                "GET /api/v1/repos/fixture/specimen/branch_protections",
            ],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "settings",
            known: false,
            read: Read::Settings,
            replies: &[(200, REPOSITORY)],
            expected: Ok(Answer::Settings(api::Settings {
                default_branch: Box::from(&b"main"[..]),
                merge: true,
                squash: false,
                rebase: false,
            })),
            sends: &["GET /api/v1/repositories/1"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "collaborators",
            known: false,
            read: Read::Collaborators { page: 1 },
            replies: &[(200, REPOSITORY), (200, b"[]")],
            expected: Ok(Answer::Collaborators { collaborators: Box::new([]), more: false }),
            sends: &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/collaborators?page=1&limit=2"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "permission",
            known: false,
            read: Read::Permission { user: 7 },
            replies: &[
                (200, REPOSITORY),
                (200, br#"{"ok":true,"data":[{"id":7,"login":"seven"}]}"#),
                (200, br#"{"permission":"write","user":{"id":7,"login":"seven"}}"#),
            ],
            expected: Ok(Answer::Permission(api::UserPermission {
                user: 7,
                login: Box::from(b"seven".as_slice()),
                permission: api::Permission::Write,
            })),
            sends: &[
                "GET /api/v1/repositories/1",
                "GET /api/v1/users/search?uid=7&limit=1",
                "GET /api/v1/repos/fixture/specimen/collaborators/seven/permission",
            ],
            range: None,
            kept: true,
            login: Some(b"seven"),
        },
        ReadCase {
            name: "branch fills settings",
            known: false,
            read: Read::Branch { branch: Box::from(*b"work") },
            replies: &[(200, REPOSITORY), (200, BRANCH)],
            expected: Ok(Answer::Commit(HEAD)),
            sends: &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/branches/work"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "missing pull for keeps the row",
            known: false,
            read: Read::PullFor { head: Box::from(*b"work"), base: Box::from(*b"main") },
            replies: &[(200, REPOSITORY), (404, b"{}")],
            expected: Err(Error::Missing),
            sends: &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/pulls/main/work"],
            range: None,
            kept: true,
            login: None,
        },
        ReadCase {
            name: "missing branch keeps the row",
            known: false,
            read: Read::Branch { branch: Box::from(&b"gone"[..]) },
            replies: &[(200, REPOSITORY), (404, b"{}")],
            expected: Err(Error::Missing),
            sends: &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/branches/gone"],
            range: None,
            kept: true,
            login: None,
        },
    ]
}
#[test]
fn read_calls_follow_canned_exchanges() {
    for case in read_cases().into_iter().flatten() {
        let mut run = if case.known { Run::known(Op::Read(case.read)) } else { Run::new(Op::Read(case.read)) };
        let mut step = run.begin();
        for (status, body) in case.replies {
            step = run.answer(*status, body);
        }
        assert_eq!(step, Step::Done(case.expected), "{}", case.name);
        run.assert_sent(case.sends);
        assert_eq!(run.last_range(), case.range, "{}", case.name);
        assert_eq!(
            run.tables().repository(REPO).cloned(),
            if case.kept { Some(fixtures::specimen()) } else { None },
            "{}",
            case.name
        );
        assert_eq!(run.tables().user(REPO.forge, 7), case.login, "{}", case.name);
        if case.kept {
            for _ in 0_u8..2 {
                let mut settings = super::super::plan::Plan::new(REPO, Op::Read(Read::Settings));
                assert_eq!(
                    settings.begin(BOUNDS, fixtures::WORD, run.tables()),
                    Ok(Step::Done(Ok(Answer::Settings(api::Settings {
                        default_branch: Box::from(*b"main"),
                        merge: true,
                        squash: false,
                        rebase: false,
                    })))),
                    "settings after {} sends nothing",
                    case.name,
                );
            }
        }
    }
}
#[test]
fn user_search_uses_id_and_one_result() {
    let mut run = Run::known(Op::Read(Read::Permission { user: 7 }));
    run.begin();
    run.assert_sent(&["GET /api/v1/users/search?uid=7&limit=1"]);
    assert_eq!(run.answer(200, br#"{"ok":true,"data":[]}"#), Step::Done(Err(Error::Missing)));
}
#[test]
fn page_size_is_passed_to_each_listing() {
    for (size, limit) in [(PAGE_SIZE, b"2".as_slice()), (ROWS, b"8".as_slice()), (ONE_ROW_PAGE, b"1".as_slice())] {
        for (read, before, target) in listings() {
            let mut run = Run::with(crate::forgejo::plans::plan::Bounds { page_size: size, ..BOUNDS }, Op::Read(read));
            run.begin();
            for body in before {
                run.answer(200, body);
            }
            run.assert_listing(target, limit);
        }
    }
}
type Listing = (Read, &'static [&'static [u8]], &'static str);
fn listings() -> [Listing; 7] {
    [
        (
            Read::Items { single: false, since: Time::ZERO, page: 1, kind: None },
            &[],
            "issues?state=all&since=1970-01-01T00%3A00%3A00Z&sort=leastupdate&page=1&limit=",
        ),
        (Read::Reviews { number: 1, page: 1, skip: 0 }, &[], "pulls/1/reviews?page=1&limit="),
        (
            Read::Statuses { commit: HEAD, page: 1 },
            &[],
            "commits/0100000000000000000000000000000000000000/status?page=1&limit=",
        ),
        (Read::Collaborators { page: 1 }, &[], "collaborators?page=1&limit="),
        (
            Read::PullFiles { number: 1, head: HEAD, page: 1, skip: 0, sides: true },
            &[PULL],
            "pulls/1/files?page=1&limit=",
        ),
        (
            Read::PullFor { head: Box::from(&b"work"[..]), base: Box::from(&b"main"[..]) },
            &[CLOSED_PULL],
            "pulls?state=open&head=work&base=main&sort=oldest&page=1&limit=",
        ),
        (Read::Checks { commit: HEAD }, &[], "commits/0100000000000000000000000000000000000000/status?page=1&limit="),
    ]
}
const CLOSED_PULL: &[u8] = br#"{"number":1,"state":"closed","head":{"repo_id":1,"ref":"work","sha":"0100000000000000000000000000000000000000"},"base":{"repo_id":1,"ref":"main","sha":"0300000000000000000000000000000000000000"},"merged":false,"merge_commit_sha":null,"merge_base":"0300000000000000000000000000000000000000","mergeable":true,"requested_reviewers":[]}"#;
#[test]
fn remarks_are_sliced_without_wire_paging() {
    let page_rows = usize::try_from(PAGE_SIZE).expect("fixture page fits usize");
    let one_row = usize::try_from(ONE_ROW_PAGE).expect("fixture page fits usize");
    for (size, page, skip, answer_bytes, expected, more, cut) in [
        (PAGE_SIZE, 1, 0, BOUNDS.answer_bytes, &REMARKS[..page_rows], true, false),
        (ROWS, 1, 0, BOUNDS.answer_bytes, &REMARKS[..], false, false),
        (ONE_ROW_PAGE, 1, 0, BOUNDS.answer_bytes, &REMARKS[..one_row], true, false),
        (PAGE_SIZE, 2, 0, BOUNDS.answer_bytes, &REMARKS[page_rows..], false, false),
        (ROWS, 1, 0, 66, &REMARKS[..one_row], true, true),
        (ROWS, 1, 1, 66, &REMARKS[one_row..page_rows], true, true),
        (ROWS, 1, 2, 66, &REMARKS[page_rows..], false, false),
    ] {
        let mut run = Run::with(
            crate::forgejo::plans::plan::Bounds { page_size: size, answer_bytes, ..BOUNDS },
            Op::Read(Read::Remarks { number: 1, review: 7, page, skip }),
        );
        run.begin();
        let result = run.answer(200, REMARKS_JSON);
        assert_eq!(
            result,
            Step::Done(Ok(Answer::Remarks { remarks: remarks(expected), more, cut })),
            "size {size}/page {page}/skip {skip}"
        );
        run.assert_sent(&["GET /api/v1/repos/fixture/specimen/pulls/1/reviews/7/comments"]);
    }
}
// A page one row below the protocol fixture exercises the smallest nonempty page.
const ONE_ROW_PAGE: u32 = PAGE_SIZE.saturating_sub(1);
// Three comments straddle a two-row page, with a one-row page as its lower bound.
const REMARKS: [u64; 3] = [1, 2, 3];
const REMARKS_JSON: &[u8] = br#"[{"id":1,"user":{"id":2,"login":"writer"},"path":"x","position":1,"body":"b"},{"id":2,"user":{"id":2,"login":"writer"},"path":"x","position":1,"body":"b"},{"id":3,"user":{"id":2,"login":"writer"},"path":"x","position":1,"body":"b"}]"#;
fn remark(id: u64) -> api::Remark {
    api::Remark { id, author: 2, path: Box::from(&b"x"[..]), line: 1, body: Box::from(&b"b"[..]) }
}
#[test]
fn read_plans_emit_escaped_targets_for_supplied_inputs() {
    skein_rows::check(
        [
            (
                "pull_listing_branches_have_golden_targets",
                (
                    Read::PullFor { head: Box::from(&b"feat/x y"[..]), base: Box::from(&b"main/z &"[..]) },
                    [CLOSED_PULL].as_slice(),
                    [
                        "GET /api/v1/repos/fixture/specimen/pulls/main%2Fz%20%26/feat%2Fx%20y",
                        "GET /api/v1/repos/fixture/specimen/pulls?state=open&head=feat%2Fx%20y&base=main%2Fz%20%26&sort=oldest&page=1&limit=2",
                    ].as_slice(),
                ),
            ),
            (
                "branch_slashes_have_golden_targets",
                (
                    Read::Branch { branch: Box::from(&b"a/b"[..]) },
                    [].as_slice(),
                    ["GET /api/v1/repos/fixture/specimen/branches/a%2Fb"].as_slice(),
                ),
            ),
            (
                "far_future_times_have_golden_targets",
                (
                    Read::Items { single: false, since: Time::from_nanos(u64::MAX), page: 1, kind: None },
                    [].as_slice(),
                    ["GET /api/v1/repos/fixture/specimen/issues?state=all&since=2554-07-21T23%3A34%3A33Z&sort=leastupdate&page=1&limit=2"].as_slice(),
                ),
            ),

            (
                "comments_escape_far_future_time",
                (
                    Read::Item { number: 1, after: 0, since: Time::from_nanos(u64::MAX) },
                    [ISSUE].as_slice(),
                    [
                        "GET /api/v1/repos/fixture/specimen/issues/1",
                        "GET /api/v1/repos/fixture/specimen/issues/1/comments?since=2554-07-21T23%3A34%3A33Z",
                    ].as_slice(),
                ),
            ),
        ],
        |(read, before, targets)| {
            let mut run = Run::known(Op::Read(read));
            run.begin();
            for body in before {
                run.answer(200, body);
            }
            run.assert_sent(targets);
        },
    );
}
#[test]
fn body_revisions_match_literal_big_endian_digest_goldens() {
    skein_rows::check(
        [
            (
                "long_body_revision_has_big_endian_golden",
                (crate::forgejo::document::Body::Long(5), Some(skein_lib::Wall::from_nanos(3)), 0x46c3_95ed_52b0_b56f),
            ),
            (
                "kept_body_revision_has_big_endian_golden",
                (crate::forgejo::document::Body::Kept(Box::from(&b"abc"[..])), None, 0xba78_16bf_8f01_cfea),
            ),
        ],
        |(body, timestamp, expected)| assert_eq!(super::super::reading::revision(&body, timestamp), expected),
    );
}

fn past_document_length(op: Op, body: &[u8]) -> (Run, u32) {
    let length = u32::try_from(body.len()).unwrap().checked_sub(1).unwrap();
    let mut limits = fixtures::limits();
    limits.json.length = length;
    let mut run = Run::with_limits(limits, BOUNDS, op);
    run.begin();
    (run, length)
}

#[test]
fn item_past_document_length_is_too_large() {
    let (mut run, length) =
        past_document_length(Op::Read(Read::Item { number: 1, after: 0, since: Time::ZERO }), ISSUE);
    assert_eq!(
        run.answer(200, ISSUE),
        Step::Done(Err(Error::TooLarge(api::Excess::at_least(
            api::Bound::Forge,
            u64::from(length),
            skein_lib::len_u64(ISSUE.len())
        ))))
    );
}

#[test]
fn open_pull_preflight_past_document_length_is_too_large() {
    let (mut run, length) = past_document_length(Op::Write(open_pull()), BRANCH);
    assert_eq!(
        run.answer(200, BRANCH),
        Step::Done(Err(Error::TooLarge(api::Excess::at_least(
            api::Bound::Forge,
            u64::from(length),
            skein_lib::len_u64(BRANCH.len()),
        ))))
    );
}

#[test]
fn write_past_document_length_stays_timeout() {
    let (mut run, _) = past_document_length(Op::Write(create_issue()), ISSUE);
    assert_eq!(run.answer(201, ISSUE), Step::Done(Err(Error::Timeout)));
}

#[test]
fn read_string_past_cap_is_unnamed_too_large() {
    let mut limits = fixtures::limits();
    limits.string_bytes = 0;
    let mut run = Run::with_limits(limits, BOUNDS, Op::Read(Read::Item { number: 1, after: 0, since: Time::ZERO }));
    run.begin();
    assert_eq!(run.answer(200, ISSUE), Step::Done(Err(Error::TooLarge(api::Excess::unnamed()))));
}

#[test]
fn malformed_json_stays_invalid_answer() {
    let mut run = Run::known(Op::Read(Read::Item { number: 1, after: 0, since: Time::ZERO }));
    run.begin();
    assert_eq!(run.answer(200, b"{"), Step::Done(Err(Error::InvalidAnswer)));
}

#[test]
fn comparison_past_document_length_is_too_large() {
    let (mut run, length) = past_document_length(Op::Read(Read::Compare { before: BASE, after: HEAD }), LARGE_COMPARE);
    assert_eq!(
        run.answer(200, LARGE_COMPARE),
        Step::Done(Err(Error::TooLarge(api::Excess {
            bound: api::Bound::Forge,
            limit: u64::from(length),
            size: skein_lib::len_u64(LARGE_COMPARE.len()),
            exact: false,
        })))
    );
}

const LARGE_COMPARE: &[u8] = br#"{"total_commits":22,"commits":[{"sha":"not read"}],"files":[{"filename":"a","status":null},{"filename":"b"},{"filename":"c"},{"filename":"d"},{"filename":"e"},{"filename":"f"},{"filename":"g"},{"filename":"h"},{"filename":"i"},{"filename":"j"},{"filename":"k"},{"filename":"l"}]}"#;
const REVERSE_COMPARE: &[u8] = br#"{"total_commits":3,"commits":null,"files":[]}"#;
// Reconstructed from walk-103's measured shape: two commits, three paths, six rows.
// The walk record does not contain the complete Forgejo response body.
const REPEATED_COMPARE: &[u8] = br#"{"total_commits":2,"commits":[{"sha":"0200000000000000000000000000000000000000"},{"sha":"0100000000000000000000000000000000000000"}],"files":[{"filename":"a","status":"added"},{"filename":"a","status":"modified"},{"filename":"b","status":"renamed","previous_filename":"old"},{"filename":"b","status":"modified"},{"filename":"c","status":"deleted"},{"filename":"c","status":"modified"}]}"#;
const COMPARE_SENDS: [&str; 2] = [
    "GET /api/v1/repos/fixture/specimen/compare/0300000000000000000000000000000000000000...0100000000000000000000000000000000000000?verification=false",
    "GET /api/v1/repos/fixture/specimen/compare/0100000000000000000000000000000000000000...0300000000000000000000000000000000000000?files=false&verification=false",
];
fn repeated_compare(bounds: crate::forgejo::plans::plan::Bounds) -> Step {
    let mut limits = fixtures::limits();
    limits.json.length = u32::try_from(REPEATED_COMPARE.len()).unwrap();
    let mut run = Run::with_limits(limits, bounds, Op::Read(Read::Compare { before: BASE, after: HEAD }));
    run.begin();
    run.answer(200, REPEATED_COMPARE);
    let terminal = run.answer(200, EMPTY_COMPARE);
    run.assert_sent(&COMPARE_SENDS);
    terminal
}
fn repeated_compare_answer(kept: usize) -> Step {
    let paths: &[&[u8]] = &[b"a", b"a", b"b", b"b", b"c", b"c"];
    let mut files = skein_lib::List::with_capacity(6);
    for path in paths.iter().take(kept) {
        files.push(Box::from(*path)).unwrap();
    }
    Step::Done(Ok(Answer::Compare {
        before: BASE,
        after: HEAD,
        ahead: 2,
        behind: 0,
        changed: 6,
        files: files.into_boxed(),
    }))
}
#[test]
fn repeated_comparison_paths_obey_file_and_answer_byte_bounds() {
    skein_rows::check(
        [
            ("all names", (BOUNDS, repeated_compare_answer(6))),
            (
                "exact bytes",
                (crate::forgejo::plans::plan::Bounds { answer_bytes: 102, ..BOUNDS }, repeated_compare_answer(6)),
            ),
            ("past rows", (crate::forgejo::plans::plan::Bounds { rows: 5, ..BOUNDS }, repeated_compare_answer(5))),
            (
                "past bytes",
                (crate::forgejo::plans::plan::Bounds { answer_bytes: 101, ..BOUNDS }, repeated_compare_answer(5)),
            ),
        ],
        |(bounds, expected)| assert_eq!(repeated_compare(bounds), expected),
    );
}

fn remarks(ids: &[u64]) -> Box<[api::Remark]> {
    let mut rows = skein_lib::List::with_capacity(ROWS);
    for id in ids {
        rows.push(remark(*id)).unwrap();
    }
    rows.into_boxed()
}

pub(super) mod write_cases;
use crate::api::Write;
use crate::forgejo::plans::{
    Body, Failure, Heard,
    failure::{self, Asked, Classified, Drop},
    tables::Tables,
    writes,
};
use write_cases::{REVIEWERS, begun, create_issue, open_pull};

#[test]
fn write_calls_follow_canned_exchanges() {
    for group in write_cases::cases() {
        for case in group {
            let mut run = Run::known(Op::Write(case.write));
            for (id, login) in case.users {
                run.tables().keep_user(REPO.forge, *id, Box::from(*login));
            }
            let mut step = run.begin();
            for (status, body) in case.replies {
                step = run.answer(*status, body);
            }
            assert_eq!(step, Step::Done(case.expected), "{}", case.name);
            run.assert_sent(case.sends);
            run.assert_bodies(case.bodies);
        }
    }
}

#[test]
fn refusals_match_status_and_whole_message() {
    for (asked, status, message, retry, error, drop) in write_cases::failures() {
        let refusal = crate::forgejo::failure::refusal(status, message);
        assert_eq!(
            failure::classify(asked, status, refusal, retry),
            Classified { error, drop },
            "{asked:?} {status} {message:?} {retry:?}"
        );
    }
    for status in [401, 403] {
        let mut run = begun(create_issue());
        assert_eq!(run.answer(status, b"{}"), Step::Done(Err(Error::Forbidden)));
        assert_eq!(run.tables().repository(REPO), None);
        let mut next = super::super::plan::Plan::new(REPO, Op::Read(Read::Settings));
        assert_eq!(
            next.begin(BOUNDS, fixtures::WORD, run.tables()),
            Ok(Step::send(crate::forgejo::request::Request::RepositoryById { id: 1 },))
        );
    }
    let mut run = begun(create_issue());
    let retry = skein_lib::Duration::from_secs(5);
    let step = run.answer_with(
        429,
        br#"{"message":"slow"}"#,
        crate::forgejo::plans::Fields { retry_after: Some(retry), ..fixtures::fields() },
    );
    assert_eq!(step, Step::Done(Err(Error::RateLimited { after: retry })));
}

#[test]
fn first_failure_ends_the_plan() {
    for (factory, replies, sends) in first_failures() {
        let mut run = factory(Op::Read(Read::Item { number: 1, after: 0, since: Time::ZERO }));
        run.begin();
        for (status, body) in replies {
            run.answer(*status, body);
        }
        assert_eq!(run.answer(500, br#"{"message":"failed"}"#), Step::Done(Err(Error::Timeout)));
        run.assert_sent(sends);
    }
}

#[test]
fn write_failures_classify_whether_the_exchange_was_seen() {
    skein_rows::check(
        [
            ("unseen_write_is_unavailable", (Failure::Unseen, Error::Unavailable)),
            ("seen_write_is_timeout", (Failure::Seen, Error::Timeout)),
        ],
        |(failure, error)| {
            let mut run = begun(create_issue());
            assert_eq!(run.fail(failure), Step::Done(Err(error)));
            run.assert_sent(&["POST /api/v1/repos/fixture/specimen/issues"]);
        },
    );
}

#[test]
fn open_pull_read_failure_is_unavailable() {
    let failures: [fn(&mut Run) -> Step; 2] = [seen, malformed];
    for (replies, sends) in write_cases::open_failures() {
        for fail in failures {
            let mut run = begun(open_pull());
            for (status, body) in replies {
                run.answer(*status, body);
            }
            assert_eq!(fail(&mut run), Step::Done(Err(Error::Unavailable)), "{replies:?}");
            run.assert_sent(sends);
        }
    }
}

#[test]
fn reviewers_are_added_before_removed() {
    let mut run = Run::known(Op::Write(Write::SetReviewers { number: 1, reviewers: Box::from([7]) }));
    run.tables().keep_user(REPO.forge, 7, Box::from(&b"seven"[..]));
    run.begin();
    run.answer(200, REVIEWERS);
    run.answer(201, b"");
    assert_eq!(run.answer(204, b""), Step::Done(Ok(Answer::Done)));
    run.assert_sent(&[
        "GET /api/v1/repos/fixture/specimen/pulls/1",
        "POST /api/v1/repos/fixture/specimen/pulls/1/requested_reviewers",
        "DELETE /api/v1/repos/fixture/specimen/pulls/1/requested_reviewers",
    ]);
    run.assert_bodies(&[None, Some(br#"{"reviewers":["seven"]}"#), Some(br#"{"reviewers":["eight"]}"#)]);
}

fn seen(run: &mut Run) -> Step {
    run.fail(Failure::Seen)
}
fn malformed(run: &mut Run) -> Step {
    run.answer(200, b"{")
}
type FirstFailure = (fn(Op) -> Run, &'static [(u16, &'static [u8])], &'static [&'static str]);
fn first_failures() -> [FirstFailure; 2] {
    [
        (Run::known, &[], &["GET /api/v1/repos/fixture/specimen/issues/1"]),
        (
            Run::new,
            &[(200, REPOSITORY)],
            &["GET /api/v1/repositories/1", "GET /api/v1/repos/fixture/specimen/issues/1"],
        ),
    ]
}

const DIRECTORY: &[u8] = br#"[{"name":"a","type":"file"},{"name":"b","type":"dir"},{"name":"c","type":"symlink"},{"name":"d","type":"submodule"},{"name":"ignored","type":"socket"}]"#;
// Four one-byte names, each charged the fixed 32-byte entry storage.
const DIRECTORY_BYTES: u32 = 132;
fn directory_entries() -> Box<[api::Entry]> {
    Box::new([
        api::Entry { name: Box::from(*b"a"), kind: api::EntryKind::File },
        api::Entry { name: Box::from(*b"b"), kind: api::EntryKind::Directory },
        api::Entry { name: Box::from(*b"c"), kind: api::EntryKind::File },
        api::Entry { name: Box::from(*b"d"), kind: api::EntryKind::Submodule },
    ])
}
#[test]
fn a_raw_miss_reads_the_contents_and_lists_a_directory() {
    for path in [b"src".as_slice(), b""] {
        let mut run =
            Run::known(Op::Read(Read::File { head: HEAD, path: Box::from(path), max_bytes: DIRECTORY_BYTES }));
        run.begin();
        if !path.is_empty() {
            run.answer(404, b"{}");
        }
        assert_eq!(
            run.answer(200, DIRECTORY),
            Step::Done(Ok(Answer::Directory {
                head: HEAD,
                path: Box::from(path),
                entries: directory_entries(),
                truncated: false
            }))
        );
    }
}
#[test]
fn a_raw_and_contents_miss_is_missing() {
    let mut run =
        Run::known(Op::Read(Read::File { head: HEAD, path: Box::from(*b"gone"), max_bytes: DIRECTORY_BYTES }));
    run.begin();
    run.answer(404, b"{}");
    assert_eq!(run.answer(404, b"{}"), Step::Done(Err(Error::Missing)));
}
#[test]
fn directory_entries_fill_the_byte_bound_or_are_cut() {
    for (max_bytes, kept, truncated) in [(DIRECTORY_BYTES, 4, false), (DIRECTORY_BYTES - 1, 3, true)] {
        for path in [b"src".as_slice(), b""] {
            let answer_bytes = max_bytes + u32::try_from(path.len()).unwrap();
            let mut run = Run::with(
                crate::forgejo::plans::plan::Bounds { answer_bytes, rows: 1, ..BOUNDS },
                Op::Read(Read::File { head: HEAD, path: Box::from(path), max_bytes: DIRECTORY_BYTES }),
            );
            run.begin();
            if !path.is_empty() {
                run.answer(404, b"{}");
            }
            let entries = Box::from(directory_entries().get(..kept).unwrap());
            assert_eq!(
                run.answer(200, DIRECTORY),
                Step::Done(Ok(Answer::Directory { head: HEAD, path: Box::from(path), entries, truncated }))
            );
        }
    }
}
mod turn;

mod artifact;
#[test]
fn write_errors_keep_the_exchange_state() {
    let limits = fixtures::limits();
    // Write, bodies heard, what Timeout becomes, what Forbidden becomes.
    for (write, bodies, expected, forbidden) in [
        (open_pull(), &[][..], Error::Unavailable, Error::Forbidden),
        (open_pull(), &[REPOSITORY][..], Error::Unavailable, Error::Forbidden),
        (open_pull(), &[REPOSITORY, BRANCH][..], Error::Unavailable, Error::Forbidden),
        (open_pull(), &[REPOSITORY, BRANCH, BRANCH][..], Error::Unavailable, Error::Forbidden),
        (
            Write::Merge { number: 1, head: HEAD, how: api::MergeHow::Style },
            &[REPOSITORY, b""][..],
            Error::Timeout,
            Error::Timeout,
        ),
    ] {
        let mut tables = Tables::new(ROWS, ROWS);
        let mut plan = writes::Plan::new(REPO, write.clone());
        let mut step = plan.begin(BOUNDS, fixtures::WORD, &mut tables).unwrap();
        write_hear(&mut plan, &mut tables, Heard::Head { status: 200, fields: fixtures::fields() }).unwrap();
        for body in bodies {
            let Step::Send(exchange) = step else { panic!("exchange expected") };
            let body = match crate::forgejo::client::decoder(&exchange.request, &limits) {
                Some(decoder) => Body::Document(fixtures::decode(decoder, body, &limits).document.unwrap()),
                None => Body::Empty,
            };
            step = write_hear(&mut plan, &mut tables, Heard::Done(body)).unwrap().unwrap();
        }
        assert_eq!(plan.error(Error::Timeout), expected, "{write:?} {}", bodies.len());
        assert_eq!(plan.error(Error::Forbidden), forbidden);
        assert_eq!(write_hear(&mut plan, &mut tables, Heard::Done(Body::Empty)), Err(expected));
    }
}
#[test]
fn beginning_a_write_touches_repository_recency() {
    let mut run = Run::known(Op::Write(create_issue()));
    for repository in 2..=ROWS {
        run.tables().keep_repository(api::Repository { repository, ..REPO }, fixtures::specimen());
    }
    run.begin();
    run.tables().keep_repository(api::Repository { repository: ROWS + 1, ..REPO }, fixtures::specimen());
    assert!(run.tables().repository(REPO).is_some());
    assert!(run.tables().repository(api::Repository { repository: 2, ..REPO }).is_none());
}
#[test]
fn mismatched_pull_ids_are_rejected_before_keeping_reviewers() {
    let mut run = Run::known(Op::Write(Write::SetReviewers { number: 1, reviewers: Box::from([8]) }));
    let mut row = fixtures::specimen();
    row.object_format = crate::forgejo::id::ObjectFormat::Sha256;
    run.tables().keep_repository(REPO, row);
    run.begin();
    assert_eq!(run.answer(200, REVIEWERS), Step::Done(Err(Error::Timeout)));
    assert_eq!(run.tables().user(REPO.forge, 8), None);
}
fn write_hear(plan: &mut writes::Plan, tables: &mut Tables, heard: Heard) -> Result<Option<Step>, Error> {
    plan.heard(heard, BOUNDS, fixtures::WORD, tables)
}

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "step tests edit recorded JSON as text"
)]
fn forgejo_pull_sizes_are_optional_lenient_and_follow_pull_for() {
    let json = core::str::from_utf8(PULL).unwrap();
    let baseline = pull();
    for field in ["changed_files", "additions", "deletions"] {
        for bad in [
            None,
            Some("null"),
            Some("-1"),
            Some("1.5"),
            Some("18446744073709551616"),
            Some("true"),
            Some("{}"),
            Some("\"bad\""),
        ] {
            let mut counts = String::new();
            for key in ["changed_files", "additions", "deletions"] {
                let value = if key == field { bad } else { Some("3") };
                if let Some(value) = value {
                    counts.push('"');
                    counts.push_str(key);
                    counts.push_str("\":");
                    counts.push_str(value);
                    counts.push(',');
                }
            }
            let body = format!("{{{counts}{}", json.strip_prefix('{').unwrap());
            let mut run =
                Run::with_limits(fixtures::profile_limits(), BOUNDS, Op::Read(Read::Pull { number: 1, ci: false }));
            run.begin();
            assert_eq!(
                run.answer(200, body.as_bytes()),
                Step::Done(Ok(Answer::Pull(baseline.clone()))),
                "{field}: {bad:?}"
            );
        }
    }
    let body = format!(
        "{{\"changed_files\":18446744073709551615,\"additions\":3,\"deletions\":0,{}",
        json.strip_prefix('{').unwrap()
    );
    let mut run = Run::with_limits(
        fixtures::profile_limits(),
        BOUNDS,
        Op::Read(Read::PullFor { head: Box::from(*b"work"), base: Box::from(*b"main") }),
    );
    run.begin();
    let mut expected = baseline;
    expected.size = Some(api::Size { files: u64::MAX, additions: 3, deletions: 0 });
    assert_eq!(run.answer(200, body.as_bytes()), Step::Done(Ok(Answer::Pull(expected))));
    let recorded = include_str!("../../../../../../tests/http/transcripts/forgejo-pull-request.http");
    let body = &recorded[recorded.find('{').unwrap()..];
    let mut limits = fixtures::profile_limits();
    limits.json.length = u32::try_from(body.len()).unwrap();
    let mut run = Run::with_limits(limits, BOUNDS, Op::Read(Read::Pull { number: 58, ci: false }));
    run.begin();
    let Step::Done(Ok(Answer::Pull(pull))) = run.answer(200, body.as_bytes()) else { panic!("recorded pull") };
    assert_eq!(pull.size, Some(api::Size { files: 19, additions: 2841, deletions: 12 }));
}
