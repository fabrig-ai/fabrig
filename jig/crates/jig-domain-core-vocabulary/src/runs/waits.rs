//! A task's waits (jig/docs/design/domain/engine/views.md, 3).

use jig_domain_people as people;
use skein_lib::List;

/// What a task waits on, computed for each document and never stored.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum WaitsOn {
    /// Pending prerequisites, oldest first, bounded by document lines (domain/engine/views.md, 3).
    Dependencies { pending: List<u64>, count: u32 },
    /// The oldest waiting entry and the number of waiting entries in the subtree.
    Party { whom: people::Whom, entries: u32, task: u64, kind: people::EntryKind },
    /// Live delegates, with no party waiting in the subtree.
    Delegates,
    /// A message, a run's resumption or a procedure's facts.
    News,
    /// A working run, a phase that explains its own wait, or an ended task.
    Nothing,
}
