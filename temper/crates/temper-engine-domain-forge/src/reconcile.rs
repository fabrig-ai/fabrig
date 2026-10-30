//! Reference reconciliation before Switch (domain/root.md, 9.1; protocol/store.md, 2).
use crate::{Domain, Limits, Name, Stored, Topic};
use skein_forge::api::Repository;
use skein_lib::List;

/// A restored record's family, retained for compatibility diagnostics.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecordKind {
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
    Change,
    Issue,
    Release,
    Application,
    Writer,
    Demo,
    Binding,
    DemoAct,
    Flight,
    Landing,
    Receipt,
}
/// Compatibility diagnostic; missing required links assert during reconciliation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unresolved {
    pub records: u32,
    pub first: RecordKind,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Need {
    Forge(u16),
    Demo(u16),
    Repository(Repository),
    Task { task: u64, retired: Option<u16> },
    EntryOwner { task: u64, entry: u64 },
    Entry { task: u64, entry: u64 },
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Reference {
    kind: RecordKind,
    need: Need,
}
#[derive(Debug)]
pub(crate) struct Reconciliation {
    references: List<Reference>,
    pub(crate) verdict: Result<(), Unresolved>,
}
// Retain the opaque client's references too: families can arrive in any order.
pub(crate) fn capacity(l: &Limits) -> Option<u32> {
    l.named_calls // Call: task.
        .checked_add(l.tasks.checked_mul(2)?)? // ProposedEffect: repository and task.
        .checked_add(l.repositories)? // Repository: forge.
        .checked_add(l.holds.checked_mul(3)?)? // Hold: name, task, writer task.
        .checked_add(l.tasks.checked_mul(l.resources_per_task.checked_add(1)?)?)? // Names.
        .checked_add(l.subscriptions.checked_mul(2)?)? // Subscription: repository and owner.
        .checked_add(l.client.resources.checked_mul(3)?)? // BranchHead, PullState, Client::Live.
        .checked_add(l.subscriptions)? // Ci: repository.
        .checked_add(l.entries.checked_mul(3)?)? // Entry: repository/task; Flight: entry/task.
        .checked_add(l.client.repositories)? // Client::Repository.
        .checked_add(l.changes)? // Change: repository; task is historical.
        .checked_add(l.issues.checked_mul(2)?)? // Issue: repository and live goal.
        .checked_add(l.tasks.checked_mul(3)?)? // Release: task, root, pending name.
        .checked_add(l.applications)? // Application: adopted repository, if any.
        .checked_add(l.client.repositories)? // Writer: configured forge.
        .checked_add(l.landings.checked_mul(l.landing_members)?)?
        .checked_add(l.repositories.checked_mul(l.gates)?.checked_mul(l.receipts_per_gate)?)?
        .checked_add(1) // Demo: configured demo's forge.
}
impl Reconciliation {
    pub(crate) fn new(l: &Limits) -> Self {
        Self { references: List::with_capacity(capacity(l).expect("checked reconciliation capacity")), verdict: Ok(()) }
    }
    fn need(&mut self, kind: RecordKind, need: Need) -> Result<(), crate::RestoreFit> {
        let held = self.references.len();
        match self.references.push(Reference { kind, need }) {
            Ok(()) => Ok(()),
            Err(_) => Err(crate::RestoreFit {
                family: Some(kind),
                set: crate::RestoreSet::References,
                held: u64::from(held),
                additional: 1,
                limit: u64::from(self.references.capacity()),
            }),
        }
    }
    fn repository(&mut self, kind: RecordKind, repository: Repository) -> Result<(), crate::RestoreFit> {
        self.need(kind, Need::Repository(repository))?;
        Ok(())
    }
    fn task(&mut self, kind: RecordKind, task: u64, retired: Option<u16>) -> Result<(), crate::RestoreFit> {
        self.need(kind, Need::Task { task, retired })?;
        Ok(())
    }
    fn name(&mut self, kind: RecordKind, name: &Name) -> Result<(), crate::RestoreFit> {
        self.repository(kind, Repository { forge: name.forge, repository: name.repository })?;
        Ok(())
    }
    fn entry(&mut self, kind: RecordKind, entry: &skein_forge_client::Entry) -> Result<(), crate::RestoreFit> {
        self.repository(kind, entry.repository)?;
        self.task(kind, entry.task, Some(entry.repository.forge))?;
        Ok(())
    }
    pub(crate) fn restore(&mut self, row: &Stored) -> Result<(), crate::RestoreFit> {
        match row {
            Stored::Landing(row) => {
                for member in &row.landing.members {
                    self.repository(
                        RecordKind::Landing,
                        Repository { forge: member.repository.forge, repository: member.repository.repository },
                    )?;
                }
                Ok(())
            }
            Stored::Receipt { key, .. } => self.repository(RecordKind::Receipt, key.repository),
            Stored::Call { key, .. } => self.task(RecordKind::Call, key.task, None),
            Stored::ProposedEffect(row) => self.entry(RecordKind::ProposedEffect, &row.entry),
            Stored::Repository(row) => self.need(RecordKind::Repository, Need::Forge(row.provider.forge)),
            Stored::Hold(row) => {
                self.name(RecordKind::Hold, &row.name)?;
                self.task(RecordKind::Hold, row.task, Some(row.name.forge))?;
                if let Some(crate::Writer::Run { task, .. } | crate::Writer::Lost { task, .. }) = row.writer {
                    self.task(RecordKind::Hold, task, Some(row.name.forge))?;
                }
                Ok(())
            }
            Stored::Names { task, resources } => {
                let mut retired = None;
                if let Some(name) = resources.first() {
                    retired = Some(name.forge);
                }
                for name in resources {
                    if retired != Some(name.forge) {
                        retired = None;
                    }
                }
                self.task(RecordKind::Names, *task, retired)?;
                for name in resources {
                    self.name(RecordKind::Names, name)?;
                }
                Ok(())
            }
            Stored::Subscription(row) => {
                let repository = match row.topic {
                    Topic::Landings { repository, .. }
                    | Topic::Ci { repository, .. }
                    | Topic::Pull { repository, .. }
                    | Topic::Participation { repository, .. } => repository,
                };
                self.repository(RecordKind::Subscription, repository)?;
                self.task(RecordKind::Subscription, row.task, Some(repository.forge))?;
                // own_change and goal_tasks classify historical landings.
                Ok(())
            }
            Stored::BranchHead(row) => self.name(RecordKind::BranchHead, &row.name),
            Stored::PullState(row) => self.name(RecordKind::PullState, &row.name),
            Stored::Ci(row) => self.repository(RecordKind::Ci, row.repository),
            Stored::Entry { entry, .. } => {
                self.repository(RecordKind::Entry, entry.repository)?;
                self.need(RecordKind::Entry, Need::EntryOwner { task: entry.task, entry: entry.number })?;
                Ok(())
            }
            Stored::Client(row) => match row {
                skein_forge_client::Stored::Live(row) => {
                    self.repository(RecordKind::Client, row.watch.resource.repository)
                }
                skein_forge_client::Stored::Repository(row) => self.repository(RecordKind::Client, row.repository),
                skein_forge_client::Stored::Writer(row) => self.need(RecordKind::Client, Need::Forge(row.forge)),
            },
            // Change panels outlive tasks (domain/forge.md, 8.8).
            Stored::Change(row) => self.repository(RecordKind::Change, row.repository),
            Stored::Issue(row) => {
                self.repository(RecordKind::Issue, row.repository)?;
                let ending = match &row.desired {
                    Some(view) => view.finished.is_some(),
                    None => false,
                };
                // Failed projections settle at goal close, even with an unfinished
                // or absent desired view (domain/forge.md, 12).
                if !row.failed && !row.state.closed && !ending {
                    self.task(RecordKind::Issue, row.goal, Some(row.repository.forge))?;
                }
                Ok(())
            }
            Stored::Release(row) => {
                self.task(RecordKind::Release, row.task, None)?;
                self.task(RecordKind::Release, row.root, None)?;
                if let Some((_, Some(name))) = &row.pending {
                    self.name(RecordKind::Release, name)?;
                }
                Ok(())
            }
            Stored::Application { outcome, .. } => match outcome {
                crate::ApplicationOutcome::Adopted(row) => self.repository(RecordKind::Application, row.repository),
                crate::ApplicationOutcome::CoreRefused(_) | crate::ApplicationOutcome::ForgeRefused(_) => Ok(()),
            },
            Stored::Writer(row) => self.need(RecordKind::Writer, Need::Forge(row.forge)),
            Stored::Demo(row) => self.need(RecordKind::Demo, Need::Demo(row.forge)),
            // Landing history outlives tasks (domain/forge.md, 7.2).
            // Bindings outlive members. End acts retry until the saved answer,
            // including after their target ended or was removed.
            Stored::Landed { .. } | Stored::Binding { .. } | Stored::DemoAct(_) => Ok(()),
            Stored::Flight { task, entry } => self.need(RecordKind::Flight, Need::Entry { task: *task, entry: *entry }),
        }
    }
}
impl Domain {
    /// Link restored rows to retained targets; a missing target is a writer bug.
    pub fn reconcile(&mut self, tasks: &jig_domain_tasks::Domain) {
        if self.restore_fit_failure().is_some() {
            return;
        }
        for reference in &self.reconciliation.references {
            let live = match reference.need {
                Need::Task { task, .. } | Need::EntryOwner { task, .. } | Need::Entry { task, .. } => {
                    tasks.task(task).is_some()
                }
                Need::Forge(_) | Need::Demo(_) | Need::Repository(_) => false,
            };
            assert!(
                self.reference_resolves(reference.need, live),
                "forge link {:?} -> {:?}",
                reference.kind,
                reference.need
            );
        }
        self.reconciliation.verdict = Ok(());
    }
    /// Retained compatibility result; missing targets panic before this can report failure.
    pub const fn reconciliation(&self) -> Result<(), Unresolved> {
        self.reconciliation.verdict
    }
    fn configured_forge(&self, forge: u16) -> bool {
        self.configured.contains_key(&forge) || self.identities.contains_key(&forge)
    }
    fn reference_resolves(&self, need: Need, live: bool) -> bool {
        match need {
            // Identities are constructor facts too; a restored writer is never one.
            Need::Forge(forge) => self.configured_forge(forge),
            Need::Demo(forge) => match &self.demo {
                Some(demo) => demo.adoption.forge == forge,
                None => false,
            },
            Need::Repository(repository) => {
                self.configured_forge(repository.forge) && self.repositories.contains_key(&repository)
            }
            Need::Task { retired, .. } => {
                live || match retired {
                    Some(forge) => self.forge_ended(forge),
                    None => false,
                }
            }
            Need::EntryOwner { task, entry } => {
                live || match self.entries.get(&entry) {
                    Some(row) => {
                        self.forge_ended(row.entry.repository.forge) || self.closing_projection_entry(task, entry)
                    }
                    None => false,
                }
            }
            Need::Entry { task, entry } => match self.entries.get(&entry) {
                Some(row) => {
                    live || self.forge_ended(row.entry.repository.forge) || self.closing_projection_entry(task, entry)
                }
                None => false,
            },
        }
    }

    fn closing_projection_entry(&self, task: u64, entry: u64) -> bool {
        // Closing projections outlive their goals (domain/forge.md, 12).
        // The pending identity reconstructs ownership without a live task.
        match self.issues.get(&task) {
            Some(row) if row.pending == Some(entry) => match &row.desired {
                Some(view) => view.finished.is_some(),
                None => false,
            },
            Some(_) | None => false,
        }
    }
}

#[cfg(test)]
pub(crate) fn references_resolve(d: &Domain, limits: &Limits, row: &Stored, live_tasks: &[u64]) -> bool {
    let mut reconciliation = Reconciliation::new(limits);
    reconciliation.restore(row).expect("oracle row fits references");
    for reference in &reconciliation.references {
        let live = match reference.need {
            Need::Task { task, .. } | Need::EntryOwner { task, .. } | Need::Entry { task, .. } => {
                live_tasks.contains(&task)
            }
            Need::Forge(_) | Need::Demo(_) | Need::Repository(_) => false,
        };
        // Writer rules are an oracle concern, separate from runtime target links.
        if let Need::EntryOwner { task, entry } | Need::Entry { task, entry } = reference.need
            && let Some(row) = d.entries.get(&entry)
        {
            let check_owner = match reference.need {
                Need::Entry { .. } => true,
                Need::EntryOwner { .. } => !live && !d.forge_ended(row.entry.repository.forge),
                Need::Forge(_) | Need::Demo(_) | Need::Repository(_) | Need::Task { .. } => false,
            };
            if check_owner && row.entry.task != task {
                return false;
            }
            if !live
                && !d.forge_ended(row.entry.repository.forge)
                && let Some(issue) = d.issues.get(&task)
                && issue.repository != row.entry.repository
            {
                return false;
            }
        }
        if !d.reference_resolves(reference.need, live) {
            return false;
        }
    }
    true
}
