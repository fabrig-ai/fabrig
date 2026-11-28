//! Checked translations reached by the link's maps (skein/docs/design/data/codec/maps.md, 4.2).
use super::facts;
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use skein_lib::bytes;

pub(crate) use super::asks::wire_decision as decision_to_record;
pub(crate) use super::facts::{retry_notice_to_domain, run_work_to_domain};
pub(crate) use super::llm::{model_effort_to_domain, model_effort_to_record};

pub(crate) fn cursor_to_record(value: &c::Cursor) -> w::Cursor {
    w::Cursor { value: bytes::copy_of(value.value.bytes()) }
}

pub(crate) fn document_text_to_domain(record: w::DocumentText, limits: &Limits) -> Result<c::DocumentText, w::Problem> {
    let w::DocumentText { bytes, cut } = record;
    match c::Text::new(bytes, limits.client().long_text) {
        Ok(bytes) => Ok(c::DocumentText { bytes, cut }),
        Err(_) => Err(super::text_problem()),
    }
}

pub(crate) fn hold_reason_to_domain(record: w::HoldReason, limits: &Limits) -> Result<c::HoldReason, w::Problem> {
    facts::hold_reason(&record, limits)
}

pub(crate) fn revision_to_domain(value: u64) -> c::Revision {
    c::Revision { value }
}

// The generated copies call the owners for checked text, pieces and other
// shapes that need the link's receiving limits.

pub(crate) fn activity_to_domain(record: w::Activity, limits: &Limits) -> Result<c::Activity, w::Problem> {
    super::pages::activity_from_record(record, limits)
}

pub(crate) fn word_result_to_domain(record: w::WordResult, limits: &Limits) -> Result<c::WordResult, w::Problem> {
    super::pages::word_result_fields(&record, limits)
}

pub(crate) fn archived_turn_to_domain(record: w::ArchivedTurn, limits: &Limits) -> c::ArchivedTurn {
    super::pages::archived_turn_fields(&record, limits)
}

pub(crate) fn chip_to_domain(record: w::Chip, limits: &Limits) -> Result<c::Chip, w::Problem> {
    super::documents::chip_fields(&record, limits)
}

pub(crate) fn authority_to_domain(record: w::Authority, limits: &Limits) -> Result<c::Authority, w::Problem> {
    super::documents::authority_fields(&record, limits)
}

pub(crate) fn document_specification_to_domain(
    record: w::DocumentSpecification,
    limits: &Limits,
) -> Result<c::DocumentSpecification, w::Problem> {
    super::documents::document_specification_fields(&record, limits)
}

pub(crate) fn retained_action_to_domain(
    record: w::RetainedAction,
    limits: &Limits,
) -> Result<c::RetainedAction, w::Problem> {
    super::pages::retained_action_fields(&record, limits)
}

pub(crate) fn history_line_to_domain(record: w::HistoryLine, limits: &Limits) -> Result<c::HistoryLine, w::Problem> {
    super::pages::history_line_fields(record, limits)
}

pub(crate) fn chip_ending_to_domain(record: w::ChipEnding, limits: &Limits) -> Result<c::ChipEnding, w::Problem> {
    super::documents::chip_ending_fields(&record, limits)
}

pub(crate) fn task_line_to_domain(record: w::TaskLine, limits: &Limits) -> Result<c::TaskLine, w::Problem> {
    super::queries::line_from_record(record, limits)
}

pub(crate) fn task_contract_document_to_domain(
    record: w::TaskContractDocument,
    limits: &Limits,
) -> Result<c::TaskContractDocument, w::Problem> {
    super::documents::task_contract_document_fields(&record, limits)
}

pub(crate) fn result_document_to_domain(record: w::ResultDocument, limits: &Limits) -> Result<c::Result, w::Problem> {
    super::documents::result_fields(&record, limits)
}

pub(crate) fn call_to_domain(record: w::Call, limits: &Limits) -> Result<c::Call, w::Problem> {
    super::documents::call_fields(&record, limits)
}

pub(crate) fn current_call_to_domain(record: w::CurrentCall, limits: &Limits) -> Result<c::CurrentCall, w::Problem> {
    super::documents::current_call_fields(&record, limits)
}

pub(crate) fn run_text_to_domain(record: w::RunText, limits: &Limits) -> Result<c::RunText, w::Problem> {
    super::documents::run_text_fields(&record, limits)
}

