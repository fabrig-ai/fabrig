//! Whole immutable activity and admitted words (domain/engine/README.md, 5.4, and domain/engine/executors.md, 2;
//! client/domain/contract/documents.md and client/domain/contract/README.md, 5). Commit position orders lines; wall time
//! is supplied metadata. Turns here are archive references, never decoded content.
use super::{FailureTries, HoldFailure, Requester, ResultField, Text};
use skein_lib::{List, Wall};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivityLine {
    pub task: u64,
    pub position: u64,
    pub at: Wall,
    pub line: Activity,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Activity {
    Words { message: u64, by: Requester, kind: MessageKind, words: Text },
    Heard(Word),
    Notified { recipient: Requester, word: Word },
    RunBegan { attempt: u64, why: RunWhy, resumed: bool },
    RunEnded { attempt: u64, end: RunEnd, spent: u64 },
    Turn { reply: u64, attempt: u64, turn: u32, read: Option<u64> },
    Decided { revision: u64, proposal: Option<DecidedProposal> },
    Milestone { position: u64 },
    Stood { attempt: u64, words: Text },
    SessionEnded { attempt: u64, end: SessionEnd },
    Kept { attempt: u64, pending: List<u64>, question: Option<u64> },
}
/// The end of a standing session (jig/docs/design/domain/tasks/standing.md, 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SessionEnd {
    Quiet,
    Bound,
    Unoffered,
    Fresh,
    Closed,
    Rules,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RunWhy {
    Made,
    Ready,
    Message { message: u64 },
    Released,
    Retry { tries: FailureTries },
    Narrowed,
    Changed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RunEnd {
    Parked,
    Finished,
    Cancelled,
    Failed {
        /// Failure class used by the retry policy.
        class: HoldFailure,
        /// Present only for invalid results, including an unreadable result.
        invalid_reason: Option<InvalidReason>,
    },
    Refused,
}
/// Public cause of an invalid run result (protocol/client/wire.md, 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InvalidReason {
    /// The result used a different contract form.
    Form,
    /// The verdict code was outside the contract's choices.
    Verdict,
    /// The result's words exceeded the contract's bound.
    Words,
    /// The change named a different connector or result kind.
    Change,
    /// The result proposed too many follow-up tasks.
    Followups,
    /// The adapter could not decode or translate the result.
    UnreadableResult,
}
/// All bytes are whole admitted words. No text-loss marker exists in this source.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Word {
    pub number: u64,
    pub from: Requester,
    pub kind: MessageKind,
    pub words: Text,
    pub at: Wall,
    pub hits: u32,
    pub eligible: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MessageKind {
    QuestionOwed,
    NothingAwaited,
    SettingsDecision {
        proposal: u64,
        outcome: SettingsOutcome,
    },
    Escalation {
        task: u64,
        revision: u64,
    },
    /// The supplied proposer number has no Task/Person category in this source.
    Proposal {
        proposer: u64,
        proposal: u64,
        kind: MessageProposalKind,
    },
    PersonStalled {
        task: u64,
    },
    ProposalDecision {
        proposal: u64,
        accepted: bool,
    },
    QuestionAnswered {
        question: u64,
    },
    QuestionEnded {
        question: u64,
    },
    Words,
    EffectNotMade {
        entry: u64,
    },
    Amendment {
        revision: u64,
    },
    Question,
    Answer {
        question: u64,
    },
    Notice {
        subscription: u64,
        target: u64,
        state: NoticeState,
    },
    Timer {
        subscription: u64,
    },
    News {
        subscription: u64,
        class: NewsClass,
    },
    Result(WordResult),
    Met {
        dependency: u64,
        removed: bool,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MessageProposalKind {
    Settings,
    Batch,
    Effect,
    Amend,
    Widen,
    Release,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoticeState {
    Held,
    Done,
    Failed,
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NewsClass {
    Wakes,
    Kept,
    Dropped,
}
/// Message metadata, separate from a result document or its connector's part.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WordResult {
    /// Presence is independent of the associated whole words, including empty words.
    Approved {
        state: Text,
        words_present: bool,
    },
    Report,
    Verdict {
        code: u32,
        fields: List<ResultField>,
    },
    Change {
        connector: u16,
        kind: u16,
        resource: u64,
    },
    Failed,
    Cancelled,
}

/// The facts supplied by a settings proposal's durable inbox outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingsOutcome {
    Accepted { revision: u32 },
    Rejected,
    Stale { newest: u32 },
}

/// Which proposal a decision line names, and whether it records its making.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DecidedProposal {
    pub number: u64,
    pub made: bool,
}
