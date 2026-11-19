//! Settings' whole values and bounded history on the client wire (client/domain/contract/README.md, 4).
use super::{me, objects, owner, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_authority as a;
use jig_domain_core_vocabulary::{self as core, documents as d, reads, settings as s};
use jig_domain_tasks as t;
use skein_lib::{List, Writer};

mod change;
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
    match value {
        w::Setting::Charter(v) => s::Setting::Charter(v.number),
        w::Setting::Template(v) => s::Setting::Template(v.number),
        w::Setting::Policy(v) => s::Setting::Policy(v.number),
        w::Setting::Model(v) => s::Setting::Model(v.number),
        w::Setting::Worker(v) => s::Setting::Worker(v.number),
        w::Setting::Provider(v) => s::Setting::Provider(v.number),
        w::Setting::Rules => s::Setting::Rules,
        w::Setting::Limits => s::Setting::Limits,
        w::Setting::ProjectPolicy(v) => s::Setting::ProjectPolicy { project: v.project },
        w::Setting::Application(v) => s::Setting::Application { kind: v.kind, scope: v.scope, number: v.number },
    }
}
pub fn name(value: s::Setting, _limits: &w::Limits) -> Result<w::Setting, w::Problem> {
    Ok(match value {
        jig_domain_people::Setting::ProjectPolicy { project } => {
            w::Setting::ProjectPolicy(w::ProjectPolicySetting { project })
        }
        s::Setting::Charter(number) => w::Setting::Charter(w::SettingNumber { number }),
        s::Setting::Template(number) => w::Setting::Template(w::SettingNumber { number }),
        s::Setting::Policy(number) => w::Setting::Policy(w::SettingNumber { number }),
        s::Setting::Model(n) => w::Setting::Model(w::SettingNumber { number: n }),
        s::Setting::Worker(n) => w::Setting::Worker(w::SettingNumber { number: n }),
        s::Setting::Provider(n) => w::Setting::Provider(w::SettingNumber { number: n }),
        s::Setting::Rules => w::Setting::Rules,
        s::Setting::Limits => w::Setting::Limits,
        s::Setting::Application { kind, scope, number } => {
            w::Setting::Application(w::ApplicationSetting { kind, scope, number })
        }
    })
}
pub fn head(value: &s::Head, limits: &w::Limits) -> Result<w::RevisionHead, w::Problem> {
    Ok(w::RevisionHead {
        number: value.number,
        against: value.against,
        at: value.at.as_nanos(),
        by: match value.by {
            s::Decider::FirstStart => w::RevisionDecider::FirstStart,
            s::Decider::Release => w::RevisionDecider::Release,
            s::Decider::List { provider } => w::RevisionDecider::List(w::RevisionList { provider }),
            s::Decider::Party(number) => w::RevisionDecider::Party(w::RevisionParty { number }),
        },
        proposal: match value.proposal {
            Some((proposer, number)) => {
                Some(super::task::proposal(&d::ProposalRef { proposer, number: u64::from(number) }, limits)?)
            }
            None => None,
        },
        moved: value.moved,
    })
}
pub fn document(value: &d::Settings, limits: &w::Limits) -> Result<w::SettingsDocument, w::Problem> {
    let mut settings = List::with_capacity(value.settings.len());
    for line in &value.settings {
        settings
            .push(w::SettingLine {
                setting: name(line.setting, limits)?,
                name: match &line.name {
                    Some(value) => Some(owner::text(value, limits)?),
                    None => None,
                },
                follows: match line.follows {
                    d::Follows::Overlay => w::SettingFollows::Overlay,
                    d::Follows::Default => w::SettingFollows::Default,
                    d::Follows::Change => w::SettingFollows::Change,
                },
                last: line.last,
                application: match &line.application {
                    Some(v) => Some(application_state(v, limits)?),
                    None => None,
                },
            })
            .expect("settings line count");
    }
    let mut held = List::with_capacity(value.held.len());
    for row in &value.held {
        held.push(w::HeldRevision { head: head(&row.head, limits)?, holders: row.holders }).expect("held count");
    }
    let mut pending = List::with_capacity(value.pending.len());
    for row in &value.pending {
        let proposer =
            super::task::proposal(&d::ProposalRef { proposer: row.proposer, number: u64::from(row.number) }, limits)?
                .proposer;
        pending.push(w::SettingsProposal { proposer, number: row.number }).expect("proposal count");
    }
    Ok(w::SettingsDocument { newest: value.newest, settings, held, pending })
}
fn model(value: &core::Model, _limits: &w::Limits) -> Result<w::SettingModel, w::Problem> {
    let mut efforts = List::with_capacity(super::count(value.efforts.len())?);
    for effort in &value.efforts {
        efforts.push(effort.clone()).expect("effort count");
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
    })
}
fn charter(value: &core::Charter, limits: &w::Limits) -> Result<w::Charter, w::Problem> {
    let policy = &value.policy;
    let mut alternatives = List::with_capacity(super::count(policy.alternatives.len())?);
    for row in &policy.alternatives {
        alternatives.push(model(row, limits)?).expect("model count");
    }
    let mut documents = List::with_capacity(super::count(value.documents.len())?);
    for item in &value.documents {
        let mut name = List::with_capacity(super::count(item.name.segments.len())?);
        for segment in &item.name.segments {
            name.push(segment.clone()).expect("segment count");
        }
        documents
            .push(w::CharterDocument {
                connector: item.connector,
                name,
                path: item.path.clone(),
                heading: item.heading.clone(),
            })
            .expect("document count");
    }
    let mut parts = List::with_capacity(super::count(value.parts.len())?);
    for item in &value.parts {
        parts.push(w::CharterPart { on: item.on.0, instructions: item.instructions.clone() }).expect("part count");
    }
    Ok(w::Charter {
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
        model: model(&policy.model, limits)?,
        alternatives,
        call_timeout: policy.call_timeout,
        tools: policy.tools.0,
        hosts: match value.hosts {
            jig_domain_fleet::Kinds::Workers => w::SettingHosts::Workers,
            jig_domain_fleet::Kinds::Engine => w::SettingHosts::Engine,
            jig_domain_fleet::Kinds::Both => w::SettingHosts::Both,
        },
        spend: value.spend,
        wake: owner::wake(value.wake, limits)?,
    })
}
fn authority(value: &a::Authority, models: &[crate::Model], limits: &w::Limits) -> Result<w::Authority, w::Problem> {
    objects::goal::authority(models, &core::task_authority(value), limits)
}
fn pattern(value: &a::Pattern, limits: &w::Limits) -> Result<w::Pattern, w::Problem> {
    objects::goal::pattern(
        &t::Pattern {
            segments: value.segments.clone(),
            last: match &value.last {
                a::Last::Exact(v) => t::Last::Exact(v.clone()),
                a::Last::Open(v) => t::Last::Open(v.clone()),
            },
        },
        limits,
    )
}
fn grant(value: &a::Grant, limits: &w::Limits) -> Result<w::Grant, w::Problem> {
    Ok(w::Grant { connector: value.connector, kind: value.kind, pattern: pattern(&value.pattern, limits)? })
}
fn requirements(values: &[a::Requirement], limits: &w::Limits) -> Result<List<w::PolicyRequirement>, w::Problem> {
    let mut rows = List::with_capacity(super::count(values.len())?);
    for v in values {
        rows.push(w::PolicyRequirement {
            connector: v.connector,
            kind: v.kind,
            pattern: pattern(&v.pattern, limits)?,
            judge: w::PolicyJudge {
                connector: v.judge.connector,
                requirement: v.judge.requirement,
                parameters: v.judge.parameters,
            },
            guard: match v.guard {
                a::Guard::Guarded => w::PolicyGuard::Guarded,
                a::Guard::Observed { freshness } => w::PolicyGuard::Observed(w::PolicyGuardObserved { freshness }),
            },
            must_be_guarded: v.must_be_guarded,
        })
        .expect("requirement count");
    }
    Ok(rows)
}
fn policy(
    value: &core::ConfiguredPolicy,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::SettingPolicy, w::Problem> {
    let p = &value.policy;
    let mut projections = List::with_capacity(super::count(p.projections.len())?);
    for v in &p.projections {
        projections.push(grant(v, limits)?).expect("projection count");
    }
    let mut roles = List::with_capacity(super::count(p.roles.len())?);
    for v in &p.roles {
        roles
            .push(w::PolicyEditRole {
                number: v.number,
                authority: authority(&v.authority, models, limits)?,
                period_spend: v.period_spend,
                requests: v.requests.0,
                decides: v.decides.0,
            })
            .expect("role count");
    }
    let mut permissions = List::with_capacity(super::count(value.permissions.len())?);
    for v in &value.permissions {
        permissions
            .push(w::PolicyPermission { connector: v.connector, permission: v.permission, role: v.role })
            .expect("permission count");
    }
    let mut overrides = List::with_capacity(super::count(value.templates.len())?);
    for v in &value.templates {
        overrides.push(v.clone()).expect("template override count");
    }
    Ok(w::SettingPolicy {
        escalation_role: p.escalation_role,
        ceiling: authority(&p.ceiling, models, limits)?,
        projections,
        period_spend: p.period_spend,
        roles,
        requirements: requirements(&p.requirements, limits)?,
        permissions,
        templates: me::overrides(&overrides, limits)?,
    })
}
fn rules(value: &a::Rules, models: &[crate::Model], limits: &w::Limits) -> Result<w::Rules, w::Problem> {
    let mut implies = List::with_capacity(super::count(value.implies.entries().len())?);
    for v in value.implies.entries() {
        implies
            .push(w::KindImplication { connector: v.connector, kind: v.kind, implies: v.implies })
            .expect("implication count");
    }
    Ok(w::Rules {
        ceiling: authority(&value.ceiling, models, limits)?,
        period_spend: value.period_spend,
        minimum_run_spend: value.minimum_run_spend,
        maximum_run_spend: value.maximum_run_spend,
        implies,
        requirements: requirements(&value.requirements, limits)?,
    })
}
pub fn value(value: &s::Value, models: &[crate::Model], limits: &w::Limits) -> Result<w::SettingValue, w::Problem> {
    Ok(match value {
        s::Value::Provider(v) => w::SettingValue::Provider(Box::new(providers::encode(v, limits)?)),
        s::Value::Worker(v) => w::SettingValue::Worker(Box::new(w::WorkerSetting {
            number: v.number,
            name: v.name.clone(),
            places: v.places,
            digest: v.digest,
            withdrawn: v.withdrawn,
        })),
        s::Value::Model(v) => w::SettingValue::Model(Box::new(catalogue(v, limits)?)),
        s::Value::Overlay(v) => w::SettingValue::Overlay(Box::new(overlay(v, limits)?)),
        s::Value::Charter(v) => w::SettingValue::Charter(Box::new(charter(v, limits)?)),
        s::Value::Template(v) => w::SettingValue::Template(Box::new(me::template(v, models, limits)?)),
        s::Value::Policy(v) => w::SettingValue::Policy(Box::new(policy(v, models, limits)?)),
        s::Value::Rules(v) => w::SettingValue::Rules(Box::new(rules(v, models, limits)?)),
        s::Value::Limits(v) => w::SettingValue::Limits(limits_value(v, limits)?),
        s::Value::Application(bytes) => w::SettingValue::Application(w::ApplicationValue { bytes: bytes.clone() }),
    })
}
pub fn setting_document(
    v: &reads::SettingDocument,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::SettingDocument, w::Problem> {
    Ok(w::SettingDocument {
        overlay: match &v.overlay {
            Some(item) => Some(overlay(item, limits)?),
            None => None,
        },
        setting: name(v.setting, limits)?,
        revision: v.revision,
        value: match &v.value {
            Some(v) => Some(value(v, models, limits)?),
            None => None,
        },
        application: match &v.application {
            Some(v) => Some(application_state(v, limits)?),
            None => None,
        },
        proposed: match &v.proposed {
            Some(v) => Some(sized_edit(v, limits)?),
            None => None,
        },
    })
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
                    setting: name(item.setting, limits)?,
                    edit,
                    replaced: replaced(&item.replaced, *value_bytes, models, limits)?,
                };
                let mut out = Writer::new(skein_lib::length::usize_of(item.measure(limits)?));
                item.write(limits, &mut out).expect("measured item");
                pieces.push(out.finish()).expect("item count");
            }
            for item in &revision.applications {
                let setting = s::Setting::Application { kind: item.kind, scope: item.scope, number: item.number };
                let edit = w::RevisionEdit::Application(w::ApplicationEdit { edit: sized_edit(&item.edit, limits)? });
                let row = w::RevisionItem {
                    setting: name(setting, limits)?,
                    edit,
                    replaced: application_replaced(&item.replaced, limits)?,
                };
                let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
                row.write(limits, &mut out).expect("measured application item");
                pieces.push(out.finish()).expect("application item count");
            }
            w::RevisionAnswer::Revision(w::RevisionDocument {
                head: head(&revision.head, limits)?,
                items: super::pieces(&pieces, limits.revision_document_items, w::paths::REVISION_DOCUMENT_ITEMS)?,
            })
        }
    })
}
pub fn revisions(v: &reads::RevisionsPage, limits: &w::Limits) -> Result<w::RevisionsPage, w::Problem> {
    let mut rows = List::with_capacity(v.rows.len());
    for row in &v.rows {
        let mut items = List::with_capacity(row.items.len());
        for item in &row.items {
            items
                .push(w::RevisionItemHead {
                    setting: name(item.setting, limits)?,
                    edit: match item.edit {
                        reads::RevisionEdit::Set => w::RevisionItemEdit::Set,
                        reads::RevisionEdit::Removed => w::RevisionItemEdit::Removed,
                        reads::RevisionEdit::Reset => w::RevisionItemEdit::Reset,
                    },
                })
                .expect("preview count");
        }
        rows.push(w::RevisionRow { head: head(&row.head, limits)?, items, more: row.more }).expect("page count");
    }
    Ok(w::RevisionsPage { rows, next: v.next, forgotten: v.forgotten })
}

