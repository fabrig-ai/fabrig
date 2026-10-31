//! Entrance and reclaim observations for root handoff order scenarios.
//! The recorder copies facts and request identities; it never schedules work.
//! Contract: smith/docs/design/domain/run.md, sections 3.2, 5.3 and 13;
//! skein/docs/design/draft/dynamic-memory.md, section 4.6.

use skein_lib::{Time, Token};
use smith_domain::{Fact, Request, llm};

/// An entrance into the root or its iteration's reclaim point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entrance {
    /// One external event was handled.
    Step,
    /// The host cancelled the active run.
    Cancel,
    /// One eligible ready item was resumed.
    Resume,
    /// One due alarm was fired.
    Fire,
    /// The root and its children crossed the reclaim point.
    Reclaim,
}

/// A root output relevant to causal run translation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandoffRequest {
    /// Main's concrete turn reached the root output.
    Turn { sequence: u32 },
    /// A session requested its next provider completion.
    Complete { owner: Token, name: llm::CompletionName, thread: u32 },
    /// A pending provider completion was cancelled.
    Cancel { owner: Token },
}

impl HandoffRequest {
    pub(crate) fn from_request(request: &Request) -> Option<Self> {
        if let Request::Turn { turn, .. } = request {
            Some(Self::Turn { sequence: turn.sequence })
        } else if let Request::Complete { owner, name, prompt, .. } = request {
            Some(Self::Complete { owner: *owner, name: *name, thread: prompt.affinity.thread })
        } else if let Request::Cancel { owner } = request {
            Some(Self::Cancel { owner: *owner })
        } else {
            None
        }
    }
}

/// Facts and ordered outputs of one completed root entrance, before the down pass.
/// Round numbers count iterations even when the injected clock does not advance.
#[derive(Clone, Debug)]
pub struct EntranceObservation {
    pub round: u64,
    pub at: Time,
    pub entrance: Entrance,
    /// Whether another ready item is eligible at this observation point.
    pub ready: bool,
    pub facts: Vec<Fact>,
    pub requests: Vec<HandoffRequest>,
}

/// Optional recording state, independent of the world's schedule.
#[derive(Debug, Default)]
pub(crate) struct Recorder {
    pub(crate) round: u64,
    pub(crate) facts: Vec<Fact>,
    pub(crate) entrances: Vec<EntranceObservation>,
}
