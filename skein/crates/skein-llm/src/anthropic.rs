//! Anthropic subscription Messages documents and a bounded streaming decoder.
pub(crate) mod filter;
pub mod identity;
mod peer;
mod request;
mod response;

pub use crate::openai::{DecodeError, Failure, Json, Limits, ProviderError, RateLimit, Stop, Usage, classify};
pub use peer::{Mark, Request, decode_request, encode_event};
pub(crate) use request::encode_measured_request;
pub use request::{encode_request, measure_request};
pub use response::{
    BlockStart, Delta, Event, Output, Part, StreamDecoder, UsagePatch, decode_error, decode_event, decoder_bound,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod request_tests;

/// Bounds whole-value admission and decoded output.
/// A conservative per-exchange heap bound, including common JSON processing
/// and provider request/response ownership.
#[must_use]
pub fn decoded_bound(limits: &Limits) -> Option<u64> {
    crate::openai::decoded_bound(limits)?
        .checked_add(decoder_bound(limits)?)?
        .checked_add(skein_lib::List::<Mark>::price(4)?.checked_mul(2)?)
}
