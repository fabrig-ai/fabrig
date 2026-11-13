//! The core's vocabulary toward its root (`domain/engine/README.md`, section 4).

pub use jig_domain_core_vocabulary::vocabulary::*;

use jig_domain_brief as brief;
use jig_domain_notes as notes;
use skein_lib::List;

/// Core-owned continuations drained within one decision (jig/docs/design/domain/engine/README.md, 4.3).
#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "public entrances move through the decision queue without a separate allocation"
)]
pub(crate) enum Work {
    Event(Event),
    /// A required preparation read or connector section could not be assembled.
    PreparationFailed {
        task: u64,
        failure: brief::GatherFailure,
    },
    /// Begin core-owned brief planning for one preparing task.
    StartBrief {
        task: u64,
    },
    /// Forget a fully read ended task and its read marker in one decision.
    Forget {
        owner: crate::reads::forget::Owner,
    },
    /// The notes child supplied the brief's bounded index before connector gathering.
    BriefNotes {
        task: u64,
        lines: List<notes::Line>,
        more: u32,
    },
}
