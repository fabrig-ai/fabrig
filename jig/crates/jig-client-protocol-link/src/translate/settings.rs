//! Exhaustive settings and history translations (protocol/client/wire.md, 1).
use crate::Limits;
use crate::generated::client;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Reader, Writer, bytes, length::len_u64};
#[must_use]
pub fn wire_setting_number(value: &c::SettingNumber) -> w::SettingNumber {
    client::setting_number_to_record(value)
}
#[must_use]
pub fn restore_setting_number(value: &w::SettingNumber) -> c::SettingNumber {
    client::setting_number_to_domain(value.clone())
}
#[must_use]
pub fn wire_setting(value: &c::Setting) -> w::Setting {
    client::setting_to_record(value)
}
#[must_use]
pub fn restore_setting(value: &w::Setting) -> c::Setting {
    client::setting_to_domain(value.clone())
}
#[must_use]
pub fn wire_setting_model(value: &c::SettingModel) -> w::SettingModel {
    let mut efforts = List::with_capacity(value.efforts.len());
    for item in &value.efforts {
        efforts.push(bytes::copy_of(item.bytes())).expect("same capacity");
    }
    let mut speeds = List::with_capacity(value.speeds.len());
    for speed in &value.speeds {
        let mut also = List::with_capacity(speed.also.len());
        for name in &speed.also {
            also.push(bytes::copy_of(name.bytes())).expect("same capacity");
        }
        speeds
            .push(w::SettingSpeed {
                name: bytes::copy_of(speed.name.bytes()),
                also,
                input_price: speed.input_price,
                cached_price: speed.cached_price,
                cache_write_price: speed.cache_write_price,
                output_price: speed.output_price,
            })
            .expect("same capacity");
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
        speeds,
        speed: match &value.speed {
            Some(name) => Some(bytes::copy_of(name.bytes())),
            None => None,
        },
    }
}
pub fn restore_setting_model(value: &w::SettingModel) -> Result<c::SettingModel, w::Problem> {
    let mut efforts = List::with_capacity(value.efforts.len());
    for item in &value.efforts {
        efforts.push(text(item, 32)?).expect("same capacity");
    }
    let mut speeds = List::with_capacity(value.speeds.len());
    for speed in &value.speeds {
        let mut also = List::with_capacity(speed.also.len());
        for name in &speed.also {
            also.push(text(name, 32)?).expect("same capacity");
        }
        speeds
            .push(c::SettingSpeed {
                name: text(&speed.name, 32)?,
                also,
                input_price: speed.input_price,
                cached_price: speed.cached_price,
                cache_write_price: speed.cache_write_price,
                output_price: speed.output_price,
            })
            .expect("same capacity");
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
        speeds,
        speed: match &value.speed {
            Some(name) => Some(text(name, 32)?),
            None => None,
        },
    })
}
#[must_use]
pub fn wire_setting_hosts(value: &c::SettingHosts) -> w::SettingHosts {
    client::setting_hosts_to_record(value)
}
#[must_use]
pub fn restore_setting_hosts(value: &w::SettingHosts) -> c::SettingHosts {
    client::setting_hosts_to_domain(value.clone())
}
#[must_use]
pub fn wire_charter(value: &c::Charter) -> w::Charter {
    let mut alternatives = List::with_capacity(value.alternatives.len());
    for item in &value.alternatives {
        alternatives.push(wire_setting_model(item)).expect("same capacity");
    }
    let mut documents = List::with_capacity(value.documents.len());
    for item in &value.documents {
        documents.push(client::charter_document_to_record(item)).expect("same capacity");
    }
    let mut parts = List::with_capacity(value.parts.len());
    for item in &value.parts {
        parts
            .push(w::CharterPart { on: item.on, instructions: bytes::copy_of(item.instructions.bytes()) })
            .expect("same capacity");
    }
    w::Charter {
        name: value.name.clone(),
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
        documents.push(client::charter_document_to_domain(item.clone())).expect("same capacity");
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
        name: value.name.clone(),
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
        name: value.name.clone(),
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
        name: value.name.clone(),
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
        implies.push(client::kind_implication_to_record(item)).expect("same capacity");
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
    client::rules_to_domain(value.clone(), limits)
}
pub fn wire_setting_value(value: &c::SettingValue, limits: &w::Limits) -> Result<w::SettingValue, w::Problem> {
    Ok(match value {
        c::SettingValue::Overlay(item) => w::SettingValue::Overlay(Box::new(wire_overlay(item))),
        c::SettingValue::Provider(v) => w::SettingValue::Provider(Box::new(providers::encode(v, limits)?)),
        c::SettingValue::Model(v) => w::SettingValue::Model(Box::new(wire_catalogue(v))),
        c::SettingValue::Worker(v) => w::SettingValue::Worker(Box::new(client::worker_setting_to_record(v))),
        c::SettingValue::Application(v) => w::SettingValue::Application(w::ApplicationValue { bytes: v.clone() }),
        c::SettingValue::Charter(item) => w::SettingValue::Charter(Box::new(wire_charter(item))),
        c::SettingValue::Template(item) => w::SettingValue::Template(Box::new(wire_template(item, limits)?)),
        c::SettingValue::Policy(item) => w::SettingValue::Policy(Box::new(wire_setting_policy(item, limits)?)),
        c::SettingValue::Rules(item) => w::SettingValue::Rules(Box::new(wire_rules(item, limits)?)),
        c::SettingValue::Limits(item) => w::SettingValue::Limits(wire_limits(item)),
    })
}
pub fn restore_setting_value(value: &w::SettingValue, limits: &Limits) -> Result<c::SettingValue, w::Problem> {
    client::setting_value_to_domain(value.clone(), limits)
}
pub fn wire_setting_set(value: &c::SettingSet, limits: &w::Limits) -> Result<w::SettingSet, w::Problem> {
    Ok(w::SettingSet { value: wire_setting_value(&value.value, limits)? })
}
pub fn restore_setting_set(value: &w::SettingSet, limits: &Limits) -> Result<c::SettingSet, w::Problem> {
    client::setting_set_to_domain(value.clone(), limits)
}
pub fn wire_setting_edit(value: &c::SettingEdit, limits: &w::Limits) -> Result<w::SettingEdit, w::Problem> {
    Ok(match value {
        c::SettingEdit::Set(item) => w::SettingEdit::Set(wire_setting_set(item, limits)?),
        c::SettingEdit::Removed => w::SettingEdit::Removed,
        c::SettingEdit::Reset => w::SettingEdit::Reset,
    })
}
pub fn restore_setting_edit(value: &w::SettingEdit, limits: &Limits) -> Result<c::SettingEdit, w::Problem> {
    client::setting_edit_to_domain(value.clone(), limits)
}
pub fn wire_setting_item(value: &c::SettingItem, limits: &w::Limits) -> Result<w::SettingItem, w::Problem> {
    Ok(w::SettingItem { setting: wire_setting(&value.setting), edit: wire_setting_edit(&value.edit, limits)? })
}
pub fn restore_setting_item(value: &w::SettingItem, limits: &Limits) -> Result<c::SettingItem, w::Problem> {
    client::setting_item_to_domain(value.clone(), limits)
}
#[must_use]
pub fn wire_running(value: &c::Running) -> w::Running {
    client::running_to_record(value)
}
#[must_use]
pub fn restore_running(value: &w::Running) -> c::Running {
    client::running_to_domain(value.clone())
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
    client::outcome_settings_changed_to_domain(value.clone())
}
#[must_use]
pub fn restore_refusal_stale(value: &w::RefusalStale) -> c::RefusalStale {
    client::refusal_stale_to_domain(value.clone())
}
#[must_use]
pub fn restore_refusal_full(value: &w::RefusalFull) -> c::RefusalFull {
    client::refusal_full_to_domain(value.clone())
}
#[must_use]
pub fn restore_setting_fault(value: &w::SettingFault) -> c::SettingFault {
    client::setting_fault_to_domain(value.clone())
}
#[must_use]
pub fn restore_malformed_settings(value: &w::MalformedSettings) -> c::MalformedSettings {
    client::malformed_settings_to_domain(value.clone())
}
#[must_use]
pub fn wire_read_setting(value: &c::ReadSetting) -> w::ReadSetting {
    client::read_setting_to_record(value)
}
#[must_use]
pub fn wire_read_revision(value: &c::ReadRevision) -> w::ReadRevision {
    client::read_revision_to_record(value)
}
#[must_use]
pub fn wire_read_revisions(value: &c::ReadRevisions) -> w::ReadRevisions {
    client::read_revisions_to_record(value)
}
#[must_use]
pub fn restore_setting_follows(value: &w::SettingFollows) -> c::SettingFollows {
    client::setting_follows_to_domain(value.clone())
}
pub fn restore_setting_line(value: &w::SettingLine, limits: &Limits) -> Result<c::SettingLine, w::Problem> {
    client::setting_line_to_domain(value.clone(), limits)
}
#[must_use]
pub fn restore_revision_party(value: &w::RevisionParty) -> c::RevisionParty {
    client::revision_party_to_domain(value.clone())
}
#[must_use]
pub fn restore_revision_decider(value: &w::RevisionDecider) -> c::RevisionDecider {
    client::revision_decider_to_domain(value.clone())
}
#[must_use]
pub fn restore_revision_head(value: &w::RevisionHead) -> c::RevisionHead {
    client::revision_head_to_domain(value.clone())
}
#[must_use]
pub fn restore_held_revision(value: &w::HeldRevision) -> c::HeldRevision {
    client::held_revision_to_domain(value.clone())
}
#[must_use]
pub fn restore_settings_proposal(value: &w::SettingsProposal) -> c::SettingsProposal {
    client::settings_proposal_to_domain(value.clone())
}
pub fn restore_settings_document(
    value: &w::SettingsDocument,
    limits: &Limits,
) -> Result<c::SettingsDocument, w::Problem> {
    client::settings_document_to_domain(value.clone(), limits)
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
    client::size_to_domain(value.clone())
}
pub fn restore_revision_edit(value: &w::RevisionEdit, limits: &Limits) -> Result<c::RevisionEdit, w::Problem> {
    client::revision_edit_to_domain(value.clone(), limits)
}
pub fn restore_revision_item(value: &w::RevisionItem, limits: &Limits) -> Result<c::RevisionItem, w::Problem> {
    client::revision_item_to_domain(value.clone(), limits)
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
    client::revision_forgotten_to_domain(value.clone())
}
pub fn restore_revision_answer(value: &w::RevisionAnswer, limits: &Limits) -> Result<c::RevisionAnswer, w::Problem> {
    client::revision_answer_to_domain(value.clone(), limits)
}
#[must_use]
pub fn restore_revision_item_edit(value: &w::RevisionItemEdit) -> c::RevisionItemEdit {
    client::revision_item_edit_to_domain(value.clone())
}
#[must_use]
pub fn restore_revision_item_head(value: &w::RevisionItemHead) -> c::RevisionItemHead {
    client::revision_item_head_to_domain(value.clone())
}
#[must_use]
pub fn restore_revision_row(value: &w::RevisionRow) -> c::RevisionRow {
    client::revision_row_to_domain(value.clone())
}
#[must_use]
pub fn restore_revisions_page(value: &w::RevisionsPage) -> c::RevisionsPage {
    client::revisions_page_to_domain(value.clone())
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
        if rows.push(client::revision_item_to_domain(row, limits)?).is_err() {
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
    client::template_override_to_record(value)
}
fn restore_template_override(value: &w::TemplateOverride) -> c::TemplateOverride {
    client::template_override_to_domain(value.clone())
}
fn wire_template(value: &c::Template, limits: &w::Limits) -> Result<w::Template, w::Problem> {
    Ok(w::Template {
        name: value.name.clone(),
        resources: super::documents::me::wire_symbols(&value.resources),
        number: value.number,
        shown: wire_document_text(&value.shown),
        executor: client::template_executor_to_record(&value.executor),
        contract: wire_task_contract(&value.contract, limits)?,
        shallower: value.shallower,
        budget: client::template_budget_to_record(&value.budget),
        deadline: client::template_deadline_to_record(&value.deadline),
        tracked: value.tracked,
        flags: client::task_flags_to_record(&value.flags),
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
                let row = client::verdict_choice_to_record(choice);
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
    client::part_to_record(value)
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
    client::application_state_to_domain(v.clone())
}
pub(super) fn restore_sized_edit(v: &w::SizedEdit) -> c::SizedEdit {
    client::sized_edit_to_domain(v.clone())
}

pub fn restore_revision_replaced(
    value: &w::RevisionReplaced,
    limits: &Limits,
) -> Result<c::RevisionReplaced, w::Problem> {
    client::revision_replaced_to_domain(value.clone(), limits)
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
        source: client::model_source_to_record(&value.source),
    }
}
pub(super) fn restore_catalogue(value: &w::CatalogueEntry) -> Result<c::CatalogueEntry, w::Problem> {
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
        source: client::model_source_to_domain(value.source.clone()),
    })
}

#[must_use]
pub fn wire_change_model(value: &c::ChangeModel) -> w::ChangeModel {
    client::change_model_to_record(value)
}
pub fn restore_change_model(value: &w::ChangeModel, limits: &Limits) -> Result<c::ChangeModel, w::Problem> {
    client::change_model_to_domain(value.clone(), limits)
}

pub(crate) fn model_act_to_record(value: &c::ModelAct) -> w::ModelAct {
    match value {
        c::ModelAct::Picker { number, on } => w::ModelAct::Picker(w::ModelPicker { number: *number, on: *on }),
        c::ModelAct::Add { provider, name } => {
            w::ModelAct::Add(w::ModelAdd { provider: *provider, name: bytes::copy_of(name.bytes()) })
        }
    }
}

pub(crate) fn model_act_to_domain(value: w::ModelAct, limits: &Limits) -> Result<c::ModelAct, w::Problem> {
    Ok(match value {
        w::ModelAct::Picker(value) => c::ModelAct::Picker { number: value.number, on: value.on },
        w::ModelAct::Add(value) => {
            c::ModelAct::Add { provider: value.provider, name: text(&value.name, limits.receiving().model_add_name)? }
        }
    })
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
                documents.push(client::charter_document_to_record(item)).expect("same capacity");
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
                documents.push(client::charter_document_to_domain(item.clone())).expect("same capacity");
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
        overrides.push(client::override_to_record(value)).expect("override count");
    }
    w::LimitsValue { overrides }
}

pub(super) mod providers;
