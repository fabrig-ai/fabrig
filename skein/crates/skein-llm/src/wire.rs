//! Shared native codec vocabulary (llm/vocabulary.md, section 1, and llm/wire.md, section 1).
mod detail;
mod failure;
pub(crate) mod json;
mod limits;
mod output;
pub(crate) mod reasoning;

pub(crate) use detail::{append, clipped, limit_detail, measured};
pub(crate) use failure::read_error;
pub use failure::{DecodeError, ProviderError, RateLimit, classify, decode_error};
pub use json::Json;
pub use limits::{Limits, encoder_bound};
pub use output::Output;
