use super::*;
use jig_client_protocol_wire::v20 as w;
use skein_http::Method;
use skein_lib::stream::{Down, Up};
mod fixtures;
#[test]
fn a_read_post_becomes_the_component_s_read_exchange() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &fixtures::bearer(), b"\x01\x02");
    assert_eq!(
        *fixtures::component(&mut out),
        [kit::FromTransport::Exchange {
            browser: false,
            exchange: fixtures::EXCHANGE,
            credential: Some(fixtures::credential()),
            message: kit::Message::Read(Box::new([1, 2]))
        }]
    );
}
#[test]
fn a_reply_is_written_as_200_with_its_record_and_no_store() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &[], b"");
    fixtures::component(&mut out);
    fixtures::reply(&mut mount, &mut out, call, kit::Reply::Record(Box::new([3, 4])));
    let (status, body, headers) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (200, &[3, 4][..]));
    assert_eq!(fixtures::header(&headers, b"cache-control"), Some(b"no-store".as_slice()));
}
#[test]
fn a_record_the_component_cannot_read_is_written_as_400() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &[], b"");
    fixtures::component(&mut out);
    fixtures::reply(&mut mount, &mut out, call, kit::Reply::NotARecord);
    let (status, body, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (400, &[][..]));
}
#[test]
fn a_body_past_the_route_s_bound_is_413_and_reaches_nothing() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(32);
    let length = fixtures::env().limits.body.checked_add(1).expect("past body bound");
    let mut body = List::with_capacity(length);
    for _ in 0..length {
        body.push(7).expect("body");
    }
    let body = body.into_boxed();
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &[], body.as_ref());
    let mut count = 0_u32;
    for request in &out {
        if let Request::Component(_) = request {
            count = count.saturating_add(1);
        }
    }
    assert_eq!(count, 0);
    let (status, body, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (413, &[][..]));
}
#[test]
fn a_path_under_the_prefix_with_no_route_is_404() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/absent", &[], b"");
    let (status, body, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (404, &[][..]));
}
#[test]
fn a_streamed_record_is_one_event_and_taken_once_written() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Get, b"/jig/v1/stream", &[], b"");
    fixtures::component(&mut out);
    fixtures::streamed(&mut mount, &mut out, kit::Streamed::Record(Box::new([1, 2, 3])));
    let (bytes, taken) = fixtures::sent(&mut mount, &mut out);
    assert_eq!((bytes.as_ref(), taken), (b"data: AQID\n\n".as_slice(), 1));
    assert!(out.is_empty());
}
#[test]
fn a_heartbeat_is_written_as_a_comment_line() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Get, b"/jig/v1/stream", &[], b"");
    fixtures::component(&mut out);
    fixtures::streamed(&mut mount, &mut out, kit::Streamed::Heartbeat);
    let (bytes, taken) = fixtures::sent(&mut mount, &mut out);
    assert_eq!((bytes.as_ref(), taken), (b":\n\n".as_slice(), 1));
}
#[test]
fn the_same_routes_answer_under_another_prefix() {
    let mut mount = fixtures::prefixed(b"/api");
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Post, b"/api/v1/read", &[], b"\x01\x02");
    assert_eq!(
        *fixtures::component(&mut out),
        [kit::FromTransport::Exchange {
            browser: false,
            exchange: fixtures::EXCHANGE,
            credential: None,
            message: kit::Message::Read(Box::new([1, 2]))
        }]
    );
}
#[test]
fn a_lost_connection_drops_the_answer_and_nothing_else() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(32);
    fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/request", &[], b"");
    fixtures::component(&mut out);
    fixtures::call_named(&mut mount, &mut out, fixtures::OTHER, Method::Post, b"/jig/v1/request", &[], b"");
    fixtures::component(&mut out);
    fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Closed);
    out = Queue::with_capacity(32);
    fixtures::reply(&mut mount, &mut out, fixtures::EXCHANGE, kit::Reply::Record(Box::new([7])));
    assert!(out.is_empty());
    fixtures::reply(&mut mount, &mut out, fixtures::OTHER, kit::Reply::Record(Box::new([9])));
    let (status, body, _) = fixtures::answered(&mut mount, &mut out, fixtures::OTHER);
    assert_eq!((status, body.as_ref()), (200, &[9][..]));
}
#[test]
fn a_chunked_request_body_read_as_it_arrives_is_the_same_whatever_its_cuts() {
    const BODY: &[u8] = b"a chunked record";
    for size in [BODY.len(), 1] {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(64);
        mount.down(
            &fixtures::env(),
            Event::Call {
                call: fixtures::EXCHANGE,
                head: fixtures::head(Method::Post, b"/jig/v1/request", &fixtures::bearer(), http::Body::Chunked),
            },
        );
        fixtures::pump(&mut mount, &mut out);
        for piece in BODY.chunks(size) {
            fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Body(Up::Bytes(Box::from(piece))));
        }
        fixtures::event(&mut mount, &mut out, fixtures::EXCHANGE, http::Event::Body(Up::End));
        assert_eq!(
            *fixtures::component(&mut out),
            [kit::FromTransport::Exchange {
                browser: false,
                exchange: fixtures::EXCHANGE,
                credential: Some(fixtures::credential()),
                message: kit::Message::Request(Box::from(BODY))
            }]
        );
    }
}

