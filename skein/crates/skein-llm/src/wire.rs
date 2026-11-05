//! Shared native codec vocabulary (llm.md, sections 2 and 4).
pub(crate) mod json;
mod limits;

pub use limits::{Limits, encoder_bound};
pub use json::Json;
