//! Settings restore precedes live work; stored defaults survive until its holders are known (engine.md, 17.7–17.8).
use crate::{
    Core, CoreKey, CoreRecord, HeldLimit, Limits, Request, Write,
    settings::{
        CharterVersion, Decider, Default, Edit, Failure, Fault, Head, Item, Key, Record, Revision, Setting, VersionKey,
    },
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Map, Queue};

#[derive(Debug)]
pub(crate) struct Restore {
    pending: Map<Setting, bool>,
    // Stored defaults until settings_restored swaps them with the supplied
    // defaults. Tasks restore against the stored overlay, before a release.
    defaults: Map<Setting, Default>,
    held: bool,
    extent: bool,
    ready: bool,
    release: Option<u32>,
}
impl Restore {
    pub(crate) fn new(limits: &Limits) -> Self {
        let count = super::capacity(limits).expect("settings bound");
        Self {
            pending: Map::with_capacity(count.checked_mul(2).expect("default union")),
            defaults: Map::with_capacity(count),
            held: false,
            extent: false,
            ready: false,
            release: None,
        }
    }
}
impl Core {
    pub(crate) fn restore_setting(&mut self, record: Record, limits: &Limits) -> bool {
        if self.settings_restore.ready {
            return false;
        }
        match record {
            Record::Default { setting, default } => self.restore_default(setting, default, limits),
            Record::Changed { setting, changed } => {
                {
                    assert!(
                        !self.current.changes.contains_key(&setting),
                        "duplicate {:?}",
                        crate::Key::Core(CoreKey::Settings(Key::Changed(setting)))
                    );
                }
                if let Err(failure) = super::check::admits_change(&self.current, setting, limits) {
                    self.refuse_restore(crate::start::Failure::Settings { failure, changed: true });
                    return false;
                }
                if let Some(value) = &changed.value
                    && let Err(failure) = super::check::restore_fit(setting, value, limits)
                {
                    self.refuse_restore(crate::start::Failure::Settings { failure, changed: true });
                    return false;
                }
                self.current.changes.insert(setting, changed).is_ok()
            }
            Record::Revision { history, revision } => self.restore_head(history, revision, limits),
            Record::Extent { oldest } => {
                {
                    assert!(
                        !self.settings_restore.extent,
                        "duplicate {:?}",
                        crate::Key::Core(CoreKey::Settings(Key::Extent))
                    );
                }
                self.current.oldest = oldest;
                self.settings_restore.extent = true;
                true
            }
            Record::Charter { charter, revision, value } => self.restore_charter(charter, revision, value, limits),
            Record::Proposal { proposer, proposal, value } => {
                self.restore_settings_proposal(proposer, proposal, value, limits)
            }
        }
    }

    fn restore_settings_proposal(
        &mut self,
        proposer: jig_domain_people::Proposer,
        proposal: u32,
        value: super::Proposal,
        limits: &Limits,
    ) -> bool {
        {
            assert!(
                !self.settings_proposals.contains_key(&(proposer, proposal)),
                "duplicate {:?}",
                crate::Key::Core(CoreKey::Settings(Key::Proposal { proposer, proposal }))
            );
            if value.items.len() > usize::try_from(tasks::SETTINGS_ITEMS).expect("item bound")
                || self.settings_proposals.len() >= self.settings_proposals.capacity()
            {
                self.refuse_restore(crate::start::Failure::Settings {
                    failure: Failure { setting: Setting::Rules, fault: Fault::Count },
                    changed: true,
                });
                return false;
            }
            for item in &value.items {
                if let Edit::Set(value) = &item.edit
                    && let Err(failure) = super::check::restore_fit(item.setting, value, limits)
                {
                    self.refuse_restore(crate::start::Failure::Settings { failure, changed: true });
                    return false;
                }
            }
        }
        self.settings_proposals.insert((proposer, proposal), value).is_ok()
    }

    fn restore_default(&mut self, setting: Setting, default: Default, limits: &Limits) -> bool {
        {
            assert!(
                !self.settings_restore.defaults.contains_key(&setting),
                "duplicate {:?}",
                crate::Key::Core(CoreKey::Settings(Key::Default(setting)))
            );
        }
        match self.current.defaults.get_mut(&setting) {
            Some(given) if given.value == default.value => given.revision = default.revision,
            Some(_) => {
                if self.settings_restore.pending.insert(setting, true).is_err() {
                    return false;
                }
            }
            None => {
                if self.settings_restore.pending.insert(setting, false).is_err() {
                    return false;
                }
            }
        }
        if let Err(failure) = super::check::restore_fit(setting, &default.value, limits) {
            let _refused = self.refuse_settings(failure);
            return false;
        }
        let mut counts = [0_u32; 3];
        for (&stored, _) in &self.settings_restore.defaults {
            if super::check::count(stored, &mut counts, limits).is_err() {
                return false;
            }
        }
        if let Err(failure) = super::check::count(setting, &mut counts, limits) {
            let _refused = self.refuse_settings(failure);
            return false;
        }
        if self.settings_restore.defaults.insert(setting, default).is_err() {
            let _refused = self.refuse_settings(Failure { setting, fault: Fault::Count });
            return false;
        }
        true
    }

