//! Charter model and effort rules (domain/tasks/README.md, 3;
//! domain/engine/executors.md, 1).

use crate::{Model, RunPolicy};
use jig_domain_tasks as tasks;

/// The same charter model/effort rules used to construct a run's chosen policy, without
/// allocating its models. Templates use these before publishing their retained choice.
pub fn takes(models: &[Model], policy: &RunPolicy, choice: &tasks::LlmChoice) -> bool {
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

pub fn takes_model(models: &[Model], policy: &RunPolicy, chosen: tasks::ModelEffort) -> bool {
    let Some(configured) = models.get(usize::from(chosen.model)) else { return false };
    let Some(model) = charter_model(policy, &configured.name) else { return false };
    match chosen.effort {
        Some(effort) => usize::from(effort) < model.efforts.len(),
        None => model.efforts.is_empty(),
    }
}

#[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
pub fn charter_model<'policy>(policy: &'policy RunPolicy, name: &[u8]) -> Option<&'policy Model> {
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
