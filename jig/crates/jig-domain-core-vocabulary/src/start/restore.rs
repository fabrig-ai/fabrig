//! Store rows enter through the start that owns their child routing (jig/docs/design/domain/engine/restart.md).

/// The information a root needs after routing one restored row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Restore {
    Kept,
    Refused,
    Deployment {
        commits: u64,
    },
    /// A people row queued for the root's next sweep decision.
    Swept,
}
