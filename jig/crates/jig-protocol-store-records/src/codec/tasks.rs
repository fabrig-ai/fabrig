//! Tasks rows translated to their generated family (protocol/store/records.md, 1).
pub mod activity;
pub mod carried;
mod lifecycle;
mod models;
pub mod phase;
mod pieces;
pub mod proposals;
pub mod record;
pub mod state;
mod summary;
pub mod value;
use crate::generated::tasks as mapped;
use crate::tasks_v13::{self as wire, Record};
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::Reader;

/// Fixed item sizes without any collection or byte payload (protocol/store/records.md, 1).
#[cfg(test)]
pub(crate) const EMPTY: wire::Limits = wire::Limits {
    question_relay_words: 0,
    question_answer_notice_words: 0,
    task_summary_title: 0,
    task_summary_reason: 0,
    summary_result_words: 0,
    end_failed_with_reason_reason: 0,
    end_refused_with_reason_reason: 0,
    failed_retry_reason_some: 0,
    refused_retry_reason_some: 0,
    hold_preparation_words: 0,
    bytes_parameter_value: 0,
    job_spec_bytes: 0,
    job_result_bytes: 0,
    spec_words: 0,
    spec_parameters: 0,
    spec_inputs: 0,
    part_segment_bytes: 0,
    part_path_segments: 0,
    field_rule_name: 0,
    verdict_fields: 0,
    approval_state: 0,
    choices_choices: 0,
    change_contract_choices: 0,
    llm_choice_subagents: 0,
    llm_later_subagents: 0,
    authority_segment_bytes: 0,
    pattern_segments: 0,
    delegation_kinds: 0,
    authority_grants: 0,
    authority_note_resources: 0,
    segment_bytes: 0,
    name_path: 0,
    member_dependencies: 0,
    member_holdings: 0,
    recurring_template_batch: 0,
    result_field_name: 0,
    result_field_value: 0,
    approved_state: 0,
    approved_words_some: 0,
    report_result_words: 0,
    verdict_result_fields: 0,
    verdict_result_words: 0,
    change_result_title: 0,
    change_result_words: 0,
    failure_result_reason: 0,
    cancelled_reason: 0,
    circle_tasks: 0,
    hold_account_words: 0,
    hold_wait_words: 0,
    rejected_proposal_reason: 0,
    amendment_instructions_some: 0,
    amendment_dependencies_some: 0,
    amendment_reason: 0,
    proposed_batch_members: 0,
    proposed_settings_items: 0,
    proposed_settings_projects: 0,
    proposal_reason: 0,
    settings_instructions_some: 0,
    settings_dependencies: 0,
    history_reason: 0,
    rejected_person_proposal_reason: 0,
    person_proposal_reason: 0,
    grace_reason: 0,
    approval_message_state: 0,
    verdict_message_fields: 0,
    word_words: 0,
    activity_words_words: 0,
    kept_line_pending: 0,
    stood_line_words: 0,
    now_written_line_line: 0,
    rejected_escalation_reason: 0,
    recurring_state_last_batch: 0,
    saved_resource_path: 0,
    task_history_question_lines: 0,
    hot_history_question_lines: 0,
    task_history_proposal_rows: 0,
    hot_history_proposal_rows: 0,
    task_history_instructions_some: 0,
    live_cold_instructions_some: 0,
    task_history_refused: 0,
    hot_history_refused: 0,
    task_run_replies: 0,
    task_definition_dependencies: 0,
    hot_definition_dependencies: 0,
    task_definition_waiting_on: 0,
    hot_definition_waiting_on: 0,
    task_definition_added: 0,
    hot_definition_added: 0,
    task_interests_holdings: 0,
    task_interests_delegates: 0,
    task_interests_result_followups: 0,
    task_interests_references: 0,
    task_interests_subscriptions: 0,
    task_interests_inbox: 0,
    task_interests_saved: 0,
    before_talk_started_from: 0,
    live_hot_started_from: 0,
    placed_hold_reason: 0,
    placed_hold_exempt: 0,
    model_name_value: 0,
    row_models: 0,
    task_problem_blocked_by_some: 0,
    result_followups_delegates_value: 0,
};

