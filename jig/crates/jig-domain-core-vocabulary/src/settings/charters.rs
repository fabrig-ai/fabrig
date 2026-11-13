//! Configured charter values and resolution (domain/engine/executors.md, 1).
use super::CatalogueModel;
use crate::{Limits, RunPolicy};
use alloc::boxed::Box;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::List;

/// A charter's document (domain/engine/settings.md, 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Document {
    pub connector: u16,
    /// The resource name, within the authority's pattern bounds.
    pub name: jig_domain_authority::Name,
    /// 1 to `crate::DOCUMENT_PATH_BYTES` bytes.
    pub path: Box<[u8]>,
    /// 1 to `crate::DOCUMENT_HEADING_BYTES` bytes when present.
    pub heading: Option<Box<[u8]>>,
}
/// A charter's part: instructions for runs whose reasons meet `on`
/// (jig/docs/design/domain/tasks/messages.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CharterPart {
    /// Nonempty, known bits (`tasks::Reasons::is_known`).
    pub on: tasks::Reasons,
    pub instructions: Box<[u8]>,
}
/// One numbered charter offered by the deployment, with its documents
/// (protocol/configuration.md, 3; domain/engine/settings.md, 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Charter {
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
/// Resolve names against the current catalogue (domain/engine/settings.md, 11).
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
    let mut value = charter.clone();
    let mut chosen = List::with_capacity(tasks::MODELS);
    let mut main = false;
    let mut omitted = 0_u32;
    for (at, number) in candidates.as_slice().iter().enumerate() {
        let rest = at >= skein_lib::length::usize_of(named);
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
                    break;
                }
            }
        }
        if rest && main && chosen.len() >= limits.settings.alternatives {
            omitted = omitted.checked_add(1).expect("bounded candidates");
            continue;
        }
        let mut trial = value.policy.clone();
        if main {
            let mut alternatives = chosen.clone();
            alternatives.push(model.clone()).expect("alternative room checked");
            trial.alternatives = alternatives.into_boxed();
        } else {
            trial.model = model.clone();
            trial.alternatives = Box::new([]);
        }
        let fits = match crate::limits::policy_owned_bytes(&trial) {
            Some(bytes) => bytes <= u64::from(limits.run_bytes),
            None => false,
        };
        if rest && !fits {
            omitted = omitted.checked_add(1).expect("bounded candidates");
            continue;
        }
        if main {
            chosen.push(model.clone()).expect("alternative room checked");
        }
        value.policy = trial;
        main = true;
    }
    (if main { Some(value) } else { None }, omitted)
}
