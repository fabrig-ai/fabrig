//! Wire projections of settings facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, try_list, try_pieces, w};
use super::{
    wire_authority, wire_document_text, wire_grant, wire_kind_implication, wire_policy_edit_role,
    wire_policy_permission, wire_policy_requirement, wire_proposal_key, wire_proposer, wire_template,
    wire_template_override,
};
use skein_lib::List;
#[path = "settings/providers.rs"]
mod providers;
pub fn wire_settings_document(value: &c::SettingsDocument) -> w::SettingsDocument {
    w::SettingsDocument {
        newest: value.newest,
        settings: list(value.settings.iter().map(wire_setting_line)),
        held: list(value.held.iter().map(wire_held_revision)),
        pending: list(value.pending.iter().map(wire_settings_proposal)),
    }
}

pub(in crate::codec) fn wire_setting_line(value: &c::SettingLine) -> w::SettingLine {
    w::SettingLine {
        setting: wire_setting(&value.setting),
        shown: value.shown.as_ref().map(wire_document_text),
        follows: wire_setting_follows(&value.follows),
        last: value.last,
        application: value.application.as_ref().map(wire_application_state),
    }
}

pub(in crate::codec) fn wire_setting(value: &c::Setting) -> w::Setting {
    match value {
        c::Setting::Charter(value) => w::Setting::Charter(wire_setting_number(value)),
        c::Setting::Template(value) => w::Setting::Template(wire_setting_number(value)),
        c::Setting::Policy(value) => w::Setting::Policy(wire_setting_number(value)),
        c::Setting::Application { kind, scope, number } => {
            w::Setting::Application(w::ApplicationSetting { kind: *kind, scope: *scope, number: *number })
        }
        c::Setting::Model(item) => w::Setting::Model(w::SettingNumber { number: item.number }),
        c::Setting::Worker(item) => w::Setting::Worker(w::SettingNumber { number: item.number }),
        c::Setting::Provider(item) => w::Setting::Provider(w::SettingNumber { number: item.number }),
        c::Setting::Rules => w::Setting::Rules,
        c::Setting::Limits => w::Setting::Limits,
        c::Setting::ProjectPolicy { project } => {
            w::Setting::ProjectPolicy(w::ProjectPolicySetting { project: *project })
        }
    }
}

pub(in crate::codec) fn wire_setting_number(value: &c::SettingNumber) -> w::SettingNumber {
    w::SettingNumber { number: value.number }
}

pub(in crate::codec) fn wire_setting_follows(value: &c::SettingFollows) -> w::SettingFollows {
    match value {
        c::SettingFollows::Overlay => w::SettingFollows::Overlay,
        c::SettingFollows::Default => w::SettingFollows::Default,
        c::SettingFollows::Change => w::SettingFollows::Change,
    }
}

pub(in crate::codec) fn wire_held_revision(value: &c::HeldRevision) -> w::HeldRevision {
    w::HeldRevision { head: wire_revision_head(&value.head), holders: value.holders }
}

pub(in crate::codec) fn wire_revision_head(value: &c::RevisionHead) -> w::RevisionHead {
    w::RevisionHead {
        number: value.number,
        against: value.against,
        at: value.at.as_nanos(),
        by: wire_revision_decider(&value.by),
        proposal: value.proposal.as_ref().map(wire_proposal_key),
        moved: value.moved,
    }
}

pub(in crate::codec) fn wire_revision_decider(value: &c::RevisionDecider) -> w::RevisionDecider {
    match value {
        c::RevisionDecider::FirstStart => w::RevisionDecider::FirstStart,
        c::RevisionDecider::Release => w::RevisionDecider::Release,
        c::RevisionDecider::List { provider } => w::RevisionDecider::List(w::RevisionList { provider: *provider }),
        c::RevisionDecider::Party(value) => w::RevisionDecider::Party(wire_revision_party(value)),
    }
}

