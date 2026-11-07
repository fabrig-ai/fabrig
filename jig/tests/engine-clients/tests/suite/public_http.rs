//! The public mapping over HTTP bytes (jig/docs/design/protocol/client.md, 17.5 and 19).
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{FromTransport, Message, PublicRead, PublicReply, Reply, ToTransport};
use jig_test_service::server::policy::FIELDS;
use skein_http::{Header, Method};
use skein_lib::{Time, Token};

const PATH: &[u8] = b"/jig/v1/public/0/1/1";
const NO_STORE: &[(&str, &str)] = &[("cache-control", "no-store"), ("x-content-type-options", "nosniff")];
const BUSY: &[(&str, &str)] =
    &[("cache-control", "no-store"), ("retry-after", "1"), ("x-content-type-options", "nosniff")];
fn peer() -> Peer {
    let mut bounds = fixtures::http_limits();
    // One call in each room exposes both admission boundaries.
    bounds.calls = 1;
    bounds.public_calls = 1;
    Peer::with_bounds(&fixtures::settings(), fixtures::limits(), bounds)
}
fn headers(extra: &[(&[u8], &[u8])]) -> Box<[Header]> {
    let mut headers = vec![Header { name: Box::from(b"host".as_slice()), value: Box::from(fixtures::HOST.as_bytes()) }];
    for (name, value) in extra {
        headers.push(Header { name: Box::from(*name), value: Box::from(*value) });
    }
    headers.into_boxed_slice()
}
fn admitted(messages: &[FromTransport]) -> Token {
    assert_eq!(messages.len(), 1, "wire seeds 1,2: one public call");
    let Some(FromTransport::PublicRead { exchange, read }) = messages.first() else {
        panic!("wire seeds 1,2: public read: {messages:?}");
    };
    assert_eq!(*read, PublicRead { owner: 0, version: 1, kind: 1, ids: Box::new([]) }, "wire seeds 1,2");
    *exchange
}
fn answer(peer: &mut Peer, reply: PublicReply, now: Time) -> Token {
    let (socket, messages) = peer.public_http(Method::Get, PATH, headers(&[]));
    let exchange = admitted(&messages);
    peer.public_http_answer(now, ToTransport::Public { exchange, reply });
    socket
}
fn assert_answer(peer: &Peer, socket: Token, status: &str, body: &str, fields: &[(&str, &str)], close: bool) {
    let bytes = std::str::from_utf8(peer.public_http_bytes(socket)).expect("wire seeds 1,2: ASCII answer");
    let (head, got_body) = bytes.split_once("\r\n\r\n").expect("wire seeds 1,2: HTTP head");
    let mut lines = head.split("\r\n");
    assert_eq!(lines.next(), Some(status), "wire seeds 1,2");
    let got: Vec<_> = lines.map(|line| line.split_once(": ").expect("wire seeds 1,2: header")).collect();
    let mut expected = vec![("Date", "Thu, 01 Jan 1970 00:00:00 GMT")];
    expected.extend_from_slice(fields);
    for (name, value) in FIELDS {
        let name = std::str::from_utf8(name).expect("policy name");
        if !expected.iter().any(|(present, _)| present.eq_ignore_ascii_case(name)) {
            expected.push((name, std::str::from_utf8(value).expect("policy value")));
        }
    }
    let length = body.len().to_string();
    expected.push(("Content-Length", &length));
    if close {
        expected.push(("Connection", "close"));
    }
    assert_eq!(got, expected, "wire seeds 1,2");
    assert_eq!(got_body, body, "wire seeds 1,2");
}

