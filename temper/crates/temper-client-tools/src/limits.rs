//! The tools' preset and host bounds (client/ctl/README.md, 9).
pub use crate::jig::call::{KEY_BYTES, MOVED_BYTES};
use skein_lib::{Duration, RequestLimits};
use temper_client_domain as client;
pub const ROWS: u32 = 32;
pub const TEXT: u32 = 81_920;
/// The client's text and turn text; jig's wire caps these at 65,536 (`task_text_words`).
/// `task text` joins the engine's chunks (client/ctl/README.md, 9).
pub const CLIENT_TEXT: u32 = 32_768;
/// Whole document bodies and goal words (client/view.md, 4).
pub const LONG_TEXT: u32 = 131_072;
/// jig's portable login-directory name and configured adapter-name bounds.
pub use crate::jig::call::{PROGRAM_ADAPTER, PROGRAM_NAME};
pub const PART: u32 = 65_536;
pub const DOCUMENT: u64 = 1_048_576;
/// A document's bytes can each become six JSON bytes (`\u00XX` in skein's
/// encoder). Reserve the same 1 KiB envelope as the me window for the printed
/// object's digest, continuation and result framing, beyond the document.
pub const RESULT_BYTES: u64 = DOCUMENT * 6 + 1024;
pub const DEADLINE: Duration = Duration::from_secs(30);
pub const LOCK: Duration = Duration::from_secs(60);
pub const LOCK_TRY: Duration = Duration::from_millis(100);
pub const WAIT: Duration = Duration::from_secs(300);
pub const WAIT_MOST: Duration = Duration::from_secs(3600);
/// The tools' client preset, with their windows and retry spans (client/ctl/README.md, 9).
#[must_use]
#[expect(clippy::arithmetic_side_effects, reason = "the request count is the nonzero constant declared here")]
pub fn client() -> client::Limits {
    let requests = RequestLimits {
        requests: 16,                   // Durable keyed requests.
        bytes: 16_777_216,              // Whole request-table payload bytes.
        out: 32,                        // Table outputs per iteration.
        first: Duration::from_secs(1),  // First request retry span.
        most: Duration::from_secs(2),   // Longest request retry span.
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
            handles: 24, // Holds four projects' decided watches.
            watches: 22, // Holds four projects' decided and the shared session watches.
            turn_bytes: 1_048_576,
            backoff: jig_client::Backoff { first: Duration::from_secs(1), most: Duration::from_secs(2) },
            authority: 22, // The stock's 21 executors and one deployment charter fit whole.
            heartbeat: Duration::from_secs(30),
            reads: 32, // Sixteen inbox cards, seven decisions and nine other Home/session reads.
            parts: 16,
            delegates: 80, // Live delegates in a chat's close.
            document_bytes: DOCUMENT,
            records: 16,
            kept: 16,
            projects: 9, // Covers Small and Standard deployment membership capacity.
            text: CLIENT_TEXT,
            long_text: LONG_TEXT,
            turn_text: CLIENT_TEXT,
            rows: ROWS, // Entries in a document's short lists and result windows.
            fields: 16,
            segments: 9,
        },
        exchanges: 8,
    };

    client::Limits {
        forge: client::forge::Limits { field_bytes: 256, checks: 8 },
        client,
        // An account's row names eight uses.
        row_uses: 8,
        window_bytes: None,
    }
}
/// Bounds consumed by the generic tools.
#[must_use]
pub fn tools() -> crate::jig::Limits {
    let preset = client().client;
    let client = preset.domain;
    let mut settings =
        *jig_client_protocol_link::Limits::new(client, preset.exchanges).expect("checked host client limits").wire();
    // The settings form's executor bound is independent of the client's grant capacity.
    settings.delegation_kinds = settings_form::Limits::standard().settings.delegation_kinds;
    crate::jig::Limits {
        rows: client.rows,
        settings,
        program_login: b"temper program-login --config CONFIG --data DATA --cache CACHE",
        application: b"temper",
        ask_bytes: client.ask_bytes,
        result_bytes: RESULT_BYTES,
        text: TEXT,
        words: client.text,
        part: PART,
        wait: WAIT,
        wait_most: WAIT_MOST,
    }
}

#[must_use]
pub fn form() -> settings_form::Limits {
    crate::jig::form_limits(&tools())
}