pub(crate) fn person_proposal_goal_to_domain(
    record: w::PersonProposalGoal,
    limits: &Limits,
) -> Result<c::PersonProposalGoal, w::Problem> {
    super::pages::person_proposal_goal_fields(&record, limits)
}

pub(crate) fn person_proposal_state_to_domain(
    record: w::PersonProposalState,
    limits: &Limits,
) -> Result<c::PersonProposalState, w::Problem> {
    super::pages::person_proposal_state_fields(&record, limits)
}

pub(crate) fn escalation_decision_to_domain(
    record: w::EscalationDecision,
    limits: &Limits,
) -> Result<c::EscalationDecision, w::Problem> {
    super::pages::escalation_decision_fields(&record, limits)
}

pub(crate) fn ended_value_to_domain(record: w::EndedValue, limits: &Limits) -> Result<c::EndedValue, w::Problem> {
    super::pages::ended_value_fields(record, limits)
}

pub(crate) fn wait_resource_name_to_domain(
    record: w::WaitResourceName,
    limits: &Limits,
) -> Result<c::ResourceName, w::Problem> {
    if !c::resource_path_fits(&record.path, limits.client().segments, limits.client().text) {
        return Err(super::too_large());
    }
    Ok(c::ResourceName { connector: record.connector, path: record.path })
}

pub(crate) fn proposal_decision_created_to_domain(record: w::ProposalDecisionCreated) -> (u64, u64) {
    (record.first, record.last)
}

/// A basis-point measurement fits a whole share (jig/docs/design/client/domain/contract/documents.md, 1).
pub(crate) fn basis_to_domain(value: u16) -> Result<u16, skein_codec::Reason> {
    if value > 10_000 {
        return Err(skein_codec::Reason::Bound);
    }
    Ok(value)
}

pub(crate) use super::settings::providers::{provider_address_to_domain, provider_address_to_record};
pub(crate) use super::settings::{model_act_to_domain, model_act_to_record};

pub(crate) fn revision_document_to_domain(
    value: w::RevisionDocument,
    limits: &Limits,
) -> Result<c::RevisionDocument, w::Problem> {
    super::settings::restore_revision_document(&value, limits)
}

pub(crate) use super::asks::wire_note_scope as note_scope_to_record;
pub(crate) use super::documents::bounds_create_to_domain;

pub(crate) fn client_text_to_record(value: &c::Text) -> Box<[u8]> {
    bytes::copy_of(value.bytes())
}

pub(crate) use super::settings::charter_words_to_domain;
pub(crate) use client_text_to_record as charter_words_to_record;

pub(crate) fn document_outcome_to_domain(
    value: w::DocumentOutcome,
    limits: &Limits,
) -> Result<c::DocumentOutcome, w::Problem> {
    super::answers::document_outcome(&value, limits)
}

pub(crate) fn made_to_domain(value: w::Made) -> Result<c::Made, w::Problem> {
    super::answers::made(&value)
}

pub(crate) fn refusal_to_domain(value: w::Refusal, limits: &Limits) -> Result<c::Refusal, w::Problem> {
    super::answers::refusal(&value, limits)
}

pub(crate) fn bounds_to_domain(value: w::Bounds, limits: &Limits) -> Result<c::Bounds, w::Problem> {
    super::documents::bounds_fields(&value, limits)
}

pub(crate) fn decision_to_domain(value: w::Decision, limits: &Limits) -> Result<c::Decision, w::Problem> {
    super::answers::decision(&value, limits)
}

pub(crate) fn charter_to_domain(value: w::Charter) -> Result<c::Charter, w::Problem> {
    super::settings::restore_charter(&value)
}

pub(crate) fn catalogue_entry_to_domain(value: w::CatalogueEntry) -> Result<c::CatalogueEntry, w::Problem> {
    super::settings::restore_catalogue(&value)
}

pub(crate) fn overlay_to_domain(value: w::Overlay) -> Result<c::Overlay, w::Problem> {
    super::settings::restore_overlay(&value)
}

pub(crate) fn setting_policy_to_domain(
    value: w::SettingPolicy,
    limits: &Limits,
) -> Result<c::SettingPolicy, w::Problem> {
    super::settings::restore_setting_policy(&value, limits)
}
