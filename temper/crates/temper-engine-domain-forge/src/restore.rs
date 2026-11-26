//! Configured restoration fit before linking (temper/docs/design/domain/forge/effects.md, 2).
use super::Domain;
use crate::{CallPayload, Key, Limits, MissingLink, RecordKind, Request, Stored, What};
use alloc::boxed::Box;
use skein_lib::{Env, Map, Queue, Wall};
pub use temper_engine_domain_forge_vocabulary::restore::*;

/// The forge's half of a start's Link phase (temper/docs/design/testing/store-check.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StartLink {
    Row(Key),
    Fit(RestoreFit),
    BranchWrites(crate::Name, usize),
    Link(MissingLink),
}

impl Domain {
    /// Link restored records in the same order at a root start and a store check.
    pub fn start_links(
        &mut self,
        tasks: &jig_domain_tasks::Domain,
        env: &Env<Limits>,
        out: &mut Queue<Request>,
    ) -> Result<(), StartLink> {
        if let Some(key) = self.unjoined_change() {
            return Err(StartLink::Row(key));
        }
        let branches = crate::conversion::branches::prepare(self, &env.limits)?;
        self.link_names(tasks, &env.limits);
        self.link_tiers(tasks, &env.limits);
        if let Err(failure) = self.restore_memberships() {
            return Err(StartLink::Fit(failure));
        }
        if let Err(failure) = self.convert_previous(out) {
            return Err(StartLink::Fit(failure));
        }
        crate::conversion::branches::write(self, branches, out);
        self.link_landings(tasks, env, out);
        self.link_writers(tasks, env, out);
        if let Err(failure) = self.fit_restored(&env.limits) {
            return Err(StartLink::Fit(failure));
        }
        if let Some(link) = self.missing_link(tasks) {
            return Err(StartLink::Link(link));
        }
        Ok(())
    }
    /// A retired run cannot own a slot. Keep its old durable row until the
    /// post-switch fresh read saves the writerless hold; a crash repeats the read
    /// (jig's domain/tasks/holds.md, 3).
    fn link_writers(&mut self, tasks: &jig_domain_tasks::Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
        use super::{LostReadState, LostWriterKind, PendingLost};
        let mut retired = skein_lib::List::with_capacity(self.holds.len());
        for (name, hold) in &self.holds {
            if let Some(crate::Writer::Run { task, attempt } | crate::Writer::Lost { task, attempt }) = hold.writer
                && tasks.task(task).is_none()
                && !self.forge_ended(name.forge)
            {
                retired.push((name.clone(), task, attempt)).expect("retired writers bounded by holds");
            }
        }
        for (name, task, attempt) in retired.into_boxed() {
            self.finished(task, attempt, true, out);
            let hold = self.holds.get_mut(&name).expect("selected retired writer");
            let previous = hold.clone();
            hold.writer = None;
            let repaired = hold.clone();
            self.reconciliation.forget(&Stored::Hold(previous), &env.limits);
            self.reconciliation.restore(&Stored::Hold(repaired.clone())).expect("repair removes one reference");
            self.sequence = self.sequence.checked_add(1).expect("restored writer read sequence fits");
            let owner = skein_lib::Token::new(self.sequence | (1_u64 << 62));
            self.lost
                .insert(
                    owner,
                    PendingLost {
                        task,
                        attempt,
                        name,
                        state: LostReadState::Restoring,
                        last_answer: None,
                        kind: LostWriterKind::Retired,
                    },
                )
                .expect("one retired read per hold");
        }
    }

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
    /// A change with only one part is an unreadable row (protocol/store-restart.md, 1).
    #[must_use]
    pub fn unjoined_change(&self) -> Option<Key> {
        for (task, _) in &self.change_cold {
            if !self.changes.contains_key(task) && !self.change_hot.contains_key(task) {
                return Some(Key::ChangeCold(*task));
            }
        }
        for (task, _) in &self.change_hot {
            if !self.change_cold.contains_key(task) {
                return Some(Key::ChangeHot(*task));
            }
        }
        None
    }
    pub(crate) fn join_changes(&mut self) {
        while let Some((task, _)) = self.change_hot.first() {
            let task = *task;
            let Some(cold) = self.change_cold.get(&task).cloned() else { break };
            let hot = self.change_hot.remove(&task).expect("pending change part");
            let mut row = crate::ChangeRow::join(cold, hot);
            if row.frozen() {
                row.room = None;
            }
            self.changes.insert(task, row).expect("change room checked");
        }
    }
    fn restore_memberships(&mut self) -> Result<(), RestoreFit> {
        if let Some(failure) = self.restore_fit_failure() {
            return Err(*failure);
        }
        self.join_changes();
        if let Err(failure) = self.landing.restore_memberships() {
            self.restore_fit = Some(Box::new(failure));
            return Err(failure);
        }
        Ok(())
    }
    /// Fit reconstruction before reference linking or client recovery.
    pub fn fit_restored(&mut self, limits: &Limits) -> Result<(), RestoreFit> {
        self.restore_memberships()?;
        let result = derived(self, limits);
        if let Err(failure) = result {
            self.restore_fit = Some(Box::new(failure));
        }
        result
    }
}

