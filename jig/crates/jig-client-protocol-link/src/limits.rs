//! Fixed attempt admission and receiving bounds (protocol/client/README.md, 10; client/domain/client/README.md, 11).
use jig_client_protocol_wire::v22 as w;
use skein_lib::Duration;
/// Retained connector accounts have their own bound (client/domain/contract/documents.md, 3).
pub(crate) const HOLD_ACCOUNT_BYTES: u32 = 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    client: jig_client_domain::Limits,
    exchanges: u32,
    wire: w::Limits,
    receiving: Receiving,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Receiving {
    wire: w::Limits,
    document_room: u64,
    fact_rows: u32,
    // A page's archived turns retain the client's display budget.
    archived_turn: jig_protocol_transcript::Limits,
    // A turn read alone keeps everything the archive's byte bound admits.
    turn_read: jig_protocol_transcript::Limits,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unusable {
    Limits,
}
impl Limits {
    pub fn new(client: jig_client_domain::Limits, exchanges: u32) -> Result<Self, Unusable> {
        if client.owners.len() >= skein_lib::length::usize_of(w::CEILINGS.hello_served) {
            return Err(Unusable::Limits);
        }
        for (i, owner) in client.owners.iter().enumerate() {
            for earlier in client.owners.get(..i).expect("declared prefix") {
                if earlier.family == owner.family {
                    return Err(Unusable::Limits);
                }
            }
        }
        if client.accounts.accounts == 0 || client.accounts.sources == 0 || client.accounts.sign_ins == 0 {
            return Err(Unusable::Limits);
        }
        let wire = wire(&client).ok_or(Unusable::Limits)?;
        let receiving = receiving(wire, &client).ok_or(Unusable::Limits)?;
        let turn = jig_client_domain::contract::TurnLimits {
            messages: client.rows,
            blocks: client.rows,
            text: client.turn_text,
            outcome: client.text,
        };
        let bytes = wire.owned_record.min(receiving.turn_carried_bytes);
        let archived_turn = jig_protocol_transcript::Limits::new(bytes, turn);
        let turn_read = jig_protocol_transcript::Limits::new(
            bytes,
            jig_client_domain::contract::TurnLimits {
                messages: archived_turn.smith.turn_messages.checked_add(1).ok_or(Unusable::Limits)?,
                blocks: archived_turn.smith.message_blocks,
                text: bytes.checked_mul(2).ok_or(Unusable::Limits)?,
                outcome: client.text,
            },
        );
        let receiving = Receiving {
            wire: receiving,
            document_room: client.document_bytes.saturating_mul(2),
            fact_rows: wire.owned_record.checked_div(4).expect("nonzero piece prefix"),
            archived_turn,
            turn_read,
        };
        let value = Self { client, exchanges, wire, receiving };
        if !value.valid() {
            return Err(Unusable::Limits);
        }
        Ok(value)
    }
    #[must_use]
    pub fn client(&self) -> &jig_client_domain::Limits {
        &self.client
    }
    #[must_use]
    pub fn exchanges(&self) -> u32 {
        self.exchanges
    }
    #[must_use]
    pub fn wire(&self) -> &w::Limits {
        &self.wire
    }
    #[must_use]
    pub fn receiving(&self) -> &w::Limits {
        &self.receiving.wire
    }
    pub(crate) fn document_room(&self) -> u64 {
        self.receiving.document_room
    }
    pub(crate) fn fact_rows(&self) -> u32 {
        self.receiving.fact_rows
    }
    pub(crate) fn archived_turn(&self) -> &jig_protocol_transcript::Limits {
        &self.receiving.archived_turn
    }
    pub(crate) fn turn_read(&self) -> &jig_protocol_transcript::Limits {
        &self.receiving.turn_read
    }
    fn valid(&self) -> bool {
        self.exchanges != 0
            && self.client.watches != 0
            && self.client.heartbeat != Duration::ZERO
            && crate::store::valid(self)
            && w::Authorize::encoded_bound(&self.wire).is_some()
            && w::AuthorizeAnswer::encoded_bound(&self.wire).is_some()
    }
}
/// One class of the wire's bounds, sized by one knob. A member the schema bounds below its class keeps its ceiling.
fn narrow(knob: u32, members: &mut [&mut u32]) {
    for member in members.iter_mut() {
        **member = knob.min(**member);
    }
}
fn wire(client: &jig_client_domain::Limits) -> Option<w::Limits> {
    if client.rows > w::CEILINGS.circle_tasks
        || client.text > w::CEILINGS.task_text_words
        || client.segments > w::CEILINGS.pattern_segments
        || client.authority > w::CEILINGS.delegation_kinds
        || client.document_bytes > u64::from(w::CEILINGS.owned_record)
    {
        return None;
    }
    let mut wire = w::CEILINGS;
    let document_bytes = u32::try_from(client.document_bytes).ok()?;
    let ask_bytes = u32::try_from(client.ask_bytes).ok()?;
    wire.owned_record = document_bytes;
    // Retained connector accounts fit beside text previews (client/domain/contract/documents.md, 3).
    wire.document_text_bytes = client.long_text.clamp(HOLD_ACCOUNT_BYTES, w::CEILINGS.document_text_bytes);
    #[rustfmt::skip]
    narrow(client.text, &mut [
        &mut wire.task_text_words, &mut wire.placed_line_reason, &mut wire.model_set_names_names_item,
        &mut wire.call_identity_id, &mut wire.run_work_tool_tool, &mut wire.taken_by_name_some,
        &mut wire.party_name, &mut wire.decision_rejected_reason, &mut wire.decision_left_held_reason,
        &mut wire.attempt_failure_reason, &mut wire.part_key_path_value_item, &mut wire.result_field_name,
        &mut wire.result_field_value, &mut wire.ending_failed_fields_reason_some,
        &mut wire.ending_cancelled_fields_reason_some, &mut wire.ending_withdrawn_fields_reason,
        &mut wire.hold_reason_stalled_fields_waiting, &mut wire.result_value_approved_fields_state,
        &mut wire.person_answer_choice_reason, &mut wire.person_answer_approved_words_some,
        &mut wire.person_answer_words_value, &mut wire.person_answer_cannot_reason,
        &mut wire.parameter_bytes_value, &mut wire.pattern_end_exact_value,
        &mut wire.pattern_end_open_value, &mut wire.pattern_segments_item, &mut wire.amendment_instructions_some,
        &mut wire.amend_reason, &mut wire.cancel_reason,
        &mut wire.move_reason, &mut wire.edit_note_description,
        &mut wire.make_service_name, &mut wire.read_notes_description_some, &mut wire.word_result_approved_state,
        &mut wire.field_rule_name,
        &mut wire.task_contract_document_approval_state, &mut wire.runtime_running_tool_some,
        &mut wire.runtime_backing_off_reason, &mut wire.setting_model_name, &mut wire.setting_model_program_some,
        &mut wire.setting_model_effort_some, &mut wire.setting_model_efforts_item,
        &mut wire.catalogue_entry_shown_some, &mut wire.catalogue_entry_description_some,
        &mut wire.charter_document_name_item,
    ]);
    narrow(
        client.long_text,
        &mut [
            &mut wire.specification_words,
            &mut wire.edit_note_body,
            &mut wire.word_words,
            &mut wire.activity_words_words,
            &mut wire.message_words,
            &mut wire.answer_question_words,
        ],
    );
    #[rustfmt::skip]
    narrow(document_bytes, &mut [
        &mut wire.me_providers, &mut wire.accounts_readiness, &mut wire.deployment_providers,
        &mut wire.deployment_readiness, &mut wire.chip_details_fields_resources, &mut wire.chip_rows,
        &mut wire.chip_phase_facts_rows, &mut wire.closing_rows, &mut wire.ending_rows,
        &mut wire.held_runtime_rows, &mut wire.hold_reason_rows, &mut wire.chip_details_rows,
        &mut wire.chip_ending_rows, &mut wire.result_value_rows, &mut wire.create_tasks, &mut wire.new_task_named,
        &mut wire.authority_grants, &mut wire.authority_note_resources,
        &mut wire.policy_edit_requirements_requirements, &mut wire.policy_edit_projections_grants,
        &mut wire.task_contract_document_verdict_choices, &mut wire.result_details_delegates,
        &mut wire.result_details_followups_some, &mut wire.bounds_amend_grants, &mut wire.result_document_acts,
        &mut wire.activity_page_rows, &mut wire.proposed_task_resources, &mut wire.retained_action_batch_value,
        &mut wire.history_page_rows, &mut wire.note_acts, &mut wire.note_page_rows, &mut wire.object_page_rows,
        &mut wire.ended_page_rows, &mut wire.inbox_page_selection_some, &mut wire.inbox_page_rows,
        &mut wire.settings_review_items, &mut wire.overlay_instructions, &mut wire.charter_instructions, &mut wire.charter_part_instructions, &mut wire.revision_document_items,
        &mut wire.settings_document_settings,
    ]);
    #[rustfmt::skip]
    narrow(client.rows, &mut [
        &mut wire.provider_offer_programs, &mut wire.chip_phase_waiting_dependencies,
        &mut wire.chip_details_fields_dependencies, &mut wire.task_dependency_facts, &mut wire.circle_tasks,
        &mut wire.refusal_circle_tasks, &mut wire.refusal_kept_pending, &mut wire.activity_kept_pending,
        &mut wire.waits_on_dependencies_pending, &mut wire.recipient_roles_roles,
        &mut wire.closing_fields_connectors_some, &mut wire.hold_reason_tries_fields_failures,
        &mut wire.chip_details_fields_closing_connectors_some, &mut wire.result_value_owner_verdict_fields_fields,
        &mut wire.result_value_verdict_fields_fields, &mut wire.result_value_verdict_fields_tasks,
        &mut wire.new_task_dependencies, &mut wire.contract_verdict_choices, &mut wire.specification_parameters,
        &mut wire.specification_inputs, &mut wire.amendment_dependencies_some,
        &mut wire.policy_edit_permissions_mappings, &mut wire.policy_edit_templates_overrides,
        &mut wire.edit_note_references, &mut wire.set_roles_holdings, &mut wire.made_tasks_tasks,
        &mut wire.spend_page_rows, &mut wire.word_result_verdict_fields, &mut wire.verdict_choice_fields,
        &mut wire.bounds_create_charters, &mut wire.bounds_create_procedures, &mut wire.hold_spend,
        &mut wire.document_specification_parameters, &mut wire.document_specification_inputs,
        &mut wire.proposed_task_dependencies, &mut wire.retained_amendment_dependencies_some,
        &mut wire.amendment_state_dependencies, &mut wire.note_tasks, &mut wire.subject_chips_tasks,
        &mut wire.overlay_documents_some, &mut wire.overlay_parts, &mut wire.overlay_alternatives_some,
        &mut wire.charter_documents, &mut wire.charter_parts, &mut wire.charter_alternatives, &mut wire.setting_model_efforts,
        &mut wire.setting_policy_projections, &mut wire.setting_policy_roles,
        &mut wire.setting_policy_requirements, &mut wire.setting_policy_permissions,
        &mut wire.setting_policy_templates, &mut wire.rules_implies, &mut wire.rules_requirements,
        &mut wire.settings_document_held, &mut wire.settings_document_pending, &mut wire.revision_row_items,
        &mut wire.revisions_page_rows, &mut wire.refusal_full_tasks,
    ]);
    narrow(
        client.segments,
        &mut [&mut wire.part_key_path_value, &mut wire.pattern_segments, &mut wire.charter_document_name],
    );
    narrow(client.authority, &mut [&mut wire.delegation_kinds, &mut wire.bounds_amend_implications]);
    narrow(ask_bytes, &mut [&mut wire.propose_settings_items, &mut wire.change_settings_items]);
    w::check_limits(&wire).ok()?;
    Some(wire)
}
// Receiving records share the client's text, row, path and document bounds.
fn receiving(mut wire: w::Limits, client: &jig_client_domain::Limits) -> Option<w::Limits> {
    let document_bytes = wire.owned_record;
    narrow(client.long_text, &mut [&mut wire.run_text_text]);
    #[rustfmt::skip]
    narrow(client.text, &mut [
        &mut wire.call_tool, &mut wire.call_input, &mut wire.call_outcome_some, &mut wire.current_call_tool,
        &mut wire.current_call_input, &mut wire.chip_work_tool_call, &mut wire.chip_work_tool_tool,
        &mut wire.escalation_decision_reject_reason,
        &mut wire.person_proposal_state_rejected_reason,
    ]);
    #[rustfmt::skip]
    narrow(client.rows, &mut [
        &mut wire.call_objects, &mut wire.deployment_projects, &mut wire.deployment_hello_refusals,
        &mut wire.deployment_person_room_refusals, &mut wire.deployment_task_room_refusals,
        &mut wire.deployment_connectors, &mut wire.project_members, &mut wire.project_pools,
        &mut wire.project_configured_pools, &mut wire.project_policy_permissions_some,
        &mut wire.subscriptions_entries, &mut wire.task_lineage, &mut wire.task_proposals,
        &mut wire.task_context_spend, &mut wire.person_proposal_goal_dependencies, &mut wire.proposal_funding,
        &mut wire.tree_folded, &mut wire.tree_row_proposals,
    ]);
    #[rustfmt::skip]
    narrow(document_bytes, &mut [
        &mut wire.call_parts, &mut wire.deployment_hosts, &mut wire.deployment_attempts,
        &mut wire.deployment_accounts, &mut wire.accounts_accounts, &mut wire.accounts_signing,
        &mut wire.account_line_sources, &mut wire.account_sources, &mut wire.project_acts,
        &mut wire.project_resources, &mut wire.project_spend, &mut wire.project_recurring,
        &mut wire.project_policy_projections, &mut wire.project_policy_roles,
        &mut wire.project_policy_requirements, &mut wire.task_lineage_chips, &mut wire.task_inputs,
        &mut wire.task_dependencies, &mut wire.task_references, &mut wire.task_resources, &mut wire.task_acts,
        &mut wire.task_delegates, &mut wire.task_context_parts, &mut wire.task_context_writer_slots,
        &mut wire.tasks_rows, &mut wire.task_page_live, &mut wire.ended_rows_rows, &mut wire.escalation_acts,
        &mut wire.person_proposal_goal_holdings, &mut wire.person_task_results, &mut wire.person_task_acts,
        &mut wire.proposal_acts, &mut wire.proposal_action_batch_value, &mut wire.person_contract_choice_options,
        &mut wire.question_acts, &mut wire.chips_rows, &mut wire.tree_rows, &mut wire.tree_row_dependencies,
        &mut wire.inbox_needs, &mut wire.inbox_updates, &mut wire.inbox_selection_some,
    ]);
    w::check_limits(&wire).ok()?;
    Some(wire)
}
