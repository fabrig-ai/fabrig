use super::*;
use jig_client_protocol_link::{Heard, Message};
use skein_http::{Header, Method};
use skein_lib::{Token, bytes};
mod fixtures;
mod stream;
use fixtures::{credential, mapping};
const EXCHANGE: u64 = 11;
const ROUTES: [&[u8]; 13] = [
    b"/jig/v1/request",
    b"/jig/v1/read",
    b"/jig/v1/watch",
    b"/jig/v1/unwatch",
    b"/jig/v1/stream-gone",
    b"/jig/v1/sign-in/link",
    b"/jig/v1/sign-in/deployment",
    b"/jig/v1/sign-out",
    b"/jig/v1/authorize",
    b"/jig/v1/approval/ask",
    b"/jig/v1/approval/read",
    b"/jig/v1/approval/decide",
    b"/jig/v1/sign-in/approval",
];
const STATUSES: [u16; 4] = [201, 401, 500, 503];
#[test]
fn a_keyed_request_is_a_post_under_the_prefix_with_its_bearer() {
    let call = exchange(&mapping(), Some(&credential()), Message::Request(Box::from(*b"keyed")));
    assert_eq!(call.method, Method::Post);
    assert_eq!(call.path.as_ref(), b"/jig/v1/request");
    assert_eq!(call.body.as_ref(), b"keyed");
    let bearer = Header {
        name: Box::from(*b"Authorization"),
        value: Box::from(*b"Bearer CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ"),
    };
    assert!(call.headers.as_slice().contains(&bearer));
    assert!(call.headers.as_slice().contains(&Header {
        name: bytes::copy_of(b"Content-Type"),
        value: bytes::copy_of(b"application/octet-stream"),
    }));
}
#[test]
fn each_message_maps_to_its_route_under_the_prefix() {
    for (message, route) in fixtures::messages().into_iter().zip(ROUTES) {
        let call = exchange(&mapping(), None, message);
        assert_eq!(call.method, Method::Post, "{route:?}");
        assert_eq!(call.path.as_ref(), route, "{route:?}");
        assert!(call.headers.as_slice().contains(&Header { name: bytes::copy_of(b"Content-Type"), value: bytes::copy_of(b"application/octet-stream") }));
    }
}
#[test]
fn a_message_with_no_credential_carries_no_bearer() {
    let call = exchange(&mapping(), None, Message::SignOut);
    assert_eq!(call.headers.len(), 1);
    assert_eq!(call.headers.as_slice()[0].name.as_ref(), b"Content-Type");
}
#[test]
fn a_status_other_than_200_is_lost() {
    for status in STATUSES {
        assert_eq!(
            answered(Token::new(EXCHANGE), status, Box::new([])),
            Heard::Lost { exchange: Token::new(EXCHANGE) },
            "{status}"
        );
    }
}
#[test]
fn a_whole_successful_response_is_a_reply() {
    assert_eq!(
        answered(Token::new(EXCHANGE), 200, Box::from(*b"reply")),
        Heard::Reply { exchange: Token::new(EXCHANGE), message: Box::from(*b"reply") }
    );
}
#[test]
fn a_printed_link_gives_its_host_and_secret() {
    let result = printed(
        &mapping(),
        b"http://127.0.0.1:8080/jig/v1/sign-in/link?secret=AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM",
    )
    .unwrap();
    assert_eq!(result.host.as_ref(), b"127.0.0.1:8080");
    assert_eq!(result.secret.expose(), &fixtures::LINK_SECRET);
}
#[test]
fn a_printed_link_under_another_prefix_is_refused() {
    assert_eq!(
        printed(
            &mapping(),
            b"http://127.0.0.1:8080/other/v1/sign-in/link?secret=AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM"
        ),
        Err(Unlinked::NoSecret)
    );
}

mod public;

#[test]
fn debug_of_a_keyed_call_shows_lengths_never_the_bearer_or_body() {
    const CANARY: &[u8] = b"canary-jig-call-7f3a9c";
    const ENCODED_CREDENTIAL: &[u8] = b"CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ";
    let credential = credential();
    let call = exchange(&mapping(), Some(&credential), Message::SignInLink(bytes::copy_of(CANARY)));
    let compact = alloc::fmt::format(format_args!("{call:?}"));
    let pretty = alloc::fmt::format(format_args!("{call:#?}"));
    for debug in [&compact, &pretty] {
        assert!(bytes::find(debug.as_bytes(), credential.expose()).is_none());
        assert!(bytes::find(debug.as_bytes(), ENCODED_CREDENTIAL).is_none());
        assert!(bytes::find(debug.as_bytes(), CANARY).is_none());
        let normalized = debug.replace([' ', '\n', '\r', '\t'], "");
        for secret in [credential.expose(), ENCODED_CREDENTIAL, CANARY] {
            let numbers = alloc::fmt::format(format_args!("{secret:?}")).replace(' ', "");
            assert!(!normalized.contains(numbers.trim_matches(['[', ']'])));
        }
        let name = alloc::fmt::format(format_args!("{:?}", b"Authorization".as_slice())).replace(' ', "");
        assert!(normalized.contains(name.trim_matches(['[', ']'])));
        assert!(debug.contains("value_len: "));
        assert!(debug.contains("body_len: "));
    }
    // The fixture's 40 bytes encode to 54; "Bearer " adds 7. The canary is 22 bytes.
    assert_eq!(
        compact,
        concat!(
            "Call { method: Post, path: [47, 106, 105, 103, 47, 118, 49, 47, 115, 105, 103, 110, 45, 105, 110, 47, 108, 105, 110, 107], ",
            "headers: [Header { name: [67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101], value_len: 24 }, ",
            "Header { name: [65, 117, 116, 104, 111, 114, 105, 122, 97, 116, 105, 111, 110], value_len: 61 }], body_len: 22 }"
        )
    );
}
