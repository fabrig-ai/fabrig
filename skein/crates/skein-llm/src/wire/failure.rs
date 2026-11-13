use super::{Json, Limits, detail as common, json};
use crate::{Failure, Provider};
use alloc::boxed::Box;
use skein_json::{Document, Kind};
use skein_lib::{Duration, Wall, bytes};

/// A bounded document entrance refused its grammar, shape or receiving limits.
/// These errors can arise during local admission or an active response decode.
/// See `docs/design/llm/README.md`, Vocabulary and ownership.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DecodeError {
    /// Tokens, bytes or required framing do not form the admitted grammar.
    Malformed,
    /// A required field is absent from an otherwise bounded document.
    Missing,
    /// A field or root value has a type the entrance cannot accept.
    WrongType,
    /// Input, tokens, nesting or measured output exceed caller-supplied limits.
    TooLarge { which: crate::Cap, bound: u64 },
}
impl DecodeError {
    pub(crate) fn limit(which: crate::Cap, bound: u32) -> DecodeError {
        DecodeError::TooLarge { which, bound: u64::from(bound) }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProviderError {
    pub param: Box<[u8]>,
    pub kind: Box<[u8]>,
    pub message: Box<[u8]>,
    pub resets_in_seconds: Option<u64>,
    pub resets_at: Option<u64>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RateLimit {
    pub retry_after: Option<Duration>,
    pub reset: Option<u64>,
    pub exhausted: bool,
}
impl RateLimit {
    pub const NONE: RateLimit = RateLimit { retry_after: None, reset: None, exhausted: false };
    pub fn observe(&mut self, name: &[u8], value: &[u8]) {
        if name.eq_ignore_ascii_case(b"retry-after") {
            self.retry_after = None;
            if let Some(n) = bytes::decimal(value) {
                self.retry_after = Some(Duration::from_secs(n));
            }
        }
    }
    #[must_use]
    pub fn delay(self, error: Option<&ProviderError>, wall: Wall) -> Duration {
        self.reset(error, wall).unwrap_or(Duration::ZERO)
    }
    /// The first supplied reset, or none when the provider gives no reset.
    #[must_use]
    pub fn reset(self, error: Option<&ProviderError>, wall: Wall) -> Option<Duration> {
        if let Some(delay) = self.retry_after {
            return Some(delay);
        }
        if let Some(error) = error {
            if let Some(seconds) = error.resets_in_seconds {
                return Some(Duration::from_secs(seconds));
            }
            if let Some(reset) = error.resets_at {
                return Some(Duration::from_secs(reset.saturating_sub(wall.as_secs())));
            }
        }
        if let Some(reset) = self.reset {
            return Some(Duration::from_secs(reset.saturating_sub(wall.as_secs())));
        }
        None
    }
}
/// A route's named error decides before its HTTP status (llm/README.md, sections 4.7 and 4.8).
/// A 403 without a decoded provider error is unavailable (section 2.5).
#[must_use]
pub fn classify(
    provider: Provider,
    status: u16,
    error: Option<&ProviderError>,
    rate: RateLimit,
    wall: Wall,
) -> Failure {
    let kind = match error {
        Some(error) => error.kind.as_ref(),
        None => b"",
    };
    let named = match provider {
        Provider::Anthropic => anthropic_failure(kind, error, rate, wall),
        Provider::OpenAi => responses_failure(kind, rate, error, wall),
    };
    if let Some(failure) = named {
        return failure;
    }
    match status {
        401 => Failure::Unauthorized,
        403 => match error {
            Some(_) => Failure::Invalid,
            None => Failure::Unavailable,
        },
        404 | 422 => Failure::Invalid,
        429 => Failure::RateLimited { retry_after: rate.delay(error, wall) },
        529 => Failure::Overloaded,
        500..=599 => {
            if kind == b"overloaded_error" {
                Failure::Overloaded
            } else {
                Failure::Unavailable
            }
        }
        400 | 413 => {
            if context(kind, error) {
                Failure::ContextTooLong
            } else {
                Failure::Invalid
            }
        }
        _ => Failure::Unavailable,
    }
}

fn responses_failure(kind: &[u8], rate: RateLimit, error: Option<&ProviderError>, wall: Wall) -> Option<Failure> {
    Some(match kind {
        b"subscription_sharing_usage_limit_exceeded" => {
            Failure::Exhausted { retry_after: error_reset(error, wall).or(rate.retry_after) }
        }
        b"insufficient_quota" => Failure::Exhausted { retry_after: None },
        b"overloaded_error" => Failure::Overloaded,
        b"rate_limit_exceeded" => Failure::RateLimited { retry_after: rate.delay(error, wall) },
        b"subscription_sharing_user_not_eligible"
        | b"chatpass_v2_scope_not_authorized"
        | b"chatpass_v2_invalid_authorization_context" => Failure::Refused,
        b"subscription_sharing_usage_unavailable" | b"subscription_sharing_user_unavailable" => Failure::Unavailable,
        b"subscription_sharing_unsupported_capability" => Failure::Unsupported,
        b"subscription_sharing_route_not_supported" => Failure::Invalid,
        b"subscription_sharing_invalid_user" | b"invalid_api_key" => Failure::Unauthorized,
        b"context_length_exceeded" => Failure::ContextTooLong,
        _ => return None,
    })
}

fn error_reset(error: Option<&ProviderError>, wall: Wall) -> Option<Duration> {
    let error = error?;
    let seconds = match error.resets_in_seconds {
        Some(seconds) => seconds,
        None => error.resets_at?.saturating_sub(wall.as_secs()),
    };
    if seconds == 0 { None } else { Some(Duration::from_secs(seconds)) }
}

fn anthropic_failure(kind: &[u8], error: Option<&ProviderError>, rate: RateLimit, wall: Wall) -> Option<Failure> {
    if kind == b"authentication_error" {
        return Some(Failure::Unauthorized);
    }
    if kind == b"permission_error" {
        return Some(Failure::Refused);
    }
    if kind == b"billing_error" {
        return Some(Failure::Exhausted { retry_after: None });
    }
    if kind == b"rate_limit_error" {
        return Some(Failure::RateLimited { retry_after: rate.delay(error, wall) });
    }
    if kind == b"overloaded_error" {
        return Some(Failure::Overloaded);
    }
    if kind == b"api_error" {
        return Some(Failure::Unavailable);
    }
    if context(kind, error) {
        return Some(Failure::ContextTooLong);
    }
    if kind == b"invalid_request_error" {
        return Some(Failure::Invalid);
    }
    None
}

fn context(kind: &[u8], error: Option<&ProviderError>) -> bool {
    kind == b"context_length_exceeded"
        || kind == b"request_too_large"
        || match error {
            Some(error) => bytes::find(&error.message, b"prompt is too long").is_some(),
            None => false,
        }
}

pub fn decode_error(value: &Json, limits: &Limits) -> Result<ProviderError, DecodeError> {
    let tokens = value.view();
    let error = match json::optional_at(tokens, json::field(tokens, b"error")?)? {
        Some(value) => value,
        None => tokens,
    };
    read_error(error, limits)
}
pub(crate) fn read_error(tokens: (&Document, json::Span), limits: &Limits) -> Result<ProviderError, DecodeError> {
    let kind = match json::optional_at(tokens, json::field(tokens, b"code")?)? {
        Some(value) => {
            if json::kind(value, 0) == Some(Kind::Null) && json::len(value) == 1 {
                json::text(json::value_at(tokens, json::required(tokens, b"type")?)?)?
            } else {
                json::text(value)?
            }
        }
        None => json::text(json::value_at(tokens, json::required(tokens, b"type")?)?)?,
    };
    let message = common::clipped(
        json::text_ref(json::value_at(tokens, json::required(tokens, b"message")?)?)?,
        limits.detail_bytes,
    );
    let resets_in_seconds = match json::optional_at(tokens, json::field(tokens, b"resets_in_seconds")?)? {
        Some(value) => Some(json::unsigned(value)?),
        None => None,
    };
    let resets_at = match json::optional_at(tokens, json::field(tokens, b"resets_at")?)? {
        Some(value) => Some(json::unsigned(value)?),
        None => None,
    };
    let param = match json::optional_text(tokens, b"param")? {
        Some(param) => param,
        None => Box::new([]),
    };
    Ok(ProviderError { param, kind, message, resets_in_seconds, resets_at })
}
