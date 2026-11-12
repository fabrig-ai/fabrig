//! The shared task line, constructed from a live or loaded ended owner row.

use super::{BudgetLine, Chip, Count, PhaseCounts, WaitsOn};
use jig_domain_tasks as tasks;
use skein_lib::Wall;

/// The requesting party's position over a conversation's committed turns.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ConversationRead {
    pub seen: u64,
    pub unread: Count,
}

/// One record for every task list (client/domain/contract/documents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskLine {
    pub chip: Chip,
    pub project: u32,
    pub requester: tasks::Party,
    pub tracked: Option<u32>,
    pub label: u32,
    pub flags: tasks::Flags,
    pub changed: Wall,
    pub last_activity: Wall,
    pub ended_at: Option<Wall>,
    pub budget: BudgetLine,
    pub waits_on: WaitsOn,
    pub phases: PhaseCounts,
    /// Direct delegates ended done (jig/docs/design/client/domain/contract/documents.md, 1).
    pub delegates_done: u32,
    /// Direct delegates live or ended, cancelled ones left out.
    pub delegates_total: u32,
    pub held_below: u32,
    pub conversation: Option<ConversationRead>,
}