pub(in crate::codec) fn wire_revision_party(value: &c::RevisionParty) -> w::RevisionParty {
    w::RevisionParty { number: value.number }
}

pub(in crate::codec) fn wire_settings_proposal(value: &c::SettingsProposal) -> w::SettingsProposal {
    w::SettingsProposal { proposer: wire_proposer(&value.proposer), number: value.number }
}

pub fn wire_setting_document(value: &c::SettingDocument, limits: &w::Limits) -> Result<w::SettingDocument, w::Problem> {
    Ok(w::SettingDocument {
        overlay: value.overlay.as_deref().map(wire_overlay),
        setting: wire_setting(&value.setting),
        revision: value.revision,
        value: value.value.as_ref().map(|v| wire_setting_value(v, limits)).transpose()?,
        application: value.application.as_ref().map(wire_application_state),
        proposed: value.proposed.as_ref().map(wire_sized_edit),
    })
}

pub(in crate::codec) fn wire_setting_value(
    value: &c::SettingValue,
    limits: &w::Limits,
) -> Result<w::SettingValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingValue::Model(value) => w::SettingValue::Model(Box::new(wire_catalogue(value))),
        c::SettingValue::Provider(value) => w::SettingValue::Provider(Box::new(providers::encode(value, limits)?)),
        c::SettingValue::Worker(value) => w::SettingValue::Worker(Box::new(wire_worker(value))),
        c::SettingValue::Overlay(value) => w::SettingValue::Overlay(Box::new(wire_overlay(value))),
        c::SettingValue::Charter(value) => w::SettingValue::Charter(Box::new(wire_charter(value))),
        c::SettingValue::Template(value) => w::SettingValue::Template(Box::new(wire_template(value, limits)?)),
        c::SettingValue::Policy(value) => w::SettingValue::Policy(Box::new(wire_setting_policy(value, limits)?)),
        c::SettingValue::Application(value) => {
            w::SettingValue::Application(w::ApplicationValue { bytes: value.clone() })
        }
        c::SettingValue::Rules(value) => w::SettingValue::Rules(Box::new(wire_rules(value, limits)?)),
        c::SettingValue::Limits(value) => w::SettingValue::Limits(wire_limits(value)),
    })
}

pub(in crate::codec) fn wire_charter(value: &c::Charter) -> w::Charter {
    w::Charter {
        parts: list(
            value
                .parts
                .iter()
                .map(|item| w::CharterPart { on: item.on, instructions: bytes::copy_of(item.instructions.bytes()) }),
        ),
        documents: list(value.documents.iter().map(|item| w::CharterDocument {
            connector: item.connector,
            name: item.name.clone(),
            path: item.path.clone(),
            heading: item.heading.clone(),
        })),
        rest: value.rest,
        resume: value.resume,
        inspect: value.inspect,
        modify: value.modify,
        shell: value.shell,
        agents: value.agents,
        number: value.number,
        instructions: bytes::copy_of(value.instructions.bytes()),
        waiting: value.waiting,
        turns: value.turns,
        time: value.time,
        reserve_turns: value.reserve_turns,
        reserve_time: value.reserve_time,
        model: wire_setting_model(&value.model),
        alternatives: list(value.alternatives.iter().map(wire_setting_model)),
        call_timeout: value.call_timeout,
        tools: value.tools,
        hosts: wire_setting_hosts(&value.hosts),
        spend: value.spend,
        wake: super::wire_wake_policy(&value.wake),
    }
}

