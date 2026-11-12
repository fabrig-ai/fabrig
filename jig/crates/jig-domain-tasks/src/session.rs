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
    Talk,
    Left,
}

/// Facts loaded for a wake, never for a retry or an activation's continuation
/// (jig/docs/design/domain/tasks/standing.md, 5.1–5.3).
#[derive(Debug)]
pub struct SessionWake {
    pub talk: TalkState,
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
    if wake.talk == TalkState::Lapsed {
        Some(SessionEnd::Left)
    } else if wake.talk != TalkState::Open
        && (wake.span == Duration::ZERO
            || wake.at.as_nanos().saturating_sub(session.last.as_nanos()) > wake.span.as_nanos())
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
    if record.talk.is_some() || !record.delegates.is_empty() || !record.questions.is_empty() {
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

/// A person's talk, restored with its task (standing.md, 5.6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Talk {
    pub party: u64,
    pub opened: Wall,
    pub last: Wall,
}

/// More than this span without talk words or a session turn lapses the talk.
pub const TALK_LAPSE: Duration = Duration::from_secs(30 * 60);

/// The talk fact beside a wake's other facts (standing.md, 5.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TalkState {
    None,
    Open,
    Lapsed,
}

#[must_use]
pub fn talk_state(talk: Option<Talk>, session: Option<Session>, at: Wall) -> TalkState {
    let Some(talk) = talk else { return TalkState::None };
    let last = match session {
        Some(session) => talk.last.max(session.last),
        None => talk.last,
    };
    if at.as_nanos().saturating_sub(last.as_nanos()) > TALK_LAPSE.as_nanos() {
        TalkState::Lapsed
    } else {
        TalkState::Open
    }
}

pub(crate) fn leave(record: &mut crate::TaskRecord, at: Wall, lapsed: bool, out: &mut Queue<crate::Request>) {
    if let Some(talk) = record.talk.take() {
        crate::activity::append(record, at, crate::Line::TalkEnded { party: talk.party, lapsed }, out);
        end(record, at, SessionEnd::Left, out);
    }
}
