//! Exhaustive settings and history translations (protocol/client/wire.md, 1).
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Reader, Wall, Writer, bytes, length::len_u64};
#[must_use]
pub fn wire_setting_number(value: &c::SettingNumber) -> w::SettingNumber {
    w::SettingNumber { number: value.number }
}
#[must_use]
pub fn restore_setting_number(value: &w::SettingNumber) -> c::SettingNumber {
    c::SettingNumber { number: value.number }
}
#[must_use]
pub fn wire_setting(value: &c::Setting) -> w::Setting {
    match value {
        c::Setting::Charter(item) => w::Setting::Charter(wire_setting_number(item)),
        c::Setting::Template(item) => w::Setting::Template(wire_setting_number(item)),
        c::Setting::Policy(item) => w::Setting::Policy(wire_setting_number(item)),
        c::Setting::Model(item) => w::Setting::Model(wire_setting_number(item)),
        c::Setting::Worker(item) => w::Setting::Worker(wire_setting_number(item)),
        c::Setting::Provider(item) => w::Setting::Provider(wire_setting_number(item)),
        c::Setting::Rules => w::Setting::Rules,
        c::Setting::Limits => w::Setting::Limits,
        c::Setting::ProjectPolicy { project } => {
            w::Setting::ProjectPolicy(w::ProjectPolicySetting { project: *project })
        }
        c::Setting::Application { kind, scope, number } => {
            w::Setting::Application(w::ApplicationSetting { kind: *kind, scope: *scope, number: *number })
        }
    }
}
#[must_use]
pub fn restore_setting(value: &w::Setting) -> c::Setting {
    match value {
        w::Setting::Charter(item) => c::Setting::Charter(restore_setting_number(item)),
        w::Setting::Template(item) => c::Setting::Template(restore_setting_number(item)),
        w::Setting::Policy(item) => c::Setting::Policy(restore_setting_number(item)),
        w::Setting::Model(item) => c::Setting::Model(restore_setting_number(item)),
        w::Setting::Worker(item) => c::Setting::Worker(restore_setting_number(item)),
        w::Setting::Provider(item) => c::Setting::Provider(restore_setting_number(item)),
        w::Setting::Rules => c::Setting::Rules,
        w::Setting::Limits => c::Setting::Limits,
        w::Setting::ProjectPolicy(v) => c::Setting::ProjectPolicy { project: v.project },
        w::Setting::Application(v) => c::Setting::Application { kind: v.kind, scope: v.scope, number: v.number },
    }
}
#[must_use]
pub fn wire_setting_model(value: &c::SettingModel) -> w::SettingModel {
    let mut efforts = List::with_capacity(value.efforts.len());
    for item in &value.efforts {
        efforts.push(bytes::copy_of(item.bytes())).expect("same capacity");
    }
    w::SettingModel {
        provider: value.provider,
        dialect: value.dialect,
        endpoint: value.endpoint,
        program: match &value.program {
            Some(item) => Some(bytes::copy_of(item.bytes())),
            None => None,
        },
        name: bytes::copy_of(value.name.bytes()),
        window: value.window,
        output: value.output,
        input_price: value.input_price,
        cached_price: value.cached_price,
        cache_write_price: match &value.cache_write_price {
            Some(item) => Some(*item),
            None => None,
        },
        output_price: value.output_price,
        price_unit: value.price_unit,
        efforts,
        effort: match &value.effort {
            Some(item) => Some(bytes::copy_of(item.bytes())),
            None => None,
        },
    }
}
pub fn restore_setting_model(value: &w::SettingModel) -> Result<c::SettingModel, w::Problem> {
    let mut efforts = List::with_capacity(value.efforts.len());
    for item in &value.efforts {
        efforts.push(text(item, 32)?).expect("same capacity");
    }
    Ok(c::SettingModel {
        provider: value.provider,
        dialect: value.dialect,
        endpoint: value.endpoint,
        program: match &value.program {
            Some(item) => Some(text(item, 32)?),
            None => None,
        },
        name: text(&value.name, 4096)?,
        window: value.window,
        output: value.output,
        input_price: value.input_price,
        cached_price: value.cached_price,
        cache_write_price: match &value.cache_write_price {
            Some(item) => Some(*item),
            None => None,
        },
        output_price: value.output_price,
        price_unit: value.price_unit,
        efforts,
        effort: match &value.effort {
            Some(item) => Some(text(item, 32)?),
            None => None,
        },
    })
}
#[must_use]
pub fn wire_setting_hosts(value: &c::SettingHosts) -> w::SettingHosts {
    match value {
        c::SettingHosts::Workers => w::SettingHosts::Workers,
        c::SettingHosts::Engine => w::SettingHosts::Engine,
        c::SettingHosts::Both => w::SettingHosts::Both,
    }
}
#[must_use]
pub fn restore_setting_hosts(value: &w::SettingHosts) -> c::SettingHosts {
    match value {
        w::SettingHosts::Workers => c::SettingHosts::Workers,
        w::SettingHosts::Engine => c::SettingHosts::Engine,
        w::SettingHosts::Both => c::SettingHosts::Both,
    }
}
#[must_use]
pub fn wire_charter(value: &c::Charter) -> w::Charter {
    let mut alternatives = List::with_capacity(value.alternatives.len());
    for item in &value.alternatives {
        alternatives.push(wire_setting_model(item)).expect("same capacity");
    }
    let mut documents = List::with_capacity(value.documents.len());
    for item in &value.documents {
        documents
            .push(w::CharterDocument {
                connector: item.connector,
                name: item.name.clone(),
                path: item.path.clone(),
                heading: item.heading.clone(),
            })
            .expect("same capacity");
    }
    let mut parts = List::with_capacity(value.parts.len());
    for item in &value.parts {
        parts
            .push(w::CharterPart { on: item.on, instructions: bytes::copy_of(item.instructions.bytes()) })
            .expect("same capacity");
    }
    w::Charter {
        parts,
        documents,
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
        alternatives,
        call_timeout: value.call_timeout,
        tools: value.tools,
        hosts: wire_setting_hosts(&value.hosts),
        spend: value.spend,
        wake: super::asks::wire_owner_wake_policy(&value.wake),
    }
}
pub fn restore_charter(value: &w::Charter) -> Result<c::Charter, w::Problem> {
    let mut alternatives = List::with_capacity(value.alternatives.len());
    for item in &value.alternatives {
        alternatives.push(restore_setting_model(item)?).expect("same capacity");
    }
    let mut documents = List::with_capacity(value.documents.len());
    for item in &value.documents {
        documents
            .push(c::CharterDocument {
                connector: item.connector,
                name: item.name.clone(),
                path: bytes::copy_of(&item.path),
                heading: item.heading.clone(),
            })
            .expect("same capacity");
    }
    let mut parts = List::with_capacity(value.parts.len());
    for item in &value.parts {
        parts
            .push(c::CharterPart {
                on: item.on,
                instructions: text(&item.instructions, w::CEILINGS.charter_part_instructions)?,
            })
            .expect("same capacity");
    }
    Ok(c::Charter {
        parts,
        rest: value.rest,
        documents,
        number: value.number,
        instructions: text(&value.instructions, 1_114_240)?,
        waiting: value.waiting,
        resume: value.resume,
        turns: value.turns,
        time: value.time,
        reserve_turns: value.reserve_turns,
        reserve_time: value.reserve_time,
        model: restore_setting_model(&value.model)?,
        alternatives,
        inspect: value.inspect,
        modify: value.modify,
        shell: value.shell,
        agents: value.agents,
        call_timeout: value.call_timeout,
        tools: value.tools,
        hosts: restore_setting_hosts(&value.hosts),
        spend: value.spend,
        wake: super::facts::wake_policy(&value.wake),
    })
}
pub fn wire_setting_policy(value: &c::SettingPolicy, limits: &w::Limits) -> Result<w::SettingPolicy, w::Problem> {
    let mut projections = List::with_capacity(value.projections.len());
    for item in &value.projections {
        projections.push(super::asks::wire_owner_grant(item)).expect("same capacity");
    }
    let mut roles = List::with_capacity(value.roles.len());
    for item in &value.roles {
        roles.push(super::asks::wire_policy_edit_role(item, limits)?).expect("same capacity");
    }
    let mut requirements = List::with_capacity(value.requirements.len());
    for item in &value.requirements {
        requirements.push(super::asks::wire_policy_requirement(item)).expect("same capacity");
    }
    let mut permissions = List::with_capacity(value.permissions.len());
    for item in &value.permissions {
        permissions.push(super::asks::wire_policy_permission(*item)).expect("same capacity");
    }
    let mut templates = List::with_capacity(value.templates.len());
    for item in &value.templates {
        templates.push(wire_template_override(item)).expect("same capacity");
    }
    Ok(w::SettingPolicy {
        escalation_role: match &value.escalation_role {
            Some(item) => Some(*item),
            None => None,
        },
        ceiling: super::asks::wire_owner_authority(&value.ceiling, limits)?,
        projections,
        period_spend: value.period_spend,
        roles,
        requirements,
        permissions,
        templates,
    })
}
pub fn restore_setting_policy(value: &w::SettingPolicy, limits: &Limits) -> Result<c::SettingPolicy, w::Problem> {
    let mut projections = List::with_capacity(value.projections.len());
    for item in &value.projections {
        projections.push(super::documents::grant_fields(item)).expect("same capacity");
    }
    let mut roles = List::with_capacity(value.roles.len());
    for item in &value.roles {
        roles.push(restore_policy_edit_role(item, limits)?).expect("same capacity");
    }
    let mut requirements = List::with_capacity(value.requirements.len());
    for item in &value.requirements {
        requirements.push(super::documents::policy_requirement_fields(item)).expect("same capacity");
    }
    let mut permissions = List::with_capacity(value.permissions.len());
    for item in &value.permissions {
        permissions.push(super::documents::policy_permission_fields(item)).expect("same capacity");
    }
    let mut templates = List::with_capacity(value.templates.len());
    for item in &value.templates {
        templates.push(restore_template_override(item)).expect("same capacity");
    }
    Ok(c::SettingPolicy {
        escalation_role: match &value.escalation_role {
            Some(item) => Some(*item),
            None => None,
        },
        ceiling: super::documents::authority_fields(&value.ceiling, limits)?,
        projections,
        period_spend: value.period_spend,
        roles,
        requirements,
        permissions,
        templates,
    })
}
pub fn wire_rules(value: &c::Rules, limits: &w::Limits) -> Result<w::Rules, w::Problem> {
    let mut implies = List::with_capacity(value.implies.len());
    for item in &value.implies {
        implies
            .push(w::KindImplication { connector: item.connector, kind: item.kind, implies: item.implies })
            .expect("same capacity");
    }
    let mut requirements = List::with_capacity(value.requirements.len());
    for item in &value.requirements {
        requirements.push(super::asks::wire_policy_requirement(item)).expect("same capacity");
    }
    Ok(w::Rules {
        ceiling: super::asks::wire_owner_authority(&value.ceiling, limits)?,
        period_spend: value.period_spend,
        minimum_run_spend: value.minimum_run_spend,
        maximum_run_spend: value.maximum_run_spend,
        implies,
        requirements,
    })
}
pub fn restore_rules(value: &w::Rules, limits: &Limits) -> Result<c::Rules, w::Problem> {
    let mut implies = List::with_capacity(value.implies.len());
    for item in &value.implies {
        implies
            .push(c::KindImplication { connector: item.connector, kind: item.kind, implies: item.implies })
            .expect("same capacity");
    }
    let mut requirements = List::with_capacity(value.requirements.len());
    for item in &value.requirements {
        requirements.push(super::documents::policy_requirement_fields(item)).expect("same capacity");
    }
    Ok(c::Rules {
        ceiling: super::documents::authority_fields(&value.ceiling, limits)?,
        period_spend: value.period_spend,
        minimum_run_spend: value.minimum_run_spend,
        maximum_run_spend: value.maximum_run_spend,
        implies,
        requirements,
    })
}
pub fn wire_setting_value(value: &c::SettingValue, limits: &w::Limits) -> Result<w::SettingValue, w::Problem> {
    Ok(match value {
        c::SettingValue::Overlay(item) => w::SettingValue::Overlay(Box::new(wire_overlay(item))),
        c::SettingValue::Provider(v) => w::SettingValue::Provider(Box::new(providers::encode(v, limits)?)),
        c::SettingValue::Model(v) => w::SettingValue::Model(Box::new(wire_catalogue(v))),
        c::SettingValue::Worker(v) => w::SettingValue::Worker(Box::new(wire_worker(v))),
        c::SettingValue::Application(v) => w::SettingValue::Application(w::ApplicationValue { bytes: v.clone() }),
        c::SettingValue::Charter(item) => w::SettingValue::Charter(Box::new(wire_charter(item))),
        c::SettingValue::Template(item) => w::SettingValue::Template(Box::new(wire_template(item, limits)?)),
        c::SettingValue::Policy(item) => w::SettingValue::Policy(Box::new(wire_setting_policy(item, limits)?)),
        c::SettingValue::Rules(item) => w::SettingValue::Rules(Box::new(wire_rules(item, limits)?)),
        c::SettingValue::Limits(item) => w::SettingValue::Limits(wire_limits(item)),
    })
}
pub fn restore_setting_value(value: &w::SettingValue, limits: &Limits) -> Result<c::SettingValue, w::Problem> {
    Ok(match value {
        w::SettingValue::Overlay(item) => c::SettingValue::Overlay(Box::new(restore_overlay(item)?)),
        w::SettingValue::Provider(v) => c::SettingValue::Provider(Box::new(providers::decode(v))),
        w::SettingValue::Model(v) => c::SettingValue::Model(Box::new(restore_catalogue(v)?)),
        w::SettingValue::Worker(v) => c::SettingValue::Worker(Box::new(c::WorkerSetting {
            number: v.number,
            name: bytes::copy_of(v.name.as_ref()),
            places: v.places,
            digest: v.digest,
            withdrawn: v.withdrawn,
        })),
        w::SettingValue::Application(v) => c::SettingValue::Application(bytes::copy_of(&v.bytes)),
        w::SettingValue::Charter(item) => c::SettingValue::Charter(Box::new(restore_charter(item)?)),
        w::SettingValue::Template(item) => {
            c::SettingValue::Template(Box::new(super::documents::me::template(item, limits)?))
        }
        w::SettingValue::Policy(item) => c::SettingValue::Policy(Box::new(restore_setting_policy(item, limits)?)),
        w::SettingValue::Rules(item) => c::SettingValue::Rules(Box::new(restore_rules(item, limits)?)),
        w::SettingValue::Limits(item) => c::SettingValue::Limits(restore_limits(item)),
    })
}
pub fn wire_setting_set(value: &c::SettingSet, limits: &w::Limits) -> Result<w::SettingSet, w::Problem> {
    Ok(w::SettingSet { value: wire_setting_value(&value.value, limits)? })
}
pub fn restore_setting_set(value: &w::SettingSet, limits: &Limits) -> Result<c::SettingSet, w::Problem> {
    Ok(c::SettingSet { value: restore_setting_value(&value.value, limits)? })
}
pub fn wire_setting_edit(value: &c::SettingEdit, limits: &w::Limits) -> Result<w::SettingEdit, w::Problem> {
    Ok(match value {
        c::SettingEdit::Set(item) => w::SettingEdit::Set(wire_setting_set(item, limits)?),
        c::SettingEdit::Removed => w::SettingEdit::Removed,
        c::SettingEdit::Reset => w::SettingEdit::Reset,
    })
}
pub fn restore_setting_edit(value: &w::SettingEdit, limits: &Limits) -> Result<c::SettingEdit, w::Problem> {
    Ok(match value {
        w::SettingEdit::Set(item) => c::SettingEdit::Set(restore_setting_set(item, limits)?),
        w::SettingEdit::Removed => c::SettingEdit::Removed,
        w::SettingEdit::Reset => c::SettingEdit::Reset,
    })
}
pub fn wire_setting_item(value: &c::SettingItem, limits: &w::Limits) -> Result<w::SettingItem, w::Problem> {
    Ok(w::SettingItem { setting: wire_setting(&value.setting), edit: wire_setting_edit(&value.edit, limits)? })
}
pub fn restore_setting_item(value: &w::SettingItem, limits: &Limits) -> Result<c::SettingItem, w::Problem> {
    Ok(c::SettingItem { setting: restore_setting(&value.setting), edit: restore_setting_edit(&value.edit, limits)? })
}
#[must_use]
pub fn wire_running(value: &c::Running) -> w::Running {
    match value {
        c::Running::Stays => w::Running::Stays,
        c::Running::Moves => w::Running::Moves,
    }
}
#[must_use]
pub fn restore_running(value: &w::Running) -> c::Running {
    match value {
        w::Running::Stays => c::Running::Stays,
        w::Running::Moves => c::Running::Moves,
    }
}
pub fn wire_change_settings(value: &c::ChangeSettings, limits: &w::Limits) -> Result<w::ChangeSettings, w::Problem> {
    Ok(w::ChangeSettings {
        against: value.against,
        items: encode_setting_item_pieces(&value.items, limits)?,
        running: wire_running(&value.running),
    })
}
pub fn restore_change_settings(value: &w::ChangeSettings, limits: &Limits) -> Result<c::ChangeSettings, w::Problem> {
    Ok(c::ChangeSettings {
        against: value.against,
        items: decode_setting_item_pieces(&value.items, limits)?,
        running: restore_running(&value.running),
    })
}
#[must_use]
pub fn restore_outcome_settings_changed(value: &w::OutcomeSettingsChanged) -> c::OutcomeSettingsChanged {
    c::OutcomeSettingsChanged { revision: value.revision }
}
#[must_use]
pub fn restore_refusal_stale(value: &w::RefusalStale) -> c::RefusalStale {
    c::RefusalStale { newest: value.newest }
}
#[must_use]
pub fn restore_refusal_full(value: &w::RefusalFull) -> c::RefusalFull {
    let mut tasks = List::with_capacity(value.tasks.len());
    for item in &value.tasks {
        tasks.push(*item).expect("same capacity");
    }
    c::RefusalFull { revision: value.revision, tasks, more: value.more }
}
#[must_use]
pub fn restore_setting_fault(value: &w::SettingFault) -> c::SettingFault {
    match value {
        w::SettingFault::Shape => c::SettingFault::Shape,
        w::SettingFault::Count => c::SettingFault::Count,
        w::SettingFault::Reference => c::SettingFault::Reference,
        w::SettingFault::Rules => c::SettingFault::Rules,
        w::SettingFault::Budget => c::SettingFault::Budget,
        w::SettingFault::Grace => c::SettingFault::Grace,
        w::SettingFault::Owner => c::SettingFault::Owner,
        w::SettingFault::Judged => c::SettingFault::Judged,
        w::SettingFault::Linked => c::SettingFault::Linked,
        w::SettingFault::Accounts => c::SettingFault::Accounts,
        w::SettingFault::Range => c::SettingFault::Range,
    }
}
#[must_use]
pub fn restore_malformed_settings(value: &w::MalformedSettings) -> c::MalformedSettings {
    c::MalformedSettings {
        setting: restore_setting(&value.setting),
        fault: restore_setting_fault(&value.fault),
        detail: match &value.detail {
            Some(value) => Some(restore_setting_detail(value)),
            None => None,
        },
    }
}
#[must_use]
pub fn wire_read_setting(value: &c::ReadSetting) -> w::ReadSetting {
    w::ReadSetting {
        setting: wire_setting(&value.setting),
        revision: value.revision,
        proposal: match &value.proposal {
            Some(proposal) => Some(super::asks::wire_proposal_key(proposal)),
            None => None,
        },
    }
}
#[must_use]
pub fn wire_read_revision(value: &c::ReadRevision) -> w::ReadRevision {
    w::ReadRevision { number: value.number }
}
#[must_use]
pub fn wire_read_revisions(value: &c::ReadRevisions) -> w::ReadRevisions {
    w::ReadRevisions {
        before: match &value.before {
            Some(item) => Some(*item),
            None => None,
        },
    }
}
#[must_use]
pub fn restore_setting_follows(value: &w::SettingFollows) -> c::SettingFollows {
    match value {
        w::SettingFollows::Overlay => c::SettingFollows::Overlay,
        w::SettingFollows::Default => c::SettingFollows::Default,
        w::SettingFollows::Change => c::SettingFollows::Change,
    }
}
pub fn restore_setting_line(value: &w::SettingLine, limits: &Limits) -> Result<c::SettingLine, w::Problem> {
    Ok(c::SettingLine {
        setting: restore_setting(&value.setting),
        name: match &value.name {
            Some(item) => Some(super::facts::document_text(item, limits)?),
            None => None,
        },
        follows: restore_setting_follows(&value.follows),
        last: value.last,
        application: match &value.application {
            Some(v) => Some(restore_application_state(v)),
            None => None,
        },
    })
}
#[must_use]
pub fn restore_revision_party(value: &w::RevisionParty) -> c::RevisionParty {
    c::RevisionParty { number: value.number }
}
#[must_use]
pub fn restore_revision_decider(value: &w::RevisionDecider) -> c::RevisionDecider {
    match value {
        w::RevisionDecider::FirstStart => c::RevisionDecider::FirstStart,
        w::RevisionDecider::Release => c::RevisionDecider::Release,
        w::RevisionDecider::List(item) => c::RevisionDecider::List { provider: item.provider },
        w::RevisionDecider::Party(item) => c::RevisionDecider::Party(restore_revision_party(item)),
    }
}
#[must_use]
pub fn restore_revision_head(value: &w::RevisionHead) -> c::RevisionHead {
    c::RevisionHead {
        number: value.number,
        against: value.against,
        at: Wall::from_nanos(value.at),
        by: restore_revision_decider(&value.by),
        proposal: match &value.proposal {
            Some(item) => Some(super::answers::proposal_key(item)),
            None => None,
        },
        moved: value.moved,
    }
}
#[must_use]
pub fn restore_held_revision(value: &w::HeldRevision) -> c::HeldRevision {
    c::HeldRevision { head: restore_revision_head(&value.head), holders: value.holders }
}
#[must_use]
pub fn restore_settings_proposal(value: &w::SettingsProposal) -> c::SettingsProposal {
    c::SettingsProposal { proposer: super::answers::proposer(&value.proposer), number: value.number }
}
pub fn restore_settings_document(
    value: &w::SettingsDocument,
    limits: &Limits,
) -> Result<c::SettingsDocument, w::Problem> {
    let mut settings = List::with_capacity(value.settings.len());
    for item in &value.settings {
        settings.push(restore_setting_line(item, limits)?).expect("same capacity");
    }
    let mut held = List::with_capacity(value.held.len());
    for item in &value.held {
        held.push(restore_held_revision(item)).expect("same capacity");
    }
    let mut pending = List::with_capacity(value.pending.len());
    for item in &value.pending {
        pending.push(restore_settings_proposal(item)).expect("same capacity");
    }
    Ok(c::SettingsDocument { newest: value.newest, settings, held, pending })
}
pub fn restore_setting_document(value: &w::SettingDocument, limits: &Limits) -> Result<c::SettingDocument, w::Problem> {
    Ok(c::SettingDocument {
        overlay: match &value.overlay {
            Some(item) => Some(Box::new(restore_overlay(item)?)),
            None => None,
        },
        setting: restore_setting(&value.setting),
        revision: value.revision,
        value: match &value.value {
            Some(v) => Some(restore_setting_value(v, limits)?),
            None => None,
        },
        application: match &value.application {
            Some(v) => Some(restore_application_state(v)),
            None => None,
        },
        proposed: match &value.proposed {
            Some(v) => Some(restore_sized_edit(v)),
            None => None,
        },
    })
}
#[must_use]
pub fn restore_size(value: &w::Size) -> c::Size {
    c::Size { bytes: value.bytes }
}
pub fn restore_revision_edit(value: &w::RevisionEdit, limits: &Limits) -> Result<c::RevisionEdit, w::Problem> {
    Ok(match value {
        w::RevisionEdit::Set(item) => c::RevisionEdit::Set(restore_setting_set(item, limits)?),
        w::RevisionEdit::TooLarge(item) => c::RevisionEdit::TooLarge(restore_size(item)),
        w::RevisionEdit::Application(v) => c::RevisionEdit::Application(restore_sized_edit(&v.edit)),
        w::RevisionEdit::Removed => c::RevisionEdit::Removed,
        w::RevisionEdit::Reset => c::RevisionEdit::Reset,
    })
}
pub fn restore_revision_item(value: &w::RevisionItem, limits: &Limits) -> Result<c::RevisionItem, w::Problem> {
    Ok(c::RevisionItem {
        setting: restore_setting(&value.setting),
        edit: restore_revision_edit(&value.edit, limits)?,
        replaced: restore_revision_replaced(&value.replaced, limits)?,
    })
}
pub fn restore_revision_document(
    value: &w::RevisionDocument,
    limits: &Limits,
) -> Result<c::RevisionDocument, w::Problem> {
    Ok(c::RevisionDocument {
        head: restore_revision_head(&value.head),
        items: decode_revision_item_pieces(&value.items, limits)?,
    })
}
#[must_use]
pub fn restore_revision_forgotten(value: &w::RevisionForgotten) -> c::RevisionForgotten {
    c::RevisionForgotten { oldest: value.oldest }
}
pub fn restore_revision_answer(value: &w::RevisionAnswer, limits: &Limits) -> Result<c::RevisionAnswer, w::Problem> {
    Ok(match value {
        w::RevisionAnswer::Revision(item) => c::RevisionAnswer::Revision(restore_revision_document(item, limits)?),
        w::RevisionAnswer::Forgotten(item) => c::RevisionAnswer::Forgotten(restore_revision_forgotten(item)),
        w::RevisionAnswer::Unknown => c::RevisionAnswer::Unknown,
    })
}
#[must_use]
pub fn restore_revision_item_edit(value: &w::RevisionItemEdit) -> c::RevisionItemEdit {
    match value {
        w::RevisionItemEdit::Set => c::RevisionItemEdit::Set,
        w::RevisionItemEdit::Removed => c::RevisionItemEdit::Removed,
        w::RevisionItemEdit::Reset => c::RevisionItemEdit::Reset,
    }
}
#[must_use]
pub fn restore_revision_item_head(value: &w::RevisionItemHead) -> c::RevisionItemHead {
    c::RevisionItemHead { setting: restore_setting(&value.setting), edit: restore_revision_item_edit(&value.edit) }
}
#[must_use]
pub fn restore_revision_row(value: &w::RevisionRow) -> c::RevisionRow {
    let mut items = List::with_capacity(value.items.len());
    for item in &value.items {
        items.push(restore_revision_item_head(item)).expect("same capacity");
    }
    c::RevisionRow { head: restore_revision_head(&value.head), items, more: value.more }
}
#[must_use]
pub fn restore_revisions_page(value: &w::RevisionsPage) -> c::RevisionsPage {
    let mut rows = List::with_capacity(value.rows.len());
    for item in &value.rows {
        rows.push(restore_revision_row(item)).expect("same capacity");
    }
    c::RevisionsPage {
        rows,
        next: match &value.next {
            Some(item) => Some(*item),
            None => None,
        },
        forgotten: value.forgotten,
    }
}
fn encode_setting_item_pieces(values: &List<c::SettingItem>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    if values.len() > 1024 {
        return Err(super::too_large());
    }
    let mut rows = List::with_capacity(values.len());
    let mut size = 0_u32;
    for value in values {
        let row = wire_setting_item(value, limits)?;
        size = size
            .checked_add(row.measure(limits)?)
            .ok_or(super::too_large())?
            .checked_add(4)
            .ok_or(super::too_large())?;
        if size > limits.change_settings_items {
            return Err(super::too_large());
        }
        rows.push(row).expect("same capacity");
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(size));
    for row in rows.into_boxed() {
        writer.put(&row.measure(limits)?.to_be_bytes()).expect("measured");
        row.write(limits, &mut writer).expect("measured");
    }
    Ok(writer.finish())
}
fn decode_setting_item_pieces(values: &[u8], limits: &Limits) -> Result<List<c::SettingItem>, w::Problem> {
    let count = item_count(values, limits)?;
    let mut reader = Reader::new(values);
    let mut rows = List::with_capacity(count);
    let mut sum = 0_u64;
    for _ in 0_u32..=1024 {
        let Some(piece) = super::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let mut input = Reader::new(&piece);
        let row = w::SettingItem::decode(limits.receiving(), &mut input)?;
        if !input.is_empty() {
            return Err(super::too_large());
        }
        if rows.push(restore_setting_item(&row, limits)?).is_err() {
            return Err(super::too_large());
        }
    }
    Err(super::too_large())
}
fn decode_revision_item_pieces(values: &[u8], limits: &Limits) -> Result<List<c::RevisionItem>, w::Problem> {
    let count = item_count(values, limits)?;
    let mut reader = Reader::new(values);
    let mut rows = List::with_capacity(count);
    let mut sum = 0_u64;
    for _ in 0_u32..=1024 {
        let Some(piece) = super::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let mut input = Reader::new(&piece);
        let row = w::RevisionItem::decode(limits.receiving(), &mut input)?;
        if !input.is_empty() {
            return Err(super::too_large());
        }
        if rows.push(restore_revision_item(&row, limits)?).is_err() {
            return Err(super::too_large());
        }
    }
    Err(super::too_large())
}