#[test]
fn public_reads_with_no_credential_and_with_a_signed_in_cookie_are_answered_alike() {
    let mut peer = peer();
    let bearer = fixtures::signed_in(&mut peer);
    let token = bearer.strip_prefix(b"Bearer ").expect("fixture bearer");
    let mut cookie = b"jig-sign-in=".to_vec();
    cookie.extend_from_slice(token);
    let plain = answer(&mut peer, PublicReply::Unknown, Time::ZERO);
    let plain = peer.public_http_bytes(plain).to_vec();
    for extra in [vec![(b"cookie".as_slice(), cookie.as_slice())], vec![(b"authorization".as_slice(), bearer.as_ref())]]
    {
        let (socket, messages) = peer.public_http(Method::Get, PATH, headers(&extra));
        let exchange = admitted(&messages);
        peer.public_http_answer(Time::ZERO, ToTransport::Public { exchange, reply: PublicReply::Unknown });
        assert_eq!(peer.public_http_bytes(socket), plain, "wire seeds 1,2: credentials ignored");
    }
}

#[test]
fn each_public_answer_has_its_status_and_exact_headers() {
    for (reply, status, body, fields) in [
        (
            PublicReply::Document { bytes: Box::from(b"document".as_slice()), decided: Time::ZERO },
            "HTTP/1.1 200 OK",
            "document",
            &[
                ("content-type", "application/octet-stream"),
                ("cache-control", "public, max-age=5, must-revalidate"),
                ("x-content-type-options", "nosniff"),
            ][..],
        ),
        (
            PublicReply::Unknown,
            "HTTP/1.1 404 Not Found",
            "",
            &[("cache-control", "public, max-age=5, must-revalidate"), ("x-content-type-options", "nosniff")][..],
        ),
        (PublicReply::Busy, "HTTP/1.1 503 Service Unavailable", "", BUSY),
        (
            PublicReply::NotServed { record: Box::from(b"version".as_slice()) },
            "HTTP/1.1 200 OK",
            "version",
            &[
                ("content-type", "application/octet-stream"),
                ("cache-control", "no-store"),
                ("x-content-type-options", "nosniff"),
            ][..],
        ),
    ] {
        let mut peer = peer();
        let socket = answer(&mut peer, reply, Time::ZERO);
        assert_answer(&peer, socket, status, body, fields, false);
    }
}

#[test]
fn a_late_answer_carries_a_shortened_max_age() {
    for (now, cache) in
        [(2_100_000_000, "public, max-age=2, must-revalidate"), (9_000_000_000, "public, max-age=0, must-revalidate")]
    {
        let mut peer = peer();
        let socket = answer(
            &mut peer,
            PublicReply::Document { bytes: Box::from(b"late".as_slice()), decided: Time::ZERO },
            Time::from_nanos(now),
        );
        assert_answer(
            &peer,
            socket,
            "HTTP/1.1 200 OK",
            "late",
            &[
                ("content-type", "application/octet-stream"),
                ("cache-control", cache),
                ("x-content-type-options", "nosniff"),
            ],
            false,
        );
    }
}

#[test]
fn malformed_public_paths_are_404() {
    for (name, path) in [
        ("overflow", "/jig/v1/public/65536/1/1"),
        ("leading zero", "/jig/v1/public/0/01/1"),
        ("uppercase id", "/jig/v1/public/0/1/1/0123456789abcdeF"),
        ("short id", "/jig/v1/public/0/1/1/0"),
        ("encoded", "/jig/v1/public/%30/1/1"),
        ("dot", "/jig/v1/public/./1/1"),
        ("dot-dot", "/jig/v1/public/0/../1"),
        ("query", "/jig/v1/public/0/1/1?x=1"),
        ("trailing slash", "/jig/v1/public/0/1/1/"),
        ("empty segment", "/jig/v1/public/0//1"),
        ("third id", "/jig/v1/public/0/1/1/0000000000000000/0000000000000000/0000000000000000"),
    ] {
        let mut peer = peer();
        let (socket, messages) = peer.public_http(Method::Get, path.as_bytes(), headers(&[]));
        assert!(messages.is_empty(), "wire seeds 1,2: {name} reaches no component");
        assert_answer(&peer, socket, "HTTP/1.1 404 Not Found", "", NO_STORE, false);
    }
}