    fn restore_head(&mut self, _history: bool, revision: Revision, limits: &Limits) -> bool {
        let number = revision.head.number;
        if self.settings_restore.held {
            let duplicate = number == self.current.newest() || self.held.heads.contains_key(&number);
            assert!(!duplicate, "duplicate {:?}", crate::Key::Core(CoreKey::Settings(Key::Held(number))));
            let held = self.held.heads.len().checked_add(1).expect("held row count");
            if held > limits.settings.revisions {
                self.refuse_held(HeldLimit::Revisions, held, limits.settings.revisions);
                return false;
            }
            if number > self.current.newest() {
                let previous = core::mem::replace(&mut self.current.head, revision.head);
                self.held.heads.insert(previous.number, previous).expect("restored head bound");
            } else {
                self.held.heads.insert(number, revision.head).expect("restored head bound");
            }
        } else {
            self.current.head = revision.head;
            self.settings_restore.held = true;
        }
        true
    }

    fn restore_charter(&mut self, charter: u32, revision: u32, value: CharterVersion, limits: &Limits) -> bool {
        let key = VersionKey { charter, revision };
        {
            assert!(
                !self.held.versions.contains_key(&key),
                "duplicate {:?}",
                crate::Key::Core(CoreKey::Settings(Key::Charter { charter, revision }))
            );
        }
        let held = self.held.versions.len().checked_add(1).expect("version row count");
        if held > limits.settings.versions {
            self.refuse_held(HeldLimit::Versions, held, limits.settings.versions);
            return false;
        }
        if let Some(value) = &value.charter {
            let stored = super::Value::Charter(value.clone());
            if let Err(failure) = super::check::restore_fit(Setting::Charter(charter), &stored, limits) {
                let _refused = self.refuse_settings(failure);
                return false;
            }
        }
        self.held.versions.insert(key, value).is_ok()
    }

    pub fn settings_restored(&mut self, env: &Env<Limits>) -> bool {
        self.start_input();
        self.start_settings_finish(env).is_none()
    }

    pub(crate) fn finish_restored_settings(&mut self, env: &Env<Limits>) -> bool {
        if self.settings_restore.ready || self.restore_failure.is_some() {
            return self.restore_failure.is_none();
        }
        if self.counters.deployment().commits == 0 && !self.settings_restore.held {
            return true;
        }
        for (&setting, _) in &self.current.defaults {
            if !self.settings_restore.defaults.contains_key(&setting) {
                assert!(self.settings_restore.pending.insert(setting, true).is_ok(), "bounded added default");
            }
        }
        if let Err(failure) = super::check(&self.current, &self.settings, &self.connectors, &env.limits) {
            let _request = self.refuse_settings(failure);
            return false;
        }
        let mut changed = false;
        for (&setting, _) in &self.settings_restore.pending {
            changed |= !self.current.changed(setting);
        }
        if changed {
            let previous = self.current.newest();
            if previous.checked_add(1).is_none() {
                let _request = self.refuse_settings(Failure { setting: Setting::Rules, fault: Fault::Count });
                return false;
            }
            // The new head and defaults take effect only after tasks have
            // restored against the stored settings and established their holders.
            self.settings_restore.release = Some(previous);
        }
        if !self.settings_restore.defaults.is_empty() {
            core::mem::swap(&mut self.current.defaults, &mut self.settings_restore.defaults);
        }
        self.settings_restore.ready = true;
        self.tell_settings(env);
        true
    }
    /// The typed refusal is retained even when a later completion repeats it.
    pub fn refuse_settings(&mut self, failure: Failure) -> crate::start::Failure {
        self.start_input();
        let changed = self.current.changed(failure.setting);
        let request = crate::start::Failure::Settings { failure, changed };
        self.refuse_restore(request.clone());
        request
    }

    fn refuse_held(&mut self, which: HeldLimit, held: u32, limit: u32) {
        self.refuse_restore(crate::start::Failure::HeldPastLimit { which, held, limit });
    }

    pub(crate) fn refuse_restore(&mut self, request: crate::start::Failure) {
        if self.restore_failure.is_none() {
            self.restore_failure = Some(request);
        }
        self.start_collect_failure();
    }