/// Translate the tasks child's bounds, refusing any schema ceiling.
#[expect(clippy::too_many_lines, reason = "each tasks field derives its bound beside the complete limits mapping")]
#[must_use]
pub fn limits(domain: &domain::Limits) -> Option<wire::Limits> {
    let rules = u32::try_from(
        u64::from(domain.result_bytes.min(domain::VERDICT_FIELDS))
            .checked_mul(wire::FieldRule::ENCODED_AT_ZERO.checked_add(4)?)?
            .checked_add(u64::from(domain.result_bytes))?,
    )
    .ok()?;
    let fields = u32::try_from(
        u64::from(domain.result_bytes.min(domain::VERDICT_FIELDS))
            .checked_mul(wire::ResultField::ENCODED_AT_ZERO.checked_add(4)?)?
            .checked_add(u64::from(domain.result_bytes))?,
    )
    .ok()?;
    let creations = super::people::pieces::piece_bound(
        domain.batch_bytes,
        size_of::<domain::New>(),
        wire::Creation::ENCODED_AT_ZERO,
        true,
    )?;
    let members = super::people::pieces::piece_bound(
        domain.batch_bytes,
        size_of::<domain::New>(),
        wire::Member::ENCODED_AT_ZERO,
        true,
    )?;
    let limits = wire::Limits {
        question_relay_words: domain.message_bytes,
        question_answer_notice_words: wire::CEILINGS.question_answer_notice_words,
        task_summary_title: domain::SUMMARY_TEXT,
        task_summary_reason: domain::SUMMARY_TEXT,
        summary_result_words: domain::SUMMARY_TEXT,
        end_failed_with_reason_reason: domain::REASON_BYTES,
        end_refused_with_reason_reason: domain::REASON_BYTES,
        failed_retry_reason_some: domain::REASON_BYTES,
        refused_retry_reason_some: domain::REASON_BYTES,
        hold_preparation_words: domain::REASON_BYTES,
        hold_account_words: domain::HOLD_ACCOUNT_BYTES,
        hold_wait_words: domain::WAIT_WORDS,
        placed_hold_reason: domain.message_bytes,
        placed_hold_exempt: domain.exemptions,
        proposed_settings_items: domain::SETTINGS_ITEMS,
        proposed_settings_projects: wire::CEILINGS.proposed_settings_projects,
        row_models: jig_domain_tasks::MODELS,
        model_name_value: jig_domain_core_vocabulary::MODEL_NAME_BYTES,
        person_proposal_reason: domain.message_bytes,
        grace_reason: domain.result_bytes,
        task_problem_blocked_by_some: domain.delegates.max(domain.dependencies).max(domain.circle),
        result_followups_delegates_value: creations,
        segment_bytes: domain.hold_bytes,
        name_path: u32::try_from(
            u64::from(domain.hold_segments)
                .checked_mul(wire::Segment::ENCODED_AT_ZERO.checked_add(4)?)?
                .checked_add(u64::from(domain.hold_bytes))?,
        )
        .ok()?,
        bytes_parameter_value: domain.spec_bytes,
        spec_words: domain.spec_bytes,
        job_spec_bytes: domain.spec_bytes,
        job_result_bytes: domain.result_bytes,
        spec_parameters: u32::try_from(
            u64::from(domain.parameters)
                .checked_mul(wire::Parameter::ENCODED_AT_ZERO.checked_add(4)?)?
                .checked_add(u64::from(domain.spec_bytes))?,
        )
        .ok()?,
        spec_inputs: domain.inputs,
        part_segment_bytes: domain.hold_bytes,
        part_path_segments: u32::try_from(
            u64::from(domain.hold_segments)
                .checked_mul(wire::PartSegment::ENCODED_AT_ZERO.checked_add(4)?)?
                .checked_add(u64::from(domain.hold_bytes))?,
        )
        .ok()?,
        field_rule_name: domain.result_bytes,
        verdict_fields: rules,
        approval_state: domain.result_bytes,
        choices_choices: domain.contract_choices,
        change_contract_choices: domain.contract_choices,
        llm_choice_subagents: domain::MODELS,
        llm_later_subagents: domain::MODELS,
        authority_segment_bytes: domain.authority_bytes,
        pattern_segments: domain.authority_segments,
        authority_grants: u32::try_from(
            u64::from(domain.authority_grants)
                .checked_mul(wire::Grant::ENCODED_AT_ZERO.checked_add(4)?.checked_add(
                    u64::from(domain.authority_segments).checked_mul(wire::AuthoritySegment::ENCODED_AT_ZERO)?,
                )?)?
                .checked_add(u64::from(domain.authority_bytes))?,
        )
        .ok()?,
        authority_note_resources: u32::try_from(
            u64::from(domain.authority_grants)
                .checked_mul(wire::ResourceScope::ENCODED_AT_ZERO.checked_add(4)?.checked_add(
                    u64::from(domain.authority_segments).checked_mul(wire::AuthoritySegment::ENCODED_AT_ZERO)?,
                )?)?
                .checked_add(u64::from(domain.authority_bytes))?,
        )
        .ok()?,
        delegation_kinds: domain.executor_kinds,
        member_dependencies: domain.dependencies,
        member_holdings: domain.holdings,
        recurring_template_batch: members,
        result_field_name: domain.result_bytes,
        result_field_value: domain.result_bytes,
        approved_state: domain.result_bytes,
        approved_words_some: domain.result_bytes,
        report_result_words: domain.result_bytes,
        verdict_result_fields: fields,
        verdict_result_words: domain.result_bytes,
        change_result_title: domain.result_bytes,
        change_result_words: domain.result_bytes,
        failure_result_reason: domain.result_bytes,
        cancelled_reason: domain.result_bytes,
        rejected_proposal_reason: domain.message_bytes,
        amendment_instructions_some: domain.message_bytes,
        amendment_dependencies_some: domain.dependencies,
        amendment_reason: domain.message_bytes,
        proposed_batch_members: creations,
        proposal_reason: domain.message_bytes,
        settings_instructions_some: domain.message_bytes,
        settings_dependencies: domain.dependencies,
        history_reason: domain.result_bytes.max(domain.message_bytes),
        rejected_person_proposal_reason: domain.message_bytes,
        approval_message_state: domain.result_bytes,
        verdict_message_fields: fields,
        word_words: domain.word_bytes(),
        activity_words_words: domain.message_bytes,
        rejected_escalation_reason: domain.result_bytes,
        recurring_state_last_batch: domain.batch,
        saved_resource_path: u32::try_from(
            u64::from(domain.authority_segments)
                .checked_mul(wire::AuthoritySegment::ENCODED_AT_ZERO.checked_add(4)?)?
                .checked_add(u64::from(domain.authority_bytes))?,
        )
        .ok()?,
        task_history_question_lines: domain.inbox_messages,
        hot_history_question_lines: domain.inbox_messages,
        task_history_proposal_rows: domain.inbox_messages,
        hot_history_proposal_rows: domain.inbox_messages,
        task_history_instructions_some: domain.message_bytes,
        live_cold_instructions_some: domain.message_bytes,
        task_history_refused: domain.inbox_messages,
        hot_history_refused: domain.inbox_messages,
        task_run_replies: domain.inbox_messages,
        task_definition_dependencies: domain.dependencies,
        hot_definition_dependencies: domain.dependencies,
        before_talk_started_from: domain.dependencies,
        live_hot_started_from: domain.dependencies,
        task_definition_waiting_on: domain.dependencies,
        hot_definition_waiting_on: domain.dependencies,
        task_definition_added: domain.dependencies,
        hot_definition_added: domain.dependencies,
        circle_tasks: domain.circle,
        kept_line_pending: domain.dependencies,
        stood_line_words: jig_domain_tasks::STOOD_WORDS,
        now_written_line_line: jig_domain_tasks::NOW_LINE,
        task_interests_holdings: domain.holdings,
        task_interests_delegates: domain.delegates,
        task_interests_result_followups: domain.batch,
        task_interests_references: domain.references,
        task_interests_subscriptions: wire::CEILINGS.task_interests_subscriptions,
        task_interests_inbox: domain.inbox_messages.checked_add(2)?,
        task_interests_saved: domain.saved_resources,
    };
    wire::check_limits(&limits).ok()?;
    Some(limits)
}

