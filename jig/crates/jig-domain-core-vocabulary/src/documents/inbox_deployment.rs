//! Party pages (`client/domain/pages.md`, sections 3 and 4;
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).
//! `client/domain/contract/documents.md` and `client/domain/contract/README.md`, 5.1). Entries are bounded cache views;
//! needs count all live task projections across the party's projects. Reply
//! read positions and stored inbox rows are supplied by their owners.

use super::{Count, Cursor, Proposer, TaskLine, TaskLink};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

/// The object one inbox row opens; its own document carries content and acts.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InboxObject {
    Question { message: u64 },
    Proposal { proposer: Proposer, number: u64 },
    Escalation { revision: u64 },
    PersonTask,
    Reply { message: u64 },
    Result { position: u64 },
}
/// One owner-supplied inbox reference, with a chat header when it is a reply.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InboxEntry {
    pub reply: Option<Box<TaskLine>>,
    pub task: TaskLink,
    pub project: Option<u32>,
    pub object: InboxObject,
    /// Exact child activity to load, separate from a chat ordinal or notice message number.
    pub activity: Option<u64>,
    pub at: Option<Wall>,
    pub unread: Option<bool>,
    pub cursor: Cursor,
    pub projects: List<u32>,
}
impl InboxEntry {
    #[must_use]
    pub fn shows_in(&self, project: u32) -> bool {
        self.project == Some(project) || self.projects.as_slice().contains(&project)
    }
}
/// A bounded page across both groups. Needs precede updates. Known-time needs
/// are oldest first and updates newest first.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inbox {
    pub party: u64,
    pub needs: List<InboxEntry>,
    pub updates: List<InboxEntry>,
    pub next: Option<Cursor>,
    /// Exact needs across every project the party currently holds a role in.
    pub cached_needs: u32,
    pub cached_updates: Count,
    pub results_seen: u64,
    pub updates_time_complete: bool,
    /// None is unsupplied; Some carries complete roots for each emitted row.
    pub selection: Option<List<super::InboxSelection>>,
}
