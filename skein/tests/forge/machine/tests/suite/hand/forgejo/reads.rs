//! Document translations; the byte-level read-plan tests follow in 13.3b.
use skein_forge::api;
use skein_forge::forgejo::document as d;
use skein_forge::forgejo::plans::reading;
use skein_lib::bytes::concat;
use skein_lib::{Time, Wall};
use std::boxed::Box;

#[test]
fn revisions_and_provenance_follow_the_body_and_its_times() {
    let rows = [
        (
            "equal",
            kept(),
            Some(Wall::from_nanos(1)),
            Some(Wall::from_nanos(1)),
            api::Provenance::Original,
            0xba78_16bf_8f01_cfea,
        ),
        (
            "later",
            kept(),
            Some(Wall::from_nanos(1)),
            Some(Wall::from_nanos(3)),
            api::Provenance::Revised,
            0xba78_16bf_8f01_cfea,
        ),
        ("missing", kept(), Some(Wall::from_nanos(1)), None, api::Provenance::Unknown, 0xba78_16bf_8f01_cfea),
        (
            "long",
            d::Body::Long(5),
            Some(Wall::from_nanos(1)),
            Some(Wall::from_nanos(3)),
            api::Provenance::Unknown,
            0x46c3_95ed_52b0_b56f,
        ),
    ];
    for (name, body, created, updated, provenance, revision) in rows {
        assert_eq!(reading::provenance(created, updated, &body), provenance, "{name}: provenance");
        assert_eq!(reading::revision(&body, updated), revision, "{name}: revision");
    }
    revisions_from_bytes();
}
fn kept() -> d::Body {
    d::Body::Kept(Box::from(&b"abc"[..]))
}

#[test]
fn an_actions_status_reads_with_no_creator_and_an_absolute_link() {
    let status = d::CommitStatus {
        creator: None,
        updated_at: Wall::from_nanos(2),
        context: Box::from(&b"build"[..]),
        status: d::StatusState::Failure,
        description: Box::from(&b"failed"[..]),
        target_url: Box::from(&b"/fixture/specimen/actions/runs/1/jobs/0"[..]),
    };
    assert_eq!(
        reading::status(status, b"http://127.0.0.1:3000/fixture/specimen"),
        Ok(api::Status {
            at: Time::from_nanos(2),
            context: Box::from(&b"build"[..]),
            check: api::Check::Failed,
            description: Box::from(&b"failed"[..]),
            url: Box::from(&b"http://127.0.0.1:3000/fixture/specimen/actions/runs/1/jobs/0"[..]),
            job: None
        }),
        "Actions path has its origin, no author or job"
    );
    actions_from_bytes();
}