#[test]
fn options_head_and_post_are_404_with_no_access_control_header() {
    for method in [Method::Options, Method::Head, Method::Post] {
        let mut peer = peer();
        let (socket, messages) = peer.public_http(method, PATH, headers(&[]));
        assert!(messages.is_empty(), "wire seeds 1,2: {method:?} reaches no component");
        assert_answer(&peer, socket, "HTTP/1.1 404 Not Found", "", NO_STORE, false);
    }
}

#[test]
fn a_public_read_past_its_room_gets_the_fixed_busy_answer_and_a_bad_host_421() {
    let mut peer = peer();
    let (held, messages) = peer.public_http(Method::Get, PATH, headers(&[]));
    let exchange = admitted(&messages);
    let (busy, messages) = peer.public_http(Method::Get, PATH, headers(&[]));
    assert!(messages.is_empty(), "wire seeds 1,2: no call past the public room");
    assert_answer(&peer, busy, "HTTP/1.1 503 Service Unavailable", "", BUSY, true);
    let wrong = Box::new([Header { name: Box::from(b"host".as_slice()), value: Box::from(b"wrong".as_slice()) }]);
    let (refused, messages) = peer.public_http(Method::Get, PATH, wrong);
    assert!(messages.is_empty(), "wire seeds 1,2: refused Host reaches no component");
    assert_answer(
        &peer,
        refused,
        "HTTP/1.1 421 Misdirected Request",
        "",
        &[("x-content-type-options", "nosniff")],
        true,
    );
    peer.public_http_answer(Time::ZERO, ToTransport::Public { exchange, reply: PublicReply::Unknown });
    assert_answer(
        &peer,
        held,
        "HTTP/1.1 404 Not Found",
        "",
        &[("cache-control", "public, max-age=5, must-revalidate"), ("x-content-type-options", "nosniff")],
        false,
    );
}

#[test]
fn public_and_signed_in_calls_each_have_their_own_room() {
    let mut peer = peer();
    let bearer = fixtures::signed_in(&mut peer);
    let signed_headers = || headers(&[(b"authorization", &bearer), (b"content-type", b"application/octet-stream")]);
    let (signed, messages) = peer.public_http(Method::Post, b"/jig/v1/read", signed_headers());
    let Some(FromTransport::Exchange { exchange: signed_exchange, credential: Some(_), message: Message::Read(_), .. }) =
        messages.first()
    else {
        panic!("wire seeds 1,2: signed-in read admitted: {messages:?}");
    };
    let signed_exchange = *signed_exchange;
    // Ordinary room is full, yet the public call reaches the component and completes.
    let public = answer(&mut peer, PublicReply::Busy, Time::ZERO);
    assert_answer(&peer, public, "HTTP/1.1 503 Service Unavailable", "", BUSY, false);
    peer.public_http_answer(
        Time::ZERO,
        ToTransport::Reply { exchange: signed_exchange, reply: Reply::Record(Box::new([])) },
    );
    assert_eq!(peer.status(signed), Some(200), "wire seeds 1,2");
    let (_, messages) = peer.public_http(Method::Get, PATH, headers(&[]));
    let public_exchange = admitted(&messages);
    // Public room is full, yet the signed-in call reaches the component and completes.
    let (signed, messages) = peer.public_http(Method::Post, b"/jig/v1/read", signed_headers());
    let Some(FromTransport::Exchange { exchange, credential: Some(_), message: Message::Read(_), .. }) = messages.first()
    else {
        panic!("wire seeds 1,2: signed-in read beside full public room: {messages:?}");
    };
    peer.public_http_answer(Time::ZERO, ToTransport::Reply { exchange: *exchange, reply: Reply::Record(Box::new([])) });
    assert_eq!(peer.status(signed), Some(200), "wire seeds 1,2");
    peer.public_http_answer(Time::ZERO, ToTransport::Public { exchange: public_exchange, reply: PublicReply::Unknown });
}
