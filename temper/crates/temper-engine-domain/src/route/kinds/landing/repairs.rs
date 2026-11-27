//! Resolve named repair choices when proposed and at each start (domain/red-repairs.md, 7).
use crate::Domain;
use alloc::boxed::Box;
use jig_domain_core::{
    Model,
    settings::{CatalogueModel, catalogue, charters, check},
};
use jig_domain_tasks as tasks;
use skein_lib::List;
use temper_engine_domain_forge::settings::{NamedLlm, NamedModel, RepairGoal, RepairSettings, Repairs};

pub(crate) fn restored(domain: &mut Domain, spec_bytes: u32) {
    let projects = domain.engine.core().projects.clone();
    for &project in &projects {
        let Some(landing) = domain.forge.landing_settings(project) else { continue };
        let Some(settings) = &landing.repairs else { continue };
        let resolved = match resolve(domain, settings) {
            Ok(_) if spec_bytes < 8_192 => Err(b"tasks.spec_bytes".as_slice()),
            Ok(repairs) => Ok(repairs),
            Err(field) => Err(field),
        };
        domain.forge.take_project_repairs(project, resolved);
    }
}
pub(super) fn resolve(domain: &Domain, v: &RepairSettings) -> Result<Repairs, &'static [u8]> {
    let charter = domain.engine.core().current.charter(v.goal.charter).ok_or(b"goal.charter".as_slice())?;
    let entries = catalogue::entries(&domain.engine.core().current);
    let models = catalogue::rows(&entries);
    let charter = charters::resolved(charter, &entries, &crate::limits::core_limits(&domain.limits))
        .ok_or(b"goal.llm".as_slice())?;
    let llm = match &v.goal.llm {
        Some(named) => {
            let choice = choice(named, &entries).ok_or(b"goal.llm".as_slice())?;
            if !check::choice_valid(&models, &choice)
                || !jig_domain_core_vocabulary::runs::llm::takes(&models, &charter.policy, &choice)
            {
                return Err(b"goal.llm");
            }
            Some(choice)
        }
        None => None,
    };
    Ok(Repairs {
        goals: v.goals,
        attempts: v.attempts,
        deadline_ms: v.deadline_ms,
        goal: RepairGoal { charter: v.goal.charter, llm, priority: v.goal.priority, budget: v.goal.budget },
    })
}
fn choice(v: &NamedLlm, models: &[CatalogueModel]) -> Option<tasks::LlmChoice> {
    Some(tasks::LlmChoice {
        account: v.account,
        main: model(&v.main, models)?,
        subagents: subagents(&v.subagents, models)?,
        later: match &v.later {
            Some(v) => {
                Some(tasks::LlmLater { main: model(&v.main, models)?, subagents: subagents(&v.subagents, models)? })
            }
            None => None,
        },
    })
}
fn subagents(v: &[NamedModel], models: &[CatalogueModel]) -> Option<Box<[tasks::ModelEffort]>> {
    let mut out = List::with_capacity(u32::try_from(v.len()).ok()?);
    for v in v {
        out.push(model(v, models)?).ok()?;
    }
    Some(out.into_boxed())
}
fn model(v: &NamedModel, models: &[CatalogueModel]) -> Option<tasks::ModelEffort> {
    for configured in models {
        if !configured.withdrawn && !configured.model.is_hole() && configured.model.name == v.model {
            return Some(tasks::ModelEffort {
                model: u8::try_from(configured.number.checked_sub(1)?).ok()?,
                effort: effort(v.effort.as_deref(), &configured.model).ok()?,
                speed: match &v.speed {
                    Some(name) => Some(skein_lib::Name::from_checked(name)?),
                    None => None,
                },
            });
        }
    }
    None
}
fn effort(v: Option<&[u8]>, model: &Model) -> Result<Option<u8>, ()> {
    match v {
        None if model.efforts.is_empty() => Ok(None),
        None => Err(()),
        Some(v) => {
            for (at, named) in model.efforts.iter().enumerate() {
                if named.as_ref() == v {
                    return Ok(Some(u8::try_from(at).or(Err(()))?));
                }
            }
            Err(())
        }
    }
}
