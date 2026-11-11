//! A standing task's session record and ends (jig/docs/design/domain/tasks/standing.md, 5 and 7).
use crate::Reasons;
use skein_lib::{Duration, Queue, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Session {
    pub first: u64,
    pub reasons: Reasons,
    pub last: Wall,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SessionEnd {
    Quiet,
    Bound,
    Unoffered,
    Fresh,
    Closed,
    Rules,
}

/// Facts loaded for a wake, never for a retry or an activation's continuation
/// (jig/docs/design/domain/tasks/standing.md, 5.1–5.3).
#[derive(Debug)]
pub struct SessionWake {
    pub at: Wall,
    pub span: Duration,
    pub bytes: u64,
    pub bound: u64,
    /// Waking kinds excluding always-offered entries and a person's words.
    pub kinds: Reasons,
    pub offered: Reasons,
    pub resumable: bool,
}

/// Nine tenths of the cache life, in nanoseconds, without rounding to milliseconds.
#[must_use]
pub fn quiet_span(cache_life_ms: u32) -> Duration {
    Duration::from_nanos(u64::from(cache_life_ms).checked_mul(900_000).expect("u32 milliseconds fit nanoseconds"))
}

/// Half a token window at four bytes per token.
#[must_use]
pub fn context_bound(tokens: u32) -> u64 {
    u64::from(tokens).checked_mul(2).expect("u32 token window fits bytes")
}

/// The first applicable end at a wake; closed work and changed rules are judged
/// at their committing decisions (jig/docs/design/domain/tasks/standing.md, 5.2).
#[must_use]
pub fn session_end(session: Session, wake: SessionWake) -> Option<SessionEnd> {
    if wake.span == Duration::ZERO || wake.at.as_nanos().saturating_sub(session.last.as_nanos()) > wake.span.as_nanos()
    {
        Some(SessionEnd::Quiet)
    } else if wake.bytes > wake.bound {
        Some(SessionEnd::Bound)
    } else if !wake.offered.contains(wake.kinds) {
        Some(SessionEnd::Unoffered)
    } else if !wake.resumable {
        Some(SessionEnd::Fresh)
    } else {
        None
    }
}

pub(crate) fn end(record: &mut crate::TaskRecord, at: Wall, end: SessionEnd, out: &mut Queue<crate::Request>) {
    if let Some(session) = record.session.take() {
        crate::activity::append(record, at, crate::Line::SessionEnded { attempt: session.first, end }, out);
    }
}

pub(crate) fn work_closed(record: &crate::TaskRecord) -> bool {
    if !record.delegates.is_empty() || !record.questions.is_empty() {
        return false;
    }
    if let Some(proposal) = &record.proposal {
        match proposal.state {
            crate::ProposalState::Pending { .. } => return false,
            crate::ProposalState::Accepted { .. }
            | crate::ProposalState::Rejected { .. }
            | crate::ProposalState::Withdrawn => {}
        }
    }
    true
}
