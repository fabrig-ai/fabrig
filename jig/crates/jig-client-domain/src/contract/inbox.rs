//! Exact Inbox owner facts, separate from application supplied context.
//! See client/domain/contract.md, 5–7 and pages.md, 3. Raw row identity is
//! independent of a coarser card address and both continuation cursors.
use super::{Count, Cursor, InboxGroup, ObjectKey, ProposalKey, Proposer, TaskLine, TaskLink};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InboxObject {
    Question { message: u64 },
    Proposal { proposer: Proposer, number: u64 },
    Escalation { revision: u64 },
    PersonTask,
    Reply { message: u64 },
    Result { position: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InboxCursor {
    Number { number: u64 },
    Inbox { task: u64, object: InboxObject, at: Option<Wall> },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InboxEntry {
    pub reply: Option<Box<TaskLine>>,
    pub task: TaskLink,
    pub project: Option<u32>,
    pub object: InboxObject,
    pub activity: Option<u64>,
    pub at: Option<Wall>,
    pub unread: Option<bool>,
    pub cursor: InboxCursor,
}
/// Parallel owner selection facts; roots are self followed by the exact supplied
/// bounded ancestry. A row cursor associates context, never replaces row identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InboxSelection {
    pub cursor: InboxCursor,
    pub goals: List<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Inbox {
    pub party: u64,
    pub needs: List<InboxEntry>,
    pub updates: List<InboxEntry>,
    pub next: Option<InboxCursor>,
    pub cached_needs: u32,
    pub cached_updates: Count,
    pub results_seen: u64,
    pub updates_time_complete: bool,
    /// None is unsupplied; Some(empty) is a supplied empty selection collection.
    pub selection: Option<List<InboxSelection>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InboxPage {
    /// Original retained-row selection facts, independent of current live ancestry.
    pub selection: Option<List<InboxSelection>>,
    pub cut: bool,
    pub rows: List<InboxEntry>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}
/// A whole live envelope; no owner aggregate is split into per-kind counts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InboxDocument {
    Owner(Box<Inbox>),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InboxRow {
    Owner(Box<InboxEntry>),
}
/// Raw scoped row identity, independent of row/read cursors or card identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InboxIdentity {
    Owner { task: u64, object: InboxObject },
}
impl InboxEntry {
    #[must_use]
    pub const fn task_number(&self) -> u64 {
        match &self.task {
            TaskLink::Live(chip) | TaskLink::Retained { chip, .. } => chip.task,
            TaskLink::Stored { number, .. } => *number,
        }
    }
    #[must_use]
    pub const fn group(&self) -> InboxGroup {
        match self.object {
            InboxObject::Question { .. } => InboxGroup::Questions,
            InboxObject::Proposal { .. } => InboxGroup::Proposals,
            InboxObject::Escalation { .. } => InboxGroup::Escalations,
            InboxObject::PersonTask => InboxGroup::PersonTasks,
            InboxObject::Reply { .. } => InboxGroup::Replies,
            InboxObject::Result { .. } => InboxGroup::Results,
        }
    }
    #[must_use]
    pub const fn card_key(&self) -> ObjectKey {
        let task = self.task_number();
        match self.object {
            InboxObject::Question { message } => ObjectKey::Question { task, message },
            InboxObject::Proposal { proposer, number } => ObjectKey::Proposal(ProposalKey { proposer, number }),
            InboxObject::Escalation { .. } => ObjectKey::Escalation(task),
            InboxObject::PersonTask => ObjectKey::PersonTask(task),
            InboxObject::Reply { .. } => ObjectKey::Task(task),
            InboxObject::Result { .. } => ObjectKey::Result(task),
        }
    }
}
impl InboxRow {
    #[must_use]
    pub const fn identity(&self) -> InboxIdentity {
        match self {
            Self::Owner(value) => InboxIdentity::Owner { task: value.task_number(), object: value.object },
        }
    }
    #[must_use]
    pub const fn card_key(&self) -> ObjectKey {
        match self {
            Self::Owner(value) => value.card_key(),
        }
    }
    #[must_use]
    pub const fn group(&self) -> InboxGroup {
        match self {
            Self::Owner(value) => value.group(),
        }
    }
    #[must_use]
    pub const fn project(&self) -> Option<u32> {
        match self {
            Self::Owner(value) => value.project,
        }
    }
    #[must_use]
    pub const fn at(&self) -> Option<Wall> {
        match self {
            Self::Owner(value) => value.at,
        }
    }
    #[must_use]
    pub const fn goal(&self) -> Option<u64> {
        match self {
            Self::Owner(_) => None,
        }
    }
    #[must_use]
    pub fn reply(&self) -> Option<&TaskLine> {
        match self {
            Self::Owner(value) => value.reply.as_deref(),
        }
    }
    #[must_use]
    pub const fn turn_position(&self) -> Option<u64> {
        match self {
            Self::Owner(value) => match value.object {
                InboxObject::Reply { message } => Some(message),
                InboxObject::Question { .. }
                | InboxObject::Proposal { .. }
                | InboxObject::Escalation { .. }
                | InboxObject::PersonTask
                | InboxObject::Result { .. } => None,
            },
        }
    }
    #[must_use]
    pub const fn result_position(&self) -> Option<u64> {
        match self {
            Self::Owner(value) => match value.object {
                InboxObject::Result { position } => Some(position),
                InboxObject::Question { .. }
                | InboxObject::Proposal { .. }
                | InboxObject::Escalation { .. }
                | InboxObject::PersonTask
                | InboxObject::Reply { .. } => None,
            },
        }
    }
}
