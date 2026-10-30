//! Configured restoration fit before linking (temper/docs/design/domain/forge.md, 16).
use super::Domain;
use crate::{CallPayload, Limits, RecordKind, Stored, What};
use alloc::boxed::Box;
use skein_lib::{Map, Wall};

/// A connector-owned retained or reconstructed set.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RestoreSet {
    Rows(RecordKind),
    NamedAnswerBytes,
    NamedAnswerRows,
    BindingAddressBytes,
    BindingSubjectBytes,
    ChangeTimers,
    ProjectionTimers,
    References,
    Watches,
    LandingFlights,
    LandingMembers,
    LandingGates,
    LandingAsks,
    LandingEntries,
    NameBytes,
}

/// The first configured bound a restored row or derived set could not fit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreFit {
    pub family: Option<RecordKind>,
    pub set: RestoreSet,
    pub held: u64,
    pub additional: u64,
    pub limit: u64,
}

impl Domain {
    /// Establish the start's single expiry observation without resetting restored state.
    pub fn restore_begin(&mut self, started: Wall) {
        if self.restore_started.is_none() {
            self.restore_started = Some(started);
        }
        if self.restore_phase != super::RestorePhase::Active {
            self.restore_phase = super::RestorePhase::Restoring;
        }
    }
    /// The retained first refusal; late inputs cannot replace it.
    #[must_use]
    pub fn restore_fit_failure(&self) -> Option<&RestoreFit> {
        self.restore_fit.as_deref()
    }
    /// Fit reconstruction before reference linking or client recovery.
    pub fn fit_restored(&mut self, limits: &Limits) -> Result<(), RestoreFit> {
        if let Some(failure) = self.restore_fit_failure() {
            return Err(*failure);
        }
        let result = derived(self, limits);
        if let Err(failure) = result {
            self.restore_fit = Some(Box::new(failure));
        }
        result
    }
}

fn room(family: RecordKind, set: RestoreSet, held: u32, limit: u32, replacing: bool) -> Result<(), RestoreFit> {
    if replacing || held < limit {
        Ok(())
    } else {
        Err(RestoreFit { family: Some(family), set, held: u64::from(held), additional: 1, limit: u64::from(limit) })
    }
}
fn field(family: RecordKind, set: RestoreSet, held: u64, limit: u64) -> Result<(), RestoreFit> {
    if held <= limit { Ok(()) } else { Err(RestoreFit { family: Some(family), set, held, additional: 0, limit }) }
}
fn count(value: usize) -> u64 {
    u64::try_from(value).expect("retained length fits u64")
}