/// Encode one tasks row within the configured bounds.
pub fn encode(
    row: &domain::Stored,
    limits: &wire::Limits,
    names: &[jig_domain_core_vocabulary::settings::ModelName],
) -> Result<Box<[u8]>, wire::Problem> {
    let (models, moved) = models::encode(row, names)?;
    let context = &Context::new(limits, Some(&moved));
    let value = match row {
        domain::Stored::PlacedHold(row) => wire::Value::PlacedHold(mapped::placed_hold_to_record(row)?),
        domain::Stored::Ledger(row) => wire::Value::Ledger(mapped::funding_record_to_record(row)),
        domain::Stored::ClosedLedger(row) => wire::Value::ClosedLedger(mapped::funding_record_to_record(row)),
        domain::Stored::Writer(row) => wire::Value::Writer(mapped::writer_slot_to_record(row, context)?),
        domain::Stored::Pool(row) => wire::Value::Pool(mapped::pool_slots_to_record(row, context)?),
        domain::Stored::History(row) => wire::Value::History(Box::new(mapped::history_to_record(row, context)?)),
        domain::Stored::Attempt(row) => wire::Value::Attempt(mapped::attempt_row_to_record(row)),
        domain::Stored::Hold(row) => wire::Value::Hold(mapped::hold_row_to_record(row, context)?),
        domain::Stored::PersonProposal(row) => {
            wire::Value::PersonProposal(mapped::person_proposal_to_record(row, context)?)
        }
        domain::Stored::DecidedProposal(row) => {
            wire::Value::DecidedProposal(mapped::person_proposal_to_record(row, context)?)
        }
        domain::Stored::Update(row) => wire::Value::Update(mapped::update_to_record(row)),
        domain::Stored::Result(row) => wire::Value::Result(mapped::result_row_to_record(row)),
        domain::Stored::EndedDelegate(row) => wire::Value::EndedDelegate(mapped::edge_to_record(row)),
        domain::Stored::EndedByParty(row) => wire::Value::EndedByParty(mapped::ended_by_party_to_record(row)),
        domain::Stored::EndedByProject(row) => wire::Value::EndedByProject(mapped::ended_by_project_to_record(row)),
        domain::Stored::Delegate(row) => wire::Value::Delegate(mapped::edge_to_record(row)),
        domain::Stored::Activity(row) => wire::Value::Activity(mapped::activity_line_to_record(row, context)?),
        domain::Stored::Live(row) => wire::Value::LiveHot(record::live_hot(row, limits, Some(&moved))?),
        domain::Stored::LiveCold { spec, contract, authority, instructions, .. } => {
            wire::Value::LiveCold(record::live_cold(spec, contract, authority, instructions, limits, Some(&moved))?)
        }
        domain::Stored::Ended(row) => wire::Value::EndedTalk(record::task_record(row, limits, Some(&moved))?),
        domain::Stored::Stub(row) => wire::Value::Stub(mapped::stub_to_record(row)),
        domain::Stored::Summary(row) => wire::Value::Summary(Box::new(summary::encode(row))),
        domain::Stored::Retention(row) => wire::Value::Retention(mapped::retention_row_to_record(row)),
    };
    let row = wire::Row { value, models };
    row.encode(limits)
}

