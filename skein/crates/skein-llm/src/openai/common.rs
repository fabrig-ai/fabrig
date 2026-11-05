use crate::{
    Failure, Provider,
    openai::{Input, Tool},
};
use alloc::boxed::Box;
use skein_json::{Compact, collector, document, tokenizer, writer};
use skein_lib::{Duration, List, Wall, bytes};

pub use crate::wire::{Limits, encoder_bound};

/// Bounds whole-value admission and decoded output.
/// Conservative native whole-value admission and decoded output bounds.
/// Request encoding uses `encoder_bound`, without compact copies.
#[must_use]
pub fn decoded_bound(limits: &Limits) -> Option<u64> {
    // Native temporary values belong to the request, answer or replay
    // budgets. A discarded wire event contributes no document-sized buffer.
    let temporary = limits.request.max(limits.answer).max(limits.input).max(limits.reasoning).max(limits.metadata);
    let tokens = document::document_bound(&document::Limits { tokens: limits.tokens, text: temporary })?;
    // Standalone whole-value request decoding admits caller documents here.
    // Every JSON token costs at least one byte in that request budget.
    // Its collector and the decoded native documents may briefly coexist.
    let request_count = u64::from(limits.request);
    let request_tokens = request_count.checked_mul(u64::try_from(size_of::<Compact>()).ok()?)?.checked_mul(2)?;
    let answer_records = u64::from(limits.output_items)
        .checked_mul(u64::from(limits.tokens))?
        .min(u64::from(limits.answer))
        .checked_mul(u64::try_from(size_of::<Compact>()).ok()?)?
        .checked_mul(2)?;
    // Whole-value collection remains a neutral JSON admission entrance for
    // schemas and replay metadata. The client prices its error body separately.
    let admission = collector::collector_bound(
        &collector::Limits {
            tokenizer: limits.tokenizer_limits(),
            tokens: limits.tokens,
            text: temporary,
            skip: u64::from(temporary),
        },
        &[],
        &collector::Filter { root: collector::Keep::Value },
    )?;
    let request_slots =
        List::<Input>::price(limits.history_items)?.checked_add(List::<Tool>::price(limits.tools)?)?.checked_mul(2)?;
    let documents = u64::from(temporary).checked_mul(8)?;
    let answer = u64::from(limits.answer).checked_mul(4)?;
    tokens
        .checked_mul(4)?
        .checked_add(request_tokens)?
        .checked_add(answer_records)?
        .checked_add(admission)?
        .checked_add(request_slots)?
        .checked_add(documents)?
        .checked_add(answer)?
        .checked_add(u64::from(limits.request))?
        .checked_add(tokenizer::tokenizer_bound(&limits.tokenizer_limits())?)?
        .checked_add(writer::encoder_bound(&writer::Limits { depth: limits.depth, length: temporary })?)?
        .checked_add(crate::openai::response::decoder_bound(limits)?)
}
/// A bounded document entrance refused its grammar, shape or receiving limits.
/// These errors can arise during local admission or an active response decode.
/// See `docs/design/llm.md`, Vocabulary and ownership.
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
/// A route's named error decides before its HTTP status (llm.md, sections 4.7 and 4.8).
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
        403 | 404 | 422 => Failure::Invalid,
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

pub(crate) fn append(out: &mut List<u8>, bytes: &[u8], cap: crate::Cap) -> Result<(), DecodeError> {
    if bytes.len() > usize::try_from(out.room()).expect("u32 fits usize") {
        return Err(DecodeError::limit(cap, out.capacity()));
    }
    for &b in bytes {
        if out.push(b).is_err() {
            return Err(DecodeError::limit(cap, out.capacity()));
        }
    }
    Ok(())
}
pub(crate) fn clipped(bytes: &[u8], limit: u32) -> Box<[u8]> {
    let mut count = bytes.len().min(usize::try_from(limit).expect("u32 fits usize"));
    // Tokenizer strings are UTF-8. A prefix ends before a continuation byte,
    // so truncating diagnostics never creates a new malformed string.
    for _back in 0..4_u32 {
        match bytes.get(count) {
            Some(byte) if byte & 0xc0 == 0x80 => count = count.saturating_sub(1),
            Some(_) | None => break,
        }
    }
    bytes::copy_of(bytes.get(..count).expect("within bytes"))
}

pub(crate) fn measured(encoder: writer::Encoder, limits: writer::Limits, cap: crate::Cap) -> Result<u32, DecodeError> {
    match encoder.measured() {
        Ok(len) => Ok(len),
        Err(writer::Refusal::TooLong) => Err(DecodeError::limit(cap, limits.length)),
        Err(writer::Refusal::TooDeep) => Err(DecodeError::limit(crate::Cap::Depth, limits.depth)),
        Err(writer::Refusal::Text | writer::Refusal::Number) => Err(DecodeError::Malformed),
    }
}

/// The typed cap is the source of a local limit's diagnostic.
pub(crate) fn limit_detail(which: crate::Cap, bound: u64) -> Box<[u8]> {
    let digits = skein_lib::Decimal::of(bound);
    let length = which
        .name()
        .len()
        .checked_add(digits.as_bytes().len())
        .expect("fixed diagnostic prefix fits usize")
        .checked_add(b" exceeds bound ".len())
        .expect("fixed diagnostic fits usize");
    let mut out = skein_lib::Writer::new(length);
    out.put(which.name()).expect("measured cap name");
    out.put(b" exceeds bound ").expect("measured separator");
    out.put(digits.as_bytes()).expect("measured bound");
    out.finish()
}