fn restore_policy_edit_role(value: &w::PolicyEditRole, limits: &Limits) -> Result<c::PolicyEditRole, w::Problem> {
    Ok(c::PolicyEditRole {
        number: value.number,
        authority: super::documents::authority_fields(&value.authority, limits)?,
        period_spend: value.period_spend,
        requests: value.requests,
        decides: value.decides,
    })
}
pub(crate) fn wire_document_text(value: &c::DocumentText) -> w::DocumentText {
    w::DocumentText { bytes: bytes::copy_of(value.bytes()), cut: value.cut }
}
fn wire_template_override(value: &c::TemplateOverride) -> w::TemplateOverride {
    w::TemplateOverride {
        template: value.template,
        executor: match value.executor {
            Some(value) => Some(super::operations::executor(value)),
            None => None,
        },
        budget: match value.budget {
            Some(value) => Some(super::operations::budget(value)),
            None => None,
        },
        deadline: match value.deadline {
            Some(value) => Some(super::operations::deadline(value)),
            None => None,
        },
        priority: value.priority,
    }
}
fn restore_template_override(value: &w::TemplateOverride) -> c::TemplateOverride {
    c::TemplateOverride {
        template: value.template,
        executor: match &value.executor {
            Some(v) => Some(super::operations::restore_executor(v)),
            None => None,
        },
        budget: match &value.budget {
            Some(v) => Some(super::operations::restore_budget(v)),
            None => None,
        },
        deadline: match &value.deadline {
            Some(v) => Some(super::operations::restore_deadline(v)),
            None => None,
        },
        priority: value.priority,
    }
}
fn wire_template(value: &c::Template, limits: &w::Limits) -> Result<w::Template, w::Problem> {
    Ok(w::Template {
        resources: super::documents::me::wire_symbols(&value.resources),
        number: value.number,
        name: wire_document_text(&value.name),
        executor: super::operations::executor(value.executor),
        contract: wire_task_contract(&value.contract, limits)?,
        shallower: value.shallower,
        budget: super::operations::budget(value.budget),
        deadline: super::operations::deadline(value.deadline),
        tracked: value.tracked,
        flags: w::TaskFlags {
            ended_by: match value.flags.ended_by {
                c::EndedBy::Result => w::EndedBy::Result,
                c::EndedBy::Requester => w::EndedBy::Requester,
                c::EndedBy::Standing => w::EndedBy::Standing,
                c::EndedBy::Closer(closer) => {
                    w::EndedBy::Closer(w::Closer { connector: closer.connector, code: closer.code })
                }
            },
            conversation: value.flags.conversation,
            listed: value.flags.listed,
            retention: match value.flags.retention {
                c::Retention::Kept => w::Retention::Kept,
                c::Retention::ForgottenOnceRead => w::Retention::ForgottenOnceRead,
            },
        },
        label: value.label,
        grace: value.grace,
        llm: super::llm::wire_choice(value.llm.as_ref()),
    })
}
fn wire_task_contract(
    value: &c::TaskContractDocument,
    limits: &w::Limits,
) -> Result<w::TaskContractDocument, w::Problem> {
    Ok(match value {
        c::TaskContractDocument::Job { bytes } => {
            w::TaskContractDocument::Job(w::TaskContractDocumentJob { bytes: *bytes })
        }
        c::TaskContractDocument::Approval { state, part } => {
            w::TaskContractDocument::Approval(w::TaskContractDocumentApproval {
                state: bytes::copy_of(state),
                part: wire_owner_part(part),
            })
        }
        c::TaskContractDocument::Report { words } => {
            w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: *words })
        }
        c::TaskContractDocument::Change { connector, kind, words } => {
            w::TaskContractDocument::Change(w::TaskContractDocumentChange {
                connector: *connector,
                kind: *kind,
                words: *words,
            })
        }
        c::TaskContractDocument::Verdict { choices, cut } => {
            let mut rows = List::with_capacity(choices.len());
            let mut size = 0_u32;
            for choice in choices {
                let mut fields = List::with_capacity(choice.fields.len());
                for field in &choice.fields {
                    fields
                        .push(w::FieldRule { name: bytes::copy_of(&field.name), max: field.max })
                        .expect("same capacity");
                }
                let row =
                    w::VerdictChoice { code: choice.code, words: choice.words, followups: choice.followups, fields };
                size = size
                    .checked_add(row.measure(limits)?)
                    .ok_or(super::too_large())?
                    .checked_add(4)
                    .ok_or(super::too_large())?;
                rows.push(row).expect("same capacity");
            }
            if size > limits.task_contract_document_verdict_choices {
                return Err(super::too_large());
            }
            let mut writer = Writer::new(skein_lib::length::usize_of(size));
            for row in &rows {
                writer.put(&row.measure(limits)?.to_be_bytes()).expect("measured");
                row.write(limits, &mut writer).expect("measured");
            }
            w::TaskContractDocument::Verdict(w::TaskContractDocumentVerdict { choices: writer.finish(), cut: *cut })
        }
    })
}
fn wire_owner_part(value: &c::Part) -> w::Part {
    let kind = match value.kind {
        c::PartKind::Resource => w::PartKind::Resource,
        c::PartKind::Pool => w::PartKind::Pool,
        c::PartKind::Result => w::PartKind::Result,
        c::PartKind::Procedure => w::PartKind::Procedure,
        c::PartKind::Effect => w::PartKind::Effect,
        c::PartKind::Outbox => w::PartKind::Outbox,
        c::PartKind::Hold => w::PartKind::Hold,
    };
    let key = match &value.key {
        c::PartKey::Number(value) => w::PartKey::Number(w::PartKeyNumber { value: *value }),
        c::PartKey::Path(values) => {
            let mut rows = List::with_capacity(values.len());
            for value in values {
                rows.push(bytes::copy_of(value)).expect("same capacity");
            }
            w::PartKey::Path(w::PartKeyPath { value: rows })
        }
    };
    w::Part { connector: value.connector, kind, key }
}

