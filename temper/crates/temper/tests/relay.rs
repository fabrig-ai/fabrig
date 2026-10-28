//! Offline proof of the live relay; every upstream socket is loopback plaintext.
pub mod support;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use support::relay::{CAP, Counter, Guard, Relay};
fn guard() -> Guard {
    Guard { owner: "owner".into(), repository: "probe".into(), installation: 17, prefix: "temper/live-1/".into() }
}
/// Listener, method, target, body (`None`: encoded), allowed.
type Row<'a> = (&'a str, &'a str, &'a str, Option<&'a [u8]>, bool);
/// A receive-pack body: one pkt-line command per ref, then the flush.
fn push(refs: &[&str]) -> Vec<u8> {
    let mut body = Vec::new();
    for (index, name) in refs.iter().enumerate() {
        let caps = if index == 0 { "\0report-status" } else { "" };
        let line = format!("{} {} {name}{caps}\n", "0".repeat(40), "1".repeat(40));
        body.extend_from_slice(format!("{:04x}{line}", line.len() + 4).as_bytes());
    }
    body.extend_from_slice(b"0000PACK");
    body
}
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
fn head(socket: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        let mut b = [0];
        socket.read_exact(&mut b).expect("stub head");
        bytes.push(b[0]);
    }
    bytes
}
fn answer(socket: &mut TcpStream) -> Vec<u8> {
    let bytes = head(socket);
    let text = std::str::from_utf8(&bytes).expect("head");
    let length: usize =
        text.lines().find_map(|line| line.strip_prefix("Content-Length: ")).expect("length").parse().expect("length");
    let mut body = vec![0; length];
    socket.read_exact(&mut body).expect("answer");
    body
}
#[test]
#[expect(clippy::disallowed_methods, reason = "offline relay framing against an owned loopback socket thread")]
fn framing_preserves_requests_and_keeps_the_client_alive() {
    let scratch = support::Scratch::new("relay-framing");
    let upstream = TcpListener::bind("127.0.0.1:0").expect("stub");
    let relay = Relay::plaintext(guard(), scratch.path(), upstream.local_addr().expect("stub address"));
    let stub = thread::spawn(move || {
        let mut seen = Vec::new();
        for response in [
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\none\r\n0\r\nX-Trailer: value\r\n\r\n"
                .as_slice(),
            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\ntwo",
            b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nthree",
        ] {
            let (mut socket, _) = upstream.accept().expect("stub accepts");
            socket.set_read_timeout(Some(Duration::from_secs(2))).expect("timeout");
            let mut request = head(&mut socket);
            let mut body = vec![
                0;
                if seen.is_empty() {
                    3
                } else if seen.len() == 1 {
                    13
                } else {
                    0
                }
            ];
            socket.read_exact(&mut body).expect("request body");
            request.extend(body);
            seen.push(request);
            socket.write_all(response).expect("stub response");
        }
        seen
    });
    let mut client = TcpStream::connect(relay.api).expect("client");
    client.set_read_timeout(Some(Duration::from_secs(2))).expect("timeout");
    client.write_all(b"POST /repos/owner/probe/pulls?token=NEVER_LOG_QUERY HTTP/1.1\r\nHost: original\r\nAuthorization: SECRET_HEADER\r\nContent-Length: 3\r\n\r\nabc").expect("first request");
    assert_eq!(answer(&mut client), b"one");
    client.write_all(b"POST /repos/owner/probe/pulls HTTP/1.1\r\nHost: original\r\nAuthorization: SECRET_HEADER\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nxyz\r\n0\r\n\r\n").expect("second request");
    assert_eq!(answer(&mut client), b"two");
    assert_eq!(relay.ask("GET", "/app", b"Bearer SECRET_HEADER", b"").expect("own read"), (200, b"three".to_vec()));
    client
        .write_all(b"PATCH /app/hook/config HTTP/1.1\r\nHost: original\r\nContent-Length: 0\r\n\r\n")
        .expect("refused request");
    assert_eq!(answer(&mut client), b"relay refused request");
    let mut git = TcpStream::connect(relay.git).expect("git client");
    git.write_all(b"GET /other/probe.git/info/refs HTTP/1.1\r\nHost: original\r\n\r\n").expect("git refusal");
    assert_eq!(answer(&mut git), b"relay refused request");
    let seen = stub.join().expect("stub joined");
    assert!(seen.iter().all(|wire| String::from_utf8_lossy(wire).contains("Host: api.github.com\r\n")));
    assert!(seen[0].ends_with(b"\r\n\r\nabc"));
    assert!(seen[1].ends_with(b"3\r\nxyz\r\n0\r\n\r\n"));
    assert!(String::from_utf8_lossy(&seen[0]).contains("Authorization: SECRET_HEADER\r\n"));
    assert_eq!(relay.count(), 3);
    assert_eq!(relay.refused().len(), 2);
    let journal = fs::read_to_string(scratch.path().join("relay.jsonl")).expect("journal");
    assert!(!journal.contains("SECRET_HEADER") && !journal.contains("NEVER_LOG_QUERY") && !journal.contains("abc"));
    assert_eq!(journal.lines().count(), 5);
    for line in journal.lines() {
        let row: (String, String, String, u16) = serde_json::from_str(line).expect("closed journal schema");
        assert!(!row.2.contains('?'));
    }
    assert_eq!(fs::read_to_string(scratch.path().join("count.txt")).expect("count"), "3\n");
    drop(client);
    drop(git);
    drop(relay);
    scratch.remove();
}

