//! Offline proof of the live relay; every upstream socket is loopback plaintext.
use crate::support;
use skein_lib::Duration;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use support::relay::Relay;
use support::relay_fixture::{guard, push};
use support::socket::Socket;
fn head(socket: &mut impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        let mut b = [0];
        socket.read_exact(&mut b).expect("stub head");
        bytes.push(b[0]);
    }
    bytes
}
fn answer(socket: &mut impl Read) -> Vec<u8> {
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
            let (socket, _) = upstream.accept().expect("stub accepts");
            let mut socket = Socket::reading(socket, Duration::from_secs(2), "the offline relay stub's request");
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
    let client = TcpStream::connect(relay.api).expect("client");
    let mut client = Socket::reading(client, Duration::from_secs(2), "the offline relay client's reply");
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
            let (socket, _) = upstream.accept().expect("accept");
            let mut socket = Socket::reading(socket, Duration::from_secs(2), "the offline relay stub's request");
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
    let git = TcpStream::connect(relay.git).expect("git client");
    let mut git = Socket::reading(git, Duration::from_secs(2), "the offline relay git client's reply");
    git.write_all(b"POST /owner/probe.git/git-receive-pack HTTP/1.1\r\nHost: local\r\nAuthorization: Basic GIT_SECRET\r\nExpect: 100-continue\r\nTransfer-Encoding: chunked\r\n\r\n").expect("git head");
    assert_eq!(head(&mut git), b"HTTP/1.1 100 Continue\r\n\r\n");
    git.write_all(&chunked).expect("git body");
    assert_eq!(answer(&mut git), b"git");
    let wire = stub.join().expect("stub ended");
    assert!(String::from_utf8_lossy(&wire).contains("Host: github.com\r\n"));
    assert!(String::from_utf8_lossy(&wire).contains("Authorization: Basic GIT_SECRET\r\n"));
    relay.quiesce(); // A connected client must not delay shutdown or make a refusal.
    assert_eq!((relay.refused()).len(), 0);
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
            let (socket, _) = upstream.accept().expect("stub accepts");
            let mut socket = Socket::reading(socket, Duration::from_secs(2), "the offline relay stub's request");
            head(&mut socket);
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok").expect("stub response");
        }
    });
    // The engine's setup host asks with Connection: close, then half-closes and drains until the peer ends.
    let asked = TcpStream::connect(relay.api).expect("client");
    let mut asked = Socket::reading(asked, Duration::from_secs(2), "the relay's requested close");
    asked.write_all(b"GET /app HTTP/1.1\r\nHost: original\r\nConnection: close\r\n\r\n").expect("request");
    assert_eq!(answer(&mut asked), b"ok");
    assert_eq!(asked.read(&mut [0; 1]).expect("the relay ends an asked close"), 0);
    let halved = TcpStream::connect(relay.api).expect("client");
    let mut halved = Socket::reading(halved, Duration::from_secs(2), "the relay's half-close response");
    halved.write_all(b"GET /app HTTP/1.1\r\nHost: original\r\n\r\n").expect("request");
    assert_eq!(answer(&mut halved), b"ok");
    halved.shutdown(std::net::Shutdown::Write).expect("half-close");
    assert_eq!(halved.read(&mut [0; 1]).expect("the relay ends a half-closed connection"), 0);
    stub.join().expect("stub joined");
    assert_eq!((relay.refused()).len(), 0);
    drop(relay);
    scratch.remove();
}
