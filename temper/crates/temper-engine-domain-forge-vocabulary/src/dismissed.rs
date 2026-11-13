/// Facts a party keeps (temper/docs/design/domain/forge/attention.md, 1).

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PersonFact {
    /// "Skip" on the project's "To begin".
    ToBegin { project: u32 },
    /// The person started `goal` from `chat`'s document.
    GoalFrom { project: u32, chat: u64, goal: u64, at: u64 },
    /// Keep asking for this project and subject, at the client's clock.
    KeepAsking { project: u32, subject: temper_protocol_reaches::Subject, at: u64 },
}

/// A dismissal's answer (temper/docs/design/domain/forge/attention.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DismissAnswer {
    Dismissed,
    TooLarge,
}

/// The subject `dismissed`, outside the task numbers.
pub const DISMISSED_SUBJECT: u64 = 1 << 61;