use api::{Answer, Op, Read};
use skein_forge_machine_world::hand as fixtures;
use skein_lib::Duration;
const ISSUE: &[u8] = br#"{"number":1,"pull_request":null,"state":"open","user":{"id":2,"login":"writer"},"labels":[],"updated_at":"2020-01-01T00:00:01+05:30","title":"t","body":"b"}"#;
const SHA: [u8; 32] = [0; 32];
fn answer(h: &mut fixtures::Harness, body: &[u8]) {
    h.answer(fixtures::FORGE, &fixtures::ok(body));
    h.run(Duration::ZERO);
}
fn last(h: &fixtures::Harness, expected: &[u8]) {
    assert_eq!(h.request_lines(fixtures::FORGE).last(), Some(&Box::from(expected)), "request target");
}
fn summary() -> api::Summary {
    api::Summary {
        labels: Box::from([]),
        number: 1,
        kind: api::Kind::Issue,
        state: api::State::Open,
        author: 2,
        key: None,
        title: Box::from(&b"t"[..]),
        body: Box::from(&b"b"[..]),
        updated: Time::from_nanos(1_577_817_001_000_000_000),
    }
}
fn listed(body: &[u8]) -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(body.len().saturating_add(2));
    out.put(b"[").expect("list start");
    out.put(body).expect("row");
    out.put(b"]").expect("list end");
    out.finish()
}
#[test]
fn items_list_since_with_leastupdate_and_an_explicit_page() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Items { single: false, since: Time::ZERO, page: 1, kind: None }));
    answer(&mut h, &listed(ISSUE));
    last(&h, b"GET /api/v1/repos/fixture/specimen/issues?state=all&since=1970-01-01T00%3A00%3A00Z&sort=leastupdate&page=1&limit=2 HTTP/1.1");
    assert_eq!(
        h.answered(1),
        Some(&skein_forge_connection::Answered {
            call: skein_lib::Token::new(1),
            cost: 1,
            free: 0,
            rate: None,
            result: Ok(Answer::Items {
                cut: false,
                items: Box::new([summary()]),
                more: false,
                now: Time::from_nanos(1_577_836_800_000_000_000)
            })
        })
    );
}
#[test]
fn an_item_keeps_comments_above_after_until_the_answer_is_full() {
    let mut shared = fixtures::shared();
    shared.rows = 2;
    let mut h = fixtures::Harness::with(fixtures::limits(), shared, fixtures::settings());
    h.connected(fixtures::FORGE);
    answer(&mut h, br#"{"id":2,"login":"writer"}"#);
    answer(&mut h, br#"{"max_response_items":2,"default_paging_num":30}"#);
    h.known(fixtures::REPO, fixtures::specimen());
    h.call(1, fixtures::REPO, Op::Read(Read::Item { number: 1, after: 1, since: Time::ZERO }));
    answer(&mut h, ISSUE);
    answer(&mut h, br#"[{"id":1,"user":{"id":2,"login":"writer"},"body":"skip"},{"id":2,"user":{"id":2,"login":"writer"},"body":"abc"},{"id":3,"user":{"id":2,"login":"writer"},"body":"abc"},{"id":4,"user":{"id":2,"login":"writer"},"body":"abc"}]"#);
    last(&h, b"GET /api/v1/repos/fixture/specimen/issues/1/comments HTTP/1.1");
    assert_eq!(h.answered(1).expect("item").cost, 2);
    assert_eq!(
        h.answered(1).expect("item").result,
        Ok(Answer::Item { item: summary(), comments: Box::new([comment(2), comment(3)]), more: true })
    );
}
fn comment(id: u64) -> api::Comment {
    api::Comment {
        id,
        provenance: api::Provenance::Unknown,
        author: 2,
        created: Time::ZERO,
        revision: 0xba78_16bf_8f01_cfea,
        key: None,
        body: Box::from(&b"abc"[..]),
    }
}
#[test]
fn a_call_without_a_kept_path_resolves_by_id_and_counts_it() {
    let mut h = fixtures::Harness::ready();
    h.call(1, fixtures::REPO, Op::Read(Read::Items { single: false, since: Time::ZERO, page: 1, kind: None }));
    answer(&mut h, br#"{"id":1,"owner":{"id":2,"login":"fixture"},"name":"specimen","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":false,"allow_rebase":false,"html_url":"http://127.0.0.1:3000/fixture/specimen"}"#);
    answer(&mut h, b"[]");
    let lines = h.request_lines(fixtures::FORGE);
    assert_eq!(lines.get(2), Some(&Box::from(&b"GET /api/v1/repositories/1 HTTP/1.1"[..])), "resolution");
    assert_eq!(h.answered(1).expect("resolved answer").cost, 2);
    assert_eq!(
        h.answered(1).expect("resolved answer").result,
        Ok(Answer::Items {
            cut: false,
            items: Box::from([]),
            more: false,
            now: Time::from_nanos(1_577_836_800_000_000_000)
        })
    );
}
#[test]
fn a_pending_review_is_skipped_but_fills_the_page() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Reviews { number: 1, page: 1, skip: 0 }));
    answer(&mut h, br#"[{"id":1,"user":{"id":2,"login":"writer"},"state":"PENDING","commit_id":"0000000000000000000000000000000000000000","body":"","official":false,"dismissed":false},{"id":2,"user":{"id":2,"login":"writer"},"state":"REQUEST_REVIEW","commit_id":"0000000000000000000000000000000000000000","body":"","official":false,"dismissed":false}]"#);
    assert_eq!(
        h.answered(1).expect("reviews").result,
        Ok(Answer::Reviews { reviews: Box::from([]), more: true, cut: false })
    );
    last(&h, b"GET /api/v1/repos/fixture/specimen/pulls/1/reviews?page=1&limit=2 HTTP/1.1");
}
#[test]
fn short_status_pages_end_regardless_of_total_count() {
    let mut h = fixtures::known();
    for (name, page, total, expected_ci, expected) in [
        ("first", 1, br#"{"state":"pending","total_count":3,"statuses":[]}"#.as_slice(), api::Ci::Pending, false),
        ("later", 3, br#"{"state":"success","total_count":3,"statuses":[]}"#.as_slice(), api::Ci::Passed, false),
    ] {
        h.call(u64::from(page), fixtures::REPO, Op::Read(Read::Statuses { commit: SHA, page }));
        answer(&mut h, total);
        assert_eq!(
            h.answered(u64::from(page)).expect("statuses").result,
            Ok(Answer::Statuses { ci: expected_ci, statuses: Box::from([]), more: expected }),
            "{name}"
        );
    }
}
#[test]
fn a_malformed_read_answer_is_invalid_and_closes_its_connection() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Branch { branch: Box::from(&b"main"[..]) }));
    answer(&mut h, b"{}");
    assert_eq!(h.answered(1).expect("malformed answer").result, Err(api::Error::InvalidAnswer));
    assert_eq!(h.answered(1).expect("malformed answer").cost, 1);
    assert!(h.peers.is_empty(), "untrusted stream closed");
}
fn pull(number: u64, state: &[u8], head: &[u8]) -> Box<[u8]> {
    concat(&[b"{\"number\":", skein_lib::Decimal::of(number).as_bytes(), b",\"state\":\"", state, b"\",\"head\":{\"repo_id\":1,\"ref\":\"", head, b"\",\"sha\":\"0000000000000000000000000000000000000000\"},\"base\":{\"repo_id\":1,\"ref\":\"main\",\"sha\":\"\"},\"merged\":false,\"merge_commit_sha\":null,\"merge_base\":null,\"mergeable\":true,\"requested_reviewers\":[]}"])
}

fn expected_pull(number: u64, state: api::State) -> api::Pull {
    api::Pull {
        queued: None,
        dequeued: None,
        reviewers: Box::from([]),
        number,
        state,
        head: Box::from(&b"work"[..]),
        base: Box::from(&b"main"[..]),
        commit: SHA,
        base_commit: None,
        merged: None,
        mergeable: true,
        ci: api::Ci::None,
    }
}
#[test]
fn pull_for_lists_open_pulls_when_the_lookup_names_a_closed_one() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::PullFor { head: Box::from(&b"work"[..]), base: Box::from(&b"main"[..]) }));
    answer(&mut h, &pull(12, b"closed", b"work"));
    last(
        &h,
        b"GET /api/v1/repos/fixture/specimen/pulls?state=open&head=work&base=main&sort=oldest&page=1&limit=2 HTTP/1.1",
    );
    answer(&mut h, &concat(&[b"[", &pull(9, b"open", b"other"), b",", &pull(13, b"open", b"work"), b"]"]));
    assert_eq!(h.answered(1).expect("open pull").result, Ok(Answer::Pull(expected_pull(13, api::State::Open))));
    assert_eq!(h.answered(1).expect("open pull").cost, 2);
}
#[test]
fn pull_for_answers_the_lookups_closed_pull_when_none_is_open() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::PullFor { head: Box::from(&b"work"[..]), base: Box::from(&b"main"[..]) }));
    answer(&mut h, &pull(12, b"closed", b"work"));
    answer(&mut h, &listed(&pull(9, b"open", b"other")));
    assert_eq!(h.answered(1).expect("closed pull").result, Ok(Answer::Pull(expected_pull(12, api::State::Closed))));
    assert_eq!(h.answered(1).expect("closed pull").cost, 2);
}

