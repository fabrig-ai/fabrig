//! Shared bounded wire and domain translation (protocol/channel.md, sections 3–7;
//! protocol/charter.md, sections 2 and 5; protocol/transcript.md, sections 2–4).
//!
//! Callers supply codec limits and configured endpoint identities. Translation
//! retains no channel lifecycle, addresses or credential values; encode/decode
//! entry points return owned domain or wire values without admitting policy.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod charter;
mod credentials;
mod endpoints;
mod error;
mod result;
mod transcript;
mod translate;
mod turn;

pub use charter::encode_charter;
pub use credentials::{GrantValue, program_name};
pub use endpoints::{Endpoint, Endpoints};
pub use error::Error;
pub use result::decode_declared;
pub use transcript::{decode_transcript, decode_turn, transcript_piece};
pub use translate::{
    DecodedStart, Grant, Mount, answer_record, decode_charter, encode_result, host_path, invalid_start,
    refreshed_grant, saved_delivery, start_context,
};
pub use turn::encode_turn;
