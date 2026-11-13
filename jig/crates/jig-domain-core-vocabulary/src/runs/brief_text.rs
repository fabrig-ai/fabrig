//! Brief section vocabulary (jig/docs/design/domain/engine/briefs.md, 1).

/// Limits for rendering one core-owned brief section.
#[derive(Clone, Copy, Debug)]
pub struct BriefTextLimits {
    pub parts: u32,
    pub read_bytes: u32,
    pub brief_bytes: u32,
}

/// A root-owned part of the task context rendered for a brief.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BriefPart {
    Spec,
    /// Received open questions not carried by the run's start (domain/engine/briefs.md, 1.2).
    Waiting {
        carried: u32,
    },
    Dependencies,
    Delegates,
    Attempts,
    TranscriptTail,
    /// Whole requester preferences (domain/engine/briefs.md, 2.1).
    Preferences,
    /// A standing task's Now (domain/tasks/standing.md, 6).
    Now,
}