    pub(crate) fn tell_settings(&mut self, env: &Env<Limits>) {
        self.tell_replaced_settings(env, true, true);
    }
    pub(crate) fn tell_replaced_settings(&mut self, env: &Env<Limits>, rules: bool, charters: bool) {
        if rules {
            let mut facts = Queue::uncapped();
            crate::routing::step::step_authority(
                self,
                authority::Event::Rules(Box::new(self.current.rules().clone())),
                &mut facts,
            );
            assert!(facts.is_empty(), "rules replacement has no policy lifecycle fact");
        }
        let versions = charters || self.tasks.newest() != self.current.newest();
        if versions {
            let models = super::charters::held_models(self, &env.limits);
            let mut charters: List<tasks::CharterModels> =
                List::with_capacity(u32::try_from(models.len()).expect("charter model count"));
            for model in models {
                let mut present = false;
                for previous in &charters {
                    present |= previous.number == model.number && previous.revision == model.revision;
                }
                // Current takes precedence over a held copy of the same version,
                // as charter_at does when selecting the version for a task.
                if !present {
                    charters.push(model).expect("charter model count");
                }
            }
            let charters = charters.into_boxed();
            let mut tasks_out = Queue::uncapped();
            crate::routing::step::step_tasks(
                self,
                env,
                tasks::Event::Charters {
                    newest: self.current.newest(),
                    charters,
                    efforts: super::charters::efforts(&self.settings),
                },
                &mut tasks_out,
            );
            assert!(tasks_out.is_empty(), "charter table replacement emits nothing");
        }
        if !versions && !rules {
            return;
        }
        let needs = super::fleet_needs::current(self, &env.limits);
        let mut fleet_out = Queue::uncapped();
        crate::routing::step::step_fleet(self, env, fleet::Event::Needs(needs), &mut fleet_out);
        assert!(fleet_out.is_empty(), "needs replacement retains running attempts");
    }

    fn restored_items(&self) -> List<Item> {
        let count = self.settings_restore.pending.len();
        let mut items = List::with_capacity(count);
        for (&setting, &present) in &self.settings_restore.pending {
            if self.current.changed(setting) {
                continue;
            }
            let edit = if present {
                Edit::Set(self.settings_restore.defaults.get(&setting).expect("given default").value.clone())
            } else {
                Edit::Removed
            };
            items.push(Item { setting, edit }).expect("release item count");
        }
        items
    }

    fn release_restored_heads(&mut self, env: &Env<Limits>, after: &Map<u32, ()>, out: &mut Queue<Request>) {
        for _ in 0..self.held.heads.len() {
            let mut orphan = None;
            for (&revision, _) in &self.held.heads {
                if !after.contains_key(&revision) {
                    orphan = Some(revision);
                    break;
                }
            }
            if let Some(revision) = orphan {
                let _retained = self.archive_revision(env, revision, out);
            }
        }
    }

    pub(crate) fn commit_restored_settings(&mut self, env: &Env<Limits>, room: u32) -> Queue<Request> {
        self.write_restored_settings(env, room)
    }

