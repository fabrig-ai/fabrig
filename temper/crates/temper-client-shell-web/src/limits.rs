//! The page's bounds, the terminal's client bounds (client/README.md, 2).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use skein_lib::{Duration, RequestLimits};
use temper_client_domain as client;
/// Concurrent HTTP exchanges, beside the event stream.
pub const EXCHANGES: u32 = 8;
/// Whole small-store and HTTP response bodies.
pub const BODY_BYTES: u32 = 33_554_432;
/// Recording bytes in memory (client/web/README.md, 2.1): two of `BODY_BYTES`.
pub const RECORDING_BYTES: u32 = 67_108_864;
/// Stream body chunk bytes.
pub const PIECE: u32 = 4096;
/// Field text and a person's conversation words (client/view.md, 4).
pub const CONVERSATION_TEXT: u32 = 512;
/// The person's `long_text` for whole bodies, goal words and turns (client/view.md, 4).
pub const LONG_TEXT: u32 = 131_072;
/// The root's bounds: `temper-client-shell-terminal`'s `native().client`.
#[must_use]
pub fn client() -> client::Limits {
    client_for(false)
}
/// The page's limits with its internal small-window setting (client/web/README.md, 2.1).
#[must_use]
#[expect(clippy::arithmetic_side_effects, reason = "the request count is the nonzero constant declared here")]
pub fn client_for(small_window: bool) -> client::Limits {
    let long_text = if small_window { 32_768 } else { LONG_TEXT };
    let requests = RequestLimits {
        requests: 16,                   // Durable keyed requests.
        bytes: 16_777_216,              // Whole request-table payload bytes.
        out: 32,                        // Table outputs per iteration.
        first: Duration::from_secs(1),  // First request retry span.
        most: Duration::from_secs(30),  // Longest request retry span.
        margin: Duration::from_secs(1), // Retention's safety margin.
    };
    let client = jig_client::Limits {
        domain: jig_client::DomainLimits {
            accounts: jig_client::AccountCounts { accounts: 16, sources: 8, sign_ins: 4 },
            owners: &[jig_client::Spoken {
                family: temper_client_protocol_wire::FORGE,
                version: temper_client_protocol_wire::CLIENT_VERSION,
            }],
            requests,
            // A jig ask reserves this much, so the table holds `requests` asks.
            ask_bytes: requests.bytes / u64::from(requests.requests),
            // Hold four projects' statuses, decided and follows, and the session's own watches.
            handles: 28,
            watches: 26,
            turn_bytes: 1_048_576,
            backoff: jig_client::Backoff { first: Duration::from_secs(1), most: Duration::from_secs(30) },
            authority: 18, // Eighteen executor places size grants, delegation kinds and notes' scopes.
            heartbeat: Duration::from_secs(30),
            reads: 32, // Sixteen inbox cards, seven decisions and nine other Home/session reads.
            parts: 16,
            delegates: 80, // Live delegates in a chat's close.
            document_bytes: 1_048_576,
            records: 16,
            kept: 16,
            projects: 9, // Covers Small and Standard deployment membership capacity.
            text: CONVERSATION_TEXT,
            long_text,
            turn_text: long_text,
            rows: 16, // Entries in a document's short lists.
            fields: 16,
            segments: 9,
        },
        exchanges: EXCHANGES,
    };
    client::Limits {
        forge: client::forge::Limits { field_bytes: 256, checks: 8 },
        client,
        // An account's row names eight uses.
        row_uses: 8,
        window_bytes: if small_window { Some(32_768) } else { None },
    }
}
/// Checked words and the view bounds used at construction and accounting.
#[must_use]
pub fn view(client: &client::Limits) -> Option<(temper_client_view::Limits, temper_client_view::words::Words)> {
    let words = temper_client_view::words::Words::english(
        temper_client_protocol_wire::FORGE,
        client.client.domain.text,
        temper_client_view::words::Shell::Browser,
    )
    .ok()?;
    let view = temper_client_view::Limits::of(client, &words)?;
    Some((view, words))
}
/// The event stream's record and chunk bounds.
#[must_use]
pub fn stream() -> Option<jig_client_protocol_http::Limits> {
    Some(jig_client_protocol_http::Limits { record: BODY_BYTES.checked_add(512)?, chunk: PIECE })
}

/// Refuses unusable client, view or HTTP stream decoder bounds.
#[must_use]
pub fn valid(client: &client::Limits) -> bool {
    let Some(stream) = stream() else { return false };
    client::valid(client) && view(client).is_some() && jig_client_protocol_http::valid(&stream)
}
