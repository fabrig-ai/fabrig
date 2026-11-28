//! The core's vocabulary toward its root (`domain/engine/vocabulary.md`, section 1).

pub use jig_domain_core_vocabulary::vocabulary::*;

use jig_domain_brief as brief;
use jig_domain_notes as notes;
use skein_lib::List;

/// Queue a native connector ask under the core's synchronous hand-off mark.
pub(crate) fn connector_ask(connector: u16, ask: jig_domain_contract::Ask) -> Request {
    Request::Ask { connector, ask: Ask::Connector(ask) }
}

/// Core-owned continuations drained within one decision (jig/docs/design/domain/engine/vocabulary.md, 1.3).
#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "public entrances move through the decision queue without a separate allocation"
)]
pub(crate) enum Work {
    Event(Event),
    Store(crate::engine::store::In),
    Connector(u16, jig_domain_contract::Up),
    /// Resume a report-ready batch without resetting its first park's deadline.
    Resources(crate::effects::resources::Batch),
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
