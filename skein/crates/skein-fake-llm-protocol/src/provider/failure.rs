//! Route-specific failures and script admission (testing/fakes/llm.md, 2.3 and 2.5).
use super::{Config, Error, Limits, Server, Service, State};
use alloc::boxed::Box;
use skein_fake_llm_domain::api;
use skein_http::{Header, server as http};
use skein_lib::stream::{Down, Read};
use skein_lib::{Decimal, Env, List, bytes};
use skein_llm::{Credential, Provider, anthropic, openai};

pub(super) struct Row {
    pub status: u16,
    pub code: &'static [u8],
}

pub(super) fn row(route: Provider, error: api::Error) -> Option<Row> {
    let (status, code): (_, &[u8]) = match route {
        Provider::OpenAi => match error {
            api::Error::Spent => (429, b"insufficient_quota"),
            api::Error::SignedOut => (401, b"token_expired"),
            api::Error::Unserved => (404, b"model_not_found"),
            api::Error::Window { .. } | api::Error::RateLimited { .. } => (429, b"rate_limit_exceeded"),
            api::Error::Cut(_) => return None,
            api::Error::Exhausted { .. } => (429, b"subscription_sharing_usage_limit_exceeded"),
            api::Error::Refused => (403, b"subscription_sharing_user_not_eligible"),
            api::Error::Unsupported => (400, b"subscription_sharing_unsupported_capability"),
            api::Error::Overloaded => (503, b"overloaded_error"),
            api::Error::Unavailable => (503, b"subscription_sharing_usage_unavailable"),
            api::Error::ContextTooLong => (400, b"context_length_exceeded"),
            api::Error::InvalidRequest => (403, b"subscription_sharing_route_not_supported"),
            api::Error::Unauthorized => (401, b"invalid_api_key"),
        },
        Provider::Anthropic => match error {
            api::Error::Spent | api::Error::Exhausted { retry_after: None } => (402, b"billing_error"),
            api::Error::SignedOut | api::Error::Unauthorized => (401, b"authentication_error"),
            api::Error::Unserved => (404, b"not_found_error"),
            api::Error::Window { .. } | api::Error::RateLimited { .. } => (429, b"rate_limit_error"),
            api::Error::Refused => (403, b"permission_error"),
            api::Error::Overloaded => (529, b"overloaded_error"),
            api::Error::Unavailable => (500, b"api_error"),
            api::Error::ContextTooLong | api::Error::InvalidRequest => (400, b"invalid_request_error"),
            api::Error::Cut(_) | api::Error::Exhausted { retry_after: Some(_) } | api::Error::Unsupported => {
                return None;
            }
        },
    };
    Some(Row { status, code })
}

/// Refuses scripts whose failure class or reset has no representation on the route.
///
/// Contract: testing/fakes/llm.md, section 2.3.
pub fn scripts(config: &Config, accepted: &[Credential], scripts: &[api::Script]) -> Result<(), Error> {
    let route = super::route(config, accepted)?;
    for script in scripts {
        for turn in &script.turns {
            for line in &turn.lines {
                match line {
                    api::Line::Fail { error, .. } => {
                        if row(route, *error).is_none() {
                            return Err(Error::Limits);
                        }
                    }
                    api::Line::Drawn { .. }
                    | api::Line::Cut { .. }
                    | api::Line::Take { .. }
                    | api::Line::Text { .. }
                    | api::Line::Opaque { .. }
                    | api::Line::Call { .. }
                    | api::Line::Pieces { .. } => {}
                }
            }
        }
    }
    Ok(())
}

pub(super) fn request_header(server: &Server, request: &[u8]) -> Header {
    let number = Decimal::of(server.response_id);
    let mut generated = List::with_capacity(24);
    for byte in b"req_".iter().chain(number.as_bytes()) {
        generated.push(*byte).expect("bounded request id");
    }
    Header {
        name: bytes::copy_of(match server.route {
            Provider::OpenAi => b"x-request-id",
            Provider::Anthropic => b"request-id",
        }),
        value: if request.is_empty() { generated.into_boxed() } else { bytes::copy_of(request) },
    }
}

