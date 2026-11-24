//! Shared bounded wire and domain translation (smith/docs/design/protocol/channel.md, sections 3–7;
//! smith/docs/design/protocol/charter.md, sections 2 and 5; smith/docs/design/protocol/transcript.md, sections 2–4).
//!
//! Callers supply codec limits and configured endpoint identities. Translation
//! retains no channel lifecycle, addresses or credential values; encode/decode
//! entry points return owned domain or wire values without admitting policy.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(
    clippy::manual_map,
    reason = "closures are out of step code (programming-model.md 10.3), so an Option is mapped by a match"
)]

extern crate alloc;

#[rustfmt::skip]
mod generated {
    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "generated to-record functions uniformly borrow input (skein/docs/design/data/codec/maps.md, 4.1)"
    )]
    pub mod channel {
        include!("generated/channel.rs");
    }
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "generated translations borrow all domain values (data/codec/maps.md, 4.1)")]
    pub mod charter {
        include!("generated/charter.rs");
    }
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "generated translations borrow all domain values (data/codec/maps.md, 4.1)")]
    pub mod transcript {
        include!("generated/transcript.rs");
    }
}

mod answer;
mod channel_hand;
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
pub use transcript::{decode_part, decode_transcript, decode_turn, transcript_piece};
pub use turn::{encode_part, encode_turn};
