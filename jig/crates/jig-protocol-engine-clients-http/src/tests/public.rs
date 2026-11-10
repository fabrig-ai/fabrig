//! Public HTTP step tests (jig/docs/design/protocol/client/default-http.md, 5 and jig/docs/design/protocol/client/public-reads.md).
use super::{fixtures as f, *};
const PATH: &[u8] = b"/jig/v1/public/0/1/1";
fn mount() -> Mount {
    let mut env = f::env();
    env.limits.public_calls = 2;
    let mut mount =
        Mount::new(Settings { prefix: Box::from(b"/jig".as_slice()), public: None, max_age: 5 }, &env.limits);
    let mut hosts = List::with_capacity(1);
    hosts.push(Box::from(f::HOST)).unwrap();
    mount.down(&env, Event::Listening { hosts });
    mount
}
fn read() -> kit::PublicRead {
    kit::PublicRead { owner: 0, version: 1, kind: 1, ids: Box::new([]) }
}
#[test]
fn canonical_paths_parse_and_others_do_not() {
    for (path, owner, version, kind, ids) in [
        (b"/v1/public/0/0/0".as_slice(), 0, 0, 0, &[][..]),
        (b"/v1/public/65535/1/2/0123456789abcdef".as_slice(), 65535, 1, 2, &[0x0123_4567_89ab_cdef][..]),
        (b"/v1/public/1/65535/65535/0000000000000000/ffffffffffffffff".as_slice(), 1, 65535, 65535, &[0, u64::MAX][..]),
    ] {
        assert_eq!(public_path(path), Some(kit::PublicRead { owner, version, kind, ids: Box::from(ids) }));
    }
    for (name, path) in [
        ("missing prefix", "/public/0/1/1"),
        ("missing field", "/v1/public/0/1"),
        ("overflowing number", "/v1/public/65536/1/1"),
        ("version overflow", "/v1/public/0/65536/1"),
        ("kind overflow", "/v1/public/0/1/65536"),
        ("leading zero", "/v1/public/00/1/1"),
        ("version leading zero", "/v1/public/0/01/1"),
        ("kind leading zero", "/v1/public/0/1/01"),
        ("plus sign", "/v1/public/+0/1/1"),
        ("minus sign", "/v1/public/0/-1/1"),
        ("upper-case id", "/v1/public/0/1/1/0123456789abcdeF"),
        ("short id", "/v1/public/0/1/1/0"),
        ("long id", "/v1/public/0/1/1/00000000000000000"),
        ("non-hex id", "/v1/public/0/1/1/000000000000000g"),
        ("encoded segment", "/v1/public/%30/1/1"),
        ("encoded id", "/v1/public/0/1/1/%300000000000000"),
        ("dot segment", "/v1/public/./1/1"),
        ("dot-dot segment", "/v1/public/0/1/1/.."),
        ("empty segment", "/v1/public/0//1"),
        ("query", "/v1/public/0/1/1?x=1"),
        ("fragment", "/v1/public/0/1/1#x"),
        ("trailing slash", "/v1/public/0/1/1/"),
        ("third id", "/v1/public/0/1/1/0000000000000000/0000000000000000/0000000000000000"),
    ] {
        assert_eq!(public_path(path.as_bytes()), None, "{name}");
    }
}
fn answer(reply: kit::PublicReply, now: Time) -> (u16, Box<[u8]>, Box<[skein_http::Header]>) {
    let mut mount = mount();
    let mut out = Queue::with_capacity(32);
    f::call(&mut mount, &mut out, Method::Get, PATH, &[], b"");
    assert_eq!(
        f::component(&mut out).as_ref(),
        &[kit::FromTransport::PublicRead { exchange: f::EXCHANGE, read: read() }]
    );
    let mut env = f::env();
    env.now = now;
    mount.down(&env, Event::Component(kit::ToTransport::Public { exchange: f::EXCHANGE, reply }));
    f::pump(&mut mount, &mut out);
    f::answered(&mut mount, &mut out, f::EXCHANGE)
}
#[test]
fn each_answer_has_its_status_and_exact_headers() {
    for (reply, status, body, fields) in [
        (
            kit::PublicReply::Document { bytes: Box::from(b"doc".as_slice()), decided: Time::ZERO },
            200,
            b"doc".as_slice(),
            &[
                (b"content-type".as_slice(), b"application/octet-stream".as_slice()),
                (b"cache-control", b"public, max-age=5, must-revalidate"),
                (b"x-content-type-options", b"nosniff"),
            ][..],
        ),
        (
            kit::PublicReply::Unknown,
            404,
            b"",
            &[
                (b"cache-control".as_slice(), b"public, max-age=5, must-revalidate".as_slice()),
                (b"x-content-type-options", b"nosniff"),
            ][..],
        ),
        (
            kit::PublicReply::Busy,
            503,
            b"",
            &[
                (b"cache-control".as_slice(), b"no-store".as_slice()),
                (b"retry-after", b"1"),
                (b"x-content-type-options", b"nosniff"),
            ][..],
        ),
        (
            kit::PublicReply::NotServed { record: Box::from(b"version".as_slice()) },
            200,
            b"version",
            &[
                (b"content-type".as_slice(), b"application/octet-stream".as_slice()),
                (b"cache-control", b"no-store"),
                (b"x-content-type-options", b"nosniff"),
            ][..],
        ),
    ] {
        let (got, bytes, headers) = answer(reply, Time::ZERO);
        assert_eq!((got, bytes.as_ref()), (status, body));
        assert_eq!(headers, f::fields(fields));
    }
}
#[test]
fn a_late_answer_gets_the_rest_of_its_max_age() {
    for (now, cache) in [
        (2_100_000_000, b"public, max-age=2, must-revalidate".as_slice()),
        (9_000_000_000, b"public, max-age=0, must-revalidate"),
    ] {
        let (_, _, headers) =
            answer(kit::PublicReply::Document { bytes: Box::new([]), decided: Time::ZERO }, Time::from_nanos(now));
        assert_eq!(f::header(&headers, b"cache-control"), Some(cache));
    }
}
#[test]
fn options_head_and_post_are_404_with_no_access_control_header() {
    for method in [Method::Options, Method::Head, Method::Post] {
        let mut mount = mount();
        let mut out = Queue::with_capacity(32);
        f::call(&mut mount, &mut out, method, PATH, &[], b"");
        let (status, body, headers) = f::answered(&mut mount, &mut out, f::EXCHANGE);
        assert_eq!((status, body.len()), (404, 0));
        assert_eq!(headers, f::fields(&[(b"cache-control", b"no-store"), (b"x-content-type-options", b"nosniff")]));
    }
}
#[test]
fn a_public_head_has_no_reply_stream() {
    let mut mount = mount();
    let mut out = Queue::with_capacity(32);
    f::call(&mut mount, &mut out, Method::Head, PATH, &[], b"");
    for request in &out {
        match request {
            Request::Http { request: http::Request::Reply(_), .. } => {
                panic!("HEAD completes with its response head and has no reply stream")
            }
            Request::Http { .. } | Request::Component(_) => {}
        }
    }
    assert_eq!(f::answered(&mut mount, &mut out, f::EXCHANGE).0, 404);
}
#[test]
fn a_credential_on_the_route_is_never_read() {
    let cases: [Box<[skein_http::Header]>; 3] = [Box::new([]), Box::new([f::cookie_header()]), Box::new(f::bearer())];
    for headers in cases {
        let mut mount = mount();
        let mut out = Queue::with_capacity(32);
        f::call(&mut mount, &mut out, Method::Get, PATH, &headers, b"");
        assert_eq!(
            f::component(&mut out).as_ref(),
            &[kit::FromTransport::PublicRead { exchange: f::EXCHANGE, read: read() }]
        );
    }
}
#[test]
fn host_is_checked_before_the_path() {
    let mut mount = mount();
    let mut out = Queue::with_capacity(32);
    f::call(&mut mount, &mut out, Method::Post, b"/jig/v1/public/not/a/path", &f::fields(&[(b"host", b"wrong")]), b"");
    let (status, body, headers) = f::answered(&mut mount, &mut out, f::EXCHANGE);
    assert_eq!((status, body.len()), (421, 0));
    assert_eq!(headers, f::fields(&[(b"x-content-type-options", b"nosniff")]));
}
#[test]
fn public_calls_past_their_room_are_503_and_signed_in_calls_are_not() {
    let mut mount = mount();
    let mut out = Queue::with_capacity(32);
    for name in [f::EXCHANGE, f::OTHER] {
        f::call_named(&mut mount, &mut out, name, Method::Get, PATH, &[], b"");
    }
    assert_eq!(f::component(&mut out).len(), 2);
    let head = f::head(Method::Get, PATH, &[], http::Body::None);
    assert_eq!(mount.room_for(&head), 0);
    assert_eq!(mount.room(), f::env().limits.calls);
    assert_eq!(mount.refusal(&head).status, 503);
    f::call_named(&mut mount, &mut out, Token::new(20), Method::Post, b"/jig/v1/read", &f::bearer(), b"");
    assert_eq!(
        f::component(&mut out).as_ref(),
        &[kit::FromTransport::Exchange {
            browser: false,
            exchange: Token::new(20),
            credential: Some(f::credential()),
            message: kit::Message::Read(Box::new([]))
        }]
    );
    f::event(&mut mount, &mut out, f::EXCHANGE, http::Event::Closed);
    assert_eq!(mount.room_for(&head), 1);
    for name in [21, 22, 23] {
        f::call_named(&mut mount, &mut out, Token::new(name), Method::Post, b"/jig/v1/read", &f::bearer(), b"");
    }
    assert_eq!(mount.room(), 0);
    f::component(&mut out);
    f::call_named(&mut mount, &mut out, Token::new(24), Method::Get, PATH, &[], b"");
    assert_eq!(
        f::component(&mut out).as_ref(),
        &[kit::FromTransport::PublicRead { exchange: Token::new(24), read: read() }]
    );
}
#[test]
fn a_refused_public_call_gets_host_then_busy_and_others_bare_503() {
    let mount = mount();
    for (path, host, status, fields) in [
        (
            PATH,
            f::HOST,
            503,
            &[
                (b"cache-control".as_slice(), b"no-store".as_slice()),
                (b"retry-after", b"1"),
                (b"x-content-type-options", b"nosniff"),
            ][..],
        ),
        (PATH, b"wrong", 421, &[(b"x-content-type-options".as_slice(), b"nosniff".as_slice())][..]),
        (b"/jig/v1/read", b"wrong", 503, &[][..]),
    ] {
        let before = mount.room();
        let response = mount.refusal(&f::head(Method::Get, path, &f::fields(&[(b"host", host)]), http::Body::None));
        assert_eq!(response.status, status);
        assert_eq!(response.headers, f::fields(fields));
        assert!(response.close);
        assert_eq!(mount.room(), before);
    }
}
#[test]
fn a_declared_public_body_is_404_without_reading_it() {
    for body in [http::Body::Length(1), http::Body::Chunked] {
        let mut mount = mount();
        let mut out = Queue::with_capacity(32);
        mount.down(&f::env(), Event::Call { call: f::EXCHANGE, head: f::head(Method::Get, PATH, &[], body) });
        f::pump(&mut mount, &mut out);
        for request in &out {
            match request {
                Request::Component(_) | Request::Http { request: http::Request::Body(_), .. } => {
                    panic!("a refused body is never read or handed on")
                }
                Request::Http { .. } => {}
            }
        }
        assert_eq!(f::answered(&mut mount, &mut out, f::EXCHANGE).0, 404);
    }
}