const SIGN_IN_PATHS: [&[u8]; 3] = [b"/jig/v1/sign-in/link", b"/jig/v1/sign-in/deployment", b"/jig/v1/sign-in/approval"];
#[test]
fn a_browser_s_link_sign_in_sets_the_cookie_and_answers_no_token() {
    for path in SIGN_IN_PATHS {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(16);
        let call = fixtures::call(&mut mount, &mut out, Method::Post, path, &fixtures::browser_header(), b"");
        assert_eq!(fixtures::component(&mut out).len(), 1, "{path:?}");
        fixtures::reply(
            &mut mount,
            &mut out,
            call,
            fixtures::sign_in_reply(path, &fixtures::TOKEN, fixtures::IN_A_DAY),
        );
        let (status, body, headers) = fixtures::answered(&mut mount, &mut out, call);
        assert_eq!(status, 200, "{path:?}");
        assert_eq!(fixtures::header(&headers, b"set-cookie"), Some(fixtures::COOKIE_ON_LISTENER), "{path:?}");
        let signed_in = fixtures::signed_in(path, &body);
        assert_eq!(signed_in.token(), &None, "{path:?}");
        assert_eq!((signed_in.party(), signed_in.expires()), (fixtures::PARTY, fixtures::IN_A_DAY), "{path:?}");
    }
}
#[test]
fn a_native_sign_in_answers_its_token_in_the_body() {
    for path in SIGN_IN_PATHS {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(16);
        let call = fixtures::call(&mut mount, &mut out, Method::Post, path, &[], b"");
        assert_eq!(fixtures::component(&mut out).len(), 1, "{path:?}");
        fixtures::reply(
            &mut mount,
            &mut out,
            call,
            fixtures::sign_in_reply(path, &fixtures::TOKEN, fixtures::IN_A_DAY),
        );
        let (status, body, headers) = fixtures::answered(&mut mount, &mut out, call);
        assert_eq!(status, 200, "{path:?}");
        assert_eq!(fixtures::header(&headers, b"set-cookie"), None, "{path:?}");
        assert_eq!(fixtures::signed_in(path, &body).token(), &Some(fixtures::credential()), "{path:?}");
    }
}
const COOKIE_ROUTES: [(Method, &[u8], bool); 2] =
    [(Method::Post, b"/jig/v1/read", false), (Method::Get, b"/jig/v1/stream", true)];
#[test]
fn a_browser_exchange_s_cookie_is_its_credential() {
    for (method, path, stream) in COOKIE_ROUTES {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(16);
        fixtures::call(&mut mount, &mut out, method, path, &fixtures::browser(), b"");
        let expected = if stream {
            kit::FromTransport::StreamOpen { stream: fixtures::EXCHANGE, credential: Some(fixtures::credential()) }
        } else {
            kit::FromTransport::Exchange {
                browser: true,
                exchange: fixtures::EXCHANGE,
                credential: Some(fixtures::credential()),
                message: kit::Message::Read(Box::new([])),
            }
        };
        assert_eq!(*fixtures::component(&mut out), [expected], "{path:?}");
    }
}
#[test]
fn a_cookie_without_the_browser_header_is_403_and_reaches_nothing() {
    for (method, path, _) in COOKIE_ROUTES {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(16);
        let call = fixtures::call(&mut mount, &mut out, method, path, &[fixtures::cookie_header()], b"");
        for request in &out {
            match request {
                Request::Component(_) => panic!("forbidden call reached the component: {path:?}"),
                Request::Http { .. } => {}
            }
        }
        let (status, body, _) = fixtures::answered(&mut mount, &mut out, call);
        assert_eq!((status, body.as_ref()), (403, b"".as_slice()), "{path:?}");
    }
}
#[test]
fn a_browser_s_sign_out_clears_the_cookie() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/sign-out", &fixtures::browser(), b"");
    fixtures::component(&mut out);
    fixtures::reply(&mut mount, &mut out, call, kit::Reply::Record(Box::new([0])));
    let (status, body, headers) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!((status, body.as_ref()), (200, [0].as_slice()));
    assert_eq!(fixtures::header(&headers, b"set-cookie"), Some(fixtures::COOKIE_CLEARED));
}
#[test]
fn the_bearer_wins_when_a_cookie_also_arrives() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let [bearer] = fixtures::bearer();
    let headers = [bearer, fixtures::cookie_header()];
    fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &headers, b"");
    assert_eq!(
        *fixtures::component(&mut out),
        [kit::FromTransport::Exchange {
            browser: false,
            exchange: fixtures::EXCHANGE,
            credential: Some(fixtures::credential()),
            message: kit::Message::Read(Box::new([]))
        }]
    );
}