fn revisions_from_bytes() {
    let mut h = fixtures::known();
    let long = [b'x'; 1046];
    let rows = [
        (
            "equal",
            1,
            b"abc".as_slice(),
            b"\"1970-01-01T00:00:01Z\"".as_slice(),
            api::Provenance::Original,
            0xba78_16bf_8f01_cfea,
            b"abc".as_slice(),
        ),
        (
            "later",
            2,
            b"abc".as_slice(),
            b"\"1970-01-01T00:00:03Z\"".as_slice(),
            api::Provenance::Revised,
            0xba78_16bf_8f01_cfea,
            b"abc".as_slice(),
        ),
        (
            "missing",
            3,
            b"abc".as_slice(),
            b"null".as_slice(),
            api::Provenance::Unknown,
            0xba78_16bf_8f01_cfea,
            b"abc".as_slice(),
        ),
        (
            "long",
            4,
            long.as_slice(),
            b"\"1970-01-01T00:00:03Z\"".as_slice(),
            api::Provenance::Unknown,
            0x6dec_dbb8_004b_59a2,
            b"".as_slice(),
        ),
    ];
    for (name, call, body, updated, provenance, revision, expected_body) in rows {
        h.call(call, fixtures::REPO, Op::Read(Read::Item { number: 1, after: 0, since: Time::ZERO }));
        answer(&mut h, ISSUE);
        let comments = concat(&[
            br#"[{"id":1,"user":{"id":2,"login":"writer"},"created_at":"1970-01-01T00:00:01Z","updated_at":"#,
            updated,
            br#", "body":""#,
            body,
            br#""}]"#,
        ]);
        answer(&mut h, &comments);
        assert_eq!(
            h.answered(call).expect("comment revision").result,
            Ok(Answer::Item {
                item: summary(),
                comments: Box::new([api::Comment {
                    id: 1,
                    author: 2,
                    created: Time::from_nanos(1_000_000_000),
                    revision,
                    provenance,
                    key: None,
                    body: Box::from(expected_body)
                }]),
                more: false
            }),
            "{name}"
        );
    }
}
fn actions_from_bytes() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Statuses { commit: SHA, page: 1 }));
    answer(&mut h, br#"{"state":"failure","total_count":1,"statuses":[{"creator":null,"updated_at":"1970-01-01T00:00:02Z","context":"build","status":"failure","description":"failed","target_url":"/fixture/specimen/actions/runs/1/jobs/0"}]}"#);
    assert!(h.answered(1).is_none(), "Actions status waits for its run lookup");
    last(
        &h,
        b"GET /api/v1/repos/fixture/specimen/actions/runs?head_sha=0000000000000000000000000000000000000000&run_number=1&page=1&limit=1 HTTP/1.1",
    );
    answer(&mut h, br#"{"total_count":0,"workflow_runs":[]}"#);
    assert_eq!(
        h.answered(1).expect("Actions status").result,
        Ok(Answer::Statuses { ci: api::Ci::Failed, more: false, statuses: Box::new([actions_status()]) })
    );
    assert_eq!(h.answered(1).expect("Actions status").cost, 2);
}
fn actions_status() -> api::Status {
    api::Status {
        at: Time::from_nanos(2_000_000_000),
        context: Box::from(&b"build"[..]),
        check: api::Check::Failed,
        description: Box::from(&b"failed"[..]),
        url: Box::from(&b"http://127.0.0.1:3000/fixture/specimen/actions/runs/1/jobs/0"[..]),
        job: None,
    }
}
const REPOSITORY: &[u8] = br#"{"id":1,"owner":{"id":2,"login":"fixture"},"name":"specimen","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":false,"allow_rebase":false,"html_url":"http://127.0.0.1:3000/fixture/specimen"}"#;
const PERMISSION: &[u8] = br#"{"permission":"owner","user":{"id":2,"login":"writer"}}"#;
fn branch(name: &[u8], rule: &[u8]) -> Box<[u8]> {
    concat(&[
        br#"{"name":""#,
        name,
        br#"","commit":{"id":"0000000000000000000000000000000000000000"},"effective_branch_protection_name":""#,
        rule,
        br#""}"#,
    ])
}
#[test]
fn branches_round_trip_the_prefix_in_one_refs_request() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Branches { prefix: Box::from(&b"temper/"[..]) }));
    h.run(Duration::ZERO);
    last(&h, b"GET /api/v1/repos/fixture/specimen/git/refs/heads/temper%2F HTTP/1.1");
    answer(&mut h, br#"[{"ref":"refs/heads/temper/one"},{"ref":"refs/heads/temper/two"}]"#);
    assert_eq!(
        h.answered(1).expect("branch names").result,
        Ok(Answer::Branches(Box::new([Box::from(&b"temper/one"[..]), Box::from(&b"temper/two"[..])])))
    );
    assert_eq!(h.answered(1).expect("branch names").cost, 1);
}

