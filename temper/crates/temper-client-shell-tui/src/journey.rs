//! Recorded TUI journeys (temper/docs/design/client/tui/journeys.md, 3.2).
mod line;
mod record;
mod replay;
mod walk;

pub use line::{Line, Sent, Streamed, Unreadable, lines};
pub use record::{Facts, Recorder, event};
pub use replay::{Mark, Refusal, Replay, Stale, prove};
pub use walk::{WalkStep, browsed, follows, shows, walk};

#[cfg(test)]
mod tests;
