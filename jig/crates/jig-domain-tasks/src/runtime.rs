//! Durable lifecycle observations for chips (client/domain/objects.md, section 3).
//! Timestamps come from injected transition events; retries name the decision
//! that paused an activation, rather than a counter selected by a reader.
//! The row owns the attempt identity; a new claim clears both observations.
//! Its start report is root placement/adoption timing, not agent/tool timing.
use crate::Class;
use alloc::boxed::Box;

/// At most this many bytes of a preparation reason are kept (domain/engine/briefs.md, 1.1).
pub const REASON_BYTES: u32 = 128;

/// The decision responsible for the most recent retry pause.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RetryNotice {
    /// A classified failure, with its counter and configured allowance.
    Failed { class: Class, count: u32, allowance: u32, reason: Option<Box<[u8]>> },
    /// Preparation or placement refused before execution; no failure try spent.
    Refused { count: u32, reason: Option<Box<[u8]>> },
}