pub(in crate::codec) fn wire_setting_model(value: &c::SettingModel) -> w::SettingModel {
    w::SettingModel {
        provider: value.provider,
        dialect: value.dialect,
        endpoint: value.endpoint,
        program: value.program.as_ref().map(|item| bytes::copy_of((item).bytes())),
        name: bytes::copy_of(value.name.bytes()),
        window: value.window,
        output: value.output,
        input_price: value.input_price,
        cached_price: value.cached_price,
        cache_write_price: value.cache_write_price,
        output_price: value.output_price,
        price_unit: value.price_unit,
        efforts: list(value.efforts.iter().map(|item| bytes::copy_of((item).bytes()))),
        effort: value.effort.as_ref().map(|item| bytes::copy_of((item).bytes())),
        speeds: list(value.speeds.iter().map(|speed| w::SettingSpeed {
            name: bytes::copy_of(speed.name.bytes()),
            also: list(speed.also.iter().map(|name| bytes::copy_of(name.bytes()))),
            input_price: speed.input_price,
            cached_price: speed.cached_price,
            cache_write_price: speed.cache_write_price,
            output_price: speed.output_price,
        })),
        speed: value.speed.as_ref().map(|name| bytes::copy_of(name.bytes())),
    }
}

pub(in crate::codec) fn wire_setting_hosts(value: &c::SettingHosts) -> w::SettingHosts {
    match value {
        c::SettingHosts::Workers => w::SettingHosts::Workers,
        c::SettingHosts::Engine => w::SettingHosts::Engine,
        c::SettingHosts::Both => w::SettingHosts::Both,
    }
}

pub(in crate::codec) fn wire_setting_policy(
    value: &c::SettingPolicy,
    limits: &w::Limits,
) -> Result<w::SettingPolicy, w::Problem> {
    Ok(w::SettingPolicy {
        escalation_role: value.escalation_role,
        ceiling: wire_authority(&value.ceiling, limits)?,
        projections: list(value.projections.iter().map(wire_grant)),
        period_spend: value.period_spend,
        roles: try_list(value.roles.iter().map(|item| wire_policy_edit_role(item, limits)))?,
        requirements: list(value.requirements.iter().map(wire_policy_requirement)),
        permissions: list(value.permissions.iter().map(wire_policy_permission)),
        templates: list(value.templates.iter().map(wire_template_override)),
    })
}

pub(in crate::codec) fn wire_rules(value: &c::Rules, limits: &w::Limits) -> Result<w::Rules, w::Problem> {
    Ok(w::Rules {
        ceiling: wire_authority(&value.ceiling, limits)?,
        period_spend: value.period_spend,
        minimum_run_spend: value.minimum_run_spend,
        maximum_run_spend: value.maximum_run_spend,
        implies: list(value.implies.iter().map(wire_kind_implication)),
        requirements: list(value.requirements.iter().map(wire_policy_requirement)),
    })
}

pub fn wire_revision_answer(value: &c::RevisionAnswer, limits: &w::Limits) -> Result<w::RevisionAnswer, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionAnswer::Revision(value) => w::RevisionAnswer::Revision(wire_revision_document(value, limits)?),
        c::RevisionAnswer::Forgotten(value) => w::RevisionAnswer::Forgotten(wire_revision_forgotten(value)),
        c::RevisionAnswer::Unknown => w::RevisionAnswer::Unknown,
    })
}

pub(in crate::codec) fn wire_revision_document(
    value: &c::RevisionDocument,
    limits: &w::Limits,
) -> Result<w::RevisionDocument, w::Problem> {
    Ok(w::RevisionDocument {
        head: wire_revision_head(&value.head),
        items: try_pieces(value.items.iter().map(|item| wire_revision_item(item, limits)), limits)?,
    })
}

pub(in crate::codec) fn wire_revision_item(
    value: &c::RevisionItem,
    limits: &w::Limits,
) -> Result<w::RevisionItem, w::Problem> {
    Ok(w::RevisionItem {
        setting: wire_setting(&value.setting),
        edit: wire_revision_edit(&value.edit, limits)?,
        replaced: wire_revision_replaced(&value.replaced, limits)?,
    })
}

