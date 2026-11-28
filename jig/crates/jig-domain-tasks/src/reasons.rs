//! Run reasons and offered kinds (jig/docs/design/domain/tasks/messages.md, 6;
//! jig/docs/design/protocol/store/records.md).
pub use jig_domain_contract::ops::{REASON_NAMES, Reasons};

use crate::{MessageKind, Party, Word};

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

/// Whether the run read instructions for this entry's kind (domain/tasks/messages.md, 6).
pub(crate) fn covered(word: &Word, offered: Reasons) -> bool {
    offered.meets(Reasons::of(&word.kind))
}

/// Whether `word` goes to a run offered `offered`: always-offered kinds, entries and
/// a person's words always; other kinds when their bit meets the offering
/// (jig/docs/design/domain/tasks/messages.md, 6).
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
        | MessageKind::PassedQuestion { .. }
        | MessageKind::PassedAnswer { .. }
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