#[test]
#[expect(clippy::disallowed_methods, reason = "offline relay framing against an owned loopback socket thread")]
fn bodyless_answers_and_minted_tokens_stay_in_their_lanes() {
    let scratch = support::Scratch::new("relay-bodyless");
    let upstream = TcpListener::bind("127.0.0.1:0").expect("stub");
    let mut relay = Relay::plaintext(guard(), scratch.path(), upstream.local_addr().expect("address"));
    let pushed = push(&["refs/heads/temper/live-1/1/c2"]);
    let chunked = [format!("{:x}\r\n", pushed.len()).into_bytes(), pushed, b"\r\n0\r\n\r\n".to_vec()].concat();
    let expected = chunked.clone();
    let stub = thread::spawn(move || {
        let responses = [
            b"HTTP/1.1 201 Created\r\nContent-Length: 23\r\n\r\n{\"token\":\"MEMORY_ONLY\"}".as_slice(),
            b"HTTP/1.1 200 OK\r\nContent-Length: 9999\r\n\r\n",
            b"HTTP/1.1 204 No Content\r\nTransfer-Encoding: chunked\r\n\r\n",
            b"HTTP/1.1 304 Not Modified\r\nContent-Length: 9999\r\n\r\n",
            b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\ngit",
        ];
        let mut last = Vec::new();
        for response in responses {
            let (mut socket, _) = upstream.accept().expect("accept");
            socket.set_read_timeout(Some(Duration::from_secs(2))).expect("timeout");
            last = head(&mut socket);
            if last.starts_with(b"POST /owner/") {
                let mut chunks = vec![0; expected.len()];
                socket.read_exact(&mut chunks).expect("git chunks");
                assert_eq!(chunks, expected);
            }
            socket.write_all(response).expect("response");
        }
        last
    });
    assert_eq!(
        relay.ask("POST", "/app/installations/17/access_tokens", b"Bearer APP_SECRET", b"").expect("mint").0,
        201
    );
    assert_eq!(relay.tokens(), [b"MEMORY_ONLY".to_vec()]);
    for (method, status) in [("HEAD", 200), ("GET", 204), ("GET", 304)] {
        assert_eq!(
            relay.ask(method, "/repos/owner/probe/pulls/1", b"Bearer PERSON_SECRET", b"").expect("bodyless"),
            (status, vec![])
        );
    }
    let mut git = TcpStream::connect(relay.git).expect("git client");
    git.set_read_timeout(Some(Duration::from_secs(2))).expect("timeout");
    git.write_all(b"POST /owner/probe.git/git-receive-pack HTTP/1.1\r\nHost: local\r\nAuthorization: Basic GIT_SECRET\r\nExpect: 100-continue\r\nTransfer-Encoding: chunked\r\n\r\n").expect("git head");
    assert_eq!(head(&mut git), b"HTTP/1.1 100 Continue\r\n\r\n");
    git.write_all(&chunked).expect("git body");
    assert_eq!(answer(&mut git), b"git");
    let wire = stub.join().expect("stub ended");
    assert!(String::from_utf8_lossy(&wire).contains("Host: github.com\r\n"));
    assert!(String::from_utf8_lossy(&wire).contains("Authorization: Basic GIT_SECRET\r\n"));
    relay.quiesce(); // A connected client must not delay shutdown or make a refusal.
    assert!(relay.refused().is_empty());
    assert_eq!(relay.count(), 5);
    let journal = fs::read_to_string(scratch.path().join("relay.jsonl")).expect("journal");
    for secret in ["MEMORY_ONLY", "APP_SECRET", "PERSON_SECRET", "GIT_SECRET"] {
        assert!(!journal.contains(secret));
    }
    drop(git);
    drop(relay);
    scratch.remove();
}

#[test]
#[expect(clippy::disallowed_methods, reason = "offline relay framing against an owned loopback socket thread")]
fn a_closing_client_hears_the_relay_end_its_connection() {
    let scratch = support::Scratch::new("relay-close");
    let upstream = TcpListener::bind("127.0.0.1:0").expect("stub");
    let relay = Relay::plaintext(guard(), scratch.path(), upstream.local_addr().expect("stub address"));
    let stub = thread::spawn(move || {
        for _ in 0..2 {
            let (mut socket, _) = upstream.accept().expect("stub accepts");
            socket.set_read_timeout(Some(Duration::from_secs(2))).expect("timeout");
            head(&mut socket);
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok").expect("stub response");
        }
    });
    // The engine's setup host asks with Connection: close, then half-closes and drains until the peer ends.
    let mut asked = TcpStream::connect(relay.api).expect("client");
    asked.set_read_timeout(Some(Duration::from_secs(2))).expect("timeout");
    asked.write_all(b"GET /app HTTP/1.1\r\nHost: original\r\nConnection: close\r\n\r\n").expect("request");
    assert_eq!(answer(&mut asked), b"ok");
    assert_eq!(asked.read(&mut [0; 1]).expect("the relay ends an asked close"), 0);
    let mut halved = TcpStream::connect(relay.api).expect("client");
    halved.set_read_timeout(Some(Duration::from_secs(2))).expect("timeout");
    halved.write_all(b"GET /app HTTP/1.1\r\nHost: original\r\n\r\n").expect("request");
    assert_eq!(answer(&mut halved), b"ok");
    halved.shutdown(std::net::Shutdown::Write).expect("half-close");
    assert_eq!(halved.read(&mut [0; 1]).expect("the relay ends a half-closed connection"), 0);
    stub.join().expect("stub joined");
    assert!(relay.refused().is_empty());
    drop(relay);
    scratch.remove();
}