pub(super) fn response_error(server: &mut Server, _service: &Service, env: &Env<Limits>, failure: api::Failure) {
    let error = failure.error;
    let Some(Row { status, .. }) = row(server.route, error) else { return };
    let body = body(server.route, &failure, env);
    let mut headers = List::with_capacity(5);
    headers
        .push(Header { name: bytes::copy_of(b"content-type"), value: bytes::copy_of(b"application/json") })
        .expect("content type");
    headers.push(request_header(server, &failure.report.request)).expect("request id");
    match error {
        api::Error::Window { reset, limit, remaining } => {
            for header in window_headers(server.route, reset, limit, remaining, env.wall) {
                headers.push(header).expect("room for window headers");
            }
        }
        api::Error::RateLimited { retry_after } => {
            let seconds = Decimal::of(retry_after.as_nanos().div_euclid(1_000_000_000));
            headers
                .push(Header { name: bytes::copy_of(b"retry-after"), value: bytes::copy_of(seconds.as_bytes()) })
                .expect("retry delay");
        }
        api::Error::Spent
        | api::Error::SignedOut
        | api::Error::Unserved
        | api::Error::Cut(_)
        | api::Error::Exhausted { .. }
        | api::Error::Overloaded
        | api::Error::Unavailable
        | api::Error::ContextTooLong
        | api::Error::Unauthorized
        | api::Error::Refused
        | api::Error::Unsupported
        | api::Error::InvalidRequest => {}
    }
    server.requests.push(http::Request::Respond(http::Response {
        status,
        headers: headers.into_boxed(),
        body: http::Body::Length(skein_lib::len_u64(body.len())),
        close: false,
    }));
    server.reply = Some(body);
    server.reply_offset = 0;
    server.state = State::Erring;
    server.requests.push(http::Request::Reply(Down::Demand { read: Read::Nothing, room: env.limits.http.send }));
}

fn matches_window(error: api::Error) -> bool {
    match error {
        api::Error::Window { .. } => true,
        api::Error::Spent
        | api::Error::SignedOut
        | api::Error::Unserved
        | api::Error::Cut(_)
        | api::Error::Exhausted { .. }
        | api::Error::RateLimited { .. }
        | api::Error::Overloaded
        | api::Error::Unavailable
        | api::Error::ContextTooLong
        | api::Error::Unauthorized
        | api::Error::Refused
        | api::Error::Unsupported
        | api::Error::InvalidRequest => false,
    }
}

fn reset_duration(delay: skein_lib::Duration) -> Box<[u8]> {
    let nanos = delay.as_nanos();
    if nanos < 1_000_000_000 {
        return bytes::concat(&[Decimal::of(nanos.div_ceil(1_000_000)).as_bytes(), b"ms"]);
    }
    let seconds = nanos.div_ceil(1_000_000_000);
    if nanos < 60_000_000_000 {
        return bytes::concat(&[Decimal::of(seconds).as_bytes(), b"s"]);
    }
    bytes::concat(&[
        Decimal::of(seconds.div_euclid(60)).as_bytes(),
        b"m",
        Decimal::of(seconds.rem_euclid(60)).as_bytes(),
        b"s",
    ])
}

fn reset_wall(delay: skein_lib::Duration, wall: skein_lib::Wall) -> Box<[u8]> {
    let seconds = wall.as_nanos().saturating_add(delay.as_nanos()).div_ceil(1_000_000_000);
    let at = skein_lib::Wall::from_nanos(seconds.saturating_mul(1_000_000_000));
    let mut basic = skein_lib::Writer::new(16);
    at.write_basic_utc(&mut basic).expect("room for basic UTC");
    let basic = basic.finish();
    bytes::concat(&[
        basic.get(..4).expect("year"),
        b"-",
        basic.get(4..6).expect("month"),
        b"-",
        basic.get(6..11).expect("day and hour"),
        b":",
        basic.get(11..13).expect("minute"),
        b":",
        basic.get(13..).expect("second and Z"),
    ])
}

