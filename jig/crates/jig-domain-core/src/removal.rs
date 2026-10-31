//! Project deletion kept across starts (temper/docs/design/draft/playground-reset.md, 5).
use crate::Core;

/// A project's deletion, kept until its last stage (playground-reset.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProjectRemoval {
    pub project: u32,
    pub stage: RemovalStage,
}

/// Its stages, in order.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RemovalStage {
    /// Closed; its live tasks are being cancelled.
    Ending,
    /// Its ended tasks erased whole, scanning ended tasks after this number.
    Erasing { after: Option<u64> },
    /// Its notes erased.
    Notes,
    /// Its own rows erased, its connectors told; the row goes in that decision.
    Rows,
}

impl Core {
    /// Whether a project's deletion is kept until whole.
    #[must_use]
    pub fn removing(&self, project: u32) -> bool {
        self.removals.contains_key(&project)
    }

    /// The kept deletion's current stage.
    #[must_use]
    pub fn removal(&self, project: u32) -> Option<&ProjectRemoval> {
        self.removals.get(&project)
    }
}