/// Decode one tasks row, refusing another version or trailing bytes.
pub fn decode(
    bytes: &[u8],
    limits: &wire::Limits,
    names: &[jig_domain_core_vocabulary::settings::ModelName],
) -> Result<domain::Stored, wire::Problem> {
    let context = &Context::new(limits, None);
    let wire::Row { value, models: kept } = wire::Row::decode(limits, &mut Reader::new(bytes))?;
    let mut row = match value {
        wire::Value::Summary(row) => domain::Stored::Summary(Box::new(summary::decode(*row)?)),
        wire::Value::Retention(row) => domain::Stored::Retention(mapped::retention_row_to_domain(row)),
        wire::Value::LiveHot(_) | wire::Value::LiveCold(_) => {
            return Err(wire::Problem { path: wire::paths::ROW_VALUE, reason: skein_codec::Reason::Bound });
        }
        wire::Value::EndedTalk(row) => domain::Stored::Ended(Box::new(record::read_task_record(row, limits)?)),
        wire::Value::PlacedHold(row) => domain::Stored::PlacedHold(Box::new(mapped::placed_hold_to_domain(row))),
        wire::Value::Update(row) => domain::Stored::Update(mapped::update_to_domain(row)),
        wire::Value::Result(row) => domain::Stored::Result(mapped::result_row_to_domain(row)),
        wire::Value::EndedDelegate(row) => domain::Stored::EndedDelegate(mapped::edge_to_domain(row)),
        wire::Value::EndedByParty(row) => domain::Stored::EndedByParty(mapped::ended_by_party_to_domain(row)),
        wire::Value::EndedByProject(row) => domain::Stored::EndedByProject(mapped::ended_by_project_to_domain(row)),
        wire::Value::Delegate(row) => domain::Stored::Delegate(mapped::edge_to_domain(row)),
        wire::Value::Activity(row) => domain::Stored::Activity(mapped::activity_line_to_domain(row, context)?),
        wire::Value::Ended(row) => domain::Stored::Ended(Box::new(record::read_before_talk(row, limits)?)),
        wire::Value::Stub(row) => domain::Stored::Stub(mapped::stub_to_domain(row)),
        wire::Value::Ledger(value) => domain::Stored::Ledger(mapped::funding_record_to_domain(value)),
        wire::Value::History(value) => domain::Stored::History(mapped::history_to_domain(*value, context)?),
        wire::Value::Attempt(row) => domain::Stored::Attempt(mapped::attempt_row_to_domain(row)),
        wire::Value::Hold(row) => domain::Stored::Hold(mapped::hold_row_to_domain(row, context)?),
        wire::Value::PersonProposal(value) => {
            domain::Stored::PersonProposal(Box::new(mapped::person_proposal_to_domain(value, context)?))
        }
        wire::Value::DecidedProposal(value) => {
            domain::Stored::DecidedProposal(Box::new(mapped::person_proposal_to_domain(value, context)?))
        }
        wire::Value::ClosedLedger(value) => domain::Stored::ClosedLedger(mapped::funding_record_to_domain(value)),
        wire::Value::Writer(value) => domain::Stored::Writer(mapped::writer_slot_to_domain(value, context)?),
        wire::Value::Pool(value) => domain::Stored::Pool(mapped::pool_slots_to_domain(value, context)?),
    };
    row.remap_models(&models::moves(&kept, names));
    Ok(row)
}

