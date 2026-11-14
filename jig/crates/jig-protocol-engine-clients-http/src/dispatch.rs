//! Method and route dispatch (protocol/client/default-http.md, 1).
use alloc::boxed::Box;
use jig_client_protocol_wire::v22 as w;
use jig_protocol_engine_clients::Message;
#[derive(Debug, PartialEq)]
pub(crate) enum Route {
    Public(jig_protocol_engine_clients::PublicRead),
    Link,
    Service,
    SignInApproval,
    ApprovalAsk,
    ApprovalRead,
    ApprovalDecide,
    SignOut,
    Authorize,
    Request,
    Read,
    Watch,
    Unwatch,
    Gone,
    Stream,
    Missing,
}
pub(crate) fn route(prefix: &[u8], call: &skein_http::server::Call) -> Route {
    let Some(path) = call.target.strip_prefix(prefix) else {
        return Route::Missing;
    };
    if path.starts_with(b"/v1/public/") {
        return match (call.method, crate::public_path(path)) {
            (skein_http::Method::Get, Some(read)) => Route::Public(read),
            _ => Route::Missing,
        };
    }
    if call.method == skein_http::Method::Get && path == b"/v1/stream" {
        return Route::Stream;
    }
    if call.method != skein_http::Method::Post {
        return Route::Missing;
    }
    match path {
        b"/v1/sign-in/link" => Route::Link,
        b"/v1/sign-in/deployment" => Route::Service,
        b"/v1/sign-in/approval" => Route::SignInApproval,
        b"/v1/approval/ask" => Route::ApprovalAsk,
        b"/v1/approval/read" => Route::ApprovalRead,
        b"/v1/approval/decide" => Route::ApprovalDecide,
        b"/v1/request" => Route::Request,
        b"/v1/sign-out" => Route::SignOut,
        b"/v1/authorize" => Route::Authorize,
        b"/v1/read" => Route::Read,
        b"/v1/watch" => Route::Watch,
        b"/v1/unwatch" => Route::Unwatch,
        b"/v1/stream-gone" => Route::Gone,
        _ => Route::Missing,
    }
}
pub(crate) fn body_bound(route: &Route, configured: u32) -> u32 {
    match route {
        Route::Authorize => configured.min(
            u32::try_from(w::Authorize::encoded_bound(&w::CEILINGS).expect("record bound")).expect("record fits u32"),
        ),
        Route::Public(_)
        | Route::Link
        | Route::Service
        | Route::SignInApproval
        | Route::ApprovalAsk
        | Route::ApprovalRead
        | Route::ApprovalDecide
        | Route::SignOut
        | Route::Request
        | Route::Read
        | Route::Watch
        | Route::Unwatch
        | Route::Gone
        | Route::Stream
        | Route::Missing => configured,
    }
}
pub(crate) fn message(route: &Route, body: Box<[u8]>) -> Option<Message> {
    Some(match route {
        Route::Link => Message::SignInLink(body),
        Route::Service => Message::SignInService(body),
        Route::SignInApproval => Message::SignInApproval(body),
        Route::ApprovalAsk => Message::ApprovalAsk(body),
        Route::ApprovalRead => Message::ApprovalRead(body),
        Route::ApprovalDecide => Message::ApprovalDecide(body),
        Route::SignOut => Message::SignOut(body),
        Route::Authorize => Message::Authorize(body),
        Route::Request => Message::Request(body),
        Route::Read => Message::Read(body),
        Route::Watch => Message::Watch(body),
        Route::Unwatch => Message::Unwatch(body),
        Route::Gone => Message::Gone(body),
        Route::Public(_) | Route::Stream | Route::Missing => return None,
    })
}

pub(crate) fn reply(
    settings: &crate::Settings,
    call: &crate::call::Call,
    wall: skein_lib::Wall,
    body: Box<[u8]>,
) -> (Box<[u8]>, Option<skein_http::Header>) {
    if !call.browser {
        return (body, None);
    }
    if call.route == Route::SignOut {
        return (body, Some(crate::cleared(settings, call.via)));
    }
    let mut reader = skein_lib::Reader::new(&body);
    let value = if call.route == Route::SignInApproval {
        let Ok(w::ApprovalSignInAnswer::SignedIn(value)) = w::ApprovalSignInAnswer::decode(&w::CEILINGS, &mut reader)
        else {
            return (body, None);
        };
        value
    } else if call.route == Route::Link || call.route == Route::Service {
        let Ok(w::SignInAnswer::SignedIn(value)) = w::SignInAnswer::decode(&w::CEILINGS, &mut reader) else {
            return (body, None);
        };
        value
    } else {
        return (body, None);
    };
    if reader.remaining() != 0 {
        return (body, None);
    }
    let mut parts = value.into_parts();
    let Some(credential) = parts.token.take() else { return (body, None) };
    let life = skein_lib::Duration::from_nanos(parts.expires.saturating_sub(wall.as_nanos()));
    let cookie = crate::cookie(settings, call.via, &credential, life);
    let signed_in = w::SignedIn::new(&w::CEILINGS, parts).expect("token removed");
    let body = if call.route == Route::SignInApproval {
        let answer = w::ApprovalSignInAnswer::SignedIn(signed_in);
        let mut bytes = skein_lib::Writer::new(skein_lib::length::usize_of(answer.measure()));
        answer.encode(&mut bytes).expect("measured answer");
        bytes.finish()
    } else {
        let answer = w::SignInAnswer::SignedIn(signed_in);
        let mut bytes = skein_lib::Writer::new(skein_lib::length::usize_of(answer.measure()));
        answer.encode(&mut bytes).expect("measured answer");
        bytes.finish()
    };
    (body, Some(cookie))
}
