//! Shared native codec vocabulary (llm.md, sections 2 and 4).
mod detail;
mod failure;
pub(crate) mod json;
mod limits;

pub use limits::{Limits, encoder_bound};
pub use json::Json;
pub(crate) use detail::{append, clipped, limit_detail, measured};
pub use failure::{DecodeError, ProviderError, RateLimit, classify, decode_error};
pub(crate) use failure::read_error;
