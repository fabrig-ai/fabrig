//! Anthropic Messages documents and a bounded streaming decoder.
pub(crate) mod filter;
mod peer;
mod request;
mod response;

pub use crate::dialect::anthropic_decoded_bound as decoded_bound;
pub use crate::wire::{DecodeError, Json, Limits, ProviderError, RateLimit, classify};
pub use crate::{Failure, Stop, Usage};
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

