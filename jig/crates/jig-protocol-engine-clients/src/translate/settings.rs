//! Settings' whole values and bounded history on the client wire (client/domain/contract/reads.md, 1).
use super::{me, objects, owner, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_authority as a;
use jig_domain_core_vocabulary::{self as core, documents as d, reads, settings as s};
use jig_domain_tasks as t;
use skein_lib::{List, Writer};

mod change;
pub(crate) use change::{
    catalogue_from_record, charter_document_from_record, charter_from_record, model_from_record, policy_from_record,
    rules_from_record, template_from_record,
};
pub(crate) use change::{change, proposed};

/// A field that could not be decoded or resolved against deployment models.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputError {
    pub field: w::Path,
}

/// Decode the length-prefixed settings items carried by `ChangeSettings` and
/// `ProposeSettings`, sharing the client entrance's value and model translation.
pub fn items(items: Box<[u8]>, models: &[crate::Model], limits: &w::Limits) -> Result<Box<[s::Item]>, InputError> {
    let value = w::ChangeSettings { against: 0, items, running: w::Running::Stays };
    if let Err(problem) = value.measure(limits) {
        return Err(InputError { field: problem.path });
    }
    match change(value, models, limits) {
        Ok(value) => Ok(value.items),
        Err(problem) => Err(InputError { field: problem.path }),
    }
}