// Count the bounded pieces before reserving the actual rows; an empty change allocates no rows.
fn item_count(values: &[u8], limits: &Limits) -> Result<u32, w::Problem> {
    if len_u64(values.len()) > limits.client().document_bytes {
        return Err(super::too_large());
    }
    let mut reader = Reader::new(values);
    let mut sum = 0_u64;
    for count in 0_u32..=1024 {
        let piece = super::next_piece(&mut reader, &mut sum, limits)?;
        if piece.is_none() {
            return Ok(count);
        }
    }
    Err(super::too_large())
}

fn text(value: &[u8], bound: u32) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), bound) {
        Ok(value) => Ok(value),
        Err(_) => Err(super::too_large()),
    }
}

pub(super) fn restore_application_state(v: &w::ApplicationState) -> c::ApplicationState {
    c::ApplicationState {
        follows: match &v.follows {
            w::ApplicationFollows::Default => c::ApplicationFollows::Default,
            w::ApplicationFollows::Changed(v) => c::ApplicationFollows::Changed { revision: v.revision },
            w::ApplicationFollows::Removed(v) => c::ApplicationFollows::Removed { revision: v.revision },
        },
        size: v.size,
        charters: v.charters.clone(),
    }
}
pub(super) fn restore_sized_edit(v: &w::SizedEdit) -> c::SizedEdit {
    match v {
        w::SizedEdit::Set(v) => c::SizedEdit::Set { size: v.size },
        w::SizedEdit::Remove => c::SizedEdit::Remove,
        w::SizedEdit::Reset => c::SizedEdit::Reset,
    }
}