pub(in crate::codec) fn wire_revision_edit(
    value: &c::RevisionEdit,
    limits: &w::Limits,
) -> Result<w::RevisionEdit, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionEdit::Set(value) => w::RevisionEdit::Set(wire_setting_set(value, limits)?),
        c::RevisionEdit::TooLarge(value) => w::RevisionEdit::TooLarge(wire_size(value)),
        c::RevisionEdit::Application(value) => {
            w::RevisionEdit::Application(w::ApplicationEdit { edit: wire_sized_edit(value) })
        }
        c::RevisionEdit::Removed => w::RevisionEdit::Removed,
        c::RevisionEdit::Reset => w::RevisionEdit::Reset,
    })
}

pub(in crate::codec) fn wire_setting_set(
    value: &c::SettingSet,
    limits: &w::Limits,
) -> Result<w::SettingSet, w::Problem> {
    Ok(w::SettingSet { value: wire_setting_value(&value.value, limits)? })
}

pub(in crate::codec) fn wire_size(value: &c::Size) -> w::Size {
    w::Size { bytes: value.bytes }
}

pub(in crate::codec) fn wire_revision_forgotten(value: &c::RevisionForgotten) -> w::RevisionForgotten {
    w::RevisionForgotten { oldest: value.oldest }
}

pub fn wire_revisions_page(value: &c::RevisionsPage) -> w::RevisionsPage {
    w::RevisionsPage {
        rows: list(value.rows.iter().map(wire_revision_row)),
        next: value.next,
        forgotten: value.forgotten,
    }
}

pub(in crate::codec) fn wire_revision_row(value: &c::RevisionRow) -> w::RevisionRow {
    w::RevisionRow {
        head: wire_revision_head(&value.head),
        items: list(value.items.iter().map(wire_revision_item_head)),
        more: value.more,
    }
}

pub(in crate::codec) fn wire_revision_item_head(value: &c::RevisionItemHead) -> w::RevisionItemHead {
    w::RevisionItemHead { setting: wire_setting(&value.setting), edit: wire_revision_item_edit(&value.edit) }
}

pub(in crate::codec) fn wire_revision_item_edit(value: &c::RevisionItemEdit) -> w::RevisionItemEdit {
    match value {
        c::RevisionItemEdit::Set => w::RevisionItemEdit::Set,
        c::RevisionItemEdit::Removed => w::RevisionItemEdit::Removed,
        c::RevisionItemEdit::Reset => w::RevisionItemEdit::Reset,
    }
}

pub(in crate::codec) fn wire_refusal_stale(value: &c::RefusalStale) -> w::RefusalStale {
    w::RefusalStale { newest: value.newest }
}

pub(in crate::codec) fn wire_refusal_full(value: &c::RefusalFull) -> w::RefusalFull {
    w::RefusalFull { revision: value.revision, tasks: list(value.tasks.iter().copied()), more: value.more }
}

#[must_use]
pub fn wire_malformed_settings(value: &c::MalformedSettings) -> w::MalformedSettings {
    w::MalformedSettings {
        setting: wire_setting(&value.setting),
        fault: wire_setting_fault(&value.fault),
        detail: None,
    }
}

pub(in crate::codec) fn wire_setting_fault(value: &c::SettingFault) -> w::SettingFault {
    match value {
        c::SettingFault::Shape => w::SettingFault::Shape,
        c::SettingFault::Count => w::SettingFault::Count,
        c::SettingFault::Reference => w::SettingFault::Reference,
        c::SettingFault::Rules => w::SettingFault::Rules,
        c::SettingFault::Budget => w::SettingFault::Budget,
        c::SettingFault::Grace => w::SettingFault::Grace,
        c::SettingFault::Judged => w::SettingFault::Judged,
        c::SettingFault::Linked => w::SettingFault::Linked,
        c::SettingFault::Accounts => w::SettingFault::Accounts,
        c::SettingFault::Range => w::SettingFault::Range,
        c::SettingFault::Name => w::SettingFault::Name,
        c::SettingFault::Owner => w::SettingFault::Owner,
    }
}

