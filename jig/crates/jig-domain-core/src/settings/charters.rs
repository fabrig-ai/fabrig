//! Configured charters and readiness (domain/engine/executors.md, 1 and domain/engine/fleet.md).

pub use crate::below::settings::charters::*;

use alloc::boxed::Box;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue};

use super::{CatalogueModel, Current, Edit, Item, Setting};
use crate::{Core, Limits, Settings};

pub(crate) fn left_out(charter: &Charter, models: &[CatalogueModel], limits: &Limits) -> u32 {
    resolution(charter, models, limits).1
}

pub(crate) fn rebuild(current: &mut Current, limits: &Limits) {
    let models = super::catalogue::entries(current);
    current.resolved = skein_lib::Map::with_capacity(limits.tasks.charters);
    for number in current.offered_charters() {
        if let Some(charter) = resolved(
            &current
                .made_charter(Setting::Charter(number), current.value(Setting::Charter(number)).expect("offered value"))
                .expect("offered charter"),
            &models,
            limits,
        ) {
            current.resolved.insert(number, charter).expect("checked charter count");
        }
    }
}

/// Names alone are enough for held-version preflight; implicit changes retain the set value.
pub(crate) fn version_items(current: &Current, items: &[Item], limits: &Limits) -> Box<[Item]> {
    let models = super::catalogue::overlay(current, false, items);
    let mut result = List::with_capacity(
        u32::try_from(items.len())
            .expect("bounded items")
            .checked_add(limits.tasks.charters)
            .expect("implicit charter items"),
    );
    for item in items {
        result.push(item.clone()).expect("given items");
    }
    for number in current.offered_charters() {
        let setting = Setting::Charter(number);
        if super::check::edit(items, setting).is_some() {
            continue;
        }
        let value = current.value(setting).expect("offered value");
        let charter = current.made_charter(setting, value).expect("offered charter");
        let after = resolved(&charter, &models, limits);
        if after.as_ref() != Some(current.resolved_charter(number).expect("offered resolution")) {
            result.push(Item { setting, edit: Edit::Set(value.clone()) }).expect("implicit charter items");
        }
    }
    result.into_boxed()
}

/// Stamp only resolutions that a model edit changed; their stored charter stays as set.
pub(crate) fn stamp(current: &mut Current, items: &[Item], revision: u32) -> List<crate::Write> {
    let mut writes = List::with_capacity(u32::try_from(items.len()).expect("version items"));
    for item in items {
        match item.setting {
            Setting::Charter(_) => {}
            Setting::Template(_)
            | Setting::Policy(_)
            | Setting::Model(_)
            | Setting::Rules
            | Setting::Application { .. }
            | Setting::ProjectPolicy { .. } => continue,
        }
        let record = if let Some(change) = current.changes.get_mut(&item.setting) {
            change.revision = revision;
            super::Record::Changed { setting: item.setting, changed: change.clone() }
        } else if let Some(default) = current.defaults.get_mut(&item.setting) {
            default.revision = revision;
            super::Record::Default { setting: item.setting, default: default.clone() }
        } else {
            continue;
        };
        writes
            .push(crate::Write::Save(crate::Record::Core(crate::CoreRecord::Settings(record))))
            .expect("one stamp per charter");
    }
    writes
}

const _: () = assert!(fleet::CATALOGUE_MODELS == tasks::MODELS, "fleet evidence prices the model catalogue");
const _: () = assert!(fleet::CATALOGUE_NAME_BYTES == crate::MODEL_NAME_BYTES, "fleet evidence prices model names");

pub(crate) fn assert_catalogue(settings: &Settings) {
    assert!(settings.models.len() <= skein_lib::length::usize_of(tasks::MODELS), "model count");
    for (at, model) in settings.models.iter().enumerate() {
        if model.is_hole() {
            continue;
        }
        for earlier in settings.models.iter().take(at) {
            assert!(earlier.name != model.name, "unique configured model names");
        }
        assert!(model.name.len() <= skein_lib::length::usize_of(crate::MODEL_NAME_BYTES), "model name bound");
        if let Some(program) = &model.program {
            assert!(
                !program.is_empty() && program.len() <= skein_lib::length::usize_of(jig_domain_accounts::NAME),
                "program name bound"
            );
        }
        assert!(model.efforts.len() <= skein_lib::length::usize_of(crate::EFFORTS), "effort count");
        for (at, effort) in model.efforts.iter().enumerate() {
            assert!(
                !effort.is_empty() && effort.len() <= skein_lib::length::usize_of(crate::EFFORT_BYTES),
                "effort name bound"
            );
            for earlier in model.efforts.iter().take(at) {
                assert!(earlier != effort, "unique effort names");
            }
        }
        assert!(declares(model), "a model's effort among those it declares");
    }
}