#[test]
fn a_missing_branch_prefix_reads_its_404_body_and_keeps_the_connection() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Branches { prefix: Box::from(&b"temper/"[..]) }));
    h.run(Duration::ZERO);
    last(&h, b"GET /api/v1/repos/fixture/specimen/git/refs/heads/temper%2F HTTP/1.1");
    h.answer(fixtures::FORGE, b"HTTP/1.1 404 Not Found\r\nContent-Length: 23\r\n\r\n");
    h.run(Duration::ZERO);
    assert!(h.answered(1).is_none(), "the 404 body must be read before answering");
    h.answer(fixtures::FORGE, br#"{"message":"Not Found"}"#);
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("missing prefix").result, Ok(Answer::Branches(Box::new([]))));
    assert_eq!(h.answered(1).expect("missing prefix").cost, 1);

    let opened = h.connections_opened();
    h.call(2, fixtures::REPO, Op::Read(Read::Branch { branch: Box::from(&b"main"[..]) }));
    h.run(Duration::ZERO);
    last(&h, b"GET /api/v1/repos/fixture/specimen/branches/main HTTP/1.1");
    answer(&mut h, &branch(b"main", b""));
    assert_eq!(h.answered(2).expect("main branch").result, Ok(Answer::Commit(SHA)));
    assert_eq!(h.connections_opened(), opened, "the next call reuses the connection");
}