type Fields = &'static [(&'static [u8], &'static [u8])];
type ApprovalRoute = (&'static [u8], fn(Box<[u8]>) -> kit::Message);
const APPROVAL_ROUTES: &[ApprovalRoute] = &[
    (b"/jig/v1/approval/ask", kit::Message::ApprovalAsk),
    (b"/jig/v1/approval/read", kit::Message::ApprovalRead),
    (b"/jig/v1/approval/decide", kit::Message::ApprovalDecide),
    (b"/jig/v1/sign-in/approval", kit::Message::SignInApproval),
];
#[test]
fn the_approval_routes_reach_the_component_as_their_messages() {
    let mut exchanges = List::uncapped();
    let mut expected = List::uncapped();
    for (path, message) in APPROVAL_ROUTES {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(16);
        fixtures::call(&mut mount, &mut out, Method::Post, path, &[], b"ask");
        exchanges.push((path, fixtures::component(&mut out))).expect("route row");
        expected
            .push((
                path,
                Box::<[kit::FromTransport]>::from([kit::FromTransport::Exchange {
                    exchange: fixtures::EXCHANGE,
                    credential: None,
                    browser: false,
                    message: message(Box::from(b"ask".as_slice())),
                }]),
            ))
            .expect("expected route row");
    }
    assert_eq!(exchanges.into_boxed(), expected.into_boxed());
}
#[test]
fn the_approval_routes_refuse_every_other_method() {
    for (path, _) in APPROVAL_ROUTES {
        for method in [Method::Get, Method::Head, Method::Put, Method::Patch, Method::Delete, Method::Options] {
            let mut mount = fixtures::mount();
            let mut out = Queue::with_capacity(16);
            let call = fixtures::call(&mut mount, &mut out, method, path, &[], b"");
            fixtures::no_component(&out);
            assert_eq!(fixtures::answered(&mut mount, &mut out, call).0, 404, "{path:?} {method:?}");
        }
    }
}
const COOKIE_FIELD: (&[u8], &[u8]) = (b"cookie", b"jig-sign-in=BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw");
const BEARER_FIELD: (&[u8], &[u8]) =
    (b"authorization", b"Bearer BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw");
const BROWSER_FIELD: (&[u8], &[u8]) = (b"jig-client", b"browser");
const CARRIERS: &[(&str, Fields, Option<bool>)] = &[
    ("cookie and header", &[COOKIE_FIELD, BROWSER_FIELD], Some(true)),
    ("bearer and header", &[BEARER_FIELD, BROWSER_FIELD], Some(false)),
    ("bearer alone", &[BEARER_FIELD], Some(false)),
    ("bearer before cookie", &[BEARER_FIELD, COOKIE_FIELD, BROWSER_FIELD], Some(false)),
    ("header alone", &[BROWSER_FIELD], Some(false)),
    ("cookie alone", &[COOKIE_FIELD], None),
];
#[test]
fn an_exchange_is_a_browser_s_only_by_its_cookie() {
    let mut exchanges = List::uncapped();
    let mut expected = List::uncapped();
    for path in [b"/jig/v1/approval/decide".as_slice(), b"/jig/v1/read"] {
        for (name, fields, browser) in CARRIERS {
            let mut mount = fixtures::mount();
            let mut out = Queue::with_capacity(16);
            let call = fixtures::call(&mut mount, &mut out, Method::Post, path, &fixtures::fields(fields), b"");
            let (observed, status) = if browser.is_some() {
                let component = fixtures::component(&mut out);
                match component.as_ref() {
                    [kit::FromTransport::Exchange { browser, .. }] => (Some(*browser), 0),
                    _ => (None, 0),
                }
            } else {
                fixtures::no_component(&out);
                (None, fixtures::answered(&mut mount, &mut out, call).0)
            };
            exchanges.push((path, name, observed, status)).expect("carrier row");
            expected
                .push((path, name, *browser, if browser.is_some() { 0 } else { 403 }))
                .expect("expected carrier row");
        }
    }
    assert_eq!(exchanges.into_boxed(), expected.into_boxed());
}
const REFUSED: &[(Method, &[u8], Fields, u16)] = &[
    (
        Method::Post,
        b"/jig/v1/read",
        &[(b"host", b"evil.test:7100"), (b"content-type", b"application/octet-stream")],
        421,
    ),
    (Method::Post, b"/jig/v1/read", &[(b"host", b"127.0.0.1:8080"), (b"content-type", b"text/plain")], 415),
    (Method::Options, b"/jig/v1/read", &[(b"host", b"127.0.0.1:8080")], 404),
    (
        Method::Post,
        b"/jig/v1/approval/decide",
        &[(b"host", b"evil.test:7100"), (b"content-type", b"application/octet-stream")],
        421,
    ),
    (Method::Post, b"/jig/v1/approval/decide", &[(b"host", b"127.0.0.1:8080"), (b"content-type", b"text/plain")], 415),
    (Method::Options, b"/jig/v1/approval/decide", &[(b"host", b"127.0.0.1:8080")], 404),
];
#[test]
fn each_guard_refuses_with_nothing_reaching_the_component() {
    for (method, target, headers, status) in REFUSED {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(16);
        let call = fixtures::call(&mut mount, &mut out, *method, target, &fixtures::fields(headers), b"");
        for request in &out {
            match request {
                Request::Component(_) => panic!("refused call reached the component: {target:?} {headers:?}"),
                Request::Http { .. } => {}
            }
        }
        let (answered, _, sent) = fixtures::answered(&mut mount, &mut out, call);
        assert_eq!((answered, fixtures::component(&mut out).len()), (*status, 0), "{target:?} {headers:?}");
        assert!(!fixtures::names_a_cors_field(&sent), "no CORS header for {target:?}");
        assert_eq!(fixtures::header(&sent, b"x-content-type-options"), Some(b"nosniff".as_slice()));
        assert_eq!(fixtures::header(&sent, b"cache-control"), Some(b"no-store".as_slice()));
    }
}
#[test]
fn a_call_before_listening_is_421() {
    let mut mount = fixtures::fresh_mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &[], b"");
    let (status, _, _) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!(status, 421);
    assert!(fixtures::component(&mut out).is_empty());
}
const CLAIMED: [(&[u8], bool); 4] = [
    (b"/jig/v1/sign-in/link", false),
    (b"/jig/v1/sign-in/link?secret=not-spent", false),
    (b"/outside/v1/read", false),
    (b"/jig/v1/sign-in/link-extra", true),
];
#[test]
fn a_get_of_the_link_route_is_not_claimed_and_spends_nothing() {
    let mount = fixtures::mount();
    for (path, expected) in CLAIMED {
        let call = fixtures::head(Method::Get, path, &[], http::Body::None);
        assert_eq!(claims(&mount.settings, &call), expected, "{path:?}");
    }
    assert!(mount.calls.is_empty());
    let policy = link_page();
    assert_eq!(policy.name.as_ref(), b"referrer-policy");
    assert_eq!(policy.value.as_ref(), b"no-referrer");
}
const ANSWERS: [bool; 2] = [false, true];
#[test]
fn the_mapping_s_answers_carry_nosniff_and_no_store() {
    for stream in ANSWERS {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(16);
        let (method, path): (Method, &[u8]) =
            if stream { (Method::Get, b"/jig/v1/stream") } else { (Method::Post, b"/jig/v1/read") };
        let call = fixtures::call(&mut mount, &mut out, method, path, &[], b"");
        fixtures::component(&mut out);
        if stream {
            fixtures::streamed(&mut mount, &mut out, kit::Streamed::Heartbeat);
        } else {
            fixtures::reply(&mut mount, &mut out, call, kit::Reply::Record(Box::new([])));
        }
        let (status, _, headers) = fixtures::answered(&mut mount, &mut out, call);
        assert_eq!(status, 200);
        assert_eq!(fixtures::header(&headers, b"x-content-type-options"), Some(b"nosniff".as_slice()));
        assert_eq!(fixtures::header(&headers, b"cache-control"), Some(b"no-store".as_slice()));
        assert!(!fixtures::names_a_cors_field(&headers));
    }
}

