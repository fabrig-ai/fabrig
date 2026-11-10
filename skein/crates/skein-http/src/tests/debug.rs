//! A head prints lengths, never its target or values (skein/docs/design/net/http/README.md, 2).

use alloc::boxed::Box;
use alloc::fmt::format;

use skein_lib::Queue;
use skein_lib::stream::Up;

use super::{LIMITS, Machine, boxed, env, get, header};
use crate::client::{Call, Request};
use crate::server::{self, Server};
use crate::{Method, Version};

/// Neither text nor byte-list rendering may carry a canary, even inside a written head.
fn assert_hidden(debug: &str, pretty: &str, canary: &str) {
    let bytes = format(format_args!("{:?}", canary.as_bytes())).replace(' ', "");
    let sequence = bytes.trim_matches(['[', ']']);
    for printed in [debug, pretty] {
        assert!(!printed.contains(canary), "canary text in Debug: {printed}");
        assert!(!printed.replace([' ', '\n'], "").contains(sequence), "canary bytes in Debug: {printed}");
    }
}

#[test]
fn debug_of_a_header_shows_its_name_and_its_values_length() {
    const CANARY: &str = "canary-header";
    let field = header(b"Authorization", CANARY.as_bytes());
    let debug = format(format_args!("{field:?}"));
    let pretty = format(format_args!("{field:#?}"));
    assert_hidden(&debug, &pretty, CANARY);
    assert_eq!(
        debug,
        "Header { name: [65, 117, 116, 104, 111, 114, 105, 122, 97, 116, 105, 111, 110], value_len: 13 }"
    );
    assert!(pretty.contains("value_len: 13"));
}

#[test]
fn debug_of_a_servers_call_shows_lengths_never_its_target_or_values() {
    const TARGET: &str = "/canary-target";
    const COOKIE: &str = "canary-cookie";
    let call = server::Call {
        method: Method::Get,
        target: boxed(TARGET.as_bytes()),
        version: Version::Http11,
        headers: Box::new([header(b"Cookie", COOKIE.as_bytes())]),
        body: server::Body::None,
    };
    let debug = format(format_args!("{call:?}"));
    let pretty = format(format_args!("{call:#?}"));
    assert_hidden(&debug, &pretty, TARGET);
    assert_hidden(&debug, &pretty, COOKIE);
    assert_eq!(
        debug,
        "Call { method: Get, target_len: 14, version: Http11, headers: [Header { name: [67, 111, 111, 107, 105, 101], value_len: 13 }], body: None }"
    );
    assert!(pretty.contains("target_len: 14") && pretty.contains("value_len: 13"));
}

#[test]
fn debug_of_a_server_reading_a_head_holds_no_target() {
    const CANARY: &str = "canary-server";
    const LIMITS: server::Limits =
        server::Limits { head: 256, headers: 8, body: 1024, read: 16, response: 256, send: 16 };
    let mut server = Server::new(&LIMITS);
    let env = env(LIMITS);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    server::down(&mut server, &env, server::Request::Next, &mut above, &mut below);
    server::up(&mut server, &env, Up::Room, &mut above, &mut below);
    server::up(&mut server, &env, Up::Bytes(boxed(b"GET /canary-server HTTP/1.1\r\n")), &mut above, &mut below);
    let debug = format(format_args!("{server:?}"));
    let pretty = format(format_args!("{server:#?}"));
    assert_hidden(&debug, &pretty, CANARY);
    assert!(debug.contains("RequestLine { method: Get, target_len: 14, version: Http11 }"));
    assert!(pretty.contains("target_len: 14"));
}

#[test]
fn debug_of_a_client_holding_its_head_holds_no_value() {
    const CANARY: &str = "canary-client";
    let mut machine = Machine::new(LIMITS);
    let call = Call {
        headers: Box::new([header(b"Host", b"example.com"), header(b"Authorization", CANARY.as_bytes())]),
        ..get()
    };
    machine.down(Request::Call(call));
    let debug = format(format_args!("{:?}", machine.client));
    let pretty = format(format_args!("{:#?}", machine.client));
    assert_hidden(&debug, &pretty, CANARY);
    assert!(debug.contains("Sending { head_len: Some(67), upload: None }"));
    assert!(pretty.contains("head_len: Some("));

    machine.up(Up::Room);
    let sent = format(format_args!("{:?}", machine.client));
    assert!(sent.contains("Sending { head_len: None, upload: None }"));
}
