//! Relay route guards, request accounting and scripted connections.
use temper_box_kit::{
    relay::{CAP, Counter, tests},
    relay_fixture::{guard, push},
};
#[test]
fn framing_preserves_requests_and_keeps_the_client_alive() {
    tests::framing_preserves_requests_and_keeps_the_client_alive();
}

#[test]
fn bodyless_answers_and_minted_tokens_stay_in_their_lanes() {
    tests::bodyless_answers_and_minted_tokens_stay_in_their_lanes();
}

#[test]
fn requested_close_and_peer_eof_end_a_connection() {
    tests::requested_close_and_peer_eof_end_a_connection();
}

/// Listener, method, target, body (`None`: encoded), allowed.
type Row<'a> = (&'a str, &'a str, &'a str, Option<&'a [u8]>, bool);

fn check(rows: Vec<Row<'_>>) {
    for (listener, method, path, body, allowed) in rows {
        assert_eq!(guard().allows(listener, method, path, body, Some(123)), allowed, "{listener} {method} {path}");
    }
}

#[test]
fn guard_bounds_every_route_to_the_probe() {
    let none: &[u8] = b"";
    let status = format!("/repos/owner/probe/statuses/{}", "a".repeat(40));
    // GitHub's compare names its sides with three dots, one path segment (skein Request::Compare).
    let compare = format!("/repos/owner/probe/compare/{}...{}", "a".repeat(40), "b".repeat(40));
    check(vec![
        ("api", "GET", "/app", Some(none), true),
        ("api", "GET", "/app/hook/config", Some(none), true),
        ("api", "GET", "/app/hook/deliveries?per_page=10", Some(none), true),
        ("api", "GET", "/app/hook/deliveries/12", Some(none), true),
        ("api", "GET", "/app/installations", Some(none), true),
        ("api", "GET", "/app/installations/18", Some(none), true),
        ("api", "POST", "/app/installations/17/access_tokens", Some(none), true),
        ("api", "POST", "/app/installations/18/access_tokens", Some(none), false),
        ("api", "DELETE", "/installation/token", Some(none), true),
        ("api", "GET", "/installation/repositories", Some(none), true),
        ("api", "GET", "/users/bot", Some(none), true),
        ("api", "GET", "/user/345", Some(none), true),
        ("api", "GET", "/user/bot", Some(none), false),
        ("api", "GET", "/user", Some(none), false),
        ("api", "GET", "/repos/owner/probe", Some(none), true),
        ("api", "PATCH", "/repos/owner/probe", Some(none), false),
        ("api", "DELETE", "/repos/owner/probe", Some(none), false),
        ("api", "GET", "/repos/owner/probe/contents/probe/x.md?ref=main", Some(none), true),
        ("api", "POST", "/repos/owner/probe/pulls", Some(none), true),
        ("api", "PUT", "/repos/owner/probe/pulls/1/merge", Some(none), true),
        ("api", "PUT", "/repos/owner/probe/pulls/1/update-branch", Some(none), true),
        ("api", "PATCH", "/repos/owner/probe/pulls/1", Some(none), true),
        ("api", "PATCH", "/repos/owner/probe/issues/1", Some(none), true),
        ("api", "POST", "/repos/owner/probe/issues/1/comments", Some(none), true),
        ("api", "POST", "/repos/owner/probe/pulls/1/reviews", Some(none), true),
        ("api", "DELETE", "/repos/owner/probe/pulls/1/requested_reviewers", Some(none), true),
        ("api", "POST", &status, Some(none), true),
        ("api", "POST", "/repos/owner/probe/statuses/main", Some(none), false),
        ("api", "PUT", "/repos/owner/probe/pulls/x/merge", Some(none), false),
        ("api", "DELETE", "/repos/owner/probe/git/refs/heads/temper/live-1/1/c2", Some(none), true),
        ("api", "DELETE", "/repos/owner/probe/git/refs/heads/main", Some(none), false),
        ("api", "DELETE", "/repos/owner/probe/git/refs/heads/temper/live-1/", Some(none), false),
        ("api", "DELETE", "/repos/owner/probe/git/refs/heads/temper/live-2/1/c2", Some(none), false),
        ("api", "PUT", "/repos/owner/probe/contents/README.md", Some(none), false),
        ("api", "POST", "/repos/owner/probe/hooks", Some(none), false),
        ("api", "PUT", "/repos/owner/probe/collaborators/someone", Some(none), false),
        ("api", "GET", "/repositories/123", Some(none), true),
        ("api", "GET", "/repositories/124", Some(none), false),
        ("api", "GET", "/repositories/123/branches", Some(none), false),
        ("api", "PATCH", "/repositories/123", Some(none), false),
        ("api", "PATCH", "/app/hook/config", Some(none), false),
        ("api", "DELETE", "/app/installations/17", Some(none), false),
        ("api", "GET", "/repos/other/probe", Some(none), false),
        ("api", "POST", "/repos/owner/other/pulls", Some(none), false),
        ("api", "GET", "/repos/owner/probe/../other", Some(none), false),
        ("api", "GET", "/repos/owner/probe/..", Some(none), false),
        ("api", "GET", "/repos/owner/probe/./pulls", Some(none), false),
        ("api", "GET", &compare, Some(none), true),
        ("api", "GET", "/repos/owner/other/compare/a...b", Some(none), false),
        ("api", "GET", "/repos/owner/probe/%2E%2e/other", Some(none), false),
        ("api", "GET", "/repos/owner/probe//pulls", Some(none), false),
        ("api", "GET", "/repos/Owner/probe", Some(none), false),
        ("api", "GET", "/app/installations/17/other", Some(none), false),
        ("api", "GET", "/repositories/not-an-id", Some(none), false),
        ("api", "GET", "/users/bot/other", Some(none), false),
        ("api", "GET", "/repos/owner/probe/", Some(none), false),
        ("git", "GET", "/owner/probe.git/info/refs?service=git-upload-pack", Some(none), true),
        ("git", "POST", "/owner/probe.git/git-upload-pack", None, true),
        ("git", "GET", "/owner/other.git/info/refs", Some(none), false),
        ("git", "DELETE", "/owner/probe.git/git-receive-pack", Some(none), false),
        ("git", "GET", "/app", Some(none), false),
        ("api", "GET", "/owner/probe.git/info/refs", Some(none), false),
        ("api", "GET", "https://api.github.com/app", Some(none), false),
    ]);
    assert!(!guard().allows("api", "GET", "/repositories/123", Some(none), None), "no id read yet");
}