pub fn wire_retained_action_settings(value: &c::RetainedActionSettings) -> w::RetainedActionSettings {
    w::RetainedActionSettings {
        projects: value.projects.clone(),
        against: value.against,
        items: list(value.items.iter().map(wire_revision_item_head)),
        made: value.made,
    }
}

pub fn wire_proposal_action_settings(
    value: &c::ProposalActionSettings,
    limits: &w::Limits,
) -> Result<w::ProposalActionSettings, w::Problem> {
    Ok(w::ProposalActionSettings {
        against: value.against,
        items: list(value.items.iter().map(wire_revision_item_head)),
        review: value.review.as_ref().map(|item| wire_settings_review(item, limits)).transpose()?,
        made: value.made,
    })
}

pub(in crate::codec) fn wire_settings_review(
    value: &c::SettingsReview,
    limits: &w::Limits,
) -> Result<w::SettingsReview, w::Problem> {
    Ok(w::SettingsReview {
        key: None,
        newest: value.newest,
        items: try_pieces(value.items.iter().map(|item| wire_review_item(item, limits)), limits)?,
        cut: value.cut,
    })
}

pub(in crate::codec) fn wire_review_item(
    value: &c::ReviewItem,
    limits: &w::Limits,
) -> Result<w::ReviewItem, w::Problem> {
    Ok(w::ReviewItem {
        setting: wire_setting(&value.setting),
        edit: wire_revision_item_edit(&value.edit),
        now: wire_review_value(&value.now, limits)?,
        origin: wire_setting_origin(&value.origin),
        would_be: wire_review_value(&value.would_be, limits)?,
        application: value.application.as_ref().map(|v| w::ApplicationReview {
            edit: wire_sized_edit(&v.edit),
            now: v.now.as_ref().map(wire_application_state),
        }),
    })
}

pub(in crate::codec) fn wire_review_value(
    value: &c::ReviewValue,
    limits: &w::Limits,
) -> Result<w::ReviewValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ReviewValue::Value(value) => w::ReviewValue::Value(wire_setting_set(value, limits)?),
        c::ReviewValue::TooLarge(value) => w::ReviewValue::TooLarge(wire_size(value)),
        c::ReviewValue::None => w::ReviewValue::None,
    })
}

pub(in crate::codec) fn wire_setting_origin(value: &c::SettingOrigin) -> w::SettingOrigin {
    match value {
        c::SettingOrigin::Default(value) => w::SettingOrigin::Default(wire_origin_revision(value)),
        c::SettingOrigin::Change(value) => w::SettingOrigin::Change(wire_origin_revision(value)),
        c::SettingOrigin::NotOffered => w::SettingOrigin::NotOffered,
    }
}

pub(in crate::codec) fn wire_origin_revision(value: &c::OriginRevision) -> w::OriginRevision {
    w::OriginRevision { revision: value.revision }
}

pub(in crate::codec) fn wire_outcome_settings_changed(value: &c::OutcomeSettingsChanged) -> w::OutcomeSettingsChanged {
    w::OutcomeSettingsChanged { revision: value.revision }
}

pub(in crate::codec) fn wire_outcome_moved_to_newest(value: &c::OutcomeMovedToNewest) -> w::OutcomeMovedToNewest {
    w::OutcomeMovedToNewest { newest: value.newest, moved: value.moved, stayed: value.stayed }
}

