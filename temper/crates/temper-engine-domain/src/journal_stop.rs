//! Journal termination (temper/docs/design/domain/root.md, section 4).

/// What stopped the root's journal (domain/root.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JournalStop {
    /// The store's answer to `commit`, outside the commits sent and durable.
    Refused { commit: u64, sent: u64, durable: u64 },
    /// A decision past the store's per-commit write edge.
    PastEdge { writes: u32 },
    /// The deployment's commit counter at its end.
    Counter,
}