/// Decode a cold part with the identity carried by its key.
pub fn decode_cold(
    number: u64,
    bytes: &[u8],
    limits: &wire::Limits,
    names: &[jig_domain_core_vocabulary::settings::ModelName],
) -> Result<domain::Stored, wire::Problem> {
    let wire::Row { value: part, models: kept } = wire::Row::decode(limits, &mut Reader::new(bytes))?;
    let cold = match part {
        wire::Value::LiveCold(cold) => cold,
        wire::Value::Summary(_)
        | wire::Value::Retention(_)
        | wire::Value::Ledger(_)
        | wire::Value::ClosedLedger(_)
        | wire::Value::Writer(_)
        | wire::Value::Pool(_)
        | wire::Value::History(_)
        | wire::Value::PersonProposal(_)
        | wire::Value::DecidedProposal(_)
        | wire::Value::Activity(_)
        | wire::Value::Ended(_)
        | wire::Value::Stub(_)
        | wire::Value::Update(_)
        | wire::Value::Result(_)
        | wire::Value::EndedDelegate(_)
        | wire::Value::Delegate(_)
        | wire::Value::EndedByParty(_)
        | wire::Value::EndedByProject(_)
        | wire::Value::Attempt(_)
        | wire::Value::Hold(_)
        | wire::Value::PlacedHold(_)
        | wire::Value::EndedTalk(_)
        | wire::Value::LiveHot(_) => {
            return Err(wire::Problem { path: wire::paths::ROW_VALUE, reason: skein_codec::Reason::Bound });
        }
    };
    let wire::LiveCold { spec, contract, authority, instructions } = cold;
    let mut row = domain::Stored::LiveCold {
        number,
        spec: Box::new(value::read_spec(spec, limits)?),
        contract: value::read_contract(contract, limits)?,
        authority: value::read_authority(authority, limits)?,
        instructions,
    };
    row.remap_models(&models::moves(&kept, names));
    Ok(row)
}
/// A live read joins both parts, each with its own model-name vocabulary.
pub fn decode_live(
    hot: &[u8],
    cold: &[u8],
    limits: &wire::Limits,
    names: &[jig_domain_core_vocabulary::settings::ModelName],
) -> Result<domain::Stored, wire::Problem> {
    let wire::Row { value: part, models: hot_names } = wire::Row::decode(limits, &mut Reader::new(hot))?;
    let hot = match part {
        wire::Value::LiveHot(hot) => hot,
        wire::Value::Summary(_)
        | wire::Value::Retention(_)
        | wire::Value::Ledger(_)
        | wire::Value::ClosedLedger(_)
        | wire::Value::Writer(_)
        | wire::Value::Pool(_)
        | wire::Value::History(_)
        | wire::Value::PersonProposal(_)
        | wire::Value::DecidedProposal(_)
        | wire::Value::Activity(_)
        | wire::Value::Ended(_)
        | wire::Value::Stub(_)
        | wire::Value::Update(_)
        | wire::Value::Result(_)
        | wire::Value::EndedDelegate(_)
        | wire::Value::Delegate(_)
        | wire::Value::EndedByParty(_)
        | wire::Value::EndedByProject(_)
        | wire::Value::Attempt(_)
        | wire::Value::Hold(_)
        | wire::Value::PlacedHold(_)
        | wire::Value::EndedTalk(_)
        | wire::Value::LiveCold(_) => {
            return Err(wire::Problem { path: wire::paths::ROW_VALUE, reason: skein_codec::Reason::Bound });
        }
    };
    let wire::Row { value: part, models: cold_names } = wire::Row::decode(limits, &mut Reader::new(cold))?;
    let cold = match part {
        wire::Value::LiveCold(cold) => cold,
        wire::Value::Summary(_)
        | wire::Value::Retention(_)
        | wire::Value::Ledger(_)
        | wire::Value::ClosedLedger(_)
        | wire::Value::Writer(_)
        | wire::Value::Pool(_)
        | wire::Value::History(_)
        | wire::Value::PersonProposal(_)
        | wire::Value::DecidedProposal(_)
        | wire::Value::Activity(_)
        | wire::Value::Ended(_)
        | wire::Value::Stub(_)
        | wire::Value::Update(_)
        | wire::Value::Result(_)
        | wire::Value::EndedDelegate(_)
        | wire::Value::Delegate(_)
        | wire::Value::EndedByParty(_)
        | wire::Value::EndedByProject(_)
        | wire::Value::Attempt(_)
        | wire::Value::Hold(_)
        | wire::Value::PlacedHold(_)
        | wire::Value::EndedTalk(_)
        | wire::Value::LiveHot(_) => {
            return Err(wire::Problem { path: wire::paths::ROW_VALUE, reason: skein_codec::Reason::Bound });
        }
    };
    let record = record::read_task_record(record::join(hot, cold), limits)?;
    let models = record.authority.models.remap(&models::moves(&cold_names, names));
    let mut row = domain::Stored::Live(Box::new(record));
    row.remap_models(&models::moves(&hot_names, names));
    match &mut row {
        domain::Stored::Live(row) => row.authority.models = models,
        domain::Stored::Summary(_)
        | domain::Stored::Retention(_)
        | domain::Stored::Attempt(_)
        | domain::Stored::Hold(_)
        | domain::Stored::PlacedHold(_)
        | domain::Stored::Update(_)
        | domain::Stored::Result(_)
        | domain::Stored::EndedByParty(_)
        | domain::Stored::EndedByProject(_)
        | domain::Stored::EndedDelegate(_)
        | domain::Stored::Delegate(_)
        | domain::Stored::Activity(_)
        | domain::Stored::PersonProposal(_)
        | domain::Stored::DecidedProposal(_)
        | domain::Stored::ClosedLedger(_)
        | domain::Stored::History(_)
        | domain::Stored::Ended(_)
        | domain::Stored::Stub(_)
        | domain::Stored::Ledger(_)
        | domain::Stored::Writer(_)
        | domain::Stored::Pool(_)
        | domain::Stored::LiveCold { .. } => unreachable!("joined live task"),
    }
    Ok(row)
}

