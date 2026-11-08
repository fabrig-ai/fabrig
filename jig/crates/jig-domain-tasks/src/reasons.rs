//! Run reasons and offered kinds (jig/docs/design/domain/tasks.md, 8.6;
//! jig/docs/design/protocol/store.md, 5).
use crate::{MessageKind, Party, Word};

/// Why a run runs, and the reasons a charter's part is for: one bit each
/// (jig/docs/design/domain/tasks.md, 8.6; jig/docs/design/protocol/store.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Reasons(pub u32);

impl Reasons {
    pub const EMPTY: Self = Self(0);
    pub const FIRST: Self = Self(1 << 0);
    pub const QUESTION: Self = Self(1 << 1);
    pub const QUESTION_ANSWERED: Self = Self(1 << 2);
    pub const QUESTION_ENDED: Self = Self(1 << 3);
    pub const QUESTION_OWED: Self = Self(1 << 4);
    pub const ESCALATION: Self = Self(1 << 5);
    pub const PROPOSAL: Self = Self(1 << 6);
    pub const PERSON_STALLED: Self = Self(1 << 7);
    pub const RESULT: Self = Self(1 << 8);
    pub const NOTICE: Self = Self(1 << 9);
    pub const NOTHING_AWAITED: Self = Self(1 << 10);
    pub const NEWS: Self = Self(1 << 11);
    pub const TIMER: Self = Self(1 << 12);
    pub const WORDS: Self = Self(1 << 13);
    /// Every known bit: what a run under a charter with no parts is offered.
    pub const ALL: Self = Self((1 << 14) - 1);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// Whether the two share a bit.
    #[must_use]
    pub const fn meets(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    /// No bit outside `ALL`.
    #[must_use]
    pub const fn is_known(self) -> bool {
        self.0 & !Self::ALL.0 == 0
    }
    /// The bit of a kind that chooses parts; `EMPTY` for an always-offered kind.
    #[must_use]
    pub fn of(kind: &MessageKind) -> Self {
        match kind {
            MessageKind::Question => Self::QUESTION,
            MessageKind::QuestionAnswered { .. } => Self::QUESTION_ANSWERED,
            MessageKind::QuestionEnded { .. } => Self::QUESTION_ENDED,
            MessageKind::QuestionOwed => Self::QUESTION_OWED,
            MessageKind::Escalation { .. } => Self::ESCALATION,
            MessageKind::Proposal { .. } => Self::PROPOSAL,
            MessageKind::PersonStalled { .. } => Self::PERSON_STALLED,
            MessageKind::Result(_) => Self::RESULT,
            MessageKind::Notice { .. } => Self::NOTICE,
            MessageKind::NothingAwaited => Self::NOTHING_AWAITED,
            MessageKind::News { .. } => Self::NEWS,
            MessageKind::Timer { .. } => Self::TIMER,
            MessageKind::Words => Self::WORDS,
            MessageKind::Answer { .. }
            | MessageKind::ProposalDecision { .. }
            | MessageKind::SettingsDecision { .. }
            | MessageKind::EffectNotMade { .. }
            | MessageKind::Met { .. }
            | MessageKind::Amendment { .. }
            | MessageKind::FinishRequested { .. } => Self::EMPTY,
        }
    }
    /// `first` or a choosing kind by its store name; `None` for every other name.
    #[must_use]
    pub fn named(name: &[u8]) -> Option<Self> {
        let mut bit = 1_u32;
        for known in REASON_NAMES {
            if name == known {
                return Some(Self(bit));
            }
            bit = bit.checked_shl(1).expect("fourteen known reason bits");
        }
        None
    }
}

/// Store names of `first` and the choosing kinds, in bit order.
pub const REASON_NAMES: [&[u8]; 14] = [
    b"first",
    b"question",
    b"question_answered",
    b"question_ended",
    b"question_owed",
    b"escalation",
    b"proposal",
    b"person_stalled",
    b"result",
    b"notice",
    b"nothing_awaited",
    b"news",
    b"timer",
    b"words",
];
/// Always-offered kinds' store names, which a part's `on` refuses.
pub const ALWAYS_OFFERED_NAMES: [&[u8]; 7] = [
    b"answer",
    b"proposal_decision",
    b"settings_decision",
    b"effect_not_made",
    b"met",
    b"amendment",
    b"finish_requested",
];

/// Whether the run read instructions for this entry's kind (domain/tasks.md, 8.6).
pub(crate) fn covered(word: &Word, offered: Reasons) -> bool {
    offered.meets(Reasons::of(&word.kind))
}

/// Whether `word` goes to a run offered `offered`: always-offered kinds, entries and
/// a person's words always; other kinds when their bit meets the offering
/// (jig/docs/design/domain/tasks.md, 8.6).
#[must_use]
pub fn offered(word: &Word, offered: Reasons) -> bool {
    match &word.kind {
        MessageKind::Answer { .. }
        | MessageKind::ProposalDecision { .. }
        | MessageKind::SettingsDecision { .. }
        | MessageKind::EffectNotMade { .. }
        | MessageKind::Met { .. }
        | MessageKind::Amendment { .. }
        | MessageKind::FinishRequested { .. }
        | MessageKind::Escalation { .. }
        | MessageKind::Proposal { .. } => true,
        MessageKind::Words => match word.from {
            Party::Person(_) => true,
            Party::Task(_) | Party::Deployment { .. } => offered.meets(Reasons::WORDS),
        },
        MessageKind::Question
        | MessageKind::QuestionAnswered { .. }
        | MessageKind::QuestionEnded { .. }
        | MessageKind::QuestionOwed
        | MessageKind::PersonStalled { .. }
        | MessageKind::Result(_)
        | MessageKind::Notice { .. }
        | MessageKind::NothingAwaited
        | MessageKind::News { .. }
        | MessageKind::Timer { .. } => offered.meets(Reasons::of(&word.kind)),
    }
}
