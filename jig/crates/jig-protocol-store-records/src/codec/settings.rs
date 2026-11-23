//! Typed settings records (protocol/store/keys.md, section 1.4 and protocol/store/records.md).
#![expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "typed encoders uniformly borrow the domain values they translate"
)]
use crate::generated::settings as maps;
use crate::settings_v3::{self as wire, Record};
mod providers;
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core_vocabulary::{self as core, settings};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
pub(crate) use providers::{provider_address_to_domain, provider_address_to_record};
use settings::application;
use skein_lib::{List, Reader};
#[cfg(test)]
pub(in crate::codec) const EMPTY: wire::Limits = wire::Limits {
    provider_text_bytes: 0,
    provider_list_keep: 0,
    provider_list_drop: 0,
    provider_tls_server_name: 0,
    provider_tls_trust_some: 0,
    provider_registration_issuer: 0,
    provider_registration_client: 0,
    provider_registration_scopes: 0,
    provider_registration_name_hint_some: 0,
    provider_registration_authorization: 0,
    provider_registration_token: 0,
    provider_registration_revocation_some: 0,
    provider_registration_keys: 0,
    provider_registration_must_grant: 0,
    provider_registration_resource_some: 0,
    provider_registration_redirect_path: 0,
    provider_name: 0,
    provider_endpoint_some: 0,
    provider_direct_some: 0,
    provider_manage_usage_some: 0,
    provider_programs: 0,
    provider_host_name: 0,
    provider_header_name: 0,
    provider_header_value: 0,
    provider_endpoint_name: 0,
    provider_endpoint_provider: 0,
    provider_endpoint_program_some: 0,
    provider_endpoint_headers: 0,
    provider_setting_endpoints: 0,

    worker_setting_name: 0,
    text_bytes: 0,
    override_name: 0,
    override_reason: 0,
    limits_value_overrides: 0,
    model_name_name: 0,
    model_name_efforts: 0,
    model_name_efforts_item: 0,
    catalogue_model_shown_some: 0,
    catalogue_model_description_some: 0,
    model_program_some: 0,
    model_name: 0,
    model_efforts: 0,
    model_effort_some: 0,
    overlay_instructions: 0,
    overlay_parts: 0,
    overlay_documents_some: 0,
    overlay_alternatives_some: 0,
    run_policy_instructions: 0,
    run_policy_alternatives: 0,
    segment_bytes: 0,
    document_name: 0,
    document_path: 0,
    document_heading_some: 0,
    charter_documents: 0,
    charter_parts: 0,
    charter_part_instructions: 0,
    last_exact_bytes: 0,
    last_open_bytes: 0,
    pattern_segments: 0,
    delegation_kinds: 0,
    authority_grants: 0,
    authority_note_resources: 0,
    rules_implies: 0,
    rules_requirements: 0,
    policy_projections: 0,
    policy_roles: 0,
    policy_requirements: 0,
    configured_policy_permissions: 0,
    configured_policy_templates: 0,
    field_rule_name: 0,
    verdict_fields: 0,
    part_path_segments: 0,
    approval_state: 0,
    verdict_choices_choices: 0,
    llm_choice_subagents: 0,
    llm_later_subagents: 0,
    template_name: 0,
    template_resources: 0,
    application_state_charters: 0,
    default_models: 0,
    changed_models: 0,
    revision_models: 0,
    revision_items: 0,
    revision_applications: 0,
    proposal_models: 0,
    proposal_items: 0,
    proposal_applications: 0,
};
// Scalar pairs for this family's maps (skein/docs/design/data/codec/maps.md, 5).
pub(crate) fn run_tools_to_record(value: &tasks::Tools) -> u64 {
    value.0
}
pub(crate) fn run_tools_to_domain(value: u64) -> tasks::Tools {
    tasks::Tools(value)
}
pub(crate) fn authority_tools_to_record(value: &authority::Tools) -> u64 {
    value.0
}
pub(crate) fn authority_tools_to_domain(value: u64) -> authority::Tools {
    authority::Tools(value)
}
pub(crate) fn note_scopes_to_record(value: &authority::Scopes) -> u8 {
    value.0
}
pub(crate) fn note_scopes_to_domain(value: u8) -> authority::Scopes {
    authority::Scopes(value)
}
pub(crate) fn role_requests_to_record(value: &authority::Requests) -> u16 {
    value.0
}
pub(crate) fn role_requests_to_domain(value: u16) -> authority::Requests {
    authority::Requests(value)
}
pub(crate) fn role_proposals_to_record(value: &authority::Proposals) -> u8 {
    value.0
}
pub(crate) fn role_proposals_to_domain(value: u8) -> authority::Proposals {
    authority::Proposals(value)
}
pub(crate) fn charter_reasons_to_record(value: &tasks::Reasons) -> u32 {
    value.0
}
pub(crate) fn charter_reasons_to_domain(value: u32) -> Result<tasks::Reasons, skein_codec::Reason> {
    let reasons = tasks::Reasons(value);
    if !reasons.is_known() || reasons.is_empty() {
        return Err(skein_codec::Reason::Bound);
    }
    Ok(reasons)
}