pub(crate) fn item_heads(
    items: &List<reads::RevisionItemHead>,
    limits: &w::Limits,
) -> Result<List<w::RevisionItemHead>, w::Problem> {
    let mut result = List::with_capacity(items.len());
    for item in items {
        result
            .push(w::RevisionItemHead { setting: name(item.setting, limits)?, edit: item_edit(item.edit) })
            .expect("bounded heads");
    }
    Ok(result)
}
fn item_edit(edit: reads::RevisionEdit) -> w::RevisionItemEdit {
    match edit {
        reads::RevisionEdit::Set => w::RevisionItemEdit::Set,
        reads::RevisionEdit::Removed => w::RevisionItemEdit::Removed,
        reads::RevisionEdit::Reset => w::RevisionItemEdit::Reset,
    }
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
            setting: name(item.setting, limits)?,
            edit: item_edit(item.edit),
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
                Some(v) => Some(w::ApplicationReview {
                    edit: sized_edit(&v.edit, limits)?,
                    now: match &v.now {
                        Some(v) => Some(application_state(v, limits)?),
                        None => None,
                    },
                }),
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

fn application_state(v: &s::application::State, _limits: &w::Limits) -> Result<w::ApplicationState, w::Problem> {
    let follows = match v.follows {
        s::application::Follows::Default => w::ApplicationFollows::Default,
        s::application::Follows::Changed { revision } => {
            w::ApplicationFollows::Changed(w::ApplicationRevision { revision })
        }
        s::application::Follows::Removed { revision } => {
            w::ApplicationFollows::Removed(w::ApplicationRevision { revision })
        }
    };
    let mut charters = List::with_capacity(super::count(v.charters.len())?);
    for &number in &v.charters {
        charters.push(number).expect("charter count");
    }
    Ok(w::ApplicationState { follows, size: v.size, charters })
}
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn sized_edit(v: &s::application::SizedEdit, _limits: &w::Limits) -> Result<w::SizedEdit, w::Problem> {
    Ok(match v {
        s::application::SizedEdit::Set { size } => w::SizedEdit::Set(w::ApplicationSize { size: *size }),
        s::application::SizedEdit::Remove => w::SizedEdit::Remove,
        s::application::SizedEdit::Reset => w::SizedEdit::Reset,
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
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn application_replaced(v: &s::application::Replaced, _limits: &w::Limits) -> Result<w::RevisionReplaced, w::Problem> {
    Ok(match v {
        s::application::Replaced::Changed { size } => {
            w::RevisionReplaced::Application(w::ApplicationSize { size: *size })
        }
        s::application::Replaced::Default => w::RevisionReplaced::Default,
        s::application::Replaced::Removed => w::RevisionReplaced::Removed,
        s::application::Replaced::NotOffered => w::RevisionReplaced::NotOffered,
    })
}

fn catalogue(value: &s::CatalogueModel, limits: &w::Limits) -> Result<w::CatalogueEntry, w::Problem> {
    Ok(w::CatalogueEntry {
        number: value.number,
        model: model(&value.model, limits)?,
        shown: value.shown.clone(),
        description: value.description.clone(),
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        source: match value.source {
            s::ModelSource::Stock => w::ModelSource::Stock,
            s::ModelSource::List => w::ModelSource::List,
            s::ModelSource::Person => w::ModelSource::Person,
        },
    })
}

#[must_use]
pub fn model_act(value: w::ChangeModel) -> s::catalogue::ModelAct {
    match value.act {
        w::ModelAct::Picker(value) => s::catalogue::ModelAct::Picker { number: value.number, on: value.on },
        w::ModelAct::Add(value) => s::catalogue::ModelAct::Add { provider: value.provider, name: value.name },
    }
}

fn overlay(value: &s::Overlay, limits: &w::Limits) -> Result<w::Overlay, w::Problem> {
    let alternatives = match &value.alternatives {
        Some(items) => {
            let mut alternatives = List::with_capacity(super::count(items.len())?);
            for row in items {
                alternatives.push(model(row, limits)?).expect("model count");
            }
            Some(alternatives)
        }
        None => None,
    };
    let documents = match &value.documents {
        Some(items) => {
            let mut documents = List::with_capacity(super::count(items.len())?);
            for item in items {
                let mut name = List::with_capacity(super::count(item.name.segments.len())?);
                for segment in &item.name.segments {
                    name.push(segment.clone()).expect("segment count");
                }
                documents
                    .push(w::CharterDocument {
                        connector: item.connector,
                        name,
                        path: item.path.clone(),
                        heading: item.heading.clone(),
                    })
                    .expect("document count");
            }
            Some(documents)
        }
        None => None,
    };
    let mut parts = List::with_capacity(super::count(value.parts.len())?);
    for item in &value.parts {
        parts.push(w::CharterPart { on: item.on.0, instructions: item.instructions.clone() }).expect("part count");
    }
    Ok(w::Overlay {
        rest: value.rest,
        inspect: value.inspect,
        modify: value.modify,
        shell: value.shell,
        agents: value.agents,
        waiting: value.waiting,
        resume: value.resume,
        turns: value.turns,
        time: value.time,
        reserve_turns: value.reserve_turns,
        reserve_time: value.reserve_time,
        call_timeout: value.call_timeout,
        spend: value.spend,
        model: match &value.model {
            Some(item) => Some(model(item, limits)?),
            None => None,
        },
        alternatives,
        documents,
        parts,
        instructions: value.instructions.clone(),
        tools: match value.tools {
            Some(item) => Some(item.0),
            None => None,
        },
        hosts: match value.hosts {
            Some(jig_domain_fleet::Kinds::Workers) => Some(w::SettingHosts::Workers),
            Some(jig_domain_fleet::Kinds::Engine) => Some(w::SettingHosts::Engine),
            Some(jig_domain_fleet::Kinds::Both) => Some(w::SettingHosts::Both),
            None => None,
        },
    })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn limits_value(values: &[s::Override], _limits: &w::Limits) -> Result<w::LimitsValue, w::Problem> {
    let mut overrides = List::with_capacity(u32::try_from(values.len()).expect("bounded overrides"));
    for value in values {
        overrides
            .push(w::Override { name: value.name.clone(), value: value.value, reason: value.reason.clone() })
            .expect("override count");
    }
    Ok(w::LimitsValue { overrides })
}
mod providers;
