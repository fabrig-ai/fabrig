//! Wire projections of settings facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
use super::{
    wire_authority, wire_document_text, wire_grant, wire_kind_implication, wire_policy_edit_role,
    wire_policy_permission, wire_policy_requirement, wire_proposal_key, wire_proposer, wire_template,
    wire_template_override,
};
use skein_lib::List;
#[path = "settings/providers.rs"]
mod providers;
pub fn wire_settings_document(
    value: &c::SettingsDocument,
    limits: &w::Limits,
) -> Result<w::SettingsDocument, w::Problem> {
    Ok(w::SettingsDocument {
            newest: value.newest,
            settings: list(value.settings.iter().map(|item| wire_setting_line(item, limits)))?,
            held: list(value.held.iter().map(|item| wire_held_revision(item, limits)))?,
            pending: list(value.pending.iter().map(|item| wire_settings_proposal(item, limits)))?,
        })
}

pub(in crate::codec) fn wire_setting_line(
    value: &c::SettingLine,
    limits: &w::Limits,
) -> Result<w::SettingLine, w::Problem> {
    Ok(w::SettingLine {
            setting: wire_setting(&value.setting, limits)?,
            name: value.name.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            follows: wire_setting_follows(&value.follows, limits)?,
            last: value.last,
            application: value.application.as_ref().map(|v| wire_application_state(v, limits)).transpose()?,
        })
}

pub(in crate::codec) fn wire_setting(value: &c::Setting, limits: &w::Limits) -> Result<w::Setting, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Setting::Charter(value) => w::Setting::Charter(wire_setting_number(value, limits)?),
        c::Setting::Template(value) => w::Setting::Template(wire_setting_number(value, limits)?),
        c::Setting::Policy(value) => w::Setting::Policy(wire_setting_number(value, limits)?),
        c::Setting::Application { kind, scope, number } => w::Setting::Application(w::ApplicationSetting { kind: *kind, scope: *scope, number: *number }),
        c::Setting::Model(item) => {
            w::Setting::Model(w::SettingNumber { number: item.number })
        }
        c::Setting::Worker(item) => {
            w::Setting::Worker(w::SettingNumber { number: item.number })
        }
        c::Setting::Provider(item) => {
            w::Setting::Provider(w::SettingNumber { number: item.number })
        }
        c::Setting::Rules => w::Setting::Rules,
        c::Setting::Limits => w::Setting::Limits,
        c::Setting::ProjectPolicy { project } => w::Setting::ProjectPolicy(w::ProjectPolicySetting { project: *project }),
    })
}

pub(in crate::codec) fn wire_setting_number(
    value: &c::SettingNumber,
    _limits: &w::Limits,
) -> Result<w::SettingNumber, w::Problem> {
    Ok(w::SettingNumber { number: value.number })
}

pub(in crate::codec) fn wire_setting_follows(
    value: &c::SettingFollows,
    _limits: &w::Limits,
) -> Result<w::SettingFollows, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingFollows::Overlay => w::SettingFollows::Overlay,
        c::SettingFollows::Default => w::SettingFollows::Default,
        c::SettingFollows::Change => w::SettingFollows::Change,
    })
}

pub(in crate::codec) fn wire_held_revision(
    value: &c::HeldRevision,
    limits: &w::Limits,
) -> Result<w::HeldRevision, w::Problem> {
    Ok(w::HeldRevision { head: wire_revision_head(&value.head, limits)?, holders: value.holders })
}

pub(in crate::codec) fn wire_revision_head(
    value: &c::RevisionHead,
    limits: &w::Limits,
) -> Result<w::RevisionHead, w::Problem> {
    Ok(w::RevisionHead {
            number: value.number,
            against: value.against,
            at: value.at.as_nanos(),
            by: wire_revision_decider(&value.by, limits)?,
            proposal: value.proposal.as_ref().map(|item| wire_proposal_key(item, limits)).transpose()?,
            moved: value.moved,
        })
}

pub(in crate::codec) fn wire_revision_decider(
    value: &c::RevisionDecider,
    limits: &w::Limits,
) -> Result<w::RevisionDecider, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionDecider::FirstStart => w::RevisionDecider::FirstStart,
        c::RevisionDecider::Release => w::RevisionDecider::Release,
        c::RevisionDecider::List { provider } => {
            w::RevisionDecider::List(w::RevisionList { provider: *provider })
        }
        c::RevisionDecider::Party(value) => w::RevisionDecider::Party(wire_revision_party(value, limits)?),
    })
}