pub(super) fn named_payload(l: &Limits, answer: &CallPayload) -> Result<(), RestoreFit> {
    if let CallPayload::Read(answer) = answer
        && let Ok(answer) = answer.as_ref()
    {
        let bytes = skein_forge::measure::answer(answer).unwrap_or(u64::MAX);
        field(RecordKind::Call, RestoreSet::NamedAnswerBytes, bytes, u64::from(l.client.answer_bytes))?;
        field(RecordKind::Call, RestoreSet::NamedAnswerRows, answer_rows(answer), u64::from(l.client.rows))?;
    }
    Ok(())
}
// The largest list in an answer, including lists nested within returned rows.
fn answer_rows(answer: &skein_forge::api::Answer) -> u64 {
    use skein_forge::api::Answer;
    match answer {
        Answer::Run(run) => count(run.jobs.len()),
        Answer::Runs { runs, .. } => {
            let mut largest = count(runs.len());
            for run in runs {
                largest = largest.max(count(run.jobs.len()));
            }
            largest
        }
        Answer::Tree { entries, .. } => count(entries.len()),
        Answer::Items { items, .. } => {
            let mut largest = count(items.len());
            for item in items {
                largest = largest.max(count(item.labels.len()));
            }
            largest
        }
        Answer::Item { item, comments, .. } => count(item.labels.len()).max(count(comments.len())),
        Answer::Pull(pull) => count(pull.reviewers.len()),
        Answer::Reviews { reviews, .. } => count(reviews.len()),
        Answer::Statuses { statuses, .. } | Answer::Checks(statuses) => count(statuses.len()),
        Answer::Remarks { remarks, .. } => count(remarks.len()),
        Answer::Branches(branches) => count(branches.len()),
        Answer::PullFiles { files, .. } => count(files.len()),
        Answer::Compare { files, commits, .. } => count(files.len()).max(count(commits.len())),
        Answer::Protection(Some(value)) => count(value.contexts.len()),
        Answer::Collaborators { collaborators, .. } => count(collaborators.len()),
        Answer::Protection(None)
        | Answer::Located { .. }
        | Answer::Commit(_)
        | Answer::Job { .. }
        | Answer::Directory { .. }
        | Answer::File { .. }
        | Answer::Settings(_)
        | Answer::Permission(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        | Answer::Queued
        | Answer::Branch(_)
        | Answer::Done => 0,
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive fit map for the connector's durable rows")]
pub(super) fn row(d: &Domain, l: &Limits, row: &Stored) -> Result<(), RestoreFit> {
    use RecordKind as K;
    use RestoreSet::Rows;
    match row {
        Stored::Landing(row) => {
            field(
                K::Landing,
                RestoreSet::LandingMembers,
                count(row.landing.members.len()),
                u64::from(l.landing_members),
            )?;
            field(K::Landing, RestoreSet::LandingAsks, count(row.asks.len()), u64::from(l.landing_members))?;
            field(K::Landing, RestoreSet::LandingEntries, count(row.entries.len()), u64::from(l.entries))?;
            for member in &row.landing.members {
                field(K::Landing, RestoreSet::LandingGates, count(member.gates.len()), u64::from(l.gates))?;
            }
            room(
                K::Landing,
                Rows(K::Landing),
                d.landing.rows.len(),
                d.landing.rows.capacity(),
                d.landing.rows.contains_key(&row.landing.task),
            )
        }
        Stored::Receipt { key, .. } => room(
            K::Receipt,
            Rows(K::Receipt),
            d.landing.receipts.len(),
            d.landing.receipts.capacity(),
            d.landing.receipts.contains_key(key),
        ),
        Stored::Application { token, .. } => room(
            K::Application,
            Rows(K::Application),
            d.application.kept.len(),
            d.application.kept.capacity(),
            d.application.kept.contains_key(&skein_lib::Token::new(*token)),
        ),
        Stored::Call { key, answer } => {
            named_payload(l, answer)?;
            room(K::Call, Rows(K::Call), d.named_calls.len(), d.named_calls.capacity(), d.named_calls.contains_key(key))
        }
        Stored::Repository(row) => room(
            K::Repository,
            Rows(K::Repository),
            d.repositories.len(),
            d.repositories.capacity(),
            d.repositories.contains_key(&row.provider),
        ),
        Stored::Hold(row) => {
            room(K::Hold, Rows(K::Hold), d.holds.len(), d.holds.capacity(), d.holds.contains_key(&row.name))
        }
        Stored::Names { task, .. } => {
            room(K::Names, Rows(K::Names), d.names.len(), d.names.capacity(), d.names.contains_key(task))
        }
        Stored::Subscription(row) => room(
            K::Subscription,
            Rows(K::Subscription),
            d.subscriptions.len(),
            d.subscriptions.capacity(),
            d.subscriptions.contains_key(&(row.task, row.topic.clone())),
        ),
        Stored::BranchHead(row) => {
            room(K::BranchHead, Rows(K::BranchHead), d.heads.len(), d.heads.capacity(), d.heads.contains_key(&row.name))
        }
        Stored::PullState(row) => {
            room(K::PullState, Rows(K::PullState), d.pulls.len(), d.pulls.capacity(), d.pulls.contains_key(&row.name))
        }
        Stored::Ci(row) => {
            room(K::Ci, Rows(K::Ci), d.ci.len(), d.ci.capacity(), d.ci.contains_key(&(row.repository, row.head)))
        }
        Stored::Landed { commit, .. } => {
            room(K::Landed, Rows(K::Landed), d.landed.len(), d.landed.capacity(), d.landed.contains_key(commit))
        }
        Stored::ProposedEffect(row) => room(
            K::ProposedEffect,
            Rows(K::ProposedEffect),
            d.proposed_effects.len(),
            d.proposed_effects.capacity(),
            d.proposed_effects.contains_key(&row.number),
        ),
        Stored::Entry { entry, .. } => {
            room(K::Entry, Rows(K::Entry), d.entries.len(), d.entries.capacity(), d.entries.contains_key(&entry.number))
        }
        Stored::Binding { address, subject } => {
            field(K::Binding, RestoreSet::BindingAddressBytes, count(address.len()), 254)?;
            field(K::Binding, RestoreSet::BindingSubjectBytes, count(subject.len()), 255)?;
            room(
                K::Binding,
                Rows(K::Binding),
                d.bindings.len(),
                d.bindings.capacity(),
                d.bindings.contains_key(address),
            )
        }
        Stored::Flight { task, entry } => room(
            K::Flight,
            Rows(K::Flight),
            d.flights.len(),
            d.flights.capacity(),
            d.flights.contains_key(&(*task, *entry)),
        ),
        Stored::DemoAct(row) => room(
            K::DemoAct,
            Rows(K::DemoAct),
            d.demo_acts.len(),
            d.demo_acts.capacity(),
            d.demo_acts.contains_key(&row.key),
        ),
        Stored::Writer(row) => {
            room(K::Writer, Rows(K::Writer), d.writers.len(), d.writers.capacity(), d.writers.contains_key(&row.forge))
        }
        Stored::Change(row) => {
            room(K::Change, Rows(K::Change), d.changes.len(), d.changes.capacity(), d.changes.contains_key(&row.task))?;
            room(
                K::Change,
                RestoreSet::ChangeTimers,
                d.change_due.len(),
                d.change_due.capacity(),
                d.change_due.contains_key(&row.task),
            )
        }
        Stored::Issue(row) => {
            room(K::Issue, Rows(K::Issue), d.issues.len(), d.issues.capacity(), d.issues.contains_key(&row.goal))?;
            if row.pending.is_none() && !row.failed && !row.state.closed && row.desired.is_some() {
                room(
                    K::Issue,
                    RestoreSet::ProjectionTimers,
                    d.projection_due.len(),
                    d.projection_due.capacity(),
                    d.projection_due.contains_key(&row.goal),
                )?;
            }
            Ok(())
        }
        Stored::Release(row) => room(
            K::Release,
            Rows(K::Release),
            d.releases.len(),
            d.releases.capacity(),
            d.releases.contains_key(&row.task),
        ),
        Stored::Demo(_) | Stored::Client(_) => Ok(()),
    }
}

fn derived(d: &Domain, l: &Limits) -> Result<(), RestoreFit> {
    let mut watches = Map::with_capacity(l.client.resources);
    for (_, names) in &d.names {
        for name in names {
            if d.forge_ended(name.forge) {
                continue;
            }
            let Some(resource) = super::to_client(name, l) else {
                let mut bytes = 0_u64;
                if let What::Branch(parts) = &name.what {
                    for (index, part) in parts.iter().enumerate() {
                        bytes = bytes.checked_add(count(part.len())).expect("bounded name bytes");
                        if index > 0 {
                            bytes = bytes.checked_add(1).expect("bounded separators");
                        }
                    }
                }
                return Err(RestoreFit {
                    family: Some(RecordKind::Names),
                    set: RestoreSet::NameBytes,
                    held: bytes,
                    additional: 0,
                    limit: u64::from(l.name_bytes),
                });
            };
            let held = watches.len();
            if watches.insert(resource, ()).is_err() {
                return Err(RestoreFit {
                    family: Some(RecordKind::Names),
                    set: RestoreSet::Watches,
                    held: u64::from(held),
                    additional: 1,
                    limit: u64::from(watches.capacity()),
                });
            }
        }
    }
    // Each member/gate can contribute one rebuilt flight, before any allocation or u16 conversion.
    let members = l.landings.checked_mul(l.landing_members).expect("checked landing members");
    let limit = members.checked_mul(l.gates).expect("checked landing geometry");
    let mut held = 0_u64;
    for (_, row) in &d.landing.rows {
        for member in &row.landing.members {
            held = held.checked_add(count(member.gates.len())).expect("bounded landing geometry");
        }
    }
    if held > u64::from(limit) {
        return Err(RestoreFit {
            family: None,
            set: RestoreSet::LandingFlights,
            held,
            additional: 0,
            limit: u64::from(limit),
        });
    }
    Ok(())
}
