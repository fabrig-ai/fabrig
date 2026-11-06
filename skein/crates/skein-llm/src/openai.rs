//! `OpenAI` Responses documents, both sides, and an ordered answer decoder.
mod common;
pub(crate) mod filter;
pub(crate) use crate::wire::json;
pub(crate) use common::{limit_detail, measured};
mod request;
pub(crate) use request::VERBOSITY;
pub(crate) mod response;
pub use crate::{Failure, Stop, Usage};
pub(crate) use common::clipped as clip_detail;
pub use common::{DecodeError, Limits, ProviderError, RateLimit, classify, decoded_bound, encoder_bound};
pub use json::Json;
pub use request::{Input, Request, Role, Tool, decode_request, encode_request, measure_request};
pub use response::{
    Echo, Event, Item, Output, Part, StreamDecoder, decode_error, decode_event, encode_error, encode_event,
    encode_failed, encode_peer_event,
};

#[cfg(test)]
mod tests;
