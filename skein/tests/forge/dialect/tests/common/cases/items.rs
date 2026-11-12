#![expect(
    clippy::must_use_candidate,
    reason = "fixture-returning helpers are only consumed by assertions or other fixture construction"
)]
use super::super::forgejo;
use skein_forge::forgejo::{
    document::{
        Body, CombinedStatus, Comment, CommitStatus, Document, Issue, IssueState, PullBranch, PullRequest, Row,
        StatusState,
    },
    request::{IssueKind, Request, Sort, StateFilter},
    server::Answer,
};
use skein_lib::Wall;
pub fn issue(number: u64, pull: bool, title: &[u8], body: &[u8], author: u64, at: u64) -> Issue {
    Issue {
        number,
        pull,
        state: IssueState::Open,
        user: forgejo::user(author),
        labels: Box::from([]),
        updated_at: Wall::from_nanos(at),
        title: Box::from(title),
        body: Body::Kept(Box::from(body)),
    }
}
pub fn edited() -> Issue {
    issue(3, false, b"edited title", b"<!-- kit:key 01000001 -->\nissue text", 2, 1_577_817_005_000_000_000)
}
pub fn feature() -> Issue {
    issue(1, true, b"feature/slash", b"fixture pull", 1, 1_577_817_009_000_000_000)
}
pub fn conflict() -> Issue {
    issue(2, true, b"conflict", b"fixture pull", 1, 1_577_817_007_000_000_000)
}
pub fn writer() -> Issue {
    issue(4, true, b"writer pull", b"writer text", 2, 1_577_817_008_000_000_000)
}
pub fn listing(page: u32, kind: Option<IssueKind>, since: Wall) -> Request {
    Request::Issues {
        at: forgejo::specimen(),
        state: Some(StateFilter::All),
        kind,
        since: Some(since),
        sort: Some(Sort::LeastUpdate),
        page: forgejo::page(page, 2),
    }
}
pub fn pull(merged: bool) -> PullRequest {
    if merged {
        PullRequest {
            number: 5,
            state: IssueState::Closed,
            head: PullBranch { repo_id: 1, name: Box::from(*b"mergeable"), sha: Some(forgejo::sha(5)) },
            base: PullBranch { repo_id: 1, name: Box::from(*b"landing"), sha: Some(forgejo::sha(7)) },
            merged: true,
            merge_commit_sha: Some(forgejo::sha(7)),
            merge_base: Some(forgejo::sha(1)),
            mergeable: true,
            requested_reviewers: Box::from([]),
            changed_files: Some(1),
            additions: Some(1),
            deletions: Some(0),
        }
    } else {
        PullRequest {
            number: 1,
            state: IssueState::Open,
            head: PullBranch { repo_id: 1, name: Box::from(*b"feature/slash"), sha: Some(forgejo::sha(4)) },
            base: PullBranch { repo_id: 1, name: Box::from(*b"main"), sha: Some(forgejo::sha(3)) },
            merged: false,
            merge_commit_sha: None,
            merge_base: Some(forgejo::sha(6)),
            mergeable: true,
            requested_reviewers: Box::from([forgejo::user(2)]),
            changed_files: Some(3),
            additions: Some(3),
            deletions: Some(0),
        }
    }
}
pub fn cases() -> Vec<(&'static str, Request, Answer)> {
    vec![
        ("issues-page-1", listing(1, None, Wall::EPOCH), Answer::Issues(Box::from([conflict(), edited()]))),
        ("issues-page-2", listing(2, None, Wall::EPOCH), Answer::Issues(Box::from([writer(), feature()]))),
        ("issues-type-issues", listing(1, Some(IssueKind::Issues), Wall::EPOCH), Answer::Issues(Box::from([edited()]))),
        (
            "issues-type-pulls",
            listing(1, Some(IssueKind::Pulls), Wall::EPOCH),
            Answer::Issues(Box::from([conflict(), writer()])),
        ),
        (
            "issues-since-2554",
            listing(1, None, Wall::from_nanos(18_446_744_073_000_000_000)),
            Answer::Issues(Box::from([])),
        ),
        ("issue", Request::Issue { at: forgejo::specimen(), number: 3 }, Answer::Issue(edited())),
        ("issue-of-pull", Request::Issue { at: forgejo::specimen(), number: 1 }, Answer::Issue(feature())),
        (
            "issue-comments",
            Request::Comments { at: forgejo::specimen(), number: 3, since: None },
            Answer::Comments(Box::from([
                Comment {
                    id: 4,
                    user: forgejo::user(2),
                    created_at: Some(Wall::from_nanos(1_577_817_004_000_000_000)),
                    updated_at: Some(Wall::from_nanos(1_577_817_004_000_000_000)),
                    body: Body::Kept(Box::from(*b"<!-- kit:key 01000002 -->\ncomment text")),
                },
                Comment {
                    id: 5,
                    user: forgejo::user(2),
                    created_at: Some(Wall::from_nanos(1_577_817_004_000_000_000)),
                    updated_at: Some(Wall::from_nanos(1_577_817_004_000_000_000)),
                    body: Body::Kept(Box::from(*b"plain comment")),
                },
            ])),
        ),
        ("pull", Request::Pull { at: forgejo::specimen(), number: 1 }, Answer::Pull(pull(false))),
        ("pull-merged", Request::Pull { at: forgejo::specimen(), number: 5 }, Answer::Pull(pull(true))),
        (
            "pull-for-encoded-head",
            Request::PullFor { at: forgejo::specimen(), base: Box::from(*b"main"), head: Box::from(*b"feature/slash") },
            Answer::Pull(pull(false)),
        ),
        (
            "combined-status",
            Request::CombinedStatus { at: forgejo::specimen(), commit: forgejo::sha(4), page: forgejo::page(1, 1) },
            Answer::CombinedStatus {
                state: CombinedStatus { state: Some(StatusState::Pending), total_count: 1 },
                sha: Some(forgejo::sha(4)),
                statuses: Box::from([CommitStatus {
                    creator: Some(forgejo::user(2)),
                    updated_at: Wall::from_nanos(1_577_817_005_000_000_000),
                    context: Box::from(*b"fixture/other"),
                    status: StatusState::Pending,
                    description: Box::from([]),
                    target_url: Box::from([]),
                }]),
            },
        ),
        (
            "combined-status-empty",
            Request::CombinedStatus { at: forgejo::specimen(), commit: forgejo::sha(6), page: forgejo::page(1, 1) },
            Answer::CombinedStatus {
                state: CombinedStatus { state: None, total_count: 0 },
                sha: None,
                statuses: Box::from([]),
            },
        ),
    ]
}
// Synthetic request rows supplement the recorded exchanges without changing them.
pub fn comment_requests() -> [(Request, &'static [u8]); 1] {
    [(
        Request::Comments {
            at: forgejo::specimen(),
            number: 3,
            since: Some(Wall::from_nanos(1_577_817_004_000_000_000)),
        },
        b"/api/v1/repos/fixture/specimen/issues/3/comments?since=2019-12-31T18%3A30%3A04Z",
    )]
}
pub fn expected(answer: Answer) -> (Document, Vec<Row>) {
    match answer {
        Answer::Issues(values) => {
            (Document::Listed { skipped: 0 }, values.into_vec().into_iter().map(Row::Issue).collect())
        }
        Answer::Comments(values) => {
            (Document::Listed { skipped: 0 }, values.into_vec().into_iter().map(Row::Comment).collect())
        }
        Answer::Issue(value) => (Document::Issue(value), vec![]),
        Answer::Pull(value) => (Document::Pull(value), vec![]),
        Answer::CombinedStatus { state, statuses, .. } => {
            (Document::CombinedStatus(state), statuses.into_vec().into_iter().map(Row::Status).collect())
        }
        Answer::Org(_)
        | Answer::CreatedRepository(_)
        | Answer::CreatedFile { .. }
        | Answer::Team(_)
        | Answer::Teams(_)
        | Answer::Hook(_)
        | Answer::Hooks(_)
        | Answer::Members(_)
        | Answer::Error(_)
        | Answer::Comment(_)
        | Answer::Status(_)
        | Answer::Review(_)
        | Answer::Entries(_)
        | Answer::Files(_)
        | Answer::Compare { .. }
        | Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Job { .. }
        | Answer::Jobs(_)
        | Answer::Reviews(_)
        | Answer::ReviewComments(_)
        | Answer::Branch(_)
        | Answer::BranchRefs(_)
        | Answer::Branches(_)
        | Answer::Protection(_)
        | Answer::Protections(_)
        | Answer::Collaborators(_)
        | Answer::Tree { .. }
        | Answer::Artifacts(_)
        | Answer::Commits { .. }
        | Answer::Permission(_)
        | Answer::Pulls(_)
        | Answer::Token(_)
        | Answer::Tokens(_)
        | Answer::User(_)
        | Answer::ApiSettings(_)
        | Answer::Repository(_)
        | Answer::Reachable(_)
        | Answer::RepoSearch { .. }
        | Answer::Search { .. } => {
            panic!("an item answer")
        }
    }
}
