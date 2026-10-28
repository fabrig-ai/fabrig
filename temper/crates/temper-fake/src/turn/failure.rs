//! Standalone failing lines, one for each skein error (protocol/setup.md, 8.4).
use crate::{Refusal, read};
use jig_protocol_config::Path;
use skein_fake_llm_domain::api::{Error, Line, Report};
use skein_json::{Document, document::Kind};
use skein_lib::{Duration, List};

pub(super) fn read(d: &Document, at: u32, count: u32, p: &Path) -> Result<Line, Refusal> {
    let mut o = read::object(d, at, p)?;
    let error_at = read::required(&mut o, d, b"error", p)?;
    let credentials_at = read::required(&mut o, d, b"credentials", p)?;
    read::finish(o, d, p)?;
    let error = error(d, error_at, &p.member(b"error"))?;
    let p = p.member(b"credentials");
    let items = match jig_protocol_config::read::list(d, credentials_at, 16) {
        Ok(items) => items,
        Err(w) => return Err(read::kind(&p, w)),
    };
    let mut credentials = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let p = read::index(&p, i);
        let index = read::integer(d, *at, u64::from(u32::MAX), &p)?;
        let index = u32::try_from(index).expect("u32 bound");
        if index >= count {
            return Err(read::why(&p, b"must index an accepted credential"));
        }
        if credentials.as_slice().contains(&index) {
            return Err(read::why(&p, b"must be unique"));
        }
        credentials.push(index).expect("sixteen credentials");
    }
    Ok(Line::Fail { error, report: Report::none(), credentials: credentials.into_boxed() })
}

fn error(d: &Document, at: u32, p: &Path) -> Result<Error, Refusal> {
    if d.token(at).ok_or(read::why(p, b"missing"))?.kind == Kind::String {
        let name = read::text(d, at, 32, p)?;
        for error in [
            Error::Overloaded,
            Error::Unavailable,
            Error::ContextTooLong,
            Error::Unauthorized,
            Error::Refused,
            Error::Unsupported,
            Error::InvalidRequest,
        ] {
            if name.as_ref() == spelling(error) {
                return Ok(error);
            }
        }
        return Err(read::why(p, b"unknown error"));
    }
    let mut o = read::object(d, at, p)?;
    let limited = spelling(Error::RateLimited { retry_after: Duration::ZERO });
    let exhausted = spelling(Error::Exhausted { retry_after: None });
    let limited_at = o.member(d, limited);
    let exhausted_at = o.member(d, exhausted);
    read::finish(o, d, p)?;
    let (name, at, optional) = match (limited_at, exhausted_at) {
        (Some(at), None) => (limited, at, false),
        (None, Some(at)) => (exhausted, at, true),
        _ => return Err(read::why(p, b"expected one error")),
    };
    let p = p.member(name);
    let mut o = read::object(d, at, &p)?;
    let retry_at = read::required(&mut o, d, b"retry_after_ms", &p)?;
    read::finish(o, d, &p)?;
    let p = p.member(b"retry_after_ms");
    if optional && read::is_null(d, retry_at) {
        return Ok(Error::Exhausted { retry_after: None });
    }
    let millis = read::integer(d, retry_at, u64::MAX, &p)?;
    if millis == 0 {
        return Err(read::why(&p, b"must be at least 1"));
    }
    let retry_after = Duration::from_millis(millis);
    Ok(if optional { Error::Exhausted { retry_after: Some(retry_after) } } else { Error::RateLimited { retry_after } })
}

// Encoding direction is exhaustive: a new skein class must acquire a document spelling.
fn spelling(error: Error) -> &'static [u8] {
    match error {
        Error::Overloaded => b"overloaded",
        Error::RateLimited { .. } => b"rate_limited",
        Error::Unavailable => b"unavailable",
        Error::ContextTooLong => b"context_too_long",
        Error::Unauthorized => b"unauthorized",
        Error::Exhausted { .. } => b"exhausted",
        Error::Refused => b"refused",
        Error::Unsupported => b"unsupported",
        Error::InvalidRequest => b"invalid_request",
    }
}
