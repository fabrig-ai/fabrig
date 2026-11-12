//! A task's named or current default LLM choice (domain/tasks/README.md, section 3;
//! domain/engine/executors.md, 1).

use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

use crate::{Core, Model, RunPolicy, Settings};

impl Core {
    /// Check each agent task's named or omitted choice (domain/tasks/README.md, section 3),
    /// and the members of a recurring task's template, which tasks makes each occurrence from.
    /// A task whose charter is configured but whose authority allows none of its models is
    /// refused, naming the charter's
    /// first model; a task of an unknown charter is left to tasks, which refuses its executor.
    #[cfg_attr(
        not(target_arch = "wasm32"),
        expect(clippy::result_large_err, reason = "the refusal keeps tasks' exact problem for the creator")
    )]
    pub(crate) fn resolve_llm(&self, batch: &mut [tasks::New]) -> Result<(), tasks::Problem> {
        let mut links = List::with_capacity(u32::try_from(batch.len()).expect("bounded batch"));
        for new in &*batch {
            links
                .push(tasks::PayerLink { task: new.number, funder: new.funder })
                .expect("one financial link per member");
        }
        for new in batch {
            let number = new.number;
            self.default_llm(new, number, links.as_slice(), None)?;
            if let Some(template) = &mut new.recurring {
                for member in &mut template.batch {
                    self.default_llm(member, number, &[], Some(tasks::Payer::Project(new.project)))?;
                }
            }
        }
        Ok(())
    }

    #[cfg_attr(
        not(target_arch = "wasm32"),
        expect(clippy::result_large_err, reason = "the refusal keeps tasks' exact problem for the creator")
    )]
    fn default_llm(
        &self,
        new: &mut tasks::New,
        number: u64,
        links: &[tasks::PayerLink],
        occurrence_payer: Option<tasks::Payer>,
    ) -> Result<(), tasks::Problem> {
        if let Err(why) = super::account_preference::executor(new.executor, new.llm.as_ref()) {
            return Err(tasks::Problem::new(Some(number), why));
        }
        let charter = match new.executor {
            tasks::Executor::Agent { charter } => charter,
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => {
                return Ok(());
            }
        };
        if let Some(choice) = &new.llm {
            if choice.account.is_some() {
                let payer = occurrence_payer
                    .or(self.tasks.prospective_payer(new.funder, links))
                    .ok_or(tasks::Problem::new(Some(number), tasks::Refusal::Funding))?;
                if let Err(why) = self.account_preference(Some(choice), payer, new.project, None) {
                    return Err(tasks::Problem::new(Some(number), why));
                }
            }
            return Ok(());
        }
        if self.tasks.default_llm(charter, new.authority.models).is_some() {
            return Ok(());
        }
        if let Some(models) = self.tasks.charter_models(charter, self.tasks.newest())
            && let Some(first) = models.first()
        {
            let why = tasks::Refusal::Llm { model: *first, why: tasks::LlmFault::Authority };
            return Err(tasks::Problem::new(Some(number), why));
        }
        Ok(())
    }

    /// What a task's next run takes: its named choice, or its charter's models within
    /// its authority, the first it can use its main (domain/accounts.md, 4.3).
    pub(crate) fn next_llm(&self, row: &tasks::TaskRecord) -> Option<tasks::LlmChoice> {
        self.task_llm(row, true)
    }

    /// A task given none names no model when no candidate serves it (domain/accounts.md, 4.3).
    pub(crate) fn document_llm(&self, row: &tasks::TaskRecord) -> Option<tasks::LlmChoice> {
        self.task_llm(row, false)
    }

    fn task_llm(&self, row: &tasks::TaskRecord, fallback: bool) -> Option<tasks::LlmChoice> {
        if let Some(choice) = &row.llm {
            if !row.reasons.meets(tasks::Reasons::FIRST)
                && let Some(later) = &choice.later
            {
                let charter = match row.executor {
                    tasks::Executor::Agent { charter } => Some(charter),
                    tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => {
                        None
                    }
                };
                let revision = row.settings.run.unwrap_or(row.settings.task);
                if let Some(charter) = charter
                    && let Some(charter) = self.charter_at(charter, revision)
                    && !charter.policy.resume
                {
                    return Some(tasks::LlmChoice {
                        account: choice.account,
                        main: later.main,
                        subagents: later.subagents.clone(),
                        later: None,
                    });
                }
            }
            return Some(choice.clone());
        }
        let charter = match row.executor {
            tasks::Executor::Agent { charter } => charter,
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => {
                return None;
            }
        };
        let revision = row.settings.run.unwrap_or(row.settings.task);
        let policy = &self.charter_at(charter, revision)?.policy;
        let mut choice = self.tasks.default_llm_at(charter, revision, row.authority.models)?;
        let payer = self.tasks.payer(row.number)?;
        let coverage = self.account_coverage(payer, Some(row.project));
        let mut kept_provider = None;
        if let Some(kept) = row.kept_account
            && let Some(account) = self.accounts.account(kept)
        {
            for proof in &coverage {
                if proof.account == kept && proof.serves {
                    kept_provider = Some(account.provider());
                }
            }
        }
        let candidates = candidates(&choice);
        let mut kept = None;
        let mut usable = None;
        let mut serving = None;
        for candidate in &candidates {
            let name = &self.settings.models.get(usize::from(candidate.model))?.name;
            let model = charter_model(policy, name)?;
            let Some(provider) = self.provider(model) else { continue };
            if kept_provider == Some(provider) {
                kept = Some(*candidate);
                break;
            }
            if usable.is_none()
                && self.model_readiness(payer, coverage.as_slice(), model).0 == crate::documents::Readiness::Usable
            {
                usable = Some(*candidate);
            }
            if serving.is_none() {
                let accounts = self.accounts.candidates(
                    super::coverage::payer(payer),
                    provider,
                    self.settings.model_ways(model),
                    None,
                    None,
                    coverage.as_slice(),
                );
                for account in &accounts {
                    if self.account_allows(*account, model) {
                        serving = Some(*candidate);
                        break;
                    }
                }
            }
        }
        let selected = kept.or(usable).or(serving);
        if !fallback && selected.is_none() {
            return None;
        }
        choice.main = selected.unwrap_or(choice.main);
        Some(defaults(&self.settings, policy, choice, candidates.as_slice()))
    }

    /// A template's observation of the first offered charter candidate (domain/engine/views.md, 1).
    pub(crate) fn preview_llm(
        &self,
        charter: u32,
        revision: u32,
        models: tasks::Models,
        payer: tasks::Payer,
        project: u32,
    ) -> Option<tasks::LlmChoice> {
        let policy = &self.charter_at(charter, revision)?.policy;
        let mut choice = self.tasks.default_llm_at(charter, revision, models)?;
        let candidates = candidates(&choice);
        let offer = self.offer(payer, project, models);
        for candidate in &candidates {
            for offered in &offer {
                if offered.model == candidate.model {
                    choice.main = *candidate;
                    choice.account = None;
                    return Some(defaults(&self.settings, policy, choice, candidates.as_slice()));
                }
            }
        }
        None
    }

    /// The policy a run of `task` uses: its charter's, running its LLM choice's models, each the
    /// charter's model of that configured name with its chosen effort's name (domain/engine/executors.md,
    /// 1). A choice the catalogue or charter no longer holds falls back to the charter's own.
    pub(crate) fn chosen_policy(&self, task: u64) -> RunPolicy {
        let policy = &self.charter_of(task).policy;
        let chosen = match self.next_llm(self.tasks.task(task).expect("live agent task")) {
            Some(choice) => chosen(&self.settings, policy, &choice),
            None => None,
        };
        match chosen {
            Some(chosen) => chosen,
            None => policy.clone(),
        }
    }
}