#[test]
fn ending_a_mount_stream_while_its_record_waits_for_room_does_not_panic() {
    use skein_http::sse::writer::Waiting;
    let env = fixtures::env();
    let mut call = call::Call::new(fixtures::EXCHANGE, &env.limits);
    let mut out = Queue::with_capacity(8);
    call.stream(&env, kit::Streamed::Record(Box::new([1, 2, 3])));
    call.end(&env, kit::Ended::Stopping);
    assert_eq!(call.stream.as_ref().expect("stream").writer.waiting(), Waiting::Room);
    call.http(&env, http::Event::Reply(Up::Room), &mut out);
    while call.stream_ready() {
        call.stream_up(&env, &mut out);
    }
    assert_eq!(call.stream.as_ref().expect("stream").writer.waiting(), Waiting::Close);
    call.end(&env, kit::Ended::Stopping);
    assert_eq!(call.stream.as_ref().expect("stream").writer.waiting(), Waiting::Close);
}

#[test]
fn an_empty_response_finishes_its_http_reply() {
    const STATUS: u16 = 200;
    const FINISHES: u32 = 1;
    let env = fixtures::env();
    let mut call = call::Call::new(fixtures::EXCHANGE, &env.limits);
    call.respond(STATUS, Box::new([]), &env.limits);
    let mut finishes = 0_u32;
    while let Some(request) = call.requests.pop() {
        if let http::Request::Reply(Down::Finish) = request {
            finishes += 1;
        }
    }
    assert_eq!(finishes, FINISHES);
}

#[test]
fn room_is_released_only_after_a_calls_end_is_handled() {
    let mut mount = fixtures::mount();
    let capacity = fixtures::env().limits.calls;
    let mut out = Queue::with_capacity(capacity);
    for number in 0..capacity {
        fixtures::call_named(
            &mut mount,
            &mut out,
            Token::new(u64::from(number)),
            Method::Post,
            b"/jig/v1/read",
            &[],
            b"",
        );
        fixtures::component(&mut out);
    }
    mount.down(&fixtures::env(), Event::Http { call: Token::new(0), event: http::Event::Closed });
    assert_eq!(mount.room(), 0);
    fixtures::pump(&mut mount, &mut out);
    assert_eq!(mount.room(), 1);
}

#[test]
fn a_browser_link_keeps_its_cookie_on_a_tokenless_sign_in() {
    use jig_client_protocol_wire::v20 as w;
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call =
        fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/sign-in/link", &fixtures::browser(), b"link");
    assert_eq!(
        *fixtures::component(&mut out),
        [kit::FromTransport::Exchange {
            browser: true,
            exchange: fixtures::EXCHANGE,
            credential: Some(fixtures::credential()),
            message: kit::Message::SignInLink(Box::from(b"link".as_slice())),
        }]
    );
    let answer = w::SignInAnswer::SignedIn(
        w::SignedIn::new(
            &w::CEILINGS,
            w::SignedInParts { token: None, party: fixtures::PARTY, expires: fixtures::IN_A_DAY },
        )
        .expect("kept sign-in"),
    );
    let mut bytes = skein_lib::Writer::new(skein_lib::length::usize_of(answer.measure()));
    answer.encode(&mut bytes).expect("answer");
    fixtures::reply(&mut mount, &mut out, call, kit::Reply::Record(bytes.finish()));
    let (status, body, headers) = fixtures::answered(&mut mount, &mut out, call);
    assert_eq!(status, 200);
    assert_eq!(fixtures::header(&headers, b"set-cookie"), None);
    let answer = fixtures::signed_in(b"/jig/v1/sign-in/link", &body);
    assert_eq!(answer.token(), &None);
    assert_eq!((answer.party(), answer.expires()), (fixtures::PARTY, fixtures::IN_A_DAY));
}

