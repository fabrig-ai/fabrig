//! Kept project deletion (jig/docs/design/domain/core.md, 11;
//! jig/docs/design/domain/people.md, 5.2).

/// A project's deletion, kept until its last step (draft 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Removal {
    pub project: u32,
    pub stage: RemovalStage,
}

/// Where a deletion stands; each stage resumes at a start.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RemovalStage {
    /// Its live tasks cancelled; waiting for each to end.
    Ending,
    /// Its ended tasks being erased whole.
    Erasing,
    /// Its connectors told; waiting for each `ProjectReleased`.
    Releasing,
}