pub(super) fn room(
    family: RecordKind,
    set: RestoreSet,
    held: u32,
    limit: u32,
    replacing: bool,
) -> Result<(), RestoreFit> {
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
// The largest row-bounded list, including nested lists. Branch names are
// bounded by bytes alone (skein/docs/design/forge/vocabulary.md, 1.7).
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
        Answer::Commits { commits, .. } => {
            let mut largest = count(commits.len());
            for commit in commits {
                largest = largest.max(count(commit.parents.len()));
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
        Answer::PullFiles { files, .. } => count(files.len()),
        Answer::Compare { files, .. } => count(files.len()),
        Answer::Protection(Some(value)) => count(value.contexts.len()),
        Answer::Collaborators { collaborators, .. } => count(collaborators.len()),
        Answer::Branches(_)
        | Answer::Protection(None)
        | Answer::Located { .. }
        | Answer::Commit(_)
        | Answer::JobState { .. }
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
        Stored::HistoryProject(row) => room(
            K::HistoryProject,
            Rows(K::HistoryProject),
            d.history_projects.len(),
            d.history_projects.capacity(),
            d.history_projects.contains_key(&row.project),
        ),
        Stored::Forge(record) => {
            room(K::Forge, Rows(K::Forge), d.records.len(), l.forges, d.records.contains_key(&record.number))?;
            room(
                K::Forge,
                Rows(K::Forge),
                d.configured.len(),
                d.configured.capacity(),
                d.configured.contains_key(&record.number),
            )
        }
        Stored::Landing(row) => {
            field(K::Landing, RestoreSet::NameBytes, count(row.landing.branch.len()), u64::from(l.name_bytes))?;
            field(
                K::Landing,
                RestoreSet::LandingMembers,
                count(row.landing.members.len()),
                u64::from(l.landing_members),
            )?;
            if let Some(job) = &row.job {
                field(K::Landing, RestoreSet::LandingMembers, count(job.members.len()), u64::from(l.landing_members))?;
            }
            field(K::Landing, RestoreSet::LandingAsks, count(row.previous_asks.len()), u64::from(l.landing_members))?;
            field(K::Landing, RestoreSet::LandingEntries, count(row.entries.len()), u64::from(l.entries))?;
            field(
                K::Landing,
                RestoreSet::LandingFollows,
                count(row.follows().len()),
                u64::from(crate::landing::follow_slots(l).expect("checked follow bound")),
            )?;
            for flight in row.follows() {
                field(K::Landing, RestoreSet::LandingReruns, count(flight.reruns.len()), u64::from(l.client.rows))?;
            }
            field(K::Landing, RestoreSet::LandingFlakeReruns, count(row.flake_reruns.len()), u64::from(l.entries))?;
            for rerun in &row.flake_reruns {
                field(K::Landing, RestoreSet::LandingFlakeFindings, count(rerun.findings.len()), 64)?;
                field(K::Landing, RestoreSet::LandingFlakeJobs, count(rerun.pending.len()), 16)?;
            }
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
        Stored::Follow(row) => {
            field(K::Follow, RestoreSet::FollowQueue, count(row.queue.len()), u64::from(l.follow_queue))?;
            let mut held = 0_u32;
            for _ in d.follow_rows.range((row.project, 0)..=(row.project, u32::MAX)) {
                held = held.saturating_add(1);
            }
            let replacing = d.follow_rows.contains_key(&(row.project, row.follow));
            room(K::Follow, Rows(K::Follow), held, l.follows, replacing)?;
            room(K::Follow, Rows(K::Follow), d.follow_rows.len(), d.follow_rows.capacity(), replacing)
        }
        Stored::Tier(row) => room(
            K::Tier,
            Rows(K::Tier),
            d.tiers.len(),
            d.tiers.capacity(),
            d.tiers.contains_key(&(row.repository, row.tier.clone())),
        ),
        Stored::Milestone(row) => {
            field(
                K::Milestone,
                RestoreSet::NameBytes,
                count(row.name.len()),
                u64::from(crate::milestone_rows::MILESTONE_NAME_BYTES),
            )?;
            Ok(())
        }
        Stored::LandingLog(row) => {
            field(
                K::LandingLog,
                RestoreSet::LandingsKept,
                count(row.landings.len()),
                u64::from(l.landings_kept.checked_add(l.landings_pending).expect("checked landing count")),
            )?;
            for landing in &row.landings {
                field(
                    K::LandingLog,
                    RestoreSet::LandingMembers,
                    count(landing.changes.len()),
                    u64::from(l.landing_members),
                )?;
                field(K::LandingLog, RestoreSet::NameBytes, count(landing.branch.len()), u64::from(l.name_bytes))?;
                field(K::LandingLog, RestoreSet::LoggedTiers, count(landing.tiers.len()), 32)?;
                for tier in &landing.tiers {
                    field(K::LandingLog, RestoreSet::NameBytes, count(tier.tier.len()), 32)?;
                }
            }
            room(
                K::LandingLog,
                Rows(K::LandingLog),
                d.landing_logs.len(),
                d.landing_logs.capacity(),
                d.landing_logs.contains_key(&row.repository),
            )
        }
        Stored::Verification(row) => room(
            K::Verification,
            Rows(K::Verification),
            d.verifications.len(),
            d.verifications.capacity(),
            d.verifications.contains_key(&(row.repository, row.commit)),
        ),
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
        Stored::Repository(row) => {
            let contexts = count(row.contexts.len()).saturating_add(u64::from(row.role == crate::Role::Context));
            field(K::Repository, RestoreSet::ContextProjects, contexts, u64::from(l.judge_projects))?;
            room(
                K::Repository,
                Rows(K::Repository),
                d.repositories.len(),
                d.repositories.capacity(),
                d.repositories.contains_key(&row.provider),
            )
        }
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
        Stored::Making(row) => room(
            K::Making,
            Rows(K::Making),
            d.repository_makings.len(),
            d.repository_makings.capacity(),
            d.repository_makings.contains_key(&row.key),
        ),
        Stored::PersonBrief(row) => room(
            K::PersonBrief,
            Rows(K::PersonBrief),
            d.person_briefs.len(),
            d.person_briefs.capacity(),
            d.person_briefs.contains_key(&(row.party, row.project)),
        ),
        Stored::ProjectFacts(row) => room(
            K::ProjectFacts,
            Rows(K::ProjectFacts),
            d.project_facts.len(),
            d.project_facts.capacity(),
            d.project_facts.contains_key(&row.project),
        ),
        Stored::Device(row) => {
            let mut replacing = false;
            let mut held = 0_u32;
            for ((_, number), _) in d.devices_of(row.party) {
                held = held.saturating_add(1);
                if *number == row.number {
                    replacing = true;
                }
            }
            let total = match &d.devices {
                Some(state) => state.rows.len(),
                None => 0,
            };
            room(K::Device, Rows(K::Device), held, crate::DEVICES, replacing)?;
            room(K::Device, Rows(K::Device), total, l.devices(), replacing)
        }
        Stored::LandedDraft(row) => {
            let replacing = crate::drafts::held(d, &(row.repository, row.path.clone())).is_some();
            room(K::LandedDraft, Rows(K::LandedDraft), crate::drafts::count(d, row.project), crate::DRAFTS, replacing)?;
            room(
                K::LandedDraft,
                Rows(K::LandedDraft),
                crate::drafts::len(d),
                crate::DRAFTS.saturating_mul(l.judge_projects),
                replacing,
            )
        }
        Stored::Away(row) => room(
            K::Away,
            Rows(K::Away),
            d.aways.len(),
            d.aways.capacity(),
            d.aways.contains_key(&(row.party, row.project)),
        ),
        Stored::Red(row) => {
            let replacing = d.reds.contains_key(&(row.goal, row.key.clone()));
            let held = u32::try_from(d.reds(row.goal).count()).expect("bounded reds");
            room(K::Red, Rows(K::Red), held, l.branch_reds, replacing)?;
            room(K::Red, Rows(K::Red), d.reds.len(), d.reds.capacity(), replacing)
        }
        Stored::RedPass(row) => {
            let replacing = d.red_passes.contains_key(&(row.change, row.key.clone()));
            let held = u32::try_from(d.red_passes(row.change).count()).expect("bounded red passes");
            room(K::RedPass, Rows(K::RedPass), held, l.branch_reds, replacing)?;
            room(K::RedPass, Rows(K::RedPass), d.red_passes.len(), d.red_passes.capacity(), replacing)
        }
        Stored::RedNumber { .. } => room(K::RedNumber, Rows(K::RedNumber), u32::from(d.red_number.is_some()), 1, false),
        Stored::Finding(row) => {
            let replacing = d.finding_rows().contains_key(&(row.project, row.number));
            let mut held = 0_u32;
            for _ in d.finding_rows().range((row.project, 0)..=(row.project, u32::MAX)) {
                held = held.saturating_add(1);
            }
            room(K::Finding, Rows(K::Finding), held, l.findings_per_project, replacing)?;
            room(K::Finding, Rows(K::Finding), d.finding_rows().len(), l.findings(), replacing)
        }
        Stored::Decided(row) => {
            let replacing = d.decisions.contains_key(&(row.project, row.number));
            let mut held = 0_u32;
            for _ in d.decisions.range((row.project, 0)..=(row.project, u32::MAX)) {
                held = held.saturating_add(1);
            }
            room(K::Decided, Rows(K::Decided), held, l.decided_per_project, replacing)?;
            room(K::Decided, Rows(K::Decided), d.decisions.len(), l.decided(), replacing)
        }
        Stored::Looked(row) => room(
            K::Looked,
            Rows(K::Looked),
            d.looked.len(),
            d.looked.capacity(),
            d.looked.contains_key(&(row.party, row.project)),
        ),
        Stored::Dismissed { party, what } => room(
            K::Dismissed,
            Rows(K::Dismissed),
            d.dismissed.len(),
            d.dismissed.capacity(),
            d.dismissed.contains_key(&(*party, *what)),
        ),
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
        Stored::RefusedSetting(key) => room(
            K::Setting,
            Rows(K::Setting),
            d.setting_rows.len().saturating_add(d.refused_settings.len()),
            d.setting_rows.capacity(),
            d.refused_settings.contains_key(key),
        ),
        Stored::Setting(row) => {
            if let crate::settings::Edit::Set(crate::settings::SizedValue {
                value: crate::settings::Value::Rule(rule),
                ..
            }) = &row.edit
            {
                field(K::Setting, RestoreSet::SettingGates, count(rule.gates.len()), u64::from(l.change_policy.gates))?;
                field(K::Setting, RestoreSet::SettingApprovals, count(rule.approvals.len()), u64::from(l.roles))?;
                field(
                    K::Setting,
                    RestoreSet::SettingSegments,
                    count(rule.pattern.segments.len()),
                    u64::from(l.authority_segments),
                )?;
            }
            room(
                K::Setting,
                Rows(K::Setting),
                d.setting_rows.len().saturating_add(d.refused_settings.len()),
                d.setting_rows.capacity(),
                d.setting_rows.contains_key(&row.key),
            )
        }
        Stored::Flight { task, entry } => room(
            K::Flight,
            Rows(K::Flight),
            d.flights.len(),
            d.flights.capacity(),
            d.flights.contains_key(&(*task, *entry)),
        ),
        Stored::Writer(row) => {
            room(K::Writer, Rows(K::Writer), d.writers.len(), d.writers.capacity(), d.writers.contains_key(&row.forge))
        }
        Stored::Closer(row) => {
            room(
                K::Closer,
                Rows(K::Closer),
                d.closing.rows.len(),
                d.closing.rows.capacity(),
                d.closing.rows.contains_key(&row.task),
            )?;
            if !row.done {
                room(
                    K::Closer,
                    RestoreSet::ChangeTimers,
                    d.change_due.len(),
                    d.change_due.capacity(),
                    d.change_due.contains_key(&row.task),
                )?;
            }
            Ok(())
        }
        Stored::ChangeCold(row) => room(
            K::Change,
            Rows(K::Change),
            d.change_cold.len(),
            d.change_cold.capacity(),
            d.change_cold.contains_key(&row.task),
        ),
        Stored::ChangeHot(row) => {
            room(
                K::Change,
                Rows(K::Change),
                d.change_hot.len(),
                d.change_hot.capacity(),
                d.change_hot.contains_key(&row.task),
            )?;
            room(
                K::Change,
                RestoreSet::ChangeTimers,
                d.change_due.len(),
                d.change_due.capacity(),
                d.change_due.contains_key(&row.task),
            )
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
        Stored::Writes(_)
        | Stored::LimitHold(_)
        | Stored::HistoryEvent(_)
        | Stored::HistoryChange(_)
        | Stored::HistoryLanding(_)
        | Stored::HistoryLast(_)
        | Stored::ForgeNumber { .. }
        | Stored::Playground(_)
        | Stored::Client(_) => Ok(()),
    }
}

fn derived(d: &Domain, l: &Limits) -> Result<(), RestoreFit> {
    let mut projects = Map::uncapped();
    for (_, row) in &d.milestones {
        let (mut total, mut live) = projects.get(&row.project).copied().unwrap_or((0u32, 0u32));
        total = total.saturating_add(1);
        if row.ended.is_none() {
            live = live.saturating_add(1);
        }
        projects.insert(row.project, (total, live)).expect("derived project counts");
    }
    for (_, (total, live)) in &projects {
        field(
            RecordKind::Milestone,
            RestoreSet::Rows(RecordKind::Milestone),
            u64::from(*total),
            u64::from((*live).max(crate::milestone_rows::MILESTONES_PER_PROJECT)),
        )?;
    }
    let mut watches = Map::with_capacity(l.client.resources);
    for (_, names) in &d.names {
        for name in names {
            if d.forge_ended(name.forge) || super::goal_run(d, name) {
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
    for name in &crate::tiers::watched(d) {
        let Some(resource) = super::to_client(name, l) else {
            return Err(RestoreFit {
                family: Some(RecordKind::Tier),
                set: RestoreSet::NameBytes,
                held: u64::from(l.name_bytes),
                additional: 1,
                limit: u64::from(l.name_bytes),
            });
        };
        let held = watches.len();
        if watches.insert(resource, ()).is_err() {
            return Err(RestoreFit {
                family: Some(RecordKind::Tier),
                set: RestoreSet::Watches,
                held: u64::from(held),
                additional: 1,
                limit: u64::from(watches.capacity()),
            });
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