fn wire_application_state(v: &c::ApplicationState) -> w::ApplicationState {
    let follows = match v.follows {
        c::ApplicationFollows::Default => w::ApplicationFollows::Default,
        c::ApplicationFollows::Changed { revision } => {
            w::ApplicationFollows::Changed(w::ApplicationRevision { revision })
        }
        c::ApplicationFollows::Removed { revision } => {
            w::ApplicationFollows::Removed(w::ApplicationRevision { revision })
        }
    };
    w::ApplicationState { follows, size: v.size, charters: v.charters.clone() }
}
fn wire_sized_edit(v: &c::SizedEdit) -> w::SizedEdit {
    match v {
        c::SizedEdit::Set { size } => w::SizedEdit::Set(w::ApplicationSize { size: *size }),
        c::SizedEdit::Remove => w::SizedEdit::Remove,
        c::SizedEdit::Reset => w::SizedEdit::Reset,
    }
}

pub fn wire_revision_replaced(
    value: &c::RevisionReplaced,
    limits: &w::Limits,
) -> Result<w::RevisionReplaced, w::Problem> {
    Ok(match value {
        c::RevisionReplaced::Value(v) => w::RevisionReplaced::Value(wire_setting_set(v, limits)?),
        c::RevisionReplaced::TooLarge(v) => w::RevisionReplaced::TooLarge(wire_size(v)),
        c::RevisionReplaced::Default => w::RevisionReplaced::Default,
        c::RevisionReplaced::Removed => w::RevisionReplaced::Removed,
        c::RevisionReplaced::NotOffered => w::RevisionReplaced::NotOffered,
        c::RevisionReplaced::Application { size } => {
            w::RevisionReplaced::Application(w::ApplicationSize { size: *size })
        }
    })
}

fn wire_catalogue(value: &c::CatalogueEntry) -> w::CatalogueEntry {
    w::CatalogueEntry {
        number: value.number,
        model: wire_setting_model(&value.model),
        shown: value.shown.as_ref().map(|value| bytes::copy_of(value.bytes())),
        description: value.description.as_ref().map(|value| bytes::copy_of(value.bytes())),
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        source: match value.source {
            c::ModelSource::Stock => w::ModelSource::Stock,
            c::ModelSource::List => w::ModelSource::List,
            c::ModelSource::Person => w::ModelSource::Person,
        },
    }
}

pub(in crate::codec) fn wire_overlay(value: &c::Overlay) -> w::Overlay {
    let alternatives = match &value.alternatives {
        Some(items) => {
            let mut alternatives = List::with_capacity(items.len());
            for item in items {
                alternatives.push(wire_setting_model(item)).expect("same capacity");
            }
            Some(alternatives)
        }
        None => None,
    };
    let documents = match &value.documents {
        Some(items) => {
            let mut documents = List::with_capacity(items.len());
            for item in items {
                documents
                    .push(w::CharterDocument {
                        connector: item.connector,
                        name: item.name.clone(),
                        path: item.path.clone(),
                        heading: item.heading.clone(),
                    })
                    .expect("same capacity");
            }
            Some(documents)
        }
        None => None,
    };
    let mut parts = List::with_capacity(value.parts.len());
    for item in &value.parts {
        parts
            .push(w::CharterPart { on: item.on, instructions: bytes::copy_of(item.instructions.bytes()) })
            .expect("same capacity");
    }
    w::Overlay {
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
        model: value.model.as_ref().map(wire_setting_model),
        alternatives,
        documents,
        parts,
        tools: value.tools,
        hosts: value.hosts.as_ref().map(wire_setting_hosts),
        instructions: bytes::copy_of(value.instructions.bytes()),
    }
}

fn wire_limits(values: &[c::Override]) -> w::LimitsValue {
    let mut overrides = List::with_capacity(u32::try_from(values.len()).expect("bounded overrides"));
    for value in values {
        overrides
            .push(w::Override { name: value.name.clone(), value: value.value, reason: value.reason.clone() })
            .expect("override count");
    }
    w::LimitsValue { overrides }
}

fn wire_worker(value: &c::WorkerSetting) -> w::WorkerSetting {
    w::WorkerSetting {
        number: value.number,
        name: value.name.clone(),
        places: value.places,
        digest: value.digest,
        withdrawn: value.withdrawn,
    }
}