pub fn restore_revision_replaced(
    value: &w::RevisionReplaced,
    limits: &Limits,
) -> Result<c::RevisionReplaced, w::Problem> {
    Ok(match value {
        w::RevisionReplaced::Value(v) => c::RevisionReplaced::Value(restore_setting_set(v, limits)?),
        w::RevisionReplaced::TooLarge(v) => c::RevisionReplaced::TooLarge(restore_size(v)),
        w::RevisionReplaced::Default => c::RevisionReplaced::Default,
        w::RevisionReplaced::Removed => c::RevisionReplaced::Removed,
        w::RevisionReplaced::NotOffered => c::RevisionReplaced::NotOffered,
        w::RevisionReplaced::Application(v) => c::RevisionReplaced::Application { size: v.size },
    })
}

/// Translate the document refusal facts without checking the settings again.
fn restore_setting_detail(value: &w::SettingDetail) -> c::SettingDetail {
    c::SettingDetail {
        member: match &value.member {
            w::SettingMember::Documents => c::SettingMember::Documents,
            w::SettingMember::NameSegments => c::SettingMember::NameSegments,
            w::SettingMember::NameSegmentBytes => c::SettingMember::NameSegmentBytes,
            w::SettingMember::Path => c::SettingMember::Path,
            w::SettingMember::Heading => c::SettingMember::Heading,
            w::SettingMember::Connector => c::SettingMember::Connector,
            w::SettingMember::StoredPolicy => c::SettingMember::StoredPolicy,
        },
        rule: match &value.rule {
            w::SettingRule::Maximum => c::SettingRule::Maximum,
            w::SettingRule::Minimum => c::SettingRule::Minimum,
            w::SettingRule::Registered => c::SettingRule::Registered,
        },
        document: value.document,
        segment: value.segment,
        supplied: value.supplied,
        bound: value.bound,
    }
}

