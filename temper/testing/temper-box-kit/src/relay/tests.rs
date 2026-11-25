//! Scripted streams for the box domain checks of the owner-run relay.
use super::{Connect, ForwardError, LedgerOutput, Result, Row, Shared};
use crate::relay_fixture::{guard, push};
use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{self, Read, Write},
};

fn short_read(source: &mut &[u8], bytes: &mut [u8]) -> Result<usize> {
    let length = bytes.len().min(3);
    source.read(&mut bytes[..length])
}

enum End {
    Eof,
    Unread,
}

struct Part<'a> {
    bytes: &'a [u8],
    after: &'a [u8],
}
impl<'a> Part<'a> {
    fn immediate(bytes: &'a [u8]) -> Self {
        Self { bytes, after: b"" }
    }
}

struct Client<'a> {
    parts: VecDeque<Part<'a>>,
    end: End,
    read_eof: bool,
    outgoing: Vec<u8>,
}
impl Read for Client<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize> {
        let Some(part) = self.parts.front_mut() else {
            assert!(matches!(self.end, End::Eof), "read after the connection should have closed");
            self.read_eof = true;
            return Ok(0);
        };
        assert!(self.outgoing.ends_with(part.after), "request bytes read before the preceding reply");
        let read = short_read(&mut part.bytes, bytes)?;
        if part.bytes.is_empty() {
            self.parts.pop_front();
        }
        Ok(read)
    }
}
impl Write for Client<'_> {
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        let length = bytes.len().min(3);
        self.outgoing.extend_from_slice(&bytes[..length]);
        Ok(length)
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

struct Reply<'a> {
    bytes: &'a [u8],
    end: End,
}
impl Read for Reply<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize> {
        assert!(!self.bytes.is_empty() || matches!(self.end, End::Eof), "read past the upstream reply head");
        short_read(&mut self.bytes, bytes)
    }
}
struct Exchange<'a> {
    host: &'a str,
    request: &'a [u8],
    reply: Reply<'a>,
}
impl<'a> Exchange<'a> {
    fn api(request: &'a [u8], answer: &'a [u8]) -> Self {
        Self { host: "api.github.com", request, reply: Reply { bytes: answer, end: End::Eof } }
    }
    fn bodyless(request: &'a [u8], head: &'a [u8]) -> Self {
        Self { host: "api.github.com", request, reply: Reply { bytes: head, end: End::Unread } }
    }
}
struct Upstream<'a>(RefCell<VecDeque<Exchange<'a>>>);
impl<'a> Connect for Upstream<'a> {
    type Reply = Reply<'a>;

    fn reply(&self, host: &str, wire: &[u8]) -> std::result::Result<Self::Reply, ForwardError> {
        let exchange = self.0.borrow_mut().pop_front().expect("no unplanned upstream exchange");
        assert_eq!(host, exchange.host);
        assert_eq!(String::from_utf8_lossy(wire), String::from_utf8_lossy(exchange.request));
        Ok(exchange.reply)
    }
}

#[derive(Default)]
struct Journal {
    pending: Vec<u8>,
    flushed: Vec<u8>,
}
impl Write for Journal {
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        self.pending.write(bytes)
    }
    fn flush(&mut self) -> Result<()> {
        self.flushed.append(&mut self.pending);
        Ok(())
    }
}
#[derive(Default)]
struct Output {
    journal: Journal,
    count: Vec<u8>,
}
impl LedgerOutput for Output {
    fn journal(&mut self) -> &mut dyn Write {
        &mut self.journal
    }
    fn replace_count(&mut self, bytes: &[u8]) -> Result<()> {
        self.count.clear();
        self.count.extend_from_slice(bytes);
        Ok(())
    }
}

type Script<'a> = Shared<Output, Upstream<'a>>;