pub(in crate::codec) fn wire_revision_party(
    value: &c::RevisionParty,
    _limits: &w::Limits,
) -> Result<w::RevisionParty, w::Problem> {
    Ok(w::RevisionParty { number: value.number })
}

pub(in crate::codec) fn wire_settings_proposal(
    value: &c::SettingsProposal,
    limits: &w::Limits,
) -> Result<w::SettingsProposal, w::Problem> {
    Ok(w::SettingsProposal { proposer: wire_proposer(&value.proposer, limits)?, number: value.number })
}

pub fn wire_setting_document(value: &c::SettingDocument, limits: &w::Limits) -> Result<w::SettingDocument, w::Problem> {
    Ok(w::SettingDocument {
            overlay: value.overlay.as_ref().map(|item| wire_overlay(item, limits)).transpose()?,
            setting: wire_setting(&value.setting, limits)?,
            revision: value.revision,
            value: value.value.as_ref().map(|v| wire_setting_value(v, limits)).transpose()?,
            application: value.application.as_ref().map(|v| wire_application_state(v, limits)).transpose()?,
            proposed: value.proposed.as_ref().map(|v| wire_sized_edit(v, limits)).transpose()?,
        })
}

pub(in crate::codec) fn wire_setting_value(
    value: &c::SettingValue,
    limits: &w::Limits,
) -> Result<w::SettingValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingValue::Model(value) => w::SettingValue::Model(Box::new(wire_catalogue(value, limits)?)),
        c::SettingValue::Provider(value) => w::SettingValue::Provider(Box::new(providers::encode(value, limits)?)),
        c::SettingValue::Worker(value) => w::SettingValue::Worker(Box::new(wire_worker(value, limits)?)),
        c::SettingValue::Overlay(value) => w::SettingValue::Overlay(Box::new(wire_overlay(value, limits)?)),
        c::SettingValue::Charter(value) => w::SettingValue::Charter(Box::new(wire_charter(value, limits)?)),
        c::SettingValue::Template(value) => w::SettingValue::Template(Box::new(wire_template(value, limits)?)),
        c::SettingValue::Policy(value) => w::SettingValue::Policy(Box::new(wire_setting_policy(value, limits)?)),
        c::SettingValue::Application(value) => w::SettingValue::Application(w::ApplicationValue { bytes: value.clone() }),
        c::SettingValue::Rules(value) => w::SettingValue::Rules(Box::new(wire_rules(value, limits)?)),
        c::SettingValue::Limits(value) => w::SettingValue::Limits(wire_limits(value, limits)?),
    })
}

pub(in crate::codec) fn wire_charter(value: &c::Charter, limits: &w::Limits) -> Result<w::Charter, w::Problem> {
    Ok(w::Charter {
            parts: list(value.parts.iter().map(|item| {
                Ok(w::CharterPart { on: item.on, instructions: bytes::copy_of(item.instructions.bytes()) })
            }))?,
            documents: list(value.documents.iter().map(|item| {
                Ok(w::CharterDocument {
                        connector: item.connector,
                        name: item.name.clone(),
                        path: item.path.clone(),
                        heading: item.heading.clone(),
                    })
            }))?,
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
            model: wire_setting_model(&value.model, limits)?,
            alternatives: list(value.alternatives.iter().map(|item| wire_setting_model(item, limits)))?,
            call_timeout: value.call_timeout,
            tools: value.tools,
            hosts: wire_setting_hosts(&value.hosts, limits)?,
            spend: value.spend,
            wake: super::wire_wake_policy(&value.wake, limits)?,
        })
}

pub(in crate::codec) fn wire_setting_model(
    value: &c::SettingModel,
    _limits: &w::Limits,
) -> Result<w::SettingModel, w::Problem> {
    Ok(w::SettingModel {
            provider: value.provider,
            dialect: value.dialect,
            endpoint: value.endpoint,
            program: value
                .program
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
            name: bytes::copy_of(value.name.bytes()),
            window: value.window,
            output: value.output,
            input_price: value.input_price,
            cached_price: value.cached_price,
            cache_write_price: value.cache_write_price.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            output_price: value.output_price,
            price_unit: value.price_unit,
            efforts: list(value.efforts.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
            effort: value
                .effort
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
        })
}

pub(in crate::codec) fn wire_setting_hosts(
    value: &c::SettingHosts,
    _limits: &w::Limits,
) -> Result<w::SettingHosts, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingHosts::Workers => w::SettingHosts::Workers,
        c::SettingHosts::Engine => w::SettingHosts::Engine,
        c::SettingHosts::Both => w::SettingHosts::Both,
    })
}