pub(super) fn candidates(choice: &tasks::LlmChoice) -> List<tasks::ModelEffort> {
    let mut candidates = List::with_capacity(tasks::MODELS);
    candidates.push(choice.main).expect("charter main");
    for candidate in &choice.subagents {
        candidates.push(*candidate).expect("charter models");
    }
    candidates
}

pub(super) fn defaults(
    settings: &Settings,
    policy: &RunPolicy,
    mut choice: tasks::LlmChoice,
    candidates: &[tasks::ModelEffort],
) -> tasks::LlmChoice {
    default_effort(settings, policy, &mut choice.main);
    let mut others = List::with_capacity(tasks::MODELS);
    for candidate in candidates {
        if candidate.model != choice.main.model {
            let mut candidate = *candidate;
            default_effort(settings, policy, &mut candidate);
            others.push(candidate).expect("charter alternatives");
        }
    }
    choice.subagents = others.into_boxed();
    choice
}

/// A party's LLM choice as tasks keeps it, models and efforts by position in the catalogue
/// both: tasks checks it when it makes or amends the task (domain/tasks/README.md, section 3).
pub(crate) fn choice(value: Option<&people::LlmChoice>) -> Option<tasks::LlmChoice> {
    let value = value?;
    let later = match &value.later {
        Some(later) => {
            Some(tasks::LlmLater { main: model_effort(later.main), subagents: model_efforts(&later.subagents) })
        }
        None => None,
    };
    Some(tasks::LlmChoice {
        later,
        account: value.account,
        main: model_effort(value.main),
        subagents: model_efforts(&value.subagents),
    })
}

