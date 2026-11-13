//! Reference reconciliation before Switch (domain/root.md, 8.1; protocol/store.md, 2).
use crate::{Domain, Limits, Name, Stored, Topic};
use skein_forge::api::Repository;
use skein_lib::List;
pub use temper_engine_domain_forge_vocabulary::reconcile::*;

#[derive(Clone, Copy, Hash, Debug)]
pub(crate) struct Reference {
    kind: RecordKind,
    client_repository: bool,
    need: Need,
}
#[derive(Hash, Debug)]
pub(crate) struct Reconciliation {
    references: List<Reference>,
}
impl Reconciliation {
    pub(crate) fn new(l: &Limits) -> Self {
        Self { references: List::with_capacity(capacity(l).expect("checked reconciliation capacity")) }
    }
    fn need(&mut self, kind: RecordKind, need: Need) -> Result<(), crate::RestoreFit> {
        let held = self.references.len();
        match self.references.push(Reference { kind, client_repository: false, need }) {
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
        let before = self.references.len();
        self.restore_references(row)?;
        if let Stored::Client(skein_forge_client::Stored::Repository(_)) = row {
            for index in before..self.references.len() {
                self.references.get_mut(index).expect("bounded reference index").client_repository = true;
            }
        }
        Ok(())
    }
    #[expect(clippy::too_many_lines, reason = "one exhaustive reference map for the connector durable rows")]
    fn restore_references(&mut self, row: &Stored) -> Result<(), crate::RestoreFit> {
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
            Stored::Follow(row) => {
                self.repository(RecordKind::Follow, row.source)?;
                // A stopped follow's task number is history (domain/forge/follows.md, 4).
                if row.stopped.is_none()
                    && let Some(task) = row.task
                {
                    self.task(RecordKind::Follow, task, Some(row.source.forge))?;
                }
                Ok(())
            }
            Stored::Tier(row) => self.repository(RecordKind::Tier, row.repository),
            Stored::Verification(row) => self.repository(RecordKind::Verification, row.repository),
            Stored::LandingLog(row) => self.repository(RecordKind::LandingLog, row.repository),
            Stored::Receipt { key, .. } => self.repository(RecordKind::Receipt, key.repository),
            Stored::Call { key, .. } => self.task(RecordKind::Call, key.task, None),
            Stored::ProposedEffect(row) => self.entry(RecordKind::ProposedEffect, &row.entry),
            Stored::Repository(row) => self.need(RecordKind::Repository, Need::Forge(row.provider.forge)),
            Stored::Hold(row) => {
                self.name(RecordKind::Hold, &row.name)?;
                self.task(RecordKind::Hold, row.task, Some(row.name.forge))?;
                for writer in &row.design_writers {
                    self.task(RecordKind::Hold, writer.task, Some(row.name.forge))?;
                }
                if let Some(crate::Writer::Run { task, .. } | crate::Writer::Lost { task, .. }) = row.writer {
                    self.task(RecordKind::Hold, task, Some(row.name.forge))?;
                }
                Ok(())
            }
            Stored::Names { resources, .. } => {
                for name in resources {
                    self.name(RecordKind::Names, name)?;
                }
                Ok(())
            }
            Stored::Subscription(row) => {
                let repository = match row.topic {
                    Topic::Findings { .. } => return self.task(RecordKind::Subscription, row.task, None),
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
            Stored::Closer(row) => {
                if !row.done {
                    self.task(RecordKind::Closer, row.task, None)?;
                    self.task(RecordKind::Closer, row.goal, None)?;
                }
                Ok(())
            }
            // Change panels outlive tasks (temper/docs/design/domain/forge/change/README.md, 5).
            Stored::Change(row) => self.repository(RecordKind::Change, row.repository),
            Stored::Issue(row) => {
                self.repository(RecordKind::Issue, row.repository)?;
                let ending = match &row.desired {
                    Some(view) => view.finished.is_some(),
                    None => false,
                };
                // Failed projections settle at goal close, even with an unfinished
                // or absent desired view (domain/forge/issues.md).
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
                crate::ApplicationOutcome::CoreRefused(_)
                | crate::ApplicationOutcome::ForgeRefused(_)
                | crate::ApplicationOutcome::MakeKept
                | crate::ApplicationOutcome::Follow(_)
                | crate::ApplicationOutcome::MakeRefused(_) => Ok(()),
            },
            Stored::Red(row) => self.task(RecordKind::Red, row.goal, None),
            Stored::Writer(row) => self.need(RecordKind::Writer, Need::Forge(row.forge)),
            // Landing history outlives tasks (domain/forge/README.md, 5.2).
            // Bindings outlive members. End acts retry until the saved answer,
            // including after their target ended or was removed.
            Stored::Forge(_)
            | Stored::ForgeNumber { .. }
            | Stored::RedNumber { .. }
            | Stored::Playground(_)
            | Stored::Landed { .. }
            | Stored::RefusedSetting(_)
            | Stored::Setting(_)
            | Stored::Binding { .. }
            | Stored::Making(_)
            | Stored::ProjectFacts(_)
            | Stored::Device(_)
            | Stored::LandedDraft(_)
            | Stored::Away(_)
            | Stored::Decided(_)
            | Stored::Finding(_)
            | Stored::Looked(_)
            | Stored::Dismissed { .. } => Ok(()),
            Stored::Flight { task, entry } => self.need(RecordKind::Flight, Need::Entry { task: *task, entry: *entry }),
        }
    }
}
impl Domain {
    /// Link restored rows to retained targets; a missing target is a writer bug.
    #[expect(
        clippy::panic,
        reason = "an unmade restored link is a writer bug (jig/docs/design/domain/engine/restart.md)"
    )]
    pub fn reconcile(&mut self, tasks: &jig_domain_tasks::Domain) {
        if self.restore_fit_failure().is_some() {
            return;
        }
        if let Some(missing) = self.missing_link(tasks) {
            panic!("forge link {:?} -> {:?} for {:?}", missing.kind, missing.need, missing.key);
        }
    }
    /// The first required restored link whose target is absent (jig/docs/design/domain/engine/restart.md).
    #[must_use]
    pub fn missing_link(&self, tasks: &jig_domain_tasks::Domain) -> Option<MissingLink> {
        if self.restore_fit_failure().is_some() {
            return None;
        }
        for reference in &self.reconciliation.references {
            let live = match reference.need {
                Need::Task { task, .. } | Need::EntryOwner { task, .. } | Need::Entry { task, .. } => {
                    tasks.task(task).is_some()
                }
                Need::Forge(_) | Need::Repository(_) => false,
            };
            if !self.reference_resolves(reference.kind, reference.need, live) {
                return Some(MissingLink {
                    kind: reference.kind,
                    key: self.restored_link_key(reference.kind, reference.need, reference.client_repository),
                    need: reference.need,
                });
            }
        }
        None
    }
    fn configured_forge(&self, forge: u16) -> bool {
        self.configured.contains_key(&forge) || self.identities.contains_key(&forge)
    }
    fn reference_resolves(&self, kind: RecordKind, need: Need, live: bool) -> bool {
        match need {
            // Identities are constructor facts too; a restored writer is never one.
            Need::Forge(forge) => self.configured_forge(forge),
            Need::Repository(repository) => {
                (self.configured_forge(repository.forge) && self.repositories.contains_key(&repository))
                    || (Self::historical_repository(kind)
                        && self.forge_ended(repository.forge)
                        && repository.forge != 0
                        && repository.forge <= self.forge_number)
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

    // History survives removal without retaining its repository's row
    // (temper/docs/design/domain/forge/adoption.md, 1).
    const fn historical_repository(kind: RecordKind) -> bool {
        match kind {
            RecordKind::BranchHead
            | RecordKind::PullState
            | RecordKind::Ci
            | RecordKind::Change
            | RecordKind::Issue
            | RecordKind::Application
            | RecordKind::Landing
            | RecordKind::Receipt
            | RecordKind::Follow
            | RecordKind::Tier
            | RecordKind::Verification
            | RecordKind::LandingLog => true,
            RecordKind::Closer
            | RecordKind::Forge
            | RecordKind::Call
            | RecordKind::ProposedEffect
            | RecordKind::Repository
            | RecordKind::Hold
            | RecordKind::Names
            | RecordKind::Subscription
            | RecordKind::Landed
            | RecordKind::Entry
            | RecordKind::Client
            | RecordKind::Release
            | RecordKind::Writer
            | RecordKind::Making
            | RecordKind::Binding
            | RecordKind::ProjectFacts
            | RecordKind::Device
            | RecordKind::LandedDraft
            | RecordKind::Red
            | RecordKind::RedNumber
            | RecordKind::Away
            | RecordKind::Decided
            | RecordKind::Finding
            | RecordKind::Looked
            | RecordKind::Dismissed
            | RecordKind::Setting
            | RecordKind::Flight => false,
        }
    }

    fn closing_projection_entry(&self, task: u64, entry: u64) -> bool {
        // Closing projections outlive their goals (domain/forge/issues.md).
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
            Need::Forge(_) | Need::Repository(_) => false,
        };
        // Writer rules are an oracle concern, separate from runtime target links.
        if let Need::EntryOwner { task, entry } | Need::Entry { task, entry } = reference.need
            && let Some(row) = d.entries.get(&entry)
        {
            let check_owner = match reference.need {
                Need::Entry { .. } => true,
                Need::EntryOwner { .. } => !live && !d.forge_ended(row.entry.repository.forge),
                Need::Forge(_) | Need::Repository(_) | Need::Task { .. } => false,
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
        if !d.reference_resolves(reference.kind, reference.need, live) {
            return false;
        }
    }
    true
}