/// Each charter's models as tasks checks an LLM choice against them: its main model, then its
/// alternatives, each once, by its number's position in the catalogue (domain/tasks/README.md, 3).
pub(crate) fn models(current: &Current, settings: &Settings, limits: &Limits) -> Box<[tasks::CharterModels]> {
    let mut charters = List::with_capacity(
        limits
            .tasks
            .charters
            .checked_mul(2)
            .expect("newest charter states")
            .checked_add(limits.settings.versions)
            .expect("charter version bound"),
    );
    append_models(current, settings, &mut charters);
    charters.into_boxed()
}

fn append_models(current: &Current, settings: &Settings, charters: &mut List<tasks::CharterModels>) {
    for number in current.offered_charters() {
        let charter = current.resolved_charter(number).expect("offered charter");
        let (_, origin) = current.lookup(Setting::Charter(number));
        let (_, revision) = origin.expect("offered charter version");
        let value = one_models(charter, revision, settings);
        charters.push(value).expect("configured charter count");
    }
    for (setting, change) in &current.changes {
        if let Setting::Charter(number) = setting
            && change.value.is_none()
        {
            charters
                .push(tasks::CharterModels { resumes: false, number: *number, revision: change.revision, models: None })
                .expect("removed charter version bound");
        }
    }
}

pub(crate) fn one_models(charter: &Charter, revision: u32, settings: &Settings) -> tasks::CharterModels {
    one_models_rows(charter, revision, &settings.models)
}

pub(crate) fn one_models_rows(charter: &Charter, revision: u32, rows: &[crate::Model]) -> tasks::CharterModels {
    let count = charter.policy.alternatives.len().checked_add(1).expect("charter model count");
    let mut models = List::with_capacity(u32::try_from(count).expect("charter model count"));
    if let Some(main) = position(rows, &charter.policy.model) {
        models.push(main).expect("charter model count");
    }
    for model in &charter.policy.alternatives {
        let Some(index) = position(rows, model) else { continue };
        if !models.as_slice().contains(&index) {
            models.push(index).expect("charter model count");
        }
    }
    tasks::CharterModels {
        number: charter.number,
        revision,
        resumes: charter.policy.resume,
        models: Some(models.into_boxed()),
    }
}

/// How many efforts each catalogue row declares, by number (domain/tasks/README.md, 3).
pub(crate) fn efforts(settings: &Settings) -> Box<[u8]> {
    let mut efforts = List::with_capacity(u32::try_from(settings.models.len()).expect("model count"));
    for model in &settings.models {
        efforts.push(u8::try_from(model.efforts.len()).expect("efforts named by a byte")).expect("model count");
    }
    efforts.into_boxed()
}

/// Whether `model`'s effort, if any, is one it declares (domain/tasks/README.md, 3).
fn declares(model: &crate::Model) -> bool {
    match &model.effort {
        Some(effort) => model.efforts.contains(effort),
        None => true,
    }
}

fn position(rows: &[crate::Model], model: &crate::Model) -> Option<u8> {
    if model.is_hole() {
        return None;
    }
    for (index, configured) in rows.iter().enumerate() {
        if configured.name == model.name {
            return u8::try_from(index).ok();
        }
    }
    None
}

impl Core {
    pub(crate) fn claim_done(&mut self, task: u64, attempt: u64, out: &mut Queue<crate::Request>) {
        drop(self.replaced.remove(&task));
        let proof = self.proofs.get(&task).expect("claim proof pre-reserved");
        out.push(crate::Request::Write(crate::Write::Save(crate::Record::Core(crate::CoreRecord::RunProof(
            proof.clone(),
        )))));
        match self.tasks.task(task).expect("claimed task").executor {
            tasks::Executor::Job { .. } => {
                crate::routing::run::jobs::claimed(self, task, attempt, out);
                return;
            }
            tasks::Executor::Agent { .. } => {}
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => unreachable!("claimed executor"),
        }
        let version = self.charter_version(task).expect("agent charter version");
        let kinds = match proof.host {
            fleet::HostKind::Engine => fleet::Kinds::Engine,
            fleet::HostKind::Worker => fleet::Kinds::Workers,
        };
        for &connector in self.connectors.as_ref() {
            out.push(crate::Request::Ask { connector, ask: crate::Ask::ClaimDone { task, attempt, kinds, version } });
        }
    }

