//! Served template and creating facts (protocol/configuration-templates.md, 1; client/domain/contract/documents.md, 1).
use super::{objects, pieces, problem, task, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::acts;
use jig_domain_people as p;
use skein_lib::{List, bytes};
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn executor(value: p::TemplateExecutor, _limits: &w::Limits) -> Result<w::TemplateExecutor, w::Problem> {
    Ok(match value {
        p::TemplateExecutor::Charter(value) => w::TemplateExecutor::Charter(w::TemplateCharter { value }),
        p::TemplateExecutor::Chosen => w::TemplateExecutor::Chosen,
    })
}
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn budget(value: p::TemplateBudget, _limits: &w::Limits) -> Result<w::TemplateBudget, w::Problem> {
    Ok(match value {
        p::TemplateBudget::Amount(value) => w::TemplateBudget::Amount(w::TemplateAmount { value }),
        p::TemplateBudget::AsAllowed { cap } => w::TemplateBudget::AsAllowed(w::TemplateAsAllowed { cap }),
        p::TemplateBudget::Given => w::TemplateBudget::Given,
    })
}
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn deadline(value: p::TemplateDeadline, _limits: &w::Limits) -> Result<w::TemplateDeadline, w::Problem> {
    Ok(match value {
        p::TemplateDeadline::None => w::TemplateDeadline::None,
        p::TemplateDeadline::Span(value) => w::TemplateDeadline::Span(w::TemplateSpan { value }),
        p::TemplateDeadline::Given => w::TemplateDeadline::Given,
    })
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

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(super) fn flags(value: jig_domain_tasks::Flags, _limits: &w::Limits) -> Result<w::TaskFlags, w::Problem> {
    Ok(w::TaskFlags {
        ended_by: match value.ended_by {
            jig_domain_tasks::EndedBy::Result => w::EndedBy::Result,
            jig_domain_tasks::EndedBy::Requester => w::EndedBy::Requester,
            jig_domain_tasks::EndedBy::Standing => w::EndedBy::Standing,
            jig_domain_tasks::EndedBy::Closer(closer) => {
                w::EndedBy::Closer(w::Closer { connector: closer.connector, code: closer.code })
            }
        },
        conversation: value.conversation,
        listed: value.listed,
        retention: match value.retention {
            jig_domain_tasks::Retention::Kept => w::Retention::Kept,
            jig_domain_tasks::Retention::ForgottenOnceRead => w::Retention::ForgottenOnceRead,
        },
    })
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
        acts::Beyond::Proposed { to } => w::Beyond::Proposed(w::BeyondProposed { to: task::recipient(to, limits)? }),
        acts::Beyond::Refused(why) => {
            w::Beyond::Refused(w::BeyondRefused { value: super::refusals::refusal(why.clone(), limits)? })
        }
    };
    Ok(w::CreatingBounds { within, beyond })
}
pub fn overrides(
    values: &List<p::TemplateOverride>,
    limits: &w::Limits,
) -> Result<List<w::TemplateOverride>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(w::TemplateOverride {
            template: value.template,
            executor: match value.executor {
                Some(v) => Some(executor(v, limits)?),
                None => None,
            },
            budget: match value.budget {
                Some(v) => Some(budget(v, limits)?),
                None => None,
            },
            deadline: match value.deadline {
                Some(v) => Some(deadline(v, limits)?),
                None => None,
            },
            priority: value.priority,
        })
        .expect("same capacity");
    }
    Ok(rows)
}

pub(super) fn template(
    value: &jig_domain_core_vocabulary::Template,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::Template, w::Problem> {
    let mut resources = List::with_capacity(super::count(value.resources.len())?);
    for item in &value.resources {
        resources.push(w::Symbol { connector: item.connector, symbol: item.symbol }).expect("one row per symbol");
    }
    Ok(w::Template {
        resources,
        number: value.number,
        name: w::DocumentText { bytes: bytes::copy_of(&value.name), cut: false },
        executor: executor(value.executor, limits)?,
        contract: objects::goal::contract(&value.contract, limits)?,
        shallower: value.shallower,
        budget: budget(value.budget, limits)?,
        deadline: deadline(value.deadline, limits)?,
        tracked: value.tracked,
        flags: flags(value.flags, limits)?,
        label: value.label,
        grace: value.grace,
        llm: super::llm::choice(value.llm.as_ref(), models, limits)?,
    })
}