#[test]
fn protection_checks_admin_access_even_when_no_rule_applies() {
    let mut h = fixtures::known();
    for (name, call, rule, response, expected) in [
        ("no rule", 1, b"".as_slice(), b"[]".as_slice(), Ok(Answer::Protection(None))),
        ("effective rule", 2, b"main".as_slice(), br#"{"rule_name":"main","enable_status_check":true,"status_check_contexts":["build"],"required_approvals":2,"dismiss_stale_approvals":true,"require_signed_commits":false}"#.as_slice(), Ok(Answer::Protection(Some(api::Protection { queue: false, signatures: false, linear: false, code_owners: false, bypass: api::Bypass::Never, branch: Box::from(&b"main"[..]), contexts: Box::new([Box::from(&b"build"[..])]), approvals: 2, dismiss_stale: true })))),
    ] {
        h.call(call, fixtures::REPO, Op::Read(Read::Protection { branch: Box::from(&b"main"[..]) })); answer(&mut h, &branch(b"main", rule)); answer(&mut h, response);
        assert_eq!(h.answered(call).expect("protection").result, expected, "{name}");
        assert_eq!(h.answered(call).expect("protection").cost, 2, "{name}: cost");
    }
    h.call(3, fixtures::REPO, Op::Read(Read::Protection { branch: Box::from(&b"main"[..]) }));
    answer(&mut h, &branch(b"main", b""));
    h.answer(fixtures::FORGE, &fixtures::status(403));
    h.run(Duration::ZERO);
    assert_eq!(h.answered(3).expect("nonadmin absent rule").result, Err(api::Error::Forbidden));
}
#[test]
fn a_permission_resolves_its_user_and_reuses_the_kept_login() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Permission { user: 2 }));
    h.run(Duration::ZERO);
    last(&h, b"GET /api/v1/users/search?uid=2&limit=1 HTTP/1.1");
    answer(&mut h, br#"{"ok":true,"data":[{"id":2,"login":"writer"}]}"#);
    last(&h, b"GET /api/v1/repos/fixture/specimen/collaborators/writer/permission HTTP/1.1");
    answer(&mut h, PERMISSION);
    assert_eq!(
        h.answered(1).expect("permission").result,
        Ok(Answer::Permission(api::UserPermission {
            user: 2,
            login: Box::from(b"writer".as_slice()),
            permission: api::Permission::Admin
        }))
    );
    assert_eq!(h.answered(1).expect("permission").cost, 2);
    h.call(2, fixtures::REPO, Op::Read(Read::Permission { user: 2 }));
    answer(&mut h, PERMISSION);
    assert_eq!(h.answered(2).expect("cached login").cost, 1);
}
#[test]
fn collaborators_read_each_listed_users_permission() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Collaborators { page: 1 }));
    answer(&mut h, br#"[{"id":2,"login":"writer"}]"#);
    answer(&mut h, PERMISSION);
    assert_eq!(
        h.answered(1).expect("collaborators").result,
        Ok(Answer::Collaborators {
            collaborators: Box::new([api::Collaborator { user: 2, permission: api::Permission::Admin }]),
            more: false
        })
    );
    assert_eq!(h.answered(1).expect("collaborators").cost, 2);
}
#[test]
fn locate_keeps_the_repository() {
    let mut h = fixtures::Harness::ready();
    h.call(
        1,
        api::Repository { forge: fixtures::FORGE, repository: 0 },
        Op::Read(Read::Locate { owner: Box::from(&b"fixture"[..]), name: Box::from(&b"specimen"[..]) }),
    );
    answer(&mut h, REPOSITORY);
    assert_eq!(h.answered(1).expect("located").result, Ok(Answer::Located { repository: 1 }));
    assert_eq!(h.answered(1).expect("located").cost, 1);
    assert!(h.component.forgejo_tables().repository(fixtures::REPO).is_some(), "located row kept");
}
#[test]
fn settings_from_the_kept_repository_costs_nothing() {
    let mut h = fixtures::known();
    h.call(2, fixtures::REPO, Op::Read(Read::Settings));
    h.run(skein_lib::Duration::ZERO);
    assert_eq!(
        h.answered(2).expect("settings").result,
        Ok(Answer::Settings(api::Settings {
            default_branch: Box::from(&b"main"[..]),
            merge: true,
            squash: false,
            rebase: false
        }))
    );
    assert_eq!(h.answered(2).expect("settings").cost, 0);
    assert_eq!(
        h.component.forgejo_tables().repository(fixtures::REPO).expect("kept row").merge_style,
        d::MergeStyle::Squash
    );
}
#[test]
fn remarks_select_the_requested_slice_and_cut_after_it() {
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, Op::Read(Read::Remarks { number: 1, review: 2, page: 2, skip: 0 }));
    let row = br#"{"id":3,"user":{"id":2,"login":"writer"},"path":"file","position":4,"body":"abc"}"#;
    answer(&mut h, &concat(&[b"[", row, b",", row, b",", row, b",", row, b",", row, b"]"]));
    let remark = api::Remark { id: 3, author: 2, path: Box::from(&b"file"[..]), line: 4, body: Box::from(&b"abc"[..]) };
    assert_eq!(
        h.answered(1).expect("remarks").result,
        Ok(Answer::Remarks { remarks: Box::new([remark.clone(), remark]), more: true, cut: false })
    );
}
