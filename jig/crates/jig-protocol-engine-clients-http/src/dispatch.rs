//! Method and route dispatch (protocol/client/default-http.md, 1).
use jig_client_protocol_wire::Record as _;
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_protocol_engine_clients::Message;
#[derive(Debug, PartialEq)]
pub(crate) enum Route {
    Public(jig_protocol_engine_clients::PublicRead),
    Link,
    LinkAsk,
    Service,
    SignInApproval,
    ApprovalAsk,
    ApprovalRead,
    ApprovalDecide,
    SignOut,
    Authorize,
    Request,
    Read,
    Batch,
    Watch,
    Unwatch,
    Gone,
    Stream(Option<u32>),
    BadQuery,
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
    if call.method == skein_http::Method::Get && path.starts_with(b"/v1/stream") {
        return match stream_query(path) {
            Ok((version, _)) => Route::Stream(version),
            Err(()) => Route::BadQuery,
        };
    }
    if call.method != skein_http::Method::Post {
        return Route::Missing;
    }
    match path {
        b"/v1/link/ask" => Route::LinkAsk,
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
        b"/v1/batch" => Route::Batch,
        b"/v1/watch" => Route::Watch,
        b"/v1/unwatch" => Route::Unwatch,
        b"/v1/stream-gone" => Route::Gone,
        _ => Route::Missing,
    }
}
/// Version and predecessor are independent, each present at most once (protocol/client/default-http.md, 2).
fn stream_query(path: &[u8]) -> Result<(Option<u32>, Option<[u8; 16]>), ()> {
    if path == b"/v1/stream" {
        return Ok((None, None));
    }
    let mut query = path.strip_prefix(b"/v1/stream?").ok_or(())?;
    let mut version = None;
    let mut replaces = None;
    loop {
        let (field, rest) = match skein_lib::bytes::find(query, b"&") {
            Some(at) => {
                let (field, tail) = query.split_at_checked(at).expect("located separator");
                (field, Some(tail.strip_prefix(b"&").expect("separator")))
            }
            None => (query, None),
        };
        if let Some(digits) = field.strip_prefix(b"version=") {
            if version.is_some() || digits.is_empty() {
                return Err(());
            }
            let mut value = 0_u32;
            for digit in digits {
                if !digit.is_ascii_digit() {
                    return Err(());
                }
                value = decimal(value, *digit).ok_or(())?;
            }
            version = Some(value);
        } else if let Some(digits) = field.strip_prefix(b"replaces=") {
            if replaces.is_some() || digits.len() != 32 {
                return Err(());
            }
            let mut name = [0; 16];
            for (byte, pair) in name.iter_mut().zip(digits.as_chunks::<2>().0) {
                let mut value = 0_u8;
                for digit in pair {
                    let nibble = match digit {
                        b'0'..=b'9' => digit.checked_sub(b'0').ok_or(())?,
                        b'a'..=b'f' => digit.checked_sub(b'a').ok_or(())?.checked_add(10).ok_or(())?,
                        _ => return Err(()),
                    };
                    value = value.checked_mul(16).ok_or(())?.checked_add(nibble).ok_or(())?;
                }
                *byte = value;
            }
            replaces = Some(name);
        } else {
            return Err(());
        }
        match rest {
            Some(tail) => query = tail,
            None => break,
        }
    }
    Ok((version, replaces))
}
pub(crate) fn stream_replaces(path: &[u8]) -> Result<Option<[u8; 16]>, ()> {
    let (_, replaces) = stream_query(path)?;
    Ok(replaces)
}
pub(crate) fn body_bound(route: &Route, configured: u32) -> u32 {
    match route {
        Route::Authorize => configured.min(
            u32::try_from(w::Authorize::encoded_bound(&w::CEILINGS).expect("record bound")).expect("record fits u32"),
        ),
        Route::Public(_)
        | Route::Link
        | Route::LinkAsk
        | Route::Service
        | Route::SignInApproval
        | Route::ApprovalAsk
        | Route::ApprovalRead
        | Route::ApprovalDecide
        | Route::SignOut
        | Route::Request
        | Route::Batch
        | Route::Read
        | Route::Watch
        | Route::Unwatch
        | Route::Gone
        | Route::Stream(_)
        | Route::BadQuery
        | Route::Missing => configured,
    }
}
pub(crate) fn message(route: &Route, body: Box<[u8]>) -> Option<Message> {
    Some(match route {
        Route::LinkAsk => Message::LinkAsk(body),
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
        Route::Batch => Message::Batch(body),
        Route::Watch => Message::Watch(body),
        Route::Unwatch => Message::Unwatch(body),
        Route::Gone => Message::Gone(body),
        Route::Public(_) | Route::Stream(_) | Route::BadQuery | Route::Missing => return None,
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
    let mut parts = value;
    let Some(credential) = parts.token.take() else { return (body, None) };
    let life = skein_lib::Duration::from_nanos(parts.expires.saturating_sub(wall.as_nanos()));
    let cookie = crate::cookie(settings, call.via, &credential, life);
    let signed_in = parts;
    let body = if call.route == Route::SignInApproval {
        let answer = w::ApprovalSignInAnswer::SignedIn(signed_in);
        answer.encode(&w::CEILINGS).expect("decoded sign-in answer")
    } else {
        let answer = w::SignInAnswer::SignedIn(signed_in);
        answer.encode(&w::CEILINGS).expect("decoded sign-in answer")
    };
    (body, Some(cookie))
}

fn decimal(value: u32, digit: u8) -> Option<u32> {
    value.checked_mul(10)?.checked_add(u32::from(digit.checked_sub(b'0')?))
}

impl Route {
    pub fn stream(&self) -> bool {
        match self {
            Self::Stream(_) => true,
            Self::Public(_)
            | Self::Link
            | Self::LinkAsk
            | Self::Service
            | Self::SignInApproval
            | Self::ApprovalAsk
            | Self::ApprovalRead
            | Self::ApprovalDecide
            | Self::SignOut
            | Self::Authorize
            | Self::Request
            | Self::Read
            | Self::Batch
            | Self::Watch
            | Self::Unwatch
            | Self::Gone
            | Self::BadQuery
            | Self::Missing => false,
        }
    }
}
