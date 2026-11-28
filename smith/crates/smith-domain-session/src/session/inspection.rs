//! Read-only views of a session's waits, calls and room. The harness, not these reads,
//! checks how the records agree (skein/docs/design/testing/inspection.md, 2).

use alloc::boxed::Box;
use skein_lib::{Id, Map, Time, Token};

use super::{By, Run, Session, Slot, State, Tools, builtin, history};
use crate::Domain;

/// A session's state as it pertains to provider and retry waits.
#[derive(Clone, Copy, Debug)]
pub enum InspectionState {
    /// The session requested a completion, due by the request's whole deadline.
    Completing { deadline: Time },
    /// The session cancelled a completion and still needs its terminal.
    Cancelling,
    /// The session retries when the rate window or transient failure backoff ends.
    Backoff { until: Time, rate_limited: bool },
    /// The session is neither in a provider wait nor backing off.
    Other,
    /// The session ended; reclamation may not yet have freed its slot.
    Closed,
}

/// The session's recorded transcript and reservation counters, in bytes.
#[derive(Clone, Copy, Debug)]
pub struct RoomHeld {
    pub bytes: u64,
    pub reserved: u64,
    pub provider: Option<u64>,
}

impl Session {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_opener(&self) -> Token {
        self.conversation.opener
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_kit(&self) -> Token {
        self.conversation.kit
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_state(&self) -> InspectionState {
        match self.state {
            State::Completing { .. } => InspectionState::Completing { deadline: self.conversation.expires },
            State::Backoff { until, rate_limited, .. } => InspectionState::Backoff { until, rate_limited },
            State::Closing { waiting, .. } if waiting.completion => InspectionState::Cancelling,
            State::Tooling { .. } | State::Resting { .. } | State::Closing { .. } | State::Yielded { .. } => {
                InspectionState::Other
            }
            State::Closed => InspectionState::Closed,
        }
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_room(&self) -> RoomHeld {
        let conversation = &self.conversation;
        RoomHeld {
            bytes: conversation.bytes,
            reserved: conversation.reserved,
            provider: conversation.provider.credit(),
        }
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_batch(&self) -> Option<&history::Batch> {
        self.tools()?.batch.as_ref()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_running_slot(&self, slot: u32) -> Option<Id<Run>> {
        match self.tools()?.slots.get(slot)? {
            Slot::Running { run } => Some(*run),
            Slot::Done { .. } | Slot::Unstarted | Slot::Withheld => None,
        }
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_program(&self) -> &Map<Box<[u8]>, builtin::Claim> {
        self.conversation.builtin.inspection_entries()
    }

    fn tools(&self) -> Option<&Tools> {
        match &self.state {
            State::Tooling { tools } | State::Resting { tools } => Some(tools),
            State::Closing { tools, .. } | State::Yielded { tools } => tools.as_ref(),
            State::Completing { .. } | State::Backoff { .. } | State::Closed => None,
        }
    }
}

impl Run {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_session(&self) -> Id<Session> {
        self.session
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_position(&self) -> (u32, u32) {
        (self.block, self.slot)
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_call(&self) -> Option<&crate::ToolCall> {
        match self.by {
            By::Tools | By::Opener if !self.ended => Some(&self.metadata),
            By::Tools | By::Opener | By::Program => None,
        }
    }
}

impl Domain {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn provider_credit_for_test(&self, owner: Token) -> Option<(u64, Option<u64>)> {
        let session = self.sessions.get(Id::from_token(owner))?;
        let conversation = &session.conversation;
        Some((conversation.reserved, conversation.provider.credit()))
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn result_credit_for_test(&self, owner: Token) -> Option<(u64, Option<u64>, u64, u64)> {
        let session = self.sessions.get(Id::from_token(owner))?;
        let batch_credit = match session.inspection_batch() {
            Some(batch) => batch.inspection_credit()?,
            None => 0,
        };
        let conversation = &session.conversation;
        let program = conversation.builtin.inspection_credit()?;
        Some((conversation.reserved, conversation.provider.credit(), batch_credit, program))
    }
}