pub(in crate::codec) fn wire_setting_policy(
    value: &c::SettingPolicy,
    limits: &w::Limits,
) -> Result<w::SettingPolicy, w::Problem> {
    Ok(w::SettingPolicy {
            escalation_role: value.escalation_role.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            ceiling: wire_authority(&value.ceiling, limits)?,
            projections: list(value.projections.iter().map(|item| wire_grant(item, limits)))?,
            period_spend: value.period_spend,
            roles: list(value.roles.iter().map(|item| wire_policy_edit_role(item, limits)))?,
            requirements: list(value.requirements.iter().map(|item| wire_policy_requirement(item, limits)))?,
            permissions: list(value.permissions.iter().map(|item| wire_policy_permission(item, limits)))?,
            templates: list(value.templates.iter().map(|item| wire_template_override(item, limits)))?,
        })
}

pub(in crate::codec) fn wire_rules(value: &c::Rules, limits: &w::Limits) -> Result<w::Rules, w::Problem> {
    Ok(w::Rules {
            ceiling: wire_authority(&value.ceiling, limits)?,
            period_spend: value.period_spend,
            minimum_run_spend: value.minimum_run_spend,
            maximum_run_spend: value.maximum_run_spend,
            implies: list(value.implies.iter().map(|item| wire_kind_implication(item, limits)))?,
            requirements: list(value.requirements.iter().map(|item| wire_policy_requirement(item, limits)))?,
        })
}

pub fn wire_revision_answer(value: &c::RevisionAnswer, limits: &w::Limits) -> Result<w::RevisionAnswer, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionAnswer::Revision(value) => w::RevisionAnswer::Revision(wire_revision_document(value, limits)?),
        c::RevisionAnswer::Forgotten(value) => w::RevisionAnswer::Forgotten(wire_revision_forgotten(value, limits)?),
        c::RevisionAnswer::Unknown => w::RevisionAnswer::Unknown,
    })
}

pub(in crate::codec) fn wire_revision_document(
    value: &c::RevisionDocument,
    limits: &w::Limits,
) -> Result<w::RevisionDocument, w::Problem> {
    Ok(w::RevisionDocument {
            head: wire_revision_head(&value.head, limits)?,
            items: pieces(value.items.iter().map(|item| wire_revision_item(item, limits)), limits)?,
        })
}

pub(in crate::codec) fn wire_revision_item(
    value: &c::RevisionItem,
    limits: &w::Limits,
) -> Result<w::RevisionItem, w::Problem> {
    Ok(w::RevisionItem {
            setting: wire_setting(&value.setting, limits)?,
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
        c::RevisionEdit::TooLarge(value) => w::RevisionEdit::TooLarge(wire_size(value, limits)?),
        c::RevisionEdit::Application(value) => w::RevisionEdit::Application(w::ApplicationEdit { edit: wire_sized_edit(value, limits)? }),
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

pub(in crate::codec) fn wire_size(value: &c::Size, _limits: &w::Limits) -> Result<w::Size, w::Problem> {
    Ok(w::Size { bytes: value.bytes })
}

pub(in crate::codec) fn wire_revision_forgotten(
    value: &c::RevisionForgotten,
    _limits: &w::Limits,
) -> Result<w::RevisionForgotten, w::Problem> {
    Ok(w::RevisionForgotten { oldest: value.oldest })
}

pub fn wire_revisions_page(value: &c::RevisionsPage, limits: &w::Limits) -> Result<w::RevisionsPage, w::Problem> {
    Ok(w::RevisionsPage {
            rows: list(value.rows.iter().map(|item| wire_revision_row(item, limits)))?,
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            forgotten: value.forgotten,
        })
}

pub(in crate::codec) fn wire_revision_row(
    value: &c::RevisionRow,
    limits: &w::Limits,
) -> Result<w::RevisionRow, w::Problem> {
    Ok(w::RevisionRow {
            head: wire_revision_head(&value.head, limits)?,
            items: list(value.items.iter().map(|item| wire_revision_item_head(item, limits)))?,
            more: value.more,
        })
}

pub(in crate::codec) fn wire_revision_item_head(
    value: &c::RevisionItemHead,
    limits: &w::Limits,
) -> Result<w::RevisionItemHead, w::Problem> {
    Ok(w::RevisionItemHead {
            setting: wire_setting(&value.setting, limits)?,
            edit: wire_revision_item_edit(&value.edit, limits)?,
        })
}

pub(in crate::codec) fn wire_revision_item_edit(
    value: &c::RevisionItemEdit,
    _limits: &w::Limits,
) -> Result<w::RevisionItemEdit, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionItemEdit::Set => w::RevisionItemEdit::Set,
        c::RevisionItemEdit::Removed => w::RevisionItemEdit::Removed,
        c::RevisionItemEdit::Reset => w::RevisionItemEdit::Reset,
    })
}