#[must_use]
pub fn setting(value: &w::Setting) -> s::Setting {
    mapped::setting_to_domain(value.clone())
}
#[must_use]
pub fn name(value: s::Setting) -> w::Setting {
    mapped::setting_to_record(&value)
}
#[must_use]
pub fn head(value: &s::Head) -> w::RevisionHead {
    w::RevisionHead {
        number: value.number,
        against: value.against,
        at: value.at.as_nanos(),
        by: mapped::revision_decider_to_record(&value.by),
        proposal: match value.proposal {
            Some((proposer, number)) => {
                Some(super::task::proposal(&d::ProposalRef { proposer, number: u64::from(number) }))
            }
            None => None,
        },
        moved: value.moved,
    }
}
pub fn document(value: &d::Settings) -> Result<w::SettingsDocument, w::Problem> {
    match mapped::settings_document_to_record(value) {
        Ok(record) => Ok(record),
        Err(problem) => Err(settings_count_problem(problem)),
    }
}
pub(crate) fn model(value: &core::Model) -> Result<w::SettingModel, w::Problem> {
    let mut efforts = List::with_capacity(super::count(value.efforts.len())?);
    for effort in &value.efforts {
        efforts.push(effort.clone()).expect("effort count");
    }
    let mut speeds = List::with_capacity(super::count(value.speeds.len())?);
    for speed in &value.speeds {
        speeds.push(mapped::setting_speed_to_record(speed)?).expect("speed count");
    }
    Ok(w::SettingModel {
        provider: value.provider,
        dialect: value.dialect,
        endpoint: value.endpoint,
        program: value.program.clone(),
        name: value.name.clone(),
        window: value.window,
        output: value.output,
        input_price: value.input_price,
        cached_price: value.cached_price,
        cache_write_price: value.cache_write_price,
        output_price: value.output_price,
        price_unit: value.price_unit,
        efforts,
        effort: value.effort.clone(),
        speeds,
        speed: value.speed.clone(),
    })
}
pub(crate) fn charter(value: &core::Charter) -> Result<w::Charter, w::Problem> {
    let policy = &value.policy;
    let mut alternatives = List::with_capacity(super::count(policy.alternatives.len())?);
    for row in &policy.alternatives {
        alternatives.push(model(row)?).expect("model count");
    }
    let mut documents = List::with_capacity(super::count(value.documents.len())?);
    for item in &value.documents {
        documents.push(charter_document(item)?).expect("document count");
    }
    let mut parts = List::with_capacity(super::count(value.parts.len())?);
    for item in &value.parts {
        parts.push(mapped::charter_part_to_record(item)).expect("part count");
    }
    Ok(w::Charter {
        name: value.name.clone(),
        parts,
        documents,
        rest: value.rest,
        resume: policy.resume,
        inspect: policy.inspect,
        modify: policy.modify,
        shell: policy.shell,
        agents: policy.agents,
        number: value.number,
        instructions: policy.instructions.clone(),
        waiting: policy.waiting,
        turns: policy.turns,
        time: policy.time,
        reserve_turns: policy.reserve_turns,
        reserve_time: policy.reserve_time,
        model: model(&policy.model)?,
        alternatives,
        call_timeout: policy.call_timeout,
        tools: super::convert::tools_to_record(&policy.tools),
        hosts: mapped::setting_hosts_to_record(&value.hosts),
        spend: value.spend,
        wake: owner::wake(value.wake),
    })
}
pub(crate) fn charter_document(value: &core::Document) -> Result<w::CharterDocument, w::Problem> {
    let mut name = List::with_capacity(super::count(value.name.segments.len())?);
    for segment in &value.name.segments {
        name.push(segment.clone()).expect("segment count");
    }
    Ok(w::CharterDocument {
        connector: value.connector,
        name,
        path: value.path.clone(),
        heading: value.heading.clone(),
    })
}
fn authority(value: &a::Authority, models: &[crate::Model], limits: &w::Limits) -> Result<w::Authority, w::Problem> {
    objects::goal::authority(models, &core::task_authority(value), limits)
}
fn pattern(value: &a::Pattern) -> Result<w::Pattern, w::Problem> {
    objects::goal::pattern(&t::Pattern {
        segments: value.segments.clone(),
        last: match &value.last {
            a::Last::Exact(v) => t::Last::Exact(v.clone()),
            a::Last::Open(v) => t::Last::Open(v.clone()),
        },
    })
}
fn grant(value: &a::Grant) -> Result<w::Grant, w::Problem> {
    Ok(w::Grant { connector: value.connector, kind: value.kind, pattern: pattern(&value.pattern)? })
}
fn requirements(values: &[a::Requirement]) -> Result<List<w::PolicyRequirement>, w::Problem> {
    let mut rows = List::with_capacity(super::count(values.len())?);
    for v in values {
        rows.push(w::PolicyRequirement {
            connector: v.connector,
            kind: v.kind,
            pattern: pattern(&v.pattern)?,
            judge: mapped::policy_judge_to_record(&v.judge),
            guard: mapped::policy_guard_to_record(&v.guard),
            must_be_guarded: v.must_be_guarded,
        })
        .expect("requirement count");
    }
    Ok(rows)
}
pub(super) fn policy(
    value: &core::ConfiguredPolicy,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::SettingPolicy, w::Problem> {
    let p = &value.policy;
    let mut projections = List::with_capacity(super::count(p.projections.len())?);
    for v in &p.projections {
        projections.push(grant(v)?).expect("projection count");
    }
    let mut roles = List::with_capacity(super::count(p.roles.len())?);
    for v in &p.roles {
        roles
            .push(w::PolicyEditRole {
                number: v.number,
                authority: authority(&v.authority, models, limits)?,
                period_spend: v.period_spend,
                requests: crate::translate::convert::requests_to_record(&v.requests),
                decides: crate::translate::convert::proposals_to_record(&v.decides),
            })
            .expect("role count");
    }
    let mut permissions = List::with_capacity(super::count(value.permissions.len())?);
    for v in &value.permissions {
        permissions.push(mapped::policy_permission_to_record(v)).expect("permission count");
    }
    let mut overrides = List::with_capacity(super::count(value.templates.len())?);
    for v in &value.templates {
        overrides.push(v.clone()).expect("template override count");
    }
    Ok(w::SettingPolicy {
        name: value.name.clone(),
        escalation_role: p.escalation_role,
        ceiling: authority(&p.ceiling, models, limits)?,
        projections,
        period_spend: p.period_spend,
        roles,
        requirements: requirements(&p.requirements)?,
        permissions,
        templates: me::overrides(&overrides),
    })
}
pub(super) fn rules(value: &a::Rules, models: &[crate::Model], limits: &w::Limits) -> Result<w::Rules, w::Problem> {
    let mut implies = List::with_capacity(super::count(value.implies.entries().len())?);
    for v in value.implies.entries() {
        implies.push(mapped::kind_implication_to_record(v)).expect("implication count");
    }
    Ok(w::Rules {
        ceiling: authority(&value.ceiling, models, limits)?,
        period_spend: value.period_spend,
        minimum_run_spend: value.minimum_run_spend,
        maximum_run_spend: value.maximum_run_spend,
        implies,
        requirements: requirements(&value.requirements)?,
    })
}
pub fn value(value: &s::Value, models: &[crate::Model], limits: &w::Limits) -> Result<w::SettingValue, w::Problem> {
    check_provider(value, limits)?;
    match mapped::setting_value_to_record(value, &(models, limits)) {
        Ok(record) => Ok(record),
        Err(problem) => Err(settings_count_problem(problem)),
    }
}
pub fn setting_document(
    v: &reads::SettingDocument,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::SettingDocument, w::Problem> {
    if let Some(value) = &v.value {
        check_provider(value, limits)?;
    }
    match mapped::setting_document_to_record(v, &(models, limits)) {
        Ok(record) => Ok(record),
        Err(problem) => Err(settings_count_problem(problem)),
    }
}
pub fn revision(
    v: &reads::RevisionAnswer,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::RevisionAnswer, w::Problem> {
    Ok(match v {
        reads::RevisionAnswer::Unknown => w::RevisionAnswer::Unknown,
        reads::RevisionAnswer::Forgotten { oldest } => {
            w::RevisionAnswer::Forgotten(w::RevisionForgotten { oldest: *oldest })
        }
        reads::RevisionAnswer::Revision { revision, value_bytes } => {
            let mut pieces = List::with_capacity(super::count(
                revision.items.len().checked_add(revision.applications.len()).expect("bounded revision items"),
            )?);
            for item in &revision.items {
                let edit = match &item.edit {
                    s::Edit::Set(v) => {
                        let value = value(v, models, limits)?;
                        let bytes = u64::from(value.measure(limits)?);
                        if bytes > (*value_bytes).min(u64::from(limits.owned_record)) {
                            w::RevisionEdit::TooLarge(w::Size { bytes })
                        } else {
                            w::RevisionEdit::Set(w::SettingSet { value })
                        }
                    }
                    s::Edit::Removed => w::RevisionEdit::Removed,
                    s::Edit::Reset => w::RevisionEdit::Reset,
                };
                let item = w::RevisionItem {
                    setting: name(item.setting),
                    edit,
                    replaced: replaced(&item.replaced, *value_bytes, models, limits)?,
                };
                let mut out = Writer::new(skein_lib::length::usize_of(item.measure(limits)?));
                item.write(limits, &mut out).expect("measured item");
                pieces.push(out.finish()).expect("item count");
            }
            for item in &revision.applications {
                let setting = s::Setting::Application { kind: item.kind, scope: item.scope, number: item.number };
                let edit =
                    w::RevisionEdit::Application(w::ApplicationEdit { edit: mapped::sized_edit_to_record(&item.edit) });
                let row =
                    w::RevisionItem { setting: name(setting), edit, replaced: application_replaced(&item.replaced) };
                let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
                row.write(limits, &mut out).expect("measured application item");
                pieces.push(out.finish()).expect("application item count");
            }
            w::RevisionAnswer::Revision(w::RevisionDocument {
                head: head(&revision.head),
                items: super::pieces(&pieces, limits.revision_document_items, w::paths::REVISION_DOCUMENT_ITEMS)?,
            })
        }
    })
}
#[must_use]
pub fn revisions(v: &reads::RevisionsPage) -> w::RevisionsPage {
    mapped::revisions_page_to_record(v)
}

pub(crate) fn item_heads(items: &List<reads::RevisionItemHead>) -> List<w::RevisionItemHead> {
    let mut result = List::with_capacity(items.len());
    for item in items {
        result.push(mapped::revision_item_head_to_record(item)).expect("bounded heads");
    }
    result
}
fn review_value(
    v: Option<&s::Value>,
    bound: u64,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::ReviewValue, w::Problem> {
    Ok(match v {
        None => w::ReviewValue::None,
        Some(v) => {
            let value = value(v, models, limits)?;
            let bytes = u64::from(value.measure(limits)?);
            if bytes > bound.min(u64::from(limits.owned_record)) {
                w::ReviewValue::TooLarge(w::Size { bytes })
            } else {
                w::ReviewValue::Value(w::SettingSet { value })
            }
        }
    })
}
pub(crate) fn review(
    v: &d::SettingsReview,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::SettingsReview, w::Problem> {
    let mut pieces = List::with_capacity(v.items.len());
    let mut used = 0_u64;
    let mut cut = false;
    for item in &v.items {
        let row = w::ReviewItem {
            setting: name(item.setting),
            edit: mapped::revision_item_edit_to_record(&item.edit),
            now: review_value(item.now.as_ref(), v.value_bytes, models, limits)?,
            origin: match item.origin {
                Some((changed, revision)) => {
                    let revision = w::OriginRevision { revision };
                    if changed { w::SettingOrigin::Change(revision) } else { w::SettingOrigin::Default(revision) }
                }
                None => w::SettingOrigin::NotOffered,
            },
            would_be: review_value(item.would_be.as_ref(), v.value_bytes, models, limits)?,
            application: match &item.application {
                Some(v) => match mapped::application_review_to_record(v) {
                    Ok(review) => Some(review),
                    Err(problem) => return Err(settings_count_problem(problem)),
                },
                None => None,
            },
        };
        used = used
            .checked_add(u64::from(row.measure(limits)?))
            .expect("bounded review bytes")
            .checked_add(4)
            .expect("frame bytes");
        if used > u64::from(limits.settings_review_items) {
            cut = true;
            break;
        }
        let mut bytes = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut bytes).expect("measured review item");
        pieces.push(bytes.finish()).expect("review count");
    }
    Ok(w::SettingsReview {
        key: v.key,
        newest: v.newest,
        items: super::pieces(&pieces, limits.settings_review_items, w::paths::SETTINGS_REVIEW_ITEMS)?,
        cut,
    })
}

fn replaced(
    v: &s::Replaced,
    value_bytes: u64,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::RevisionReplaced, w::Problem> {
    Ok(match v {
        s::Replaced::Value(v) => {
            let value = value(v, models, limits)?;
            let bytes = u64::from(value.measure(limits)?);
            if bytes > value_bytes.min(u64::from(limits.owned_record)) {
                w::RevisionReplaced::TooLarge(w::Size { bytes })
            } else {
                w::RevisionReplaced::Value(w::SettingSet { value })
            }
        }
        s::Replaced::Default => w::RevisionReplaced::Default,
        s::Replaced::Removed => w::RevisionReplaced::Removed,
        s::Replaced::NotOffered => w::RevisionReplaced::NotOffered,
    })
}
fn application_replaced(v: &s::application::Replaced) -> w::RevisionReplaced {
    match v {
        s::application::Replaced::Changed { size } => {
            w::RevisionReplaced::Application(w::ApplicationSize { size: *size })
        }
        s::application::Replaced::Default => w::RevisionReplaced::Default,
        s::application::Replaced::Removed => w::RevisionReplaced::Removed,
        s::application::Replaced::NotOffered => w::RevisionReplaced::NotOffered,
    }
}

pub(crate) fn catalogue(value: &s::CatalogueModel) -> Result<w::CatalogueEntry, w::Problem> {
    Ok(w::CatalogueEntry {
        number: value.number,
        model: model(&value.model)?,
        shown: value.shown.clone(),
        description: value.description.clone(),
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        source: mapped::model_source_to_record(&value.source),
    })
}

#[must_use]
pub fn model_act(value: w::ChangeModel) -> s::catalogue::ModelAct {
    mapped::model_act_to_domain(value.act)
}

// A settings slice count that cannot fit in a List is an owned-record
// refusal. The encoder reports its own field bounds.
fn settings_count_problem(problem: w::Problem) -> w::Problem {
    if problem.path == w::paths::APPLICATION_STATE_CHARTERS
        || problem.path == w::paths::OVERLAY_ALTERNATIVES
        || problem.path == w::paths::OVERLAY_DOCUMENTS
        || problem.path == w::paths::OVERLAY_PARTS
    {
        return super::problem(problem.reason);
    }
    problem
}

fn check_provider(value: &s::Value, limits: &w::Limits) -> Result<(), w::Problem> {
    if let s::Value::Provider(value) = value {
        providers::check(value, limits)?;
    }
    Ok(())
}
mod providers;
pub(crate) use providers::{provider_address_to_domain, provider_address_to_record};
