//! Agent's host channel opening and Start translation (protocol/channel.md,
//! sections 2, 3 and 5; protocol/charter.md, sections 2 and 5).
//!
//! The component keeps Skein's framed channel, the run's channel phase and configured
//! endpoint names. It never knows host policy, credential values, addresses or
//! stream handles. `new`, `from_below`, `fire`, `send_admitted` and `send_answer`
//! return lower demands and a decoded Start with owned domain charter policy.
//!
//! | State | Input | Output |
//! |---|---|---|
//! | Opening | Open/Ready | Agree version and receive terms, then Opened |
//! | Opened | Start | Resolve charter, transcript and grants, or answer invalid |
//! | Started | Message/Cancel | Deliver a labelled message or the single cancel |
//! | Started | Admitted | Send admission; retain the live request and turn window |
//! | Admitted | `HostAnswer`/`GrantRefresh` | Settle a call or retain a new generation |
//! | Admitted | Acknowledge | Release the durably kept turn prefix |
//! | Started/Admitted | Answer | Send the final answer and finish writes |
//! | Answered | Acknowledge | Release remaining turn credit |
//! | Any | End/Closed | Emit the independent Ended terminal once |

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(any(test, feature = "fixture"))]
pub mod fixture;

mod calls;
mod component;
mod credentials;
mod facts;
mod limits;
mod live;

#[cfg(test)]
mod tests;

pub use component::{Component, OpenEvent};
pub use limits::{EdgeRoom, Error, Limits, edge_room, schema, schema_for_start, valid};
pub use live::{Piece, PieceKind};

pub use smith_protocol_translate::{
    DecodedStart, Endpoint, Endpoints, Grant, GrantValue, Mount, answer_record, decode_charter, decode_declared,
    decode_transcript, decode_turn, encode_charter, encode_result, encode_turn, host_path, program_name, start_context,
    transcript_piece,
};