/// Translation context: the family's limits and the row's model-name remap.
/// Carried values have no remap (skein/docs/design/data/codec/maps.md, 4).
#[derive(Debug)]
pub struct Context {
    limits: wire::Limits,
    moved: Option<[u64; 64]>,
}
impl Context {
    pub(crate) fn new(limits: &wire::Limits, moved: Option<&[u64; 64]>) -> Self {
        Self { limits: *limits, moved: moved.copied() }
    }
}

#[must_use]
pub fn tools_to_record(value: &domain::Tools) -> u64 {
    value.0
}
#[must_use]
pub fn tools_to_domain(value: u64) -> domain::Tools {
    domain::Tools(value)
}
#[must_use]
pub fn scopes_to_record(value: &domain::Scopes) -> u8 {
    value.0
}
#[must_use]
pub fn scopes_to_domain(value: u8) -> domain::Scopes {
    domain::Scopes(value)
}
#[must_use]
pub fn newest_to_record(value: &domain::Newest) -> u64 {
    value.value()
}
#[must_use]
pub fn newest_to_domain(value: u64) -> domain::Newest {
    domain::Newest::of(value)
}
#[must_use]
pub fn reasons_to_record(value: &domain::Reasons) -> u32 {
    value.0
}
pub fn reasons_to_domain(value: u32) -> Result<domain::Reasons, skein_codec::Reason> {
    let value = domain::Reasons(value);
    if value.is_known() { Ok(value) } else { Err(skein_codec::Reason::Bound) }
}