#[test]
fn busy_and_full_answer_200_without_setting_a_cookie() {
    let full = w::SignInAnswer::Full(
        w::SignInsFull::new(&w::CEILINGS, w::SignInsFullParts { sign_ins: 2, earliest: 400, link_until: 300 }).unwrap(),
    );
    let approval_full = w::ApprovalSignInAnswer::Full(
        w::ApprovalFull::new(&w::CEILINGS, w::ApprovalFullParts { sign_ins: 2, earliest: 400, until: 300 }).unwrap(),
    );
    let mut rows = List::uncapped();
    for record in [w::SignInAnswer::Busy, full] {
        let mut bytes = skein_lib::Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut bytes).unwrap();
        let bytes = bytes.finish();
        for path in [b"/jig/v1/sign-in/link".as_slice(), b"/jig/v1/sign-in/deployment"] {
            rows.push((path, bytes.clone())).expect("sign-in row");
        }
    }
    for record in [
        w::ApprovalSignInAnswer::Busy,
        approval_full,
        w::ApprovalSignInAnswer::Waiting,
        w::ApprovalSignInAnswer::Refused,
        w::ApprovalSignInAnswer::Expired,
        w::ApprovalSignInAnswer::Used,
        w::ApprovalSignInAnswer::Unknown,
        w::ApprovalSignInAnswer::NotReady,
    ] {
        let mut bytes = skein_lib::Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut bytes).unwrap();
        rows.push((b"/jig/v1/sign-in/approval".as_slice(), bytes.finish())).expect("approval row");
    }
    for (path, bytes) in &rows {
        for browser in [false, true] {
            let mut mount = fixtures::mount();
            let mut out = Queue::with_capacity(16);
            let browser_headers = fixtures::browser();
            let headers: &[skein_http::Header] = if browser { &browser_headers } else { &[] };
            let call = fixtures::call(&mut mount, &mut out, Method::Post, path, headers, b"");
            assert_eq!(fixtures::component(&mut out).len(), 1, "{path:?}");
            fixtures::reply(&mut mount, &mut out, call, kit::Reply::Record(bytes.clone()));
            let (status, body, headers) = fixtures::answered(&mut mount, &mut out, call);
            assert_eq!(status, 200, "{path:?}");
            assert_eq!(&body, bytes, "{path:?}");
            assert_eq!(fixtures::header(&headers, b"set-cookie"), None, "{path:?}");
        }
    }
}

#[test]
fn the_public_name_is_admitted_beside_the_listener() {
    for host in [fixtures::PUBLIC, fixtures::HOST] {
        let mut mount = fixtures::public_mount();
        let mut out = Queue::with_capacity(16);
        fixtures::read_by_host(&mut mount, &mut out, host);
        assert_eq!(
            *fixtures::component(&mut out),
            [kit::FromTransport::Exchange {
                browser: false,
                exchange: fixtures::EXCHANGE,
                credential: None,
                message: kit::Message::Read(Box::new([])),
            }],
            "{host:?}"
        );
    }
}

const PUBLIC_MISSES: [&[u8]; 3] = [b"App.example.com", b"app.example.com:443", b"other.example.com"];
#[test]
fn public_name_near_misses_reach_nothing_and_answer_421() {
    for host in PUBLIC_MISSES {
        fixtures::refused_host(&mut fixtures::public_mount(), host);
    }
}

#[test]
fn without_a_public_name_other_hosts_are_refused() {
    for host in [fixtures::PUBLIC, b"other.example.com".as_slice()] {
        fixtures::refused_host(&mut fixtures::mount(), host);
    }
}

#[test]
fn a_browser_public_sign_in_sets_a_secure_cookie() {
    for path in SIGN_IN_PATHS {
        let headers = fixtures::browser_answer(
            fixtures::PUBLIC,
            path,
            fixtures::sign_in_reply(path, &fixtures::TOKEN, fixtures::IN_A_DAY),
        );
        assert_eq!(fixtures::header(&headers, b"set-cookie"), Some(fixtures::COOKIE_SECURE));
    }
}

#[test]
fn a_browser_listener_sign_in_sets_a_plain_cookie() {
    for path in SIGN_IN_PATHS {
        let headers = fixtures::browser_answer(
            fixtures::HOST,
            path,
            fixtures::sign_in_reply(path, &fixtures::TOKEN, fixtures::IN_A_DAY),
        );
        assert_eq!(fixtures::header(&headers, b"set-cookie"), Some(fixtures::COOKIE_ON_LISTENER));
    }
}

#[test]
fn a_browser_public_sign_out_clears_a_secure_cookie() {
    let headers = fixtures::browser_answer(fixtures::PUBLIC, b"/jig/v1/sign-out", kit::Reply::Record(Box::new([0])));
    assert_eq!(fixtures::header(&headers, b"set-cookie"), Some(fixtures::COOKIE_CLEARED_SECURE));
}