fn window_headers(
    route: Provider,
    reset: skein_lib::Duration,
    limit: u64,
    remaining: u64,
    wall: skein_lib::Wall,
) -> [Header; 3] {
    let (limit_name, remaining_name, reset_name, reset) = match route {
        Provider::OpenAi => (
            b"x-ratelimit-limit-tokens".as_slice(),
            b"x-ratelimit-remaining-tokens".as_slice(),
            b"x-ratelimit-reset-tokens".as_slice(),
            reset_duration(reset),
        ),
        Provider::Anthropic => (
            b"anthropic-ratelimit-tokens-limit".as_slice(),
            b"anthropic-ratelimit-tokens-remaining".as_slice(),
            b"anthropic-ratelimit-tokens-reset".as_slice(),
            reset_wall(reset, wall),
        ),
    };
    [
        Header { name: bytes::copy_of(limit_name), value: bytes::copy_of(Decimal::of(limit).as_bytes()) },
        Header { name: bytes::copy_of(remaining_name), value: bytes::copy_of(Decimal::of(remaining).as_bytes()) },
        Header { name: bytes::copy_of(reset_name), value: reset },
    ]
}

fn rounded(delay: skein_lib::Duration) -> u64 {
    delay.as_nanos().div_ceil(1_000_000_000)
}

pub(super) fn provider_error(route: Provider, failure: &api::Failure) -> openai::ProviderError {
    let kind = if failure.report.code.is_empty() {
        row(route, failure.error).expect("admitted failure class").code
    } else {
        &failure.report.code
    };
    let message = if !failure.message.is_empty() {
        failure.message.as_ref()
    } else if route == Provider::Anthropic && failure.error == api::Error::ContextTooLong {
        b"prompt is too long".as_slice()
    } else {
        kind
    };
    let resets_in_seconds = if route == Provider::Anthropic {
        None
    } else {
        match failure.error {
            api::Error::Exhausted { retry_after: Some(delay) } => Some(rounded(delay)),
            api::Error::Spent
            | api::Error::SignedOut
            | api::Error::Unserved
            | api::Error::Window { .. }
            | api::Error::Cut(_)
            | api::Error::Exhausted { retry_after: None }
            | api::Error::RateLimited { .. }
            | api::Error::Overloaded
            | api::Error::Unavailable
            | api::Error::ContextTooLong
            | api::Error::Unauthorized
            | api::Error::Refused
            | api::Error::Unsupported
            | api::Error::InvalidRequest => None,
        }
    };
    openai::ProviderError {
        param: if route == Provider::OpenAi && failure.error == api::Error::Unserved && failure.report.field.is_empty()
        {
            bytes::copy_of(b"model")
        } else {
            failure.report.field.clone()
        },
        kind: bytes::copy_of(kind),
        message: bytes::copy_of(message),
        resets_in_seconds,
        resets_at: None,
    }
}

fn body(route: Provider, failure: &api::Failure, env: &Env<Limits>) -> Box<[u8]> {
    let error = provider_error(route, failure);
    let limits = match route {
        Provider::OpenAi => &env.limits.documents.openai,
        Provider::Anthropic => &env.limits.documents.anthropic,
    };
    if !failure.message.is_empty() || (route == Provider::OpenAi && matches_window(failure.error)) {
        let bounded = limits.writer_limits();
        let kind = if matches_window(failure.error) && route == Provider::OpenAi {
            b"tokens".as_slice()
        } else {
            row(route, failure.error).expect("admitted failure class").code
        };
        let mut measure = skein_json::writer::Encoder::measure(&bounded);
        write_message(&mut measure, kind, &error);
        let Ok(len) = measure.measured() else { return bytes::copy_of(b"{}") };
        let mut write = skein_json::writer::Encoder::write(len, &bounded);
        write_message(&mut write, kind, &error);
        return write.finish();
    }
    match openai::encode_error(&error, limits) {
        Ok(body) => body,
        Err(
            anthropic::DecodeError::Malformed
            | anthropic::DecodeError::Missing
            | anthropic::DecodeError::WrongType
            | anthropic::DecodeError::TooLarge { .. },
        ) => bytes::copy_of(b"{}"),
    }
}