    /// The value read by this live agent's run, or its task before preparation.
    pub(crate) fn charter_version(&self, task: u64) -> Option<fleet::Version> {
        let row = self.tasks.task(task).expect("live agent task");
        let charter = match row.executor {
            tasks::Executor::Agent { charter } => charter,
            tasks::Executor::Job { .. } => return None,
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => unreachable!("agent version"),
        };
        let revision = row.settings.run.unwrap_or(row.settings.task);
        let key = self.held.selected(&self.current, charter, revision).expect("run charter version");
        Some(fleet::Version { charter: key.charter, revision: key.revision })
    }

    /// The newest charter, for work about to be made (domain/engine/settings.md, 3).
    #[must_use]
    pub fn charter(&self, number: u32) -> Option<&Charter> {
        self.current.resolved_charter(number)
    }

    /// An admitted live agent task's run revision, or its own when no run lives.
    #[must_use]
    pub fn charter_of(&self, task: u64) -> &Charter {
        let row = self.tasks.task(task).expect("live agent task");
        let revision = row.settings.run.unwrap_or(row.settings.task);
        let charter = match row.executor {
            tasks::Executor::Agent { charter } => self.charter_at(charter, revision),
            tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } | tasks::Executor::Person(_) => None,
        };
        charter.expect("agent task has configured charter")
    }

    /// Whether a context's task is still due; one that left Due activates
    /// afresh if it is due again.
    fn still_due(&self, context: &tasks::RunContext) -> bool {
        let Some(current) = self.tasks.task(context.task) else { return false };
        current.phase == tasks::Phase::Active(tasks::Active::Due)
    }

    fn context_ready(&self, context: &tasks::RunContext) -> bool {
        if !self.still_due(context) {
            return false;
        }
        match self.tasks.task(context.task).expect("a due task is live").executor {
            tasks::Executor::Agent { .. } => self.agent_ready(context.task),
            tasks::Executor::Job { .. } => {
                match crate::routing::run::jobs::admission(self, context, self.run_wall, Box::new([])) {
                    Ok(()) | Err(crate::RunAdmission::Hold(_)) => true,
                    Err(crate::RunAdmission::Wait(_)) => false,
                    Err(crate::RunAdmission::Allow(_)) => unreachable!("job admission has no run allowance"),
                }
            }
            tasks::Executor::Procedure { .. } => true,
            tasks::Executor::Person(_) => false,
        }
    }

    /// Take an eligible goal-serving task first, lower priority number first,
    /// then earlier Due time and lower task number. Drop tasks no longer due.
    pub fn take_due(&mut self) -> Option<Box<tasks::RunContext>> {
        let mut taken: Option<Box<tasks::RunContext>> = None;
        for _ in 0..self.due.len() {
            let context = self.due.pop().expect("due count");
            if !self.still_due(&context) {
                continue;
            }
            let earlier = match &taken {
                Some(prior) => self.due_before(context.task, prior.task),
                None => true,
            };
            if earlier && self.context_ready(&context) {
                if let Some(prior) = taken.replace(context) {
                    self.due.push(prior);
                }
            } else {
                self.due.push(context);
            }
        }
        match taken {
            Some(context) => self.tasks.run_context(context.task),
            None => None,
        }
    }

    fn due_before(&self, left: u64, right: u64) -> bool {
        let l = self.tasks.task(left).expect("due candidate is live");
        let r = self.tasks.task(right).expect("due candidate is live");
        let lp = match self.goal_of(l) {
            Some(goal) => self.tasks.task(goal).expect("live goal").tracked,
            None => None,
        };
        let rp = match self.goal_of(r) {
            Some(goal) => self.tasks.task(goal).expect("live goal").tracked,
            None => None,
        };
        match (lp, rp) {
            (Some(lp), Some(rp)) if lp != rp => return lp < rp,
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
            (Some(_), Some(_)) | (None, None) => {}
        }
        l.phase_since < r.phase_since || (l.phase_since == r.phase_since && left < right)
    }

    /// Whether a due procedure or agent with a usable account can start.
    #[must_use]
    pub fn due_ready(&self) -> bool {
        for context in &self.due {
            if self.context_ready(context) {
                return true;
            }
        }
        false
    }
}

/// The newest view plus every retained value and erased-removal barrier.
pub(crate) fn held_models(core: &Core, limits: &Limits) -> Box<[tasks::CharterModels]> {
    let mut values = List::with_capacity(
        limits
            .tasks
            .charters
            .checked_mul(2)
            .expect("newest charter states")
            .checked_add(limits.settings.versions)
            .expect("version models"),
    );
    append_models(&core.current, &core.settings, &mut values);
    for (&key, value) in &core.held.versions {
        let models = match &value.charter {
            Some(charter) => one_models(charter, key.revision, &core.settings),
            None => tasks::CharterModels { resumes: false, number: key.charter, revision: key.revision, models: None },
        };
        values.push(models).expect("held model table");
    }
    values.into_boxed()
}