fn wire_catalogue(value: &c::CatalogueEntry) -> w::CatalogueEntry {
    w::CatalogueEntry {
        number: value.number,
        model: wire_setting_model(&value.model),
        shown: match &value.shown {
            Some(value) => Some(bytes::copy_of(value.bytes())),
            None => None,
        },
        description: match &value.description {
            Some(value) => Some(bytes::copy_of(value.bytes())),
            None => None,
        },
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
fn restore_catalogue(value: &w::CatalogueEntry) -> Result<c::CatalogueEntry, w::Problem> {
    Ok(c::CatalogueEntry {
        number: value.number,
        model: restore_setting_model(&value.model)?,
        shown: match &value.shown {
            Some(value) => Some(text(value, 4096)?),
            None => None,
        },
        description: match &value.description {
            Some(value) => Some(text(value, 512)?),
            None => None,
        },
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        source: match &value.source {
            w::ModelSource::Stock => c::ModelSource::Stock,
            w::ModelSource::List => c::ModelSource::List,
            w::ModelSource::Person => c::ModelSource::Person,
        },
    })
}

#[must_use]
pub fn wire_change_model(value: &c::ChangeModel) -> w::ChangeModel {
    let act = match &value.act {
        c::ModelAct::Picker { number, on } => w::ModelAct::Picker(w::ModelPicker { number: *number, on: *on }),
        c::ModelAct::Add { provider, name } => {
            w::ModelAct::Add(w::ModelAdd { provider: *provider, name: bytes::copy_of(name.bytes()) })
        }
    };
    w::ChangeModel { act }
}
pub fn restore_change_model(value: &w::ChangeModel, limits: &Limits) -> Result<c::ChangeModel, w::Problem> {
    let act = match &value.act {
        w::ModelAct::Picker(value) => c::ModelAct::Picker { number: value.number, on: value.on },
        w::ModelAct::Add(value) => {
            c::ModelAct::Add { provider: value.provider, name: text(&value.name, limits.receiving().model_add_name)? }
        }
    };
    Ok(c::ChangeModel { act })
}

#[must_use]
pub fn wire_overlay(value: &c::Overlay) -> w::Overlay {
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
        model: match &value.model {
            Some(item) => Some(wire_setting_model(item)),
            None => None,
        },
        alternatives,
        documents,
        parts,
        tools: value.tools,
        hosts: match &value.hosts {
            Some(item) => Some(wire_setting_hosts(item)),
            None => None,
        },
        instructions: bytes::copy_of(value.instructions.bytes()),
    }
}

pub fn restore_overlay(value: &w::Overlay) -> Result<c::Overlay, w::Problem> {
    let alternatives = match &value.alternatives {
        Some(items) => {
            let mut alternatives = List::with_capacity(items.len());
            for item in items {
                alternatives.push(restore_setting_model(item)?).expect("same capacity");
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
                    .push(c::CharterDocument {
                        connector: item.connector,
                        name: item.name.clone(),
                        path: bytes::copy_of(&item.path),
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
            .push(c::CharterPart {
                on: item.on,
                instructions: text(&item.instructions, w::CEILINGS.charter_part_instructions)?,
            })
            .expect("same capacity");
    }
    Ok(c::Overlay {
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
            Some(item) => Some(restore_setting_model(item)?),
            None => None,
        },
        alternatives,
        documents,
        parts,
        tools: value.tools,
        hosts: match &value.hosts {
            Some(item) => Some(restore_setting_hosts(item)),
            None => None,
        },
        instructions: text(&value.instructions, w::CEILINGS.overlay_instructions)?,
    })
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

fn restore_limits(value: &w::LimitsValue) -> Box<[c::Override]> {
    let mut overrides = List::with_capacity(value.overrides.len());
    for item in &value.overrides {
        overrides
            .push(c::Override { name: item.name.clone(), value: item.value, reason: item.reason.clone() })
            .expect("override count");
    }
    overrides.into_boxed()
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

mod providers;