#[test]
fn a_browser_listener_sign_out_clears_a_plain_cookie() {
    let headers = fixtures::browser_answer(fixtures::HOST, b"/jig/v1/sign-out", kit::Reply::Record(Box::new([0])));
    assert_eq!(fixtures::header(&headers, b"set-cookie"), Some(fixtures::COOKIE_CLEARED));
}

#[test]
fn via_is_public_for_the_exact_public_name() {
    let mount = fixtures::public_mount();
    let call = fixtures::head(Method::Get, b"/", &fixtures::fields(&[(b"host", fixtures::PUBLIC)]), http::Body::None);
    assert_eq!(via(&mount.settings, &call), Via::Public);
}

#[test]
fn via_is_listener_without_a_public_host_match() {
    for public in [Some(Box::from(fixtures::PUBLIC)), None] {
        let settings = Settings { max_age: 5, prefix: Box::from(b"/jig".as_slice()), public };
        for host in [
            None,
            Some(fixtures::HOST),
            Some(b"App.example.com".as_slice()),
            Some(b"app.example.com:443".as_slice()),
            Some(b"other.example.com".as_slice()),
        ] {
            let headers = match host {
                Some(host) => fixtures::fields(&[(b"host", host)]),
                None => Box::new([]),
            };
            let call = http::Call {
                method: Method::Get,
                target: Box::from(b"/".as_slice()),
                headers,
                body: http::Body::None,
                version: skein_http::Version::Http11,
            };
            assert_eq!(via(&settings, &call), Via::Listener);
        }
    }
}

#[test]
fn without_a_public_name_via_is_always_listener() {
    let mount = fixtures::mount();
    let call = fixtures::head(Method::Get, b"/", &fixtures::fields(&[(b"host", fixtures::PUBLIC)]), http::Body::None);
    assert_eq!(via(&mount.settings, &call), Via::Listener);
}

#[test]
fn public_cookie_helper_sets_a_secure_cookie() {
    let header = fixtures::cookie_for(Via::Public);
    assert_eq!((header.name.as_ref(), header.value.as_ref()), (b"set-cookie".as_slice(), fixtures::COOKIE_SECURE));
}

#[test]
fn public_clear_helper_clears_a_secure_cookie() {
    let header = cleared(&fixtures::mount().settings, Via::Public);
    assert_eq!(
        (header.name.as_ref(), header.value.as_ref()),
        (b"set-cookie".as_slice(), fixtures::COOKIE_CLEARED_SECURE)
    );
}

#[test]
fn listener_cookie_helper_sets_a_plain_cookie() {
    let header = fixtures::cookie_for(Via::Listener);
    assert_eq!((header.name.as_ref(), header.value.as_ref()), (b"set-cookie".as_slice(), fixtures::COOKIE_ON_LISTENER));
}

#[test]
fn listener_clear_helper_clears_a_plain_cookie() {
    let header = cleared(&fixtures::public_mount().settings, Via::Listener);
    assert_eq!((header.name.as_ref(), header.value.as_ref()), (b"set-cookie".as_slice(), fixtures::COOKIE_CLEARED));
}

#[test]
fn authorize_maps_under_the_prefix_and_uses_record_error_statuses() {
    for prefix in [b"/jig".as_slice(), b"/nested/api"] {
        let mut mount = fixtures::prefixed(prefix);
        let mut out = Queue::with_capacity(32);
        let mut path = skein_lib::Writer::new(prefix.len().checked_add(b"/v1/authorize".len()).unwrap());
        path.put(prefix).unwrap();
        path.put(b"/v1/authorize").unwrap();
        let path = path.finish();
        let body = [0, 0, 0, 1, b'q'];
        let call = fixtures::call(&mut mount, &mut out, Method::Post, &path, &fixtures::bearer(), &body);
        assert_eq!(
            *fixtures::component(&mut out),
            [kit::FromTransport::Exchange {
                browser: false,
                exchange: fixtures::EXCHANGE,
                credential: Some(fixtures::credential()),
                message: kit::Message::Authorize(Box::from(body))
            }]
        );
        fixtures::reply(&mut mount, &mut out, call, kit::Reply::NotARecord);
        assert_eq!(fixtures::answered(&mut mount, &mut out, call).0, 400);
    }
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(32);
    let length = usize::try_from(w::Authorize::encoded_bound(&w::CEILINGS).unwrap()).unwrap().checked_add(1).unwrap();
    let call = fixtures::call(
        &mut mount,
        &mut out,
        Method::Post,
        b"/jig/v1/authorize",
        &[],
        &skein_lib::bytes::zeroed(length),
    );
    for request in &out {
        if let Request::Component(_) = request {
            panic!("oversized authorization reaches nothing");
        }
    }
    assert_eq!(fixtures::answered(&mut mount, &mut out, call).0, 413);
    let mut mount = fixtures::mount();
    let header = skein_http::Header {
        name: Box::from(b"content-type".as_slice()),
        value: Box::from(b"application/json".as_slice()),
    };
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/authorize", &[header], b"{}");
    for request in &out {
        if let Request::Component(_) = request {
            panic!("wrong content type reaches nothing");
        }
    }
    assert_eq!(fixtures::answered(&mut mount, &mut out, call).0, 415);
}

