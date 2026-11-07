//! Recorded TUI journeys (temper/docs/design/client/tui/journeys.md, 3.2).
mod line;
mod record;
mod redact;
mod replay;
mod scan;

pub use line::{Line, Sent, Streamed, Unreadable, lines};
pub use record::{Facts, Recorder, event};
pub use redact::{Redacted, fields_mut, originals, redact};
pub use replay::{Mark, Refusal, Replay, Stale, prove};
pub use scan::{Found, Place, codes, scan, scan_bytes};
pub use temper_client_shell_web::journey::{Framing, Kind, Originals, What};

#[cfg(test)]
mod tests;