pub(in crate::codec) fn wire_refusal_stale(
    value: &c::RefusalStale,
    _limits: &w::Limits,
) -> Result<w::RefusalStale, w::Problem> {
    Ok(w::RefusalStale { newest: value.newest })
}

pub(in crate::codec) fn wire_refusal_full(
    value: &c::RefusalFull,
    _limits: &w::Limits,
) -> Result<w::RefusalFull, w::Problem> {
    Ok(w::RefusalFull {
            revision: value.revision,
            tasks: list(value.tasks.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            more: value.more,
        })
}

pub fn wire_malformed_settings(
    value: &c::MalformedSettings,
    limits: &w::Limits,
) -> Result<w::MalformedSettings, w::Problem> {
    Ok(w::MalformedSettings {
            setting: wire_setting(&value.setting, limits)?,
            fault: wire_setting_fault(&value.fault, limits)?,
            detail: None,
        })
}

pub(in crate::codec) fn wire_setting_fault(
    value: &c::SettingFault,
    _limits: &w::Limits,
) -> Result<w::SettingFault, w::Problem> {
    Ok::<_, w::Problem>(match value {
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
        c::SettingFault::Owner => w::SettingFault::Owner,
    })
}

pub fn wire_retained_action_settings(
    value: &c::RetainedActionSettings,
    limits: &w::Limits,
) -> Result<w::RetainedActionSettings, w::Problem> {
    Ok(w::RetainedActionSettings {
            projects: value.projects.clone(),
            against: value.against,
            items: list(value.items.iter().map(|item| wire_revision_item_head(item, limits)))?,
            made: value.made.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        })
}

pub fn wire_proposal_action_settings(
    value: &c::ProposalActionSettings,
    limits: &w::Limits,
) -> Result<w::ProposalActionSettings, w::Problem> {
    Ok(w::ProposalActionSettings {
            against: value.against,
            items: list(value.items.iter().map(|item| wire_revision_item_head(item, limits)))?,
            review: value.review.as_ref().map(|item| wire_settings_review(item, limits)).transpose()?,
            made: value.made.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        })
}

pub(in crate::codec) fn wire_settings_review(
    value: &c::SettingsReview,
    limits: &w::Limits,
) -> Result<w::SettingsReview, w::Problem> {
    Ok(w::SettingsReview {
            key: None,
            newest: value.newest,
            items: pieces(value.items.iter().map(|item| wire_review_item(item, limits)), limits)?,
            cut: value.cut,
        })
}

pub(in crate::codec) fn wire_review_item(
    value: &c::ReviewItem,
    limits: &w::Limits,
) -> Result<w::ReviewItem, w::Problem> {
    Ok(w::ReviewItem {
            setting: wire_setting(&value.setting, limits)?,
            edit: wire_revision_item_edit(&value.edit, limits)?,
            now: wire_review_value(&value.now, limits)?,
            origin: wire_setting_origin(&value.origin, limits)?,
            would_be: wire_review_value(&value.would_be, limits)?,
            application: value
                .application
                .as_ref()
                .map(|v| {
                    Ok(w::ApplicationReview {
                            edit: wire_sized_edit(&v.edit, limits)?,
                            now: v.now.as_ref().map(|v| wire_application_state(v, limits)).transpose()?,
                        })
                })
                .transpose()?,
        })
}

pub(in crate::codec) fn wire_review_value(
    value: &c::ReviewValue,
    limits: &w::Limits,
) -> Result<w::ReviewValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ReviewValue::Value(value) => w::ReviewValue::Value(wire_setting_set(value, limits)?),
        c::ReviewValue::TooLarge(value) => w::ReviewValue::TooLarge(wire_size(value, limits)?),
        c::ReviewValue::None => w::ReviewValue::None,
    })
}

pub(in crate::codec) fn wire_setting_origin(
    value: &c::SettingOrigin,
    limits: &w::Limits,
) -> Result<w::SettingOrigin, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingOrigin::Default(value) => w::SettingOrigin::Default(wire_origin_revision(value, limits)?),
        c::SettingOrigin::Change(value) => w::SettingOrigin::Change(wire_origin_revision(value, limits)?),
        c::SettingOrigin::NotOffered => w::SettingOrigin::NotOffered,
    })
}

