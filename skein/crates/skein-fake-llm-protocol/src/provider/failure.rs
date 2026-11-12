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
            api::Error::Exhausted { .. } => (429, b"subscription_sharing_usage_limit_exceeded"),
            api::Error::RateLimited { .. } => (429, b"rate_limit_exceeded"),
            api::Error::Refused => (403, b"subscription_sharing_user_not_eligible"),
            api::Error::Unsupported => (400, b"subscription_sharing_unsupported_capability"),
            api::Error::Overloaded => (503, b"overloaded_error"),
            api::Error::Unavailable => (503, b"subscription_sharing_usage_unavailable"),
            api::Error::ContextTooLong => (400, b"context_length_exceeded"),
            api::Error::InvalidRequest => (403, b"subscription_sharing_route_not_supported"),
            api::Error::Unauthorized => (401, b"invalid_api_key"),
        },
        Provider::Anthropic => match error {
            api::Error::RateLimited { .. } => (429, b"rate_limit_error"),
            api::Error::Exhausted { retry_after: None } => (402, b"billing_error"),
            api::Error::Refused => (403, b"permission_error"),
            api::Error::Overloaded => (529, b"overloaded_error"),
            api::Error::Unavailable => (500, b"api_error"),
            api::Error::ContextTooLong | api::Error::InvalidRequest => (400, b"invalid_request_error"),
            api::Error::Unauthorized => (401, b"authentication_error"),
            api::Error::Exhausted { retry_after: Some(_) } | api::Error::Unsupported => return None,
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
                    api::Line::Take { .. }
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
    let mut headers = List::with_capacity(4);
    headers
        .push(Header { name: bytes::copy_of(b"content-type"), value: bytes::copy_of(b"application/json") })
        .expect("content type");
    headers.push(request_header(server, &failure.report.request)).expect("request id");
    match error {
        api::Error::RateLimited { retry_after } => {
            let seconds = Decimal::of(retry_after.as_nanos().div_euclid(1_000_000_000));
            headers
                .push(Header { name: bytes::copy_of(b"retry-after"), value: bytes::copy_of(seconds.as_bytes()) })
                .expect("retry delay");
        }
        api::Error::Exhausted { .. }
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
            api::Error::Exhausted { retry_after: None }
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
        param: failure.report.field.clone(),
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
    if !failure.message.is_empty() {
        let bounded = limits.writer_limits();
        let kind = row(route, failure.error).expect("admitted failure class").code;
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