fn model_efforts(values: &[people::ModelEffort]) -> Box<[tasks::ModelEffort]> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("people bounds sub-agents"));
    for value in values {
        result.push(model_effort(*value)).expect("same number of sub-agents");
    }
    result.into_boxed()
}

fn model_effort(value: people::ModelEffort) -> tasks::ModelEffort {
    tasks::ModelEffort { model: value.model, effort: value.effort }
}

fn chosen(settings: &Settings, policy: &RunPolicy, choice: &tasks::LlmChoice) -> Option<RunPolicy> {
    let main = model(settings, policy, choice.main)?;
    let mut alternatives = List::with_capacity(u32::try_from(choice.subagents.len()).ok()?);
    for subagent in &choice.subagents {
        alternatives.push(model(settings, policy, *subagent)?).ok()?;
    }
    let mut chosen = policy.clone();
    chosen.model = main;
    chosen.alternatives = alternatives.into_boxed();
    Some(chosen)
}

/// The same charter model/effort rules used to construct a run's chosen policy, without
/// allocating its models. Templates use these before publishing their retained choice.
pub(crate) fn takes(models: &[Model], policy: &RunPolicy, choice: &tasks::LlmChoice) -> bool {
    if !takes_models(models, policy, choice.main, &choice.subagents) {
        return false;
    }
    match &choice.later {
        Some(later) => !policy.resume && takes_models(models, policy, later.main, &later.subagents),
        None => true,
    }
}

fn takes_models(
    models: &[Model],
    policy: &RunPolicy,
    main: tasks::ModelEffort,
    subagents: &[tasks::ModelEffort],
) -> bool {
    if subagents.len() > skein_lib::length::usize_of(tasks::MODELS) || !takes_model(models, policy, main) {
        return false;
    }
    for chosen in subagents {
        if !takes_model(models, policy, *chosen) {
            return false;
        }
    }
    true
}

fn takes_model(models: &[Model], policy: &RunPolicy, chosen: tasks::ModelEffort) -> bool {
    let Some(configured) = models.get(usize::from(chosen.model)) else { return false };
    let Some(model) = charter_model(policy, &configured.name) else { return false };
    match chosen.effort {
        Some(effort) => usize::from(effort) < model.efforts.len(),
        None => model.efforts.is_empty(),
    }
}

fn model(settings: &Settings, policy: &RunPolicy, chosen: tasks::ModelEffort) -> Option<Model> {
    if !takes_model(&settings.models, policy, chosen) {
        return None;
    }
    let name = &settings.models.get(usize::from(chosen.model))?.name;
    let mut model = charter_model(policy, name)?.clone();
    model.effort = match chosen.effort {
        Some(effort) => Some(model.efforts.get(usize::from(effort))?.clone()),
        None if model.efforts.is_empty() => None,
        None => return None,
    };
    Some(model)
}

#[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
pub(super) fn charter_model<'policy>(policy: &'policy RunPolicy, name: &[u8]) -> Option<&'policy Model> {
    if policy.model.name.as_ref() == name {
        return Some(&policy.model);
    }
    for model in &policy.alternatives {
        if model.name.as_ref() == name {
            return Some(model);
        }
    }
    None
}

/// A task given no choice takes each charter model's default effort.
fn default_effort(settings: &Settings, policy: &RunPolicy, choice: &mut tasks::ModelEffort) {
    let Some(configured) = settings.models.get(usize::from(choice.model)) else { return };
    let Some(model) = charter_model(policy, &configured.name) else { return };
    if let Some(effort) = &model.effort {
        for (at, offered) in model.efforts.iter().enumerate() {
            if offered == effort {
                choice.effort = Some(u8::try_from(at).expect("bounded efforts"));
                break;
            }
        }
    }
}
