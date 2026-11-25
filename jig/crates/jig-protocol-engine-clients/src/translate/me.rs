//! Served template and creating facts (protocol/configuration-templates.md, 1; client/domain/contract/documents.md, 1).
use super::{objects, pieces, problem, task, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::acts;
use jig_domain_people as p;
use skein_lib::{List, bytes};
fn executor(value: p::TemplateExecutor) -> w::TemplateExecutor {
    mapped::template_executor_to_record(&value)
}
fn budget(value: p::TemplateBudget) -> w::TemplateBudget {
    mapped::template_budget_to_record(&value)
}
fn deadline(value: p::TemplateDeadline) -> w::TemplateDeadline {
    mapped::template_deadline_to_record(&value)
}

pub fn templates(
    values: &List<jig_domain_core_vocabulary::Template>,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(template(value, models, limits)?).expect("one row per template");
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in &rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        if row.write(limits, &mut out).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
        encoded.push(out.finish()).expect("same row count");
    }
    pieces(&encoded, limits.me_project_templates, w::paths::ME_PROJECT_TEMPLATES)
}

pub(super) fn flags(value: jig_domain_tasks::Flags) -> w::TaskFlags {
    mapped::task_flags_to_record(&value)
}
pub fn creating(
    models: &[crate::Model],
    value: &acts::Creating,
    limits: &w::Limits,
) -> Result<w::CreatingBounds, w::Problem> {
    let within = match task::bounds(models, &value.within, limits)? {
        w::Bounds::Create(value) => value,
        w::Bounds::Amend(_) => unreachable!("creating has creating bounds"),
    };
    let beyond = match &value.beyond {
        acts::Beyond::Proposed { to } => w::Beyond::Proposed(w::BeyondProposed { to: task::recipient(to) }),
        acts::Beyond::Refused(why) => {
            w::Beyond::Refused(w::BeyondRefused { value: super::refusals::refusal(why.clone(), limits)? })
        }
    };
    Ok(w::CreatingBounds { within, beyond })
}
pub fn overrides(values: &List<p::TemplateOverride>) -> List<w::TemplateOverride> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(mapped::template_override_to_record(value)).expect("same count");
    }
    rows
}

pub(super) fn template(
    value: &jig_domain_core_vocabulary::Template,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::Template, w::Problem> {
    let mut resources = List::with_capacity(super::count(value.resources.len())?);
    for item in &value.resources {
        resources.push(mapped::symbol_to_record(item)).expect("one row per symbol");
    }
    Ok(w::Template {
        resources,
        number: value.number,
        name: w::DocumentText { bytes: bytes::copy_of(&value.name), cut: false },
        executor: executor(value.executor),
        contract: objects::goal::contract(&value.contract, limits)?,
        shallower: value.shallower,
        budget: budget(value.budget),
        deadline: deadline(value.deadline),
        tracked: value.tracked,
        flags: flags(value.flags),
        label: value.label,
        grace: value.grace,
        llm: super::llm::choice(value.llm.as_ref(), models)?,
    })
}