// Hold paths share Segment, but document names follow the authority's byte bound.
pub(crate) fn document_to_record(
    value: &core::Document,
    limits: &wire::Limits,
) -> Result<wire::Document, wire::Problem> {
    let name_limits = wire::Limits { segment_bytes: limits.last_exact_bytes, ..*limits };
    let mut name = List::with_capacity(limits.document_name);
    for segment in &value.name.segments {
        let segment = wire::Segment { bytes: segment.clone() };
        segment.measure(&name_limits)?;
        if name.push(segment).is_err() {
            return Err(bound());
        }
    }
    Ok(wire::Document { connector: value.connector, name, path: value.path.clone(), heading: value.heading.clone() })
}
pub(crate) fn document_to_domain(
    value: wire::Document,
    limits: &wire::Limits,
) -> Result<core::Document, wire::Problem> {
    let wire::Document { connector, name, path, heading } = value;
    let mut segments = List::with_capacity(name.len());
    for segment in name.into_boxed() {
        let wire::Segment { bytes } = segment;
        if bytes.len() > skein_lib::length::usize_of(limits.last_exact_bytes) {
            return Err(bound());
        }
        segments.push(bytes).expect("decoded document name bound");
    }
    Ok(core::Document { connector, name: authority::Name { segments: segments.into_boxed() }, path, heading })
}

fn limits_value_to_record(values: &[settings::Override]) -> wire::LimitsValue {
    let mut overrides = List::with_capacity(u32::try_from(values.len()).expect("bounded overrides"));
    for value in values {
        overrides.push(maps::override_to_record(value)).expect("override count");
    }
    wire::LimitsValue { overrides }
}
fn limits_value_to_domain(value: wire::LimitsValue) -> Box<[settings::Override]> {
    let mut overrides = List::with_capacity(value.overrides.len());
    for item in value.overrides.into_boxed() {
        overrides.push(maps::override_to_domain(item)).expect("override count");
    }
    overrides.into_boxed()
}

