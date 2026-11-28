//! Configured charter values and resolution (domain/engine/executors.md, 1).
use super::CatalogueModel;
use crate::{Limits, RunPolicy};
use alloc::boxed::Box;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::List;

pub use jig_domain_contract::parts::Document;

/// A charter's part: instructions for runs whose reasons meet `on`
/// (jig/docs/design/domain/tasks/messages.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CharterPart {
    /// Nonempty, known bits (`tasks::Reasons::is_known`).
    pub on: tasks::Reasons,
    pub instructions: Box<[u8]>,
}
/// One numbered charter offered by the deployment, with its documents
/// (protocol/configuration-engine.md, 1; domain/engine/settings.md, 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Charter {
    pub name: Option<Box<[u8]>>,
    pub rest: bool,
    pub number: u32,
    pub policy: RunPolicy,
    pub hosts: fleet::Kinds,
    pub spend: u64,
    /// Its default wake policy, which a task made under it takes when its batch entry names none (domain/tasks/messages.md, 3).
    pub wake: tasks::WakePolicy,
    pub documents: Box<[Document]>,
    pub parts: Box<[CharterPart]>,
}
/// Resolve names against the current catalogue (domain/engine/settings-catalogue.md, 1).
#[must_use]
pub fn resolved(charter: &Charter, models: &[CatalogueModel], limits: &Limits) -> Option<Charter> {
    resolution(charter, models, limits).0
}
#[must_use]
pub fn resolution(charter: &Charter, models: &[CatalogueModel], limits: &Limits) -> (Option<Charter>, u32) {
    let mut candidates = List::with_capacity(tasks::MODELS);
    for named in core::iter::once(&charter.policy.model).chain(charter.policy.alternatives.iter()) {
        for model in models {
            if model.model.name == named.name
                && !model.withdrawn
                && !model.model.is_hole()
                && !candidates.as_slice().contains(&model.number)
            {
                candidates.push(model.number).expect("unique catalogue candidates");
            }
        }
    }
    let named = candidates.len();
    if charter.rest {
        for number in 1..=tasks::MODELS {
            for model in models {
                if model.number == number
                    && !model.withdrawn
                    && !model.model.is_hole()
                    && !candidates.as_slice().contains(&number)
                {
                    candidates.push(number).expect("unique catalogue candidates");
                }
            }
        }
    }
    let mut chosen = List::with_capacity(tasks::MODELS);
    let mut main = None;
    let instructions = &charter.policy.instructions;
    let mut omitted = 0_u32;
    for (at, number) in candidates.as_slice().iter().enumerate() {
        let rest = at >= skein_lib::length::usize_of(named);
        if rest && main.is_some() && chosen.len() >= limits.settings.alternatives {
            omitted = omitted.checked_add(1).expect("bounded candidates");
            continue;
        }
        let mut record = None;
        for model in models {
            if model.number == *number {
                record = Some(&model.model);
                break;
            }
        }
        let mut model = record.expect("candidate catalogue row").clone();
        if !rest {
            for named in core::iter::once(&charter.policy.model).chain(charter.policy.alternatives.iter()) {
                if named.name == model.name {
                    if let Some(effort) = &named.effort
                        && model.efforts.contains(effort)
                    {
                        model.effort = Some(effort.clone());
                    }
                    if let Some(speed) = &named.speed {
                        if speed.as_ref() == tasks::STANDARD_SPEED {
                            model.speed = None;
                        } else if model.speed_named(speed).is_some() {
                            model.speed = Some(speed.clone());
                        }
                        // Before admission, an invalid override leaves the catalogue default in place.
                    }
                    break;
                }
            }
        }
        // Price the retained models by reference instead of cloning a trial
        // policy and all its alternatives for each candidate.
        if let Some(first) = &main {
            let before = chosen.len();
            chosen.push(model).expect("alternative room checked");
            if rest && !fits(instructions, first, chosen.as_slice(), limits.run_bytes) {
                chosen.truncate(before);
                omitted = omitted.checked_add(1).expect("bounded candidates");
            }
        } else if rest && !fits(instructions, &model, &[], limits.run_bytes) {
            omitted = omitted.checked_add(1).expect("bounded candidates");
        } else {
            main = Some(model);
        }
    }
    let Some(main) = main else { return (None, omitted) };
    let mut value = charter.clone();
    value.policy.model = main;
    value.policy.alternatives = chosen.into_boxed();
    (Some(value), omitted)
}

fn fits(instructions: &[u8], main: &crate::Model, alternatives: &[crate::Model], run_bytes: u32) -> bool {
    match crate::limits::policy_bytes(instructions, main, alternatives) {
        Some(bytes) => bytes <= u64::from(run_bytes),
        None => false,
    }
}