pub fn name_to_record(value: &domain::Name, context: &Context) -> Result<wire::Name, wire::Problem> {
    value::name(value, &context.limits)
}
pub fn name_to_domain(record: wire::Name, context: &Context) -> Result<domain::Name, wire::Problem> {
    value::read_name(record, &context.limits)
}

pub fn spec_to_record(value: &domain::Spec, context: &Context) -> Result<wire::Spec, wire::Problem> {
    value::spec(value, &context.limits)
}
pub fn spec_to_domain(record: wire::Spec, context: &Context) -> Result<domain::Spec, wire::Problem> {
    value::read_spec(record, &context.limits)
}

pub fn task_result_to_record(value: &domain::TaskResult, context: &Context) -> Result<wire::TaskResult, wire::Problem> {
    phase::result(value, &context.limits)
}
pub fn task_result_to_domain(record: wire::TaskResult, context: &Context) -> Result<domain::TaskResult, wire::Problem> {
    phase::read_result(record, &context.limits)
}

pub fn authority_to_record(value: &domain::Authority, context: &Context) -> Result<wire::Authority, wire::Problem> {
    value::authority(value, &context.limits, context.moved.as_ref())
}
pub fn authority_to_domain(record: wire::Authority, context: &Context) -> Result<domain::Authority, wire::Problem> {
    value::read_authority(record, &context.limits)
}

pub fn proposal_action_to_record(
    value: &domain::ProposalAction,
    context: &Context,
) -> Result<wire::ProposalAction, wire::Problem> {
    proposals::action(value, &context.limits, context.moved.as_ref())
}
pub fn proposal_action_to_domain(
    record: wire::ProposalAction,
    context: &Context,
) -> Result<domain::ProposalAction, wire::Problem> {
    proposals::read_action(record, &context.limits)
}