fn bound() -> wire::Problem {
    wire::Problem { path: wire::paths::DEFAULT_VALUE, reason: skein_codec::Reason::Bound }
}
pub(crate) fn contract_to_record(
    value: &tasks::Contract,
    limits: &wire::Limits,
) -> Result<wire::Contract, wire::Problem> {
    Ok(match value {
        tasks::Contract::Job { .. } => return Err(bound()),
        tasks::Contract::Approval { state, part } => {
            wire::Contract::Approval(wire::Approval { state: (*(state)).clone(), part: maps::part_to_record(part) })
        }
        tasks::Contract::Report { words } => wire::Contract::Report(wire::Report { words: *words }),
        tasks::Contract::Verdict { choices } => {
            let mut list_3 = List::with_capacity(limits.verdict_choices_choices);
            for entry in choices {
                let element = verdict_to_record(entry, limits)?;
                if list_3.push(element).is_err() {
                    return Err(bound());
                }
            }
            wire::Contract::Verdict(wire::VerdictChoices { choices: list_3 })
        }
        tasks::Contract::Change { connector, kind, words, .. } => {
            wire::Contract::Change(wire::Change { connector: *connector, kind: *kind, words: *words })
        }
    })
}
pub(crate) fn contract_to_domain(
    value: wire::Contract,
    limits: &wire::Limits,
) -> Result<tasks::Contract, wire::Problem> {
    Ok(match value {
        wire::Contract::Approval(entry) => {
            let wire::Approval { state, part } = entry;
            tasks::Contract::Approval { state, part: maps::part_to_domain(part) }
        }
        wire::Contract::Report(entry) => {
            let wire::Report { words } = entry;
            tasks::Contract::Report { words }
        }
        wire::Contract::Verdict(entry) => {
            let wire::VerdictChoices { choices } = entry;
            let mut list_4 = List::with_capacity((choices).len());
            for entry in (choices).into_boxed() {
                let element = verdict_to_domain(entry, limits)?;
                list_4.push(element).expect("decoded list bound");
            }
            tasks::Contract::Verdict { choices: list_4.into_boxed() }
        }
        wire::Contract::Change(entry) => {
            let wire::Change { connector, kind, words } = entry;
            tasks::Contract::Change { connector, kind, words, choices: Box::new([]) }
        }
    })
}
pub(crate) fn value_to_record(value: &settings::Value, limits: &wire::Limits) -> Result<wire::Value, wire::Problem> {
    Ok(match value {
        settings::Value::Charter(entry) => wire::Value::Charter(Box::new(maps::charter_to_record(entry, limits)?)),
        settings::Value::Template(entry) => wire::Value::Template(Box::new(maps::template_to_record(entry, limits)?)),
        settings::Value::Policy(entry) => wire::Value::Policy(Box::new(maps::configured_policy_to_record(entry)?)),
        settings::Value::Rules(entry) => wire::Value::Rules(Box::new(rules_to_record(entry, limits)?)),
        settings::Value::Limits(entry) => wire::Value::Limits(limits_value_to_record(entry)),
        settings::Value::Worker(value) => wire::Value::Worker(Box::new(maps::worker_setting_to_record(value))),
        settings::Value::Provider(value) => wire::Value::Provider(Box::new(maps::provider_setting_to_record(value)?)),
        settings::Value::Model(value) => wire::Value::Model(Box::new(maps::catalogue_model_to_record(value)?)),
        settings::Value::Overlay(item) => wire::Value::Overlay(Box::new(maps::overlay_to_record(item, limits)?)),
        settings::Value::Application(_) => return Err(bound()),
    })
}
pub(crate) fn value_to_domain(value: wire::Value, limits: &wire::Limits) -> Result<settings::Value, wire::Problem> {
    Ok(match value {
        wire::Value::Overlay(item) => settings::Value::Overlay(Box::new(maps::overlay_to_domain(*item, limits)?)),
        wire::Value::Charter(entry) => settings::Value::Charter(Box::new(maps::charter_to_domain(*entry, limits)?)),
        wire::Value::Template(entry) => settings::Value::Template(Box::new(maps::template_to_domain(*entry, limits)?)),
        wire::Value::Policy(entry) => settings::Value::Policy(Box::new(maps::configured_policy_to_domain(*entry))),
        wire::Value::Rules(entry) => settings::Value::Rules(Box::new(rules_to_domain(*entry, limits)?)),
        wire::Value::Limits(entry) => settings::Value::Limits(limits_value_to_domain(entry)),
        wire::Value::Worker(value) => settings::Value::Worker(Box::new(maps::worker_setting_to_domain(*value))),
        wire::Value::Provider(value) => settings::Value::Provider(Box::new(maps::provider_setting_to_domain(*value))),
        wire::Value::Model(value) => settings::Value::Model(Box::new(maps::catalogue_model_to_domain(*value))),
    })
}
pub(crate) fn models_to_record(value: &authority::Models) -> wire::Models {
    let field_bits = value.bits();
    let field_all = value.is_all();
    wire::Models { bits: field_bits, all: field_all }
}
pub(crate) fn models_to_domain(value: wire::Models) -> authority::Models {
    let wire::Models { bits, all } = value;
    let field_bits = bits;
    let field_all = all;
    authority::Models::from_parts(field_bits, field_all)
}
pub(crate) fn rules_to_record(value: &authority::Rules, limits: &wire::Limits) -> Result<wire::Rules, wire::Problem> {
    let field_ceiling = maps::authority_to_record(&value.ceiling)?;
    let field_period_spend = value.period_spend;
    let field_minimum_run_spend = value.minimum_run_spend;
    let field_maximum_run_spend = value.maximum_run_spend;
    let mut list_17 = List::with_capacity(limits.rules_implies);
    for entry in value.implies.entries() {
        let element = maps::implication_to_record(entry);
        if list_17.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_implies = list_17;
    let mut list_18 = List::with_capacity(limits.rules_requirements);
    for entry in &value.requirements {
        let element = maps::requirement_to_record(entry)?;
        if list_18.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_requirements = list_18;
    Ok(wire::Rules {
        ceiling: field_ceiling,
        period_spend: field_period_spend,
        minimum_run_spend: field_minimum_run_spend,
        maximum_run_spend: field_maximum_run_spend,
        implies: field_implies,
        requirements: field_requirements,
    })
}
pub(crate) fn rules_to_domain(value: wire::Rules, limits: &wire::Limits) -> Result<authority::Rules, wire::Problem> {
    let wire::Rules { ceiling, period_spend, minimum_run_spend, maximum_run_spend, implies, requirements } = value;
    let field_ceiling = maps::authority_to_domain(ceiling);
    let field_period_spend = period_spend;
    let field_minimum_run_spend = minimum_run_spend;
    let field_maximum_run_spend = maximum_run_spend;
    let mut list_19 = List::with_capacity((implies).len());
    for entry in (implies).into_boxed() {
        let element = maps::implication_to_domain(entry);
        list_19.push(element).expect("decoded list bound");
    }
    let field_implies = authority::Implies::new(list_19.into_boxed(), limits.rules_implies).ok_or(bound())?;
    let mut list_20 = List::with_capacity((requirements).len());
    for entry in (requirements).into_boxed() {
        let element = maps::requirement_to_domain(entry);
        list_20.push(element).expect("decoded list bound");
    }
    let field_requirements = list_20.into_boxed();
    Ok(authority::Rules {
        ceiling: field_ceiling,
        period_spend: field_period_spend,
        minimum_run_spend: field_minimum_run_spend,
        maximum_run_spend: field_maximum_run_spend,
        implies: field_implies,
        requirements: field_requirements,
    })
}
fn verdict_problem() -> wire::Problem {
    wire::Problem { path: wire::paths::VERDICT_FIELDS, reason: skein_codec::Reason::Bound }
}
fn admit_rule(used: &mut usize, name: &[u8], max: u32, limits: &wire::Limits) -> Result<(), wire::Problem> {
    *used = used.saturating_add(name.len()).saturating_add(skein_lib::length::usize_of(max));
    if name.is_empty() || *used > skein_lib::length::usize_of(limits.field_rule_name) {
        return Err(verdict_problem());
    }
    Ok(())
}
pub(crate) fn verdict_to_record(value: &tasks::Verdict, limits: &wire::Limits) -> Result<wire::Verdict, wire::Problem> {
    let Ok(count) = u32::try_from(value.fields.len()) else {
        return Err(verdict_problem());
    };
    if count > tasks::VERDICT_FIELDS {
        return Err(verdict_problem());
    }
    let mut items = List::with_capacity(count);
    let mut used = 0;
    for field in &value.fields {
        let tasks::FieldRule { name, max } = field;
        admit_rule(&mut used, name, *max, limits)?;
        let field = maps::field_rule_to_record(field);
        let item = field.encode(limits)?;
        items.push(item).expect("same field count");
    }
    let fields = match super::pieces::join(items, limits.verdict_fields) {
        Ok(bytes) => bytes,
        Err(reason) => return Err(wire::Problem { path: wire::paths::VERDICT_FIELDS, reason }),
    };
    Ok(wire::Verdict { code: value.code, words: value.words, followups: value.followups, fields })
}
pub(crate) fn verdict_to_domain(value: wire::Verdict, limits: &wire::Limits) -> Result<tasks::Verdict, wire::Problem> {
    let wire::Verdict { code, words, followups, fields } = value;
    let mut reader = Reader::new(&fields);
    let mut values = List::with_capacity(tasks::VERDICT_FIELDS.min(limits.field_rule_name));
    let mut used = 0;
    for _ in 0..=tasks::VERDICT_FIELDS {
        let bytes = match skein_codec::piece::split(&mut reader) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Ok(tasks::Verdict { code, words, followups, fields: values.into_boxed() }),
            Err(reason) => return Err(wire::Problem { path: wire::paths::VERDICT_FIELDS, reason }),
        };
        let field = wire::FieldRule::decode(limits, &mut Reader::new(&bytes))?;
        let field = maps::field_rule_to_domain(field);
        let tasks::FieldRule { name, max } = &field;
        admit_rule(&mut used, name, *max, limits)?;
        if values.push(field).is_err() {
            return Err(verdict_problem());
        }
    }
    Err(verdict_problem())
}
pub(crate) fn proposal_id_to_record(value: &(people::Proposer, u32)) -> wire::ProposalId {
    let field_proposer = maps::proposer_to_record(&value.0);
    let field_number = value.1;
    wire::ProposalId { proposer: field_proposer, number: field_number }
}
pub(crate) fn proposal_id_to_domain(value: wire::ProposalId) -> (people::Proposer, u32) {
    let wire::ProposalId { proposer, number } = value;
    let field_proposer = maps::proposer_to_domain(proposer);
    let field_number = number;
    (field_proposer, field_number)
}
fn encode_default(
    value: &settings::Default,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<wire::Default, wire::Problem> {
    let field_value = value_to_record(&value.value, limits)?;
    let field_revision = value.revision;
    Ok(wire::Default { models: encode_names(names)?, value: field_value, revision: field_revision })
}
fn decode_default(
    value: wire::Default,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Default, wire::Problem> {
    let wire::Default { models, value, revision } = value;
    let field_value = value_to_domain(value, limits)?;
    let field_revision = revision;
    let mut field_value = field_value;
    let kept = decode_names(models);
    settings::remap(&mut field_value, &settings::moves(&kept, names), &kept, names);
    Ok(settings::Default { value: field_value, revision: field_revision })
}
fn encode_changed(
    value: &settings::Changed,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<wire::Changed, wire::Problem> {
    let field_value = match &value.value {
        Some(entry) => Some(value_to_record(entry, limits)?),
        None => None,
    };
    let field_revision = value.revision;
    Ok(wire::Changed { models: encode_names(names)?, value: field_value, revision: field_revision })
}
fn decode_changed(
    value: wire::Changed,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Changed, wire::Problem> {
    let wire::Changed { models, value, revision } = value;
    let field_value = match value {
        Some(entry) => Some(value_to_domain(entry, limits)?),
        None => None,
    };
    let field_revision = revision;
    let mut field_value = field_value;
    let kept = decode_names(models);
    if let Some(value) = &mut field_value {
        settings::remap(value, &settings::moves(&kept, names), &kept, names);
    }
    Ok(settings::Changed { value: field_value, revision: field_revision })
}
fn encode_revision(
    value: &settings::Revision,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<wire::Revision, wire::Problem> {
    let mut items = List::with_capacity(limits.proposal_items);
    for item in &value.items {
        let item = maps::revision_item_to_record(item, limits)?;
        let writer = item.encode(limits)?;
        if items.push(writer).is_err() {
            return Err(bound());
        }
    }
    let items = match super::pieces::join(items, limits.revision_items) {
        Ok(bytes) => bytes,
        Err(reason) => return Err(wire::Problem { path: wire::paths::REVISION_ITEMS, reason }),
    };
    let mut applications = List::with_capacity(limits.revision_applications);
    for item in &value.applications {
        let item = maps::revision_recorded_to_record(item);
        if applications.push(item).is_err() {
            return Err(bound());
        }
    }
    Ok(wire::Revision { models: encode_names(names)?, head: maps::head_to_record(&value.head), items, applications })
}
fn decode_revision(
    value: wire::Revision,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Revision, wire::Problem> {
    let wire::Revision { models, head, items, applications } = value;
    let mut reader = Reader::new(&items);
    let mut items = List::with_capacity(limits.proposal_items);
    for _ in 0..limits.proposal_items {
        let bytes = match super::people::pieces::split(&mut reader) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => break,
            Err(problem) => return Err(wire::Problem { path: wire::paths::REVISION_ITEMS, reason: problem.reason }),
        };
        let mut piece = Reader::new(&bytes);
        let item = wire::RevisionItem::decode(limits, &mut piece)?;
        if !piece.is_empty() {
            return Err(wire::Problem { path: wire::paths::REVISION_ITEMS, reason: skein_codec::Reason::Malformed });
        }
        let item = maps::revision_item_to_domain(item, limits)?;
        if items.push(item).is_err() {
            return Err(bound());
        }
    }
    if !reader.is_empty() {
        return Err(bound());
    }
    let mut items = items.into_boxed();
    let kept = decode_names(models);
    let moved = settings::moves(&kept, names);
    for item in &mut items {
        match &mut item.edit {
            settings::Edit::Set(value) => settings::remap(value, &moved, &kept, names),
            settings::Edit::Removed | settings::Edit::Reset => {}
        }
        match &mut item.replaced {
            settings::Replaced::Value(value) => settings::remap(value, &moved, &kept, names),
            settings::Replaced::Default | settings::Replaced::Removed | settings::Replaced::NotOffered => {}
        }
    }
    let mut recorded = List::with_capacity(applications.len());
    for item in applications.into_boxed() {
        recorded.push(maps::revision_recorded_to_domain(item)).expect("decoded application item count");
    }
    Ok(settings::Revision { head: maps::head_to_domain(head), items, applications: recorded.into_boxed() })
}
fn encode_proposal(
    value: &settings::Proposal,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<wire::Proposal, wire::Problem> {
    let field_against = value.against;
    let mut list_37 = List::with_capacity(limits.proposal_items);
    for entry in &value.items {
        let element = maps::item_to_record(entry, limits)?;
        if list_37.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_items = list_37;
    Ok(wire::Proposal {
        key: value.key,
        models: encode_names(names)?,
        against: field_against,
        items: field_items,
        applications: encode_recorded(&value.applications, limits.proposal_applications)?,
    })
}
fn decode_proposal(
    value: wire::Proposal,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Proposal, wire::Problem> {
    let wire::Proposal { key, models, against, items, applications } = value;
    let field_against = against;
    let mut list_38 = List::with_capacity((items).len());
    for entry in (items).into_boxed() {
        let element = maps::item_to_domain(entry, limits)?;
        list_38.push(element).expect("decoded list bound");
    }
    let field_items = list_38.into_boxed();
    let mut field_items = field_items;
    let kept = decode_names(models);
    let moved = settings::moves(&kept, names);
    for item in &mut field_items {
        match &mut item.edit {
            settings::Edit::Set(value) => settings::remap(value, &moved, &kept, names),
            settings::Edit::Removed | settings::Edit::Reset => {}
        }
    }
    Ok(settings::Proposal {
        key,
        applications: decode_recorded(applications),
        against: field_against,
        items: field_items,
    })
}

fn encode_recorded(values: &[application::Recorded], capacity: u32) -> Result<List<wire::Recorded>, wire::Problem> {
    let mut result = List::with_capacity(capacity);
    for value in values {
        let value = maps::recorded_to_record(value);
        if result.push(value).is_err() {
            return Err(bound());
        }
    }
    Ok(result)
}
fn decode_recorded(values: List<wire::Recorded>) -> Box<[application::Recorded]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(maps::recorded_to_domain(value)).expect("decoded application item count");
    }
    result.into_boxed()
}

#[must_use]
pub fn limits(domain: &core::Limits) -> Option<wire::Limits> {
    let mut limits = wire::CEILINGS;
    limits.worker_setting_name = domain.fleet.worker_name_bytes;
    limits.text_bytes = core::EFFORT_BYTES;
    limits.catalogue_model_shown_some = core::MODEL_NAME_BYTES;
    limits.catalogue_model_description_some = core::MODEL_DESCRIPTION_BYTES;
    limits.model_name = core::MODEL_NAME_BYTES;
    limits.model_name_name = core::MODEL_NAME_BYTES;
    limits.model_name_efforts = core::EFFORTS;
    limits.model_name_efforts_item = core::EFFORT_BYTES;
    limits.run_policy_instructions = domain.run_bytes;
    limits.overlay_instructions = domain.run_bytes;
    limits.overlay_parts = domain.tasks.charter_parts;
    limits.overlay_documents_some = domain.tasks.charter_documents;
    limits.overlay_alternatives_some = domain.settings.alternatives;
    limits.segment_bytes = domain.authority.segment_bytes.max(domain.tasks.hold_bytes);
    limits.last_exact_bytes = domain.authority.segment_bytes;
    limits.last_open_bytes = domain.authority.segment_bytes;
    limits.pattern_segments = domain.authority.segments;
    limits.charter_documents = domain.tasks.charter_documents;
    limits.charter_parts = domain.tasks.charter_parts;
    limits.charter_part_instructions = domain.run_bytes;
    limits.llm_later_subagents = tasks::MODELS;
    limits.document_name = domain.authority.segments;
    limits.document_path = core::DOCUMENT_PATH_BYTES;
    limits.document_heading_some = core::DOCUMENT_HEADING_BYTES;
    limits.delegation_kinds = domain.authority.executors;
    limits.authority_grants = domain.authority.grants;
    limits.authority_note_resources = domain.authority.grants;
    limits.rules_implies = domain.authority.implications;
    limits.rules_requirements = domain.authority.requirements;
    limits.policy_projections = domain.authority.grants;
    limits.policy_roles = domain.authority.roles;
    limits.policy_requirements = domain.authority.requirements;
    limits.configured_policy_permissions =
        u32::try_from(domain.policy_bytes.checked_div(u64::try_from(size_of::<people::PermissionRole>()).ok()?)?)
            .ok()?;
    limits.configured_policy_templates = domain.templates;
    limits.field_rule_name = domain.tasks.result_bytes;
    limits.verdict_fields = u32::try_from(
        u64::from(tasks::VERDICT_FIELDS.min(domain.tasks.result_bytes))
            .checked_mul(wire::FieldRule::ENCODED_AT_ZERO.checked_add(4)?)?
            .checked_add(u64::from(domain.tasks.result_bytes))?,
    )
    .ok()?;
    limits.part_path_segments = domain.tasks.hold_segments;
    limits.approval_state = domain.tasks.result_bytes;
    limits.verdict_choices_choices = domain.tasks.contract_choices;
    limits.template_name = domain.documents.text_bytes;
    limits.revision_items = super::people::pieces::piece_bound(
        u32::try_from(settings::revision_bound(domain)?).ok()?,
        size_of::<settings::RevisionItem>(),
        wire::RevisionItem::ENCODED_AT_ZERO,
        true,
    )?;
    limits.proposal_items = settings::capacity(domain)?.checked_mul(2)?.min(tasks::SETTINGS_ITEMS);
    limits.revision_applications = tasks::SETTINGS_ITEMS;
    limits.proposal_applications = tasks::SETTINGS_ITEMS;
    if wire::check_limits(&limits).is_err() {
        return None;
    }
    Some(limits)
}

pub fn encode(
    record: &settings::Record,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<Box<[u8]>, wire::Problem> {
    match record {
        settings::Record::Application { setting, state } => {
            match setting {
                settings::Setting::Application { .. } => {}
                settings::Setting::Charter(_)
                | settings::Setting::Template(_)
                | settings::Setting::Policy(_)
                | settings::Setting::Provider(_)
                | settings::Setting::Worker(_)
                | settings::Setting::Model(_)
                | settings::Setting::Rules
                | settings::Setting::Limits
                | settings::Setting::ProjectPolicy { .. } => return Err(bound()),
            }
            let value = maps::application_state_to_record(state)?;
            value.encode(limits)
        }
        settings::Record::Default { default, .. } => {
            let value = encode_default(default, limits, names)?;
            value.encode(limits)
        }
        settings::Record::Changed { changed, .. } => {
            let value = encode_changed(changed, limits, names)?;
            value.encode(limits)
        }
        settings::Record::Revision { revision, .. } => {
            let value = encode_revision(revision, limits, names)?;
            value.encode(limits)
        }
        settings::Record::Extent { oldest, keys } => {
            let value = wire::Extent { oldest: *oldest, keys: *keys };
            value.encode(limits)
        }
        settings::Record::Charter { value, .. } => encode_version(value, limits),
        settings::Record::Proposal { value, .. } => {
            let value = encode_proposal(value, limits, names)?;
            value.encode(limits)
        }
    }
}
pub fn decode(
    key: &settings::Key,
    bytes: &[u8],
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Record, wire::Problem> {
    let mut reader = Reader::new(bytes);
    Ok(match key {
        settings::Key::Default(setting) => settings::Record::Default {
            setting: *setting,
            default: decode_default(wire::Default::decode(limits, &mut reader)?, limits, names)?,
        },
        settings::Key::Changed(setting) => match setting {
            settings::Setting::Application { .. } => settings::Record::Application {
                setting: *setting,
                state: maps::application_state_to_domain(wire::ApplicationState::decode(limits, &mut reader)?),
            },
            settings::Setting::Charter(_)
            | settings::Setting::Template(_)
            | settings::Setting::Policy(_)
            | settings::Setting::Provider(_)
            | settings::Setting::Worker(_)
            | settings::Setting::Model(_)
            | settings::Setting::Rules
            | settings::Setting::Limits
            | settings::Setting::ProjectPolicy { .. } => settings::Record::Changed {
                setting: *setting,
                changed: decode_changed(wire::Changed::decode(limits, &mut reader)?, limits, names)?,
            },
        },
        settings::Key::Held(_) | settings::Key::History(_) => {
            let history = match key {
                settings::Key::History(_) => true,
                settings::Key::Held(_) => false,
                settings::Key::Default(_)
                | settings::Key::Changed(_)
                | settings::Key::Charter { .. }
                | settings::Key::Proposal { .. }
                | settings::Key::Extent => unreachable!("revision key"),
            };
            settings::Record::Revision {
                history,
                revision: decode_revision(wire::Revision::decode(limits, &mut reader)?, limits, names)?,
            }
        }
        settings::Key::Extent => {
            let wire::Extent { oldest, keys } = wire::Extent::decode(limits, &mut reader)?;
            settings::Record::Extent { oldest, keys }
        }
        settings::Key::Charter { charter, revision } => {
            settings::Record::Charter { charter: *charter, revision: *revision, value: decode_version(bytes, limits)? }
        }
        settings::Key::Proposal { proposer, proposal } => settings::Record::Proposal {
            proposer: *proposer,
            proposal: *proposal,
            value: decode_proposal(wire::Proposal::decode(limits, &mut reader)?, limits, names)?,
        },
    })
}

// Like the people family's live policy rows, settings remember the catalogue's
// order. Frozen masks and template choices follow names when that order moves.
fn encode_names(names: &[settings::ModelName]) -> Result<List<wire::ModelName>, wire::Problem> {
    let mut result = List::with_capacity(jig_domain_tasks::MODELS);
    for name in names {
        if result.push(maps::model_name_to_record(name)?).is_err() {
            return Err(bound());
        }
    }
    Ok(result)
}
fn decode_names(names: List<wire::ModelName>) -> Box<[settings::ModelName]> {
    let mut kept = List::with_capacity(names.len());
    for name in names.into_boxed() {
        kept.push(maps::model_name_to_domain(name)).expect("decoded model count");
    }
    kept.into_boxed()
}

// One root per charter-version range (protocol/store/layouts.md, 2).
fn encode_version(value: &settings::CharterVersion, limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = maps::charter_version_to_record(value, limits)?;
    value.encode(limits)
}
fn decode_version(bytes: &[u8], limits: &wire::Limits) -> Result<settings::CharterVersion, wire::Problem> {
    maps::charter_version_to_domain(wire::CharterVersion::decode(limits, &mut Reader::new(bytes))?, limits)
}

#[cfg(test)]
mod provider_tests {
    use super::*;
    #[test]
    fn a_provider_setting_round_trips_in_the_registered_family() {
        use settings::{Endpoint, EndpointAddress, Provider, ProviderSetting, Transport};
        let provider = ProviderSetting {
            provider: Provider {
                number: 4,
                name: b"local".as_slice().into(),
                endpoint: Some(b"local".as_slice().into()),
                direct: None,
                list: None,
                signed_in_plan: false,
                manage_usage: None,
                sign_in: None,
                keys: false,
                programs: Box::new([]),
            },
            endpoints: Box::new([Endpoint {
                number: 5,
                name: b"local".as_slice().into(),
                provider: b"local".as_slice().into(),
                dialect: 3,
                program: None,
                address: EndpointAddress::Socket(::core::net::SocketAddr::from(([127, 0, 0, 1], 8080))),
                transport: Transport::Plaintext,
                headers: Box::new([]),
                connect: skein_lib::Duration::from_millis(10_000),
                handshake: skein_lib::Duration::from_millis(10_000),
            }]),
            withdrawn: true,
        };
        let key = settings::Key::Changed(settings::Setting::Provider(4));
        let record = settings::Record::Changed {
            setting: settings::Setting::Provider(4),
            changed: settings::Changed { revision: 2, value: Some(settings::Value::Provider(Box::new(provider))) },
        };
        let bytes = encode(&record, &wire::CEILINGS, &[]).unwrap();
        assert_eq!(decode(&key, &bytes, &wire::CEILINGS, &[]).unwrap(), record);
    }
}
