//! Redacted stream refusals.
//! Keeps only adapter-owned labels and counts, never program payloads.

use alloc::boxed::Box;
use skein_lib::{Decimal, bytes::concat};

/// An admission bound whose unit is fixed by its adapter-owned name.
#[derive(Clone, Copy, Debug)]
pub(super) enum Bound {
    /// Bytes in a provider message identifier.
    MessageId,
    /// Content blocks in one completion.
    Blocks,
    /// Payload bytes retained before call decoding.
    Raw,
    /// Retained bytes including block cells.
    Completion,
    /// Canonical JSON bytes in one tool input.
    Input,
    /// Text bytes in one partial observation.
    Delta,
    /// Retained replies and reserved MCP reply room.
    Reply,
    /// Text bytes in one built-in result.
    ReplyText,
}

impl Bound {
    fn name(self) -> &'static [u8] {
        match self {
            Self::MessageId => b"message id bytes",
            Self::Blocks => b"completion blocks",
            Self::Raw => b"raw completion bytes",
            Self::Completion => b"completion bytes",
            Self::Input => b"tool input bytes",
            Self::Delta => b"partial text bytes",
            Self::Reply => b"retained reply bytes",
            Self::ReplyText => b"built-in result bytes",
        }
    }
}

/// A stream operation refused before its completion can be delivered.
#[derive(Clone, Copy, Debug)]
pub(super) enum Rejection {
    /// Required fields cannot be decoded.
    Malformed,
    /// The record or block kind has no stream translation.
    Unsupported,
    /// The preceding message has not stopped or lacks blocks.
    Boundary,
    /// The result has not supplied a held structured outcome.
    StructuredBoundary,
    /// An entrance refused the measured value against its bound.
    Limit { which: Bound, value: u64, bound: u64 },
    /// The completion queue has no room for the next completion.
    Backlog { value: u64, bound: u64 },
    /// An assembly count cannot be represented.
    Overflow,
}

impl Rejection {
    /// Render only fixed labels and counts; the caller applies its failure-detail ceiling.
    pub fn detail(self, record: &[u8]) -> Box<[u8]> {
        let prefix = concat(&[b"smith: program ", record, b": "]);
        let reason = match self {
            Self::Malformed => b"decode fields: malformed record".as_slice(),
            Self::Unsupported => b"dispatch: unsupported record".as_slice(),
            Self::Boundary => b"close message: incomplete message boundary".as_slice(),
            Self::StructuredBoundary => b"close message: incomplete structured output boundary".as_slice(),
            Self::Backlog { value, bound } => return counts(&prefix, b"completion backlog", value, bound),
            Self::Overflow => b"assembly counter overflow".as_slice(),
            Self::Limit { which, value, bound } => {
                return counts(&prefix, which.name(), value, bound);
            }
        };
        concat(&[&prefix, reason])
    }
}

fn counts(prefix: &[u8], name: &[u8], value: u64, bound: u64) -> Box<[u8]> {
    let value = Decimal::of(value);
    let bound = Decimal::of(bound);
    concat(&[prefix, name, b": value ", value.as_bytes(), b", bound ", bound.as_bytes()])
}
