//! World-supplied model-list replies, independent of scripts (fake-llm.md, 3.2).
use super::{Limits, Server, Service, State};
use alloc::boxed::Box;
use skein_http::{Header, server as http};
use skein_lib::stream::{Down, Read};
use skein_lib::{Env, bytes, len_u64};

pub(super) fn answer(server: &mut Server, service: &Service, env: &Env<Limits>) {
    let (status, body) = match service.config.list_status {
        Some(status) => (status, None),
        None => match &service.config.list {
            Some(body) => (200, Some(bytes::copy_of(body))),
            None => (404, None),
        },
    };
    let framing = match &body {
        Some(body) => http::Body::Length(len_u64(body.len())),
        None => http::Body::None,
    };
    server.requests.push(http::Request::Respond(http::Response {
        status,
        headers: Box::new([
            Header { name: bytes::copy_of(b"content-type"), value: bytes::copy_of(b"application/json") },
            super::failure::request_header(server, &[]),
        ]),
        body: framing,
        close: false,
    }));
    server.state = State::Erring;
    server.reply_offset = 0;
    server.reply = body;
    if let Some(body) = &server.reply {
        let reply = if body.is_empty() {
            Down::Finish
        } else {
            Down::Demand { read: Read::Nothing, room: env.limits.http.send }
        };
        server.requests.push(http::Request::Reply(reply));
    }
}