pub fn result_kind_to_record(value: &domain::ResultKind, context: &Context) -> Result<wire::ResultKind, wire::Problem> {
    activity::result_kind(value, &context.limits)
}
pub fn result_kind_to_domain(record: wire::ResultKind, context: &Context) -> Result<domain::ResultKind, wire::Problem> {
    activity::read_result_kind(record, &context.limits)
}

#[must_use]
pub fn why_to_record(value: &domain::Why) -> wire::Why {
    value::why(*value)
}
#[must_use]
pub fn why_to_domain(record: wire::Why) -> domain::Why {
    value::read_why(record)
}

pub fn recurring_template_to_record(
    value: &domain::RecurringTemplate,
    context: &Context,
) -> Result<wire::RecurringTemplate, wire::Problem> {
    phase::template(value, &context.limits, context.moved.as_ref())
}
pub fn recurring_template_to_domain(
    record: wire::RecurringTemplate,
    context: &Context,
) -> Result<domain::RecurringTemplate, wire::Problem> {
    phase::read_template(record, &context.limits)
}

pub fn part_path_to_record(
    value: &skein_lib::List<Box<[u8]>>,
    context: &Context,
) -> Result<wire::PartPath, wire::Problem> {
    let limits = &context.limits;
    let segments =
        pieces::part_segments(value.as_slice(), limits, limits.part_path_segments, wire::paths::PART_PATH_SEGMENTS)?;
    let mut bytes = 0_usize;
    for segment in value {
        bytes = bytes.saturating_add(segment.len());
    }
    if bytes > skein_lib::length::usize_of(limits.part_segment_bytes) {
        return Err(wire::paths::PART_PATH_SEGMENTS.problem(skein_codec::Reason::Bound));
    }
    Ok(wire::PartPath { segments })
}
pub fn part_path_to_domain(
    record: wire::PartPath,
    context: &Context,
) -> Result<skein_lib::List<Box<[u8]>>, wire::Problem> {
    let limits = &context.limits;
    let segments = pieces::read_part_segments(record.segments, limits, wire::paths::PART_PATH_SEGMENTS)?;
    let mut members = skein_lib::List::with_capacity(u32::try_from(segments.len()).expect("bounded path"));
    let mut bytes = 0_usize;
    for segment in segments {
        bytes = bytes.saturating_add(segment.len());
        members.push(segment).expect("same number of segments");
    }
    if bytes > skein_lib::length::usize_of(limits.part_segment_bytes) {
        return Err(wire::paths::PART_PATH_SEGMENTS.problem(skein_codec::Reason::Bound));
    }
    Ok(members)
}
pub fn result_followups_delegates_to_record(
    value: &[domain::New],
    context: &Context,
) -> Result<wire::ResultFollowupsDelegates, wire::Problem> {
    let limits = &context.limits;
    Ok(wire::ResultFollowupsDelegates {
        value: pieces::creations(
            value,
            limits,
            limits.result_followups_delegates_value,
            wire::paths::RESULT_FOLLOWUPS_DELEGATES_VALUE,
            context.moved.as_ref(),
        )?,
    })
}
pub fn result_followups_delegates_to_domain(
    record: wire::ResultFollowupsDelegates,
    context: &Context,
) -> Result<Box<[domain::New]>, wire::Problem> {
    pieces::read_creations(record.value, &context.limits, wire::paths::RESULT_FOLLOWUPS_DELEGATES_VALUE)
}

pub use proposals::{person_action_to_domain, person_action_to_record};

/// The selector's tag, unlike a bit-set converter, is stored separately.
#[must_use]
pub fn models_to_record(value: &domain::Models) -> wire::Models {
    wire::Models { bits: value.bits(), all: value.is_all() }
}
#[must_use]
pub fn models_to_domain(record: wire::Models) -> domain::Models {
    domain::Models::from_parts(record.bits, record.all)
}