fn relay<const N: usize>(exchanges: [Exchange<'_>; N]) -> Script<'_> {
    Shared::new(guard(), Output::default(), Upstream(RefCell::new(exchanges.into())))
}

fn connection<'a>(relay: &Script<'_>, listener: &str, parts: impl Into<VecDeque<Part<'a>>>, end: End) -> Client<'a> {
    let client = Client { parts: parts.into(), end, read_eof: false, outgoing: vec![] };
    let mut client = io::BufReader::new(client);
    let result = relay.connection(listener, &mut client);
    relay.ended(listener, &result);
    let client = client.into_inner();
    assert!(client.parts.is_empty(), "scripted input not consumed");
    client
}

fn direct_request(method: &str, target: &str, authorization: &str) -> Vec<u8> {
    format!("{method} {target} HTTP/1.1\r\nUser-Agent: temper-live-relay\r\nAccept: application/vnd.github+json\r\nContent-Type: application/json\r\nAuthorization: {authorization}\r\nContent-Length: 0\r\nHost: api.github.com\r\n\r\n").into_bytes()
}

pub fn framing_preserves_requests_and_keeps_the_client_alive() {
    const REFUSED: &[u8] = b"HTTP/1.1 403 Relay\r\nContent-Length: 21\r\n\r\nrelay refused request";
    let direct = direct_request("GET", "/app", "Bearer SECRET_HEADER");
    let relay = relay([
        Exchange::api(
            b"POST /repos/owner/probe/pulls?token=NEVER_LOG_QUERY HTTP/1.1\r\nAuthorization: SECRET_HEADER\r\nContent-Length: 3\r\nHost: api.github.com\r\n\r\nabc",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\none\r\n0\r\nX-Trailer: value\r\n\r\n",
        ),
        Exchange::api(
            b"POST /repos/owner/probe/pulls HTTP/1.1\r\nAuthorization: SECRET_HEADER\r\nTransfer-Encoding: chunked\r\nHost: api.github.com\r\n\r\n3\r\nxyz\r\n0\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\ntwo",
        ),
        Exchange::api(&direct, b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nthree"),
    ]);
    let api = connection(
        &relay,
        "api",
        [
            Part::immediate(b"POST /repos/owner/probe/pulls?token=NEVER_LOG_QUERY HTTP/1.1\r\nHost: original\r\nAuthorization: SECRET_HEADER\r\nContent-Length: 3\r\n\r\nabc"),
            Part {
                bytes: b"POST /repos/owner/probe/pulls HTTP/1.1\r\nHost: original\r\nAuthorization: SECRET_HEADER\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nxyz\r\n0\r\n\r\n",
                after: b"one",
            },
            Part { bytes: b"PATCH /app/hook/config HTTP/1.1\r\nHost: original\r\nContent-Length: 0\r\n\r\n", after: b"two" },
        ],
        End::Eof,
    );
    let answers = [
        b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\none".as_slice(),
        b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\ntwo",
        REFUSED,
    ];
    assert_eq!(api.outgoing, answers.concat());
    assert_eq!(
        relay.request(false, "GET", "/app", b"Bearer SECRET_HEADER", b"").expect("own read"),
        (200, b"three".to_vec())
    );
    let git = connection(
        &relay,
        "git",
        [Part::immediate(b"GET /other/probe.git/info/refs HTTP/1.1\r\nHost: original\r\n\r\n")],
        End::Eof,
    );
    assert_eq!(git.outgoing, REFUSED);
    assert!(relay.upstream.0.borrow().is_empty());

    let ledger = relay.ledger.lock().expect("ledger");
    assert_eq!(ledger.counter.count(), 3);
    assert_eq!(ledger.refused.len(), 2);
    let journal = std::str::from_utf8(&ledger.output.journal.flushed).expect("journal");
    for secret in ["SECRET_HEADER", "NEVER_LOG_QUERY", "abc"] {
        assert!(!journal.contains(secret));
    }
    let rows = journal.lines().map(|line| serde_json::from_str::<Row>(line).expect("closed journal schema"));
    assert_eq!(rows.collect::<Vec<_>>(), ledger.journal);
    assert_eq!(ledger.journal.len(), 5);
    assert!(ledger.journal.iter().all(|row| !row.2.contains('?')));
    assert_eq!(ledger.output.count, b"3\n");
}

pub fn bodyless_answers_and_minted_tokens_stay_in_their_lanes() {
    const MINT: &str = "/app/installations/17/access_tokens";
    const PULL: &str = "/repos/owner/probe/pulls/1";
    const CONTINUE: &[u8] = b"HTTP/1.1 100 Continue\r\n\r\n";
    let mint = direct_request("POST", MINT, "Bearer APP_SECRET");
    let head = direct_request("HEAD", PULL, "Bearer PERSON_SECRET");
    let get = direct_request("GET", PULL, "Bearer PERSON_SECRET");
    let pushed = push(&["refs/heads/temper/live-1/1/c2"]);
    let chunked = [format!("{:x}\r\n", pushed.len()).into_bytes(), pushed, b"\r\n0\r\n\r\n".to_vec()].concat();
    let forwarded = [b"POST /owner/probe.git/git-receive-pack HTTP/1.1\r\nAuthorization: Basic GIT_SECRET\r\nExpect: 100-continue\r\nTransfer-Encoding: chunked\r\nHost: github.com\r\n\r\n".as_slice(), &chunked].concat();
    let relay = relay([
        Exchange::api(&mint, b"HTTP/1.1 201 Created\r\nContent-Length: 23\r\n\r\n{\"token\":\"MEMORY_ONLY\"}"),
        Exchange::bodyless(&head, b"HTTP/1.1 200 OK\r\nContent-Length: 9999\r\n\r\n"),
        Exchange::bodyless(&get, b"HTTP/1.1 204 No Content\r\nTransfer-Encoding: chunked\r\n\r\n"),
        Exchange::bodyless(&get, b"HTTP/1.1 304 Not Modified\r\nContent-Length: 9999\r\n\r\n"),
        Exchange {
            host: "github.com",
            request: &forwarded,
            reply: Reply {
                bytes: b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\ngit",
                end: End::Eof,
            },
        },
    ]);
    assert_eq!(relay.request(false, "POST", MINT, b"Bearer APP_SECRET", b"").expect("mint").0, 201);
    assert_eq!(relay.ledger.lock().expect("ledger").tokens, [b"MEMORY_ONLY".to_vec()]);
    for (method, status) in [("HEAD", 200), ("GET", 204), ("GET", 304)] {
        assert_eq!(
            relay.request(false, method, PULL, b"Bearer PERSON_SECRET", b"").expect("bodyless"),
            (status, vec![])
        );
    }
    let git = connection(
        &relay,
        "git",
        [
            Part::immediate(b"POST /owner/probe.git/git-receive-pack HTTP/1.1\r\nHost: local\r\nAuthorization: Basic GIT_SECRET\r\nExpect: 100-continue\r\nTransfer-Encoding: chunked\r\n\r\n"),
            Part { bytes: &chunked, after: CONTINUE },
        ],
        End::Eof,
    );
    assert_eq!(git.outgoing, [CONTINUE, b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\ngit"].concat());
    assert!(relay.upstream.0.borrow().is_empty());
    let ledger = relay.ledger.lock().expect("ledger");
    assert_eq!(ledger.refused, [] as [Row; 0]);
    assert_eq!(ledger.counter.count(), 5);
    assert_eq!(ledger.output.count, b"5\n");
    let journal = std::str::from_utf8(&ledger.output.journal.flushed).expect("journal");
    for secret in ["MEMORY_ONLY", "APP_SECRET", "PERSON_SECRET", "GIT_SECRET"] {
        assert!(!journal.contains(secret));
    }
}

pub fn requested_close_and_peer_eof_end_a_connection() {
    const OK: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
    let relay = relay([
        Exchange::api(b"GET /app HTTP/1.1\r\nConnection: close\r\nHost: api.github.com\r\n\r\n", OK),
        Exchange::api(b"GET /app HTTP/1.1\r\nHost: api.github.com\r\n\r\n", OK),
    ]);
    let closed = connection(
        &relay,
        "api",
        [Part::immediate(b"GET /app HTTP/1.1\r\nHost: original\r\nConnection: close\r\n\r\n")],
        End::Unread,
    );
    assert_eq!(closed.outgoing, OK);
    let eof = connection(&relay, "api", [Part::immediate(b"GET /app HTTP/1.1\r\nHost: original\r\n\r\n")], End::Eof);
    assert!(eof.read_eof);
    assert_eq!(eof.outgoing, OK);
    assert!(relay.upstream.0.borrow().is_empty());
    assert_eq!(relay.ledger.lock().expect("ledger").refused, [] as [Row; 0]);

    let partial = connection(&relay, "api", [Part::immediate(b"GET /app HTTP/1.1\r\nHost: original\r\n")], End::Eof);
    assert!(partial.read_eof);
    assert_eq!(partial.outgoing, b"");
    assert_eq!(relay.ledger.lock().expect("ledger").refused, [("api".into(), "INVALID".into(), "/".into(), 400)]);
}