/// A provider message keeps the route's type separate from its report code.
fn write_message(out: &mut skein_json::writer::Encoder, kind: &[u8], error: &openai::ProviderError) {
    out.object_start();
    out.key(b"type");
    out.string(b"error");
    out.key(b"error");
    out.object_start();
    out.key(b"type");
    out.string(kind);
    out.key(b"code");
    out.string(&error.kind);
    out.key(b"message");
    out.string(&error.message);
    if !error.param.is_empty() {
        out.key(b"param");
        out.string(&error.param);
    }
    out.object_end();
    out.object_end();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents;
    use skein_lib::Duration;
    use skein_llm::Provider::{Anthropic, OpenAi};

    fn rows() -> [(Provider, api::Error, u16, &'static [u8]); 18] {
        let none = api::Error::Exhausted { retry_after: None };
        let rate = api::Error::RateLimited { retry_after: Duration::ZERO };
        [
            (OpenAi, none, 429, b"subscription_sharing_usage_limit_exceeded"),
            (Anthropic, none, 402, b"billing_error"),
            (OpenAi, rate, 429, b"rate_limit_exceeded"),
            (Anthropic, rate, 429, b"rate_limit_error"),
            (OpenAi, api::Error::Refused, 403, b"subscription_sharing_user_not_eligible"),
            (Anthropic, api::Error::Refused, 403, b"permission_error"),
            (OpenAi, api::Error::Unsupported, 400, b"subscription_sharing_unsupported_capability"),
            (OpenAi, api::Error::Overloaded, 503, b"overloaded_error"),
            (Anthropic, api::Error::Overloaded, 529, b"overloaded_error"),
            (OpenAi, api::Error::Unavailable, 503, b"subscription_sharing_usage_unavailable"),
            (Anthropic, api::Error::Unavailable, 500, b"api_error"),
            (OpenAi, api::Error::ContextTooLong, 400, b"context_length_exceeded"),
            (Anthropic, api::Error::ContextTooLong, 400, b"invalid_request_error"),
            (OpenAi, api::Error::InvalidRequest, 403, b"subscription_sharing_route_not_supported"),
            (Anthropic, api::Error::InvalidRequest, 400, b"invalid_request_error"),
            (OpenAi, api::Error::Unauthorized, 401, b"invalid_api_key"),
            (Anthropic, api::Error::Unauthorized, 401, b"authentication_error"),
            (
                OpenAi,
                api::Error::Exhausted { retry_after: Some(Duration::ZERO) },
                429,
                b"subscription_sharing_usage_limit_exceeded",
            ),
        ]
    }

    fn bounds() -> Limits {
        let native = skein_llm::Limits::derive(&skein_llm::Declared {
            window: 171,
            output: 4096,
            reasoning_item: 2048,
            tool_payload: 342,
            calls_per_response: 7,
            conversations: 1,
            tools: 174,
            history_items: 684,
        })
        .expect("small model")
        .dialect(OpenAi)
        .native();
        Limits {
            calls: 4,
            http: http::Limits { head: 4096, headers: 32, body: 32768, read: 256, response: 4096, send: 128 },
            sse: skein_http::sse::writer::Limits { event: 16384, chunk: 128 },
            documents: documents::Limits { anthropic: native, openai: native, model_ceiling: 4096 },
        }
    }

    #[test]
    fn account_faults_render_their_status_code_field_and_headers() {
        for (route, error, status, code) in [
            (OpenAi, api::Error::Spent, 429, b"insufficient_quota".as_slice()),
            (Anthropic, api::Error::Spent, 402, b"billing_error"),
            (OpenAi, api::Error::SignedOut, 401, b"token_expired"),
            (Anthropic, api::Error::SignedOut, 401, b"authentication_error"),
            (OpenAi, api::Error::Unserved, 404, b"model_not_found"),
            (Anthropic, api::Error::Unserved, 404, b"not_found_error"),
            (
                OpenAi,
                api::Error::Window { reset: Duration::from_secs(42), limit: 1000, remaining: 40 },
                429,
                b"rate_limit_exceeded",
            ),
            (
                Anthropic,
                api::Error::Window { reset: Duration::from_secs(42), limit: 1000, remaining: 40 },
                429,
                b"rate_limit_error",
            ),
        ] {
            let row = row(route, error).expect("account class row");
            assert_eq!((row.status, row.code), (status, code));
            let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: bounds() };
            let service = Service::new(config(), &env.limits).expect("peer limits");
            let mut server = Server::new(skein_lib::Token::new(1), &env.limits).expect("server limits");
            server.route = route;
            response_error(&mut server, &service, &env, error.into());
            let Some(http::Request::Respond(head)) = server.requests.pop() else { panic!("response head") };
            assert_eq!(head.status, status);
            let mut request_id = false;
            let mut window_headers = 0_u32;
            let prefix = if route == OpenAi { b"x-ratelimit-".as_slice() } else { b"anthropic-ratelimit-" };
            for header in &head.headers {
                assert!(!header.is(b"retry-after"));
                request_id |= header.is(if route == OpenAi { b"x-request-id" } else { b"request-id" });
                window_headers += u32::from(header.name.starts_with(prefix));
            }
            assert!(request_id);
            let body = server.reply.expect("error document");
            assert!(bytes::find(&body, code).is_some());
            if error == api::Error::Unserved && route == OpenAi {
                assert!(bytes::find(&body, b"\"param\":\"model\"").is_some());
            }
            if matches_window(error) {
                assert_eq!(window_headers, 3);
                if route == OpenAi {
                    assert!(bytes::find(&body, b"\"type\":\"tokens\"").is_some());
                }
            }
        }
    }

    #[test]
    fn cuts_send_a_success_head_and_deltas_without_closing_the_last_block_or_stream() {
        for route in [OpenAi, Anthropic] {
            for parts in [
                Box::<[api::Part]>::from([]),
                Box::new([api::Part::Text { text: bytes::copy_of(b"text before cut") }]),
                Box::new([api::Part::ToolCall {
                    id: bytes::copy_of(b"call"),
                    name: bytes::copy_of(b"tool"),
                    arguments: bytes::copy_of(b"{\"x\":"),
                }]),
                Box::new([
                    api::Part::Text { text: bytes::copy_of(b"before") },
                    api::Part::ToolCall {
                        id: bytes::copy_of(b"call"),
                        name: bytes::copy_of(b"tool"),
                        arguments: bytes::copy_of(b"{\"x\":"),
                    },
                ]),
            ] {
                let count = u32::try_from(parts.len()).expect("few parts");
                let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: bounds() };
                let mut server = Server::new(skein_lib::Token::new(1), &env.limits).expect("server limits");
                server.route = route;
                server.answer = Some(api::Answer { parts, finish: api::Finish::Stop, usage: api::Usage::NONE });
                server.pieces.fail(api::Error::Cut(api::Cut::BeforeOutput).into());
                crate::provider::pieces::head(&mut server);
                let Some(http::Request::Respond(head)) = server.requests.pop() else { panic!("success head") };
                assert_eq!(head.status, 200);
                assert_eq!(head.body, http::Body::Chunked);
                let mut names = List::with_capacity(8);
                for _ in 0_u32..8 {
                    if let Some(event) = crate::provider::pieces::terminal(&mut server, &env).expect("cut events") {
                        names.push(event.name).expect("bounded cut events");
                    } else {
                        break;
                    }
                }
                assert_eq!(
                    names.as_slice().first().expect("opening").as_ref(),
                    if route == OpenAi { b"response.created".as_slice() } else { b"message_start" }
                );
                if names.len() > 1 {
                    assert_eq!(names.len(), count * 3);
                    assert!(names.last().expect("delta").ends_with(b"delta"));
                }
                assert!(server.answer.is_none(), "the writer next ends the body whole");
                for name in &names {
                    assert!(
                        !name.ends_with(b"completed")
                            && !name.ends_with(b"failed")
                            && !name.ends_with(b"message_stop")
                            && !name.ends_with(b"message_delta")
                    );
                }
            }
        }
    }

    #[test]
    fn window_resets_use_provider_durations_and_ceiled_installed_utc() {
        for (nanos, expected) in
            [(249_000_001, b"250ms".as_slice()), (41_000_000_001, b"42s"), (89_000_000_001, b"1m30s")]
        {
            let headers = window_headers(OpenAi, Duration::from_nanos(nanos), 1000, 40, skein_lib::Wall::EPOCH);
            assert_eq!(headers[0].value.as_ref(), b"1000");
            assert_eq!(headers[1].value.as_ref(), b"40");
            assert_eq!(headers[2].value.as_ref(), expected);
        }
        let wall = skein_lib::Civil { year: 2026, month: 11, day: 20 }.wall().expect("UTC midnight");
        let wall = skein_lib::Wall::from_nanos(
            wall.as_nanos() + 23 * 3_600_000_000_000 + 59 * 60_000_000_000 + 59_900_000_000,
        );
        let headers = window_headers(Anthropic, Duration::from_millis(250), 1000, 40, wall);
        assert_eq!(headers[2].name.as_ref(), b"anthropic-ratelimit-tokens-reset");
        assert_eq!(headers[2].value.as_ref(), b"2026-11-21T00:00:01Z");
    }

    #[test]
    fn each_class_renders_its_routes_status_and_code() {
        for (route, error, status, code) in rows() {
            let got = row(route, error).expect("the class has a row");
            assert_eq!((got.status, got.code), (status, code), "{route:?} {error:?}");
        }
    }

    fn config() -> Config {
        Config {
            list: None,
            list_status: None,
            without_credential: false,
            provider: documents::Provider::Anthropic,
            path: b"/v1/messages".as_slice().into(),
            headers: Box::new([]),
            echo: openai::Echo::NONE,
            usage_fields: documents::UsageFields::ALL,
        }
    }

    fn script(error: api::Error, credentials: Box<[u32]>, after: bool) -> api::Script {
        let fail = api::Line::Fail { error, report: api::Report::none(), credentials };
        let lines: Box<[_]> = if after {
            Box::new([api::Line::Text { text: b"text".as_slice().into() }, fail])
        } else {
            Box::new([fail])
        };
        api::Script {
            cue: b"cue".as_slice().into(),
            turns: Box::new([api::Turn { lines, finish: api::Finish::Stop, tokens: 1 }]),
        }
    }

    #[test]
    fn a_class_without_its_row_is_refused() {
        let script = script(api::Error::Unsupported, Box::new([]), false);
        assert_eq!(scripts(&config(), &[Credential::key(b"token".as_slice().into())], &[script]), Err(Error::Limits));
    }

    #[test]
    fn an_anthropic_exhausted_reset_has_no_row_and_is_refused() {
        let error = api::Error::Exhausted { retry_after: Some(Duration::from_secs(1)) };
        assert!(row(Anthropic, error).is_none());
        for after in [false, true] {
            for credentials in [Box::<[u32]>::from([]), Box::<[u32]>::from([0])] {
                let script = script(error, credentials, after);
                assert_eq!(
                    scripts(&config(), &[Credential::key(b"key".as_slice().into())], &[script]),
                    Err(Error::Limits)
                );
            }
        }
    }
}
