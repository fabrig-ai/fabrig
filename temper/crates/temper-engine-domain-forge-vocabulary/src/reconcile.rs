//! Reference reconciliation before Switch (domain/root.md, 8.1; protocol/store.md, 2).
use crate::{Key, Limits, Name};
use skein_forge::api::Repository;

/// A restored record's family.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RecordKind {
    Forge,
    Call,
    ProposedEffect,
    Repository,
    Hold,
    Names,
    Subscription,
    BranchHead,
    PullState,
    Ci,
    Landed,
    Entry,
    Client,
    Closer,
    Change,
    Issue,
    Release,
    Application,
    Writer,
    Binding,
    Setting,
    Flight,
    Landing,
    Receipt,
    Follow,
    Tier,
    Verification,
    LandingLog,
    Making,
    Dismissed,
    ProjectFacts,
    Looked,
    Away,
    Decided,
    Device,
    Finding,
    LandedDraft,
    HistoryProject,
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum Need {
    Forge(u16),
    Repository(Repository),
    Task { task: u64, retired: Option<u16> },
    EntryOwner { task: u64, entry: u64 },
    Entry { task: u64, entry: u64 },
}
#[derive(Debug, PartialEq, Eq)]
pub struct MissingLink {
    pub kind: RecordKind,
    pub key: Key,
    pub need: Need,
}
// Retain the opaque client's references too: families can arrive in any order.
pub fn capacity(l: &Limits) -> Option<u32> {
    l.named_calls // Call: task.
        .checked_add(l.tasks.checked_mul(2)?)? // ProposedEffect: repository and task.
        .checked_add(l.repositories)? // Repository: forge.
        .checked_add(l.holds.checked_mul(3)?)? // Hold: name, task, writer task.
        .checked_add(l.tasks.checked_add(l.repositories)?.checked_mul(l.resources_per_task)?)? // Names.
        .checked_add(l.subscriptions.checked_mul(2)?)? // Subscription: repository and owner.
        .checked_add(l.client.resources.checked_mul(3)?)? // BranchHead, PullState, Client::Live.
        .checked_add(l.subscriptions)? // Ci: repository.
        .checked_add(l.entries.checked_mul(3)?)? // Entry: repository/task; Flight: entry/task.
        .checked_add(l.client.repositories)? // Client::Repository.
        .checked_add(l.tasks.checked_mul(2)?)? // Closer: live task and goal.
        .checked_add(l.changes)? // Change: repository; task is historical.
        .checked_add(l.issues.checked_mul(2)?)? // Issue: repository and live goal.
        .checked_add(l.tasks.checked_mul(3)?)? // Release: task, root, pending name.
        .checked_add(l.applications)? // Application: adopted repository, if any.
        .checked_add(l.client.repositories)? // Writer: configured forge.
        .checked_add(l.repositories.checked_mul(l.tiers.tiers)?.checked_mul(2)?)?
        .checked_add(crate::follows::capacity(l)?.checked_mul(2)?)? // Follow: source and task.
        .checked_add(l.changes)? // Verification: repository.
        .checked_add(l.repositories)? // LandingLog: repository.
        .checked_add(l.landings.checked_mul(l.landing_members)?)?
        .checked_add(l.repositories.checked_mul(l.gates.checked_add(l.tiers.tiers)?)?.checked_mul(l.receipts_per_gate)?)
}
impl Need {
    pub fn repository(self, repository: Repository) -> bool {
        self == Self::Repository(repository)
    }
    pub fn task(self, task: u64, retired: Option<u16>) -> bool {
        self == Self::Task { task, retired }
    }
    pub fn name(self, name: &Name) -> bool {
        self.repository(Repository { forge: name.forge, repository: name.repository })
    }
    pub fn entry(self, entry: &skein_forge_client::Entry) -> bool {
        self.repository(entry.repository) || self.task(entry.task, Some(entry.repository.forge))
    }
}