pub(in crate::codec) fn wire_origin_revision(
    value: &c::OriginRevision,
    _limits: &w::Limits,
) -> Result<w::OriginRevision, w::Problem> {
    Ok(w::OriginRevision { revision: value.revision })
}

pub(in crate::codec) fn wire_outcome_settings_changed(
    value: &c::OutcomeSettingsChanged,
    _limits: &w::Limits,
) -> Result<w::OutcomeSettingsChanged, w::Problem> {
    Ok(w::OutcomeSettingsChanged { revision: value.revision })
}

pub(in crate::codec) fn wire_outcome_moved_to_newest(
    value: &c::OutcomeMovedToNewest,
    _limits: &w::Limits,
) -> Result<w::OutcomeMovedToNewest, w::Problem> {
    Ok(w::OutcomeMovedToNewest { newest: value.newest, moved: value.moved, stayed: value.stayed })
}

fn wire_application_state(v: &c::ApplicationState, _limits: &w::Limits) -> Result<w::ApplicationState, w::Problem> {
    let follows = match v.follows {
        c::ApplicationFollows::Default => w::ApplicationFollows::Default,
        c::ApplicationFollows::Changed { revision } => w::ApplicationFollows::Changed(w::ApplicationRevision { revision }),
        c::ApplicationFollows::Removed { revision } => w::ApplicationFollows::Removed(w::ApplicationRevision { revision }),
    };
    Ok(w::ApplicationState { follows, size: v.size, charters: v.charters.clone() })
}
fn wire_sized_edit(v: &c::SizedEdit, _limits: &w::Limits) -> Result<w::SizedEdit, w::Problem> {
    Ok(match v {
        c::SizedEdit::Set { size } => {
            w::SizedEdit::Set(w::ApplicationSize { size: *size })
        }
        c::SizedEdit::Remove => w::SizedEdit::Remove,
        c::SizedEdit::Reset => w::SizedEdit::Reset,
    })
}

pub fn wire_revision_replaced(
    value: &c::RevisionReplaced,
    limits: &w::Limits,
) -> Result<w::RevisionReplaced, w::Problem> {
    Ok(match value {
        c::RevisionReplaced::Value(v) => w::RevisionReplaced::Value(wire_setting_set(v, limits)?),
        c::RevisionReplaced::TooLarge(v) => w::RevisionReplaced::TooLarge(wire_size(v, limits)?),
        c::RevisionReplaced::Default => w::RevisionReplaced::Default,
        c::RevisionReplaced::Removed => w::RevisionReplaced::Removed,
        c::RevisionReplaced::NotOffered => w::RevisionReplaced::NotOffered,
        c::RevisionReplaced::Application { size } => {
            w::RevisionReplaced::Application(w::ApplicationSize { size: *size })
        }
    })
}

fn wire_catalogue(value: &c::CatalogueEntry, limits: &w::Limits) -> Result<w::CatalogueEntry, w::Problem> {
    Ok(w::CatalogueEntry {
            number: value.number,
            model: wire_setting_model(&value.model, limits)?,
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
        })
}

pub(in crate::codec) fn wire_overlay(value: &c::Overlay, limits: &w::Limits) -> Result<w::Overlay, w::Problem> {
    let alternatives = match &value.alternatives {
        Some(items) => {
            let mut alternatives = List::with_capacity(items.len());
            for item in items {
                alternatives.push(wire_setting_model(item, limits)?).expect("same capacity");
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
                Some(item) => Some(wire_setting_model(item, limits)?),
                None => None,
            },
            alternatives,
            documents,
            parts,
            tools: value.tools,
            hosts: match &value.hosts {
                Some(item) => Some(wire_setting_hosts(item, limits)?),
                None => None,
            },
            instructions: bytes::copy_of(value.instructions.bytes()),
        })
}

fn wire_limits(values: &[c::Override], _limits: &w::Limits) -> Result<w::LimitsValue, w::Problem> {
    let mut overrides = List::with_capacity(u32::try_from(values.len()).expect("bounded overrides"));
    for value in values {
        overrides
            .push(w::Override { name: value.name.clone(), value: value.value, reason: value.reason.clone() })
            .expect("override count");
    }
    Ok(w::LimitsValue { overrides })
}

fn wire_worker(value: &c::WorkerSetting, _limits: &w::Limits) -> Result<w::WorkerSetting, w::Problem> {
    Ok(w::WorkerSetting {
            number: value.number,
            name: value.name.clone(),
            places: value.places,
            digest: value.digest,
            withdrawn: value.withdrawn,
        })
}