    pub(crate) fn write_restored_settings(&mut self, env: &Env<Limits>, room: u32) -> Queue<Request> {
        let mut out = Queue::uncapped();
        if !self.settings_restore.held || self.settings_restore.defaults.is_empty() || self.restore_failure.is_some() {
            return out;
        }
        let oldest_before = self.current.oldest;
        let previous = self.current.newest();
        let release = self.settings_restore.release.is_some();
        if release && !crate::routing::settings::dispatch::withdrawal_room(self, None) {
            let _refused = self.refuse_settings(Failure { setting: Setting::Rules, fault: Fault::Count });
            return out;
        }
        let newest = if release { previous.checked_add(1).expect("release number preflighted") } else { previous };
        let reserve = if release { super::withdrawal_writes(&env.limits).expect("withdrawal room") } else { 0 };
        let count = self.settings_restore.pending.len();
        let items = self.restored_items();
        let mut after = Map::with_capacity(env.limits.settings.revisions.checked_add(1).expect("held names"));
        for (&revision, _) in &self.held.heads {
            if self.tasks.holders(revision) > 0 {
                after.insert(revision, ()).expect("restored held names");
            }
        }
        let keep_previous = release && self.tasks.holders(previous) > 0;
        if keep_previous {
            after.insert(previous, ()).expect("previous newest name");
        }
        let plan = self.held.plan_release(&self.current, items.as_slice(), newest, &after, &env.limits);
        if after.len() > env.limits.settings.revisions {
            self.refuse_held(HeldLimit::Revisions, after.len(), env.limits.settings.revisions);
            return out;
        }
        if plan.versions() > env.limits.settings.versions {
            self.refuse_held(HeldLimit::Versions, plan.versions(), env.limits.settings.versions);
            return out;
        }
        // No writes leave a refused start. Archive orphan heads and apply the
        // single preflighted version plan before bounded history pruning.
        self.release_restored_heads(env, &after, &mut out);
        self.held.retain(&self.current, &plan, &mut out);
        drop(plan);
        if keep_previous {
            self.held.heads.insert(previous, self.current.head.clone()).expect("release heads preflighted");
        }
        core::mem::swap(&mut self.current.defaults, &mut self.settings_restore.defaults);
        if release {
            self.current.head = Head {
                number: newest,
                against: previous,
                at: env.wall,
                by: Decider::Release,
                proposal: None,
                moved: false,
            };
        }
        let mut defaults = List::with_capacity(count);
        for (&setting, &present) in &self.settings_restore.pending {
            let write = if present {
                let default = self.current.defaults.get_mut(&setting).expect("given default");
                default.revision = newest;
                save(Record::Default { setting, default: default.clone() })
            } else {
                Write::Erase(crate::Key::Core(CoreKey::Settings(Key::Default(setting))))
            };
            defaults.push(write).expect("default differences");
        }
        let revision =
            if release { Some(Revision { head: self.current.head.clone(), items: items.into_boxed() }) } else { None };
        let (writes, oldest) = revision_writes_with_hold(
            revision,
            defaults,
            self.current.oldest,
            newest,
            env.limits.settings.history,
            room.saturating_sub(reserve).saturating_sub(out.len()),
            keep_previous,
        );
        self.current.oldest = oldest;
        for write in writes.into_boxed() {
            out.push(Request::Write(write));
        }
        // Archive-only starts still save their changed history extent.
        self.finish_extent(oldest_before, &mut out);
        self.tell_settings(env);
        if release {
            crate::routing::settings::dispatch::withdraw_stale(self, env, &mut out, None);
        }
        self.settings_restore = Restore::new(&env.limits);
        self.settings_restore.ready = true;
        out
    }
}
fn save(record: Record) -> Write {
    Write::Save(crate::Record::Core(CoreRecord::Settings(record)))
}

/// A revision's writes and bounded history pruning, shared with changing settings.
/// `room` excludes the deployment header which the journal's root adds.
#[must_use]
pub fn revision_writes(
    revision: Option<Revision>,
    defaults: List<Write>,
    oldest: u32,
    newest: u32,
    history: u32,
    room: u32,
) -> (List<Write>, u32) {
    revision_writes_with_hold(revision, defaults, oldest, newest, history, room, false)
}

pub(crate) fn revision_writes_with_hold(
    revision: Option<Revision>,
    defaults: List<Write>,
    mut oldest: u32,
    newest: u32,
    history: u32,
    room: u32,
    keep_previous: bool,
) -> (List<Write>, u32) {
    let mut writes = List::with_capacity(room);
    for write in defaults.into_boxed() {
        writes.push(write).expect("settings commit preflight");
    }
    if let Some(revision) = revision {
        let previous = revision.head.against;
        if previous != 0 && !keep_previous {
            if previous >= newest.saturating_sub(history).max(1) {
                writes.push(Write::CopySettingsRevision(previous)).expect("revision archive room");
                if oldest == 0 {
                    oldest = previous;
                }
            }
            writes
                .push(Write::Erase(crate::Key::Core(CoreKey::Settings(Key::Held(previous)))))
                .expect("old held row erase room");
        }
        writes.push(save(Record::Revision { history: false, revision })).expect("new held row room");
    }
    let cutoff = newest.saturating_sub(history).max(1);
    let pruning = oldest != 0 && oldest < cutoff;
    let changed = !writes.is_empty();
    if changed || pruning {
        let begin = oldest;
        for _ in begin..cutoff {
            if oldest == 0 || writes.room() <= 1 {
                break;
            }
            writes
                .push(Write::Erase(crate::Key::Core(CoreKey::Settings(Key::History(oldest)))))
                .expect("prune room reserved");
            oldest = oldest.checked_add(1).expect("bounded old revision");
        }
        writes.push(save(Record::Extent { oldest })).expect("extent room reserved");
    }
    (writes, oldest)
}

pub(crate) fn first_start(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
    if let Err(failure) = super::check(&core.current, &core.settings, &core.connectors, &env.limits) {
        let _request = core.refuse_settings(failure);
        return false;
    }
    core.settings_restore.ready = true;
    core.settings_restore.held = true;
    core.settings_restore.extent = true;
    core.current.head.at = env.wall;
    for (&setting, default) in &core.current.defaults {
        out.push(Request::Write(save(Record::Default { setting, default: default.clone() })));
    }
    out.push(Request::Write(save(Record::Revision {
        history: false,
        revision: Revision { head: core.current.head.clone(), items: Box::new([]) },
    })));
    out.push(Request::Write(save(Record::Extent { oldest: 0 })));
    true
}