#[test]
fn guard_reads_the_bodies_of_graphql_refs_and_pushes() {
    use skein_forge::github::document::{ENQUEUE, PULL, VIEWER};
    let graphql = |query: &[u8], owner: &str| {
        serde_json::to_vec(&serde_json::json!({"query": std::str::from_utf8(query).expect("document"),
            "variables": {"owner": owner, "name": "probe", "number": 1}}))
        .expect("graphql body")
    };
    let (viewer, pull, foreign, enqueue) =
        (graphql(VIEWER, "owner"), graphql(PULL, "owner"), graphql(PULL, "other"), graphql(ENQUEUE, "owner"));
    let held = push(&["refs/heads/temper/live-1/1/c2"]);
    let main = push(&["refs/heads/temper/live-1/1/c2", "refs/heads/main"]);
    let other_run = push(&["refs/heads/temper/live-2/1/c2"]);
    check(vec![
        ("api", "POST", "/graphql", Some(viewer.as_slice()), true),
        ("api", "POST", "/graphql", Some(pull.as_slice()), true),
        ("api", "POST", "/graphql", Some(foreign.as_slice()), false),
        ("api", "POST", "/graphql", Some(enqueue.as_slice()), false),
        ("api", "POST", "/graphql", Some(br#"{"query":"mutation { deleteRepository }"}"#.as_slice()), false),
        ("api", "POST", "/graphql", None, false),
        (
            "api",
            "POST",
            "/repos/owner/probe/git/refs",
            Some(br#"{"ref":"refs/heads/temper/live-1/x","sha":"a"}"#.as_slice()),
            true,
        ),
        (
            "api",
            "POST",
            "/repos/owner/probe/git/refs",
            Some(br#"{"ref":"refs/heads/main","sha":"a"}"#.as_slice()),
            false,
        ),
        ("git", "POST", "/owner/probe.git/git-receive-pack", Some(held.as_slice()), true),
        ("git", "POST", "/owner/probe.git/git-receive-pack", Some(main.as_slice()), false),
        ("git", "POST", "/owner/probe.git/git-receive-pack", Some(other_run.as_slice()), false),
        ("git", "POST", "/owner/probe.git/git-receive-pack", Some(b"0000PACK".as_slice()), false),
        ("git", "POST", "/owner/probe.git/git-receive-pack", Some(b"00zz".as_slice()), false),
        ("git", "POST", "/owner/probe.git/git-receive-pack", None, false),
        ("git", "POST", "/other/probe.git/git-receive-pack", Some(held.as_slice()), false),
    ]);
}

#[test]
fn counter_reserves_twenty_requests_for_teardown() {
    let mut counter = Counter::default();
    for _ in 0..380 {
        assert_eq!(counter.take(false), Ok(()));
    }
    assert_eq!(counter.take(false), Err(429));
    assert_eq!(counter.count(), 380);
    for _ in 0..20 {
        assert_eq!(counter.take(true), Ok(()));
    }
    assert_eq!(counter.count(), CAP);
    assert_eq!(counter.take(true), Err(429));
}