mod public;

// The listener's bound is two so that a call and an idle connection can coexist.
const SERVER_CONNECTIONS: u32 = 2;
// One connection reaches the bound with either an idle peer or a partial head.
const SERVER_ONE_CONNECTION: u32 = 1;
const SERVER_FIRST: Token = Token::new(100);
const SERVER_SECOND: Token = Token::new(101);
const SERVER_NEXT: Token = Token::new(102);
const SERVER_HEAD: &[u8] = b"GET /read HTTP/1.1\r\nHost: localhost\r\n\r\n";
fn listener(connections: u32) -> (server::Server, Queue<server::Routed>) {
    let limits = server::Limits {
        connections,
        http: http::Limits { head: 4096, headers: 32, body: 1024, read: 256, response: 4096, send: 256 },
        stall: skein_lib::Duration::from_secs(30),
        peer_end: server::PeerEnd::Connection,
    };
    let mut server = server::Server::new(server_address(), &limits);
    let mut out = Queue::uncapped();
    server.advance(&mut out);
    out.pop();
    (server, out)
}
fn server_address() -> skein_io::Addr {
    skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into())
}
fn accepted(socket: Token) -> skein_io::Event {
    skein_io::Event::Accepted { owner: Token::new(0), socket, peer: server_address() }
}
fn server_settle(server: &mut server::Server, out: &mut Queue<server::Routed>) {
    while server.advance(out) {}
}
fn server_connection(server: &mut server::Server, out: &mut Queue<server::Routed>, socket: Token) -> Token {
    server.up(accepted(socket), out);
    let Some(server::Routed::Io(skein_io::Request::Bind { owner, .. })) = out.pop() else {
        panic!("connection admitted");
    };
    server_settle(server, out);
    server.up(skein_io::Event::Stream { owner, up: Up::Room }, out);
    server_settle(server, out);
    *out = Queue::uncapped();
    owner
}
fn server_head(server: &mut server::Server, out: &mut Queue<server::Routed>, owner: Token, head: &[u8]) {
    for line in head.split_inclusive(|byte| *byte == b'\n') {
        server.up(skein_io::Event::Stream { owner, up: Up::Bytes(line.into()) }, out);
        server_settle(server, out);
    }
}
fn server_call(server: &mut server::Server, out: &mut Queue<server::Routed>, owner: Token) -> Token {
    server_head(server, out, owner, SERVER_HEAD);
    while let Some(routed) = out.pop() {
        if let server::Routed::Call { call, .. } = routed {
            return call;
        }
    }
    panic!("parsed call");
}
fn server_response(server: &mut server::Server, out: &mut Queue<server::Routed>, call: Token) {
    server.answer(call, http::Request::Discard, out);
    server.answer(
        call,
        http::Request::Respond(http::Response {
            status: 200,
            headers: Box::new([]),
            body: http::Body::None,
            close: false,
        }),
        out,
    );
    server_settle(server, out);
}
#[test]
fn a_full_listener_replaces_an_idle_connection_after_its_close() {
    let (mut server, mut out) = listener(SERVER_ONE_CONNECTION);
    let idle = server_connection(&mut server, &mut out, SERVER_FIRST);
    server.up(accepted(SERVER_NEXT), &mut out);
    let Some(server::Routed::Io(skein_io::Request::Abort { entity })) = out.pop() else {
        panic!("idle connection closes instead of rejecting the next");
    };
    assert_eq!(entity, SERVER_FIRST);
    assert!(out.is_empty(), "admission waits for the close terminal");
    server.up(skein_io::Event::Closed { owner: idle }, &mut out);
    let Some(server::Routed::Io(skein_io::Request::Bind { socket, .. })) = out.pop() else {
        panic!("the next connection is admitted");
    };
    assert_eq!(socket, SERVER_NEXT);
}
#[test]
fn a_full_listener_rejects_when_every_connection_has_a_call() {
    let (mut server, mut out) = listener(SERVER_CONNECTIONS);
    for socket in [SERVER_FIRST, SERVER_SECOND] {
        let owner = server_connection(&mut server, &mut out, socket);
        server_call(&mut server, &mut out, owner);
    }
    server.up(accepted(SERVER_NEXT), &mut out);
    let Some(server::Routed::Io(skein_io::Request::Reject { socket })) = out.pop() else {
        panic!("calls leave no idle connection to replace");
    };
    assert_eq!(socket, SERVER_NEXT);
    assert!(out.is_empty(), "no call is closed");
}
#[test]
fn a_call_continues_while_an_idle_connection_is_replaced() {
    let (mut server, mut out) = listener(SERVER_CONNECTIONS);
    let active = server_connection(&mut server, &mut out, SERVER_FIRST);
    let call = server_call(&mut server, &mut out, active);
    let idle = server_connection(&mut server, &mut out, SERVER_SECOND);
    server.up(accepted(SERVER_NEXT), &mut out);
    let Some(server::Routed::Io(skein_io::Request::Abort { entity })) = out.pop() else {
        panic!("idle connection closes");
    };
    assert_eq!(entity, SERVER_SECOND, "the call's connection stays open");
    server.up(skein_io::Event::Closed { owner: idle }, &mut out);
    server_response(&mut server, &mut out, call);
    let mut completed = None;
    while let Some(routed) = out.pop() {
        if let server::Routed::Http { call, event: http::Event::Done(_) } = routed {
            completed = Some(call);
        }
    }
    assert_eq!(completed, Some(call), "the original call still answers");
}
#[test]
fn a_partial_request_is_never_replaced_as_idle() {
    let (mut server, mut out) = listener(SERVER_ONE_CONNECTION);
    let owner = server_connection(&mut server, &mut out, SERVER_FIRST);
    server_head(&mut server, &mut out, owner, b"GET /read HTTP/1.1\r\n");
    out = Queue::uncapped();
    server.up(accepted(SERVER_NEXT), &mut out);
    let Some(server::Routed::Io(skein_io::Request::Reject { socket })) = out.pop() else {
        panic!("a request begun cannot be replaced");
    };
    assert_eq!(socket, SERVER_NEXT);
    assert!(out.is_empty());
}
#[test]
fn a_connection_is_idle_again_after_its_call_is_done() {
    let (mut server, mut out) = listener(SERVER_ONE_CONNECTION);
    let owner = server_connection(&mut server, &mut out, SERVER_FIRST);
    let call = server_call(&mut server, &mut out, owner);
    server_response(&mut server, &mut out, call);
    server.done(call, &mut out);
    server_settle(&mut server, &mut out);
    out = Queue::uncapped();
    server.up(accepted(SERVER_NEXT), &mut out);
    let Some(server::Routed::Io(skein_io::Request::Abort { entity })) = out.pop() else {
        panic!("a completed call leaves its connection idle");
    };
    assert_eq!(entity, SERVER_FIRST);
}
#[test]
fn a_stopped_listener_rejects_the_replacement_after_the_close() {
    let (mut server, mut out) = listener(SERVER_ONE_CONNECTION);
    let owner = server_connection(&mut server, &mut out, SERVER_FIRST);
    server.up(accepted(SERVER_NEXT), &mut out);
    server.stop(&mut out);
    out = Queue::uncapped();
    server.up(skein_io::Event::Closed { owner }, &mut out);
    let Some(server::Routed::Io(skein_io::Request::Reject { socket })) = out.pop() else {
        panic!("a stopped listener cannot admit the waiting connection");
    };
    assert_eq!(socket, SERVER_NEXT);
}

