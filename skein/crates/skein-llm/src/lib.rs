//! Provider-neutral LLM calls over an externally owned plaintext stream.
//!
//! Supports `OpenAI`'s public Responses route and Anthropic Messages.
//! Socket/TLS ownership, OAuth sign-in and renewal, deadlines and retry
//! policy belong to the caller. See `docs/design/llm/README.md` for the contract.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod affinity;
pub mod anthropic;
pub mod client;
mod derive;
mod dialect;
mod document;
mod filter;
pub mod list;
pub mod openai;
mod replay;
mod translate;
mod types;
mod wire;

#[cfg(test)]
#[path = "tests/list.rs"]
mod tests;

pub use derive::{Declared, ESCAPE, Limits, Rule, TOKEN_BYTES, Violation};
pub use document::DocumentLimits;
pub use replay::{REPLAY_HEADER_BYTES, replay_bound, replay_bytes};
pub use translate::decode as document_error;
pub use types::{
    Affinity, Block, Call, Cap, Completion, Credential, Delta, Encoding, Endpoint, Error, Failure, Message, Phase,
    Prompt, Provider, Replay, Report, Role, Scheme, Stop, Tool, ToolChoice, Usage,
};
/// Bounded document admission errors, independent of the configured wire dialect.
/// Callers use `document_error` to retain limit versus invalid-input refusal.
/// See `docs/design/llm/README.md`, Vocabulary and ownership.
pub use wire::DecodeError as DocumentError;
pub use wire::Json;
