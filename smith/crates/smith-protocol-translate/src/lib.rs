//! Shared bounded wire and domain translation (smith/docs/design/protocol/channel.md, sections 3–7;
//! smith/docs/design/protocol/charter.md, sections 2 and 5; smith/docs/design/protocol/transcript.md, sections 2–4).
//!
//! Callers supply codec limits and configured endpoint identities. Translation
//! retains no channel lifecycle, addresses or credential values; encode/decode
//! entry points return owned domain or wire values without admitting policy.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod answer;
mod charter;
mod credentials;
mod decode_charter;
mod delivery;
mod endpoints;
mod error;
mod host_path;
mod result;
mod start;
mod transcript;
mod turn;

pub use answer::{answer_record, invalid_start};
pub use charter::encode_charter;
pub use credentials::refreshed_grant;
pub use credentials::{GrantValue, program_name};
pub use decode_charter::decode_charter;
pub use delivery::saved_delivery;
pub use endpoints::{Endpoint, Endpoints};
pub use error::Error;
pub use host_path::host_path;
pub use result::decode_declared;
pub use result::encode_result;
pub use start::{DecodedStart, Grant, Mount, start_context};
pub use transcript::{decode_transcript, decode_turn, transcript_piece};
pub use turn::encode_turn;