#[test]
fn a_call_ended_before_its_reply_says_its_exchange_gone_once() {
    let exchange = fixtures::EXCHANGE;
    let gone = || -> Box<[kit::FromTransport]> { Box::from([kit::FromTransport::ExchangeGone { exchange }]) };
    for (label, method, path, handed, reply, expected) in [
        ("unanswered exchange", Method::Post, b"/jig/v1/read".as_slice(), true, None, gone()),
        (
            "answered exchange",
            Method::Post,
            b"/jig/v1/read",
            true,
            Some(kit::ToTransport::Reply { exchange, reply: kit::Reply::Record(Box::new([7])) }),
            Box::new([]),
        ),
        ("unfinished body", Method::Post, b"/jig/v1/read", false, None, Box::new([])),
        (
            "stream",
            Method::Get,
            b"/jig/v1/stream",
            true,
            None,
            Box::from([kit::FromTransport::StreamGone { stream: exchange }]),
        ),
        ("public read", Method::Get, b"/jig/v1/public/0/1/1", true, None, gone()),
        (
            "answered public read",
            Method::Get,
            b"/jig/v1/public/0/1/1",
            true,
            Some(kit::ToTransport::Public { exchange, reply: kit::PublicReply::Unknown }),
            Box::new([]),
        ),
    ] {
        let mut mount = fixtures::public_mount();
        // One public call fits the fixture's map, alongside its ordinary room.
        mount.public_limit = 1;
        let mut out = Queue::with_capacity(32);
        mount.down(
            &fixtures::env(),
            Event::Call { call: exchange, head: fixtures::head(method, path, &[], http::Body::None) },
        );
        fixtures::pump(&mut mount, &mut out);
        if handed {
            fixtures::event(&mut mount, &mut out, exchange, http::Event::Body(Up::End));
        }
        fixtures::component(&mut out);
        if let Some(reply) = reply {
            match reply {
                kit::ToTransport::Reply { exchange, reply } => fixtures::reply(&mut mount, &mut out, exchange, reply),
                kit::ToTransport::Public { .. } => {
                    mount.down(&fixtures::env(), Event::Component(reply));
                    fixtures::pump(&mut mount, &mut out);
                }
                kit::ToTransport::Streamed { .. } | kit::ToTransport::StreamEnded { .. } => unreachable!("reply row"),
            }
        }
        fixtures::event(&mut mount, &mut out, exchange, http::Event::Closed);
        fixtures::event(&mut mount, &mut out, exchange, http::Event::Closed);
        let mut messages = List::with_capacity(1);
        let mut next = 0_u32;
        while let Some(request) = out.pop() {
            match request {
                Request::Component(message) => {
                    assert_eq!(next, 0, "{label}: gone before Next");
                    messages.push(message).expect("one gone notification");
                }
                Request::Http { request: http::Request::Next, .. } => next = next.saturating_add(1),
                Request::Http { .. } => {}
            }
        }
        assert_eq!(messages.into_boxed().as_ref(), expected.as_ref(), "{label}");
        assert_eq!(next, 1, "{label}: ended once");
    }
}
