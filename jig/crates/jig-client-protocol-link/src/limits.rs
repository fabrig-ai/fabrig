//! Fixed attempt admission and receiving bounds (protocol/client.md, 13).
use jig_client_protocol_wire::v19 as w;
use skein_lib::Duration;
/// Retained connector accounts have their own bound (client/domain/contract.md, 5.3).
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
    archived_turn: jig_protocol_transcript::Limits,
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
        };
        let receiving = Receiving {
            wire: receiving,
            document_room: client.document_bytes.saturating_mul(2),
            fact_rows: wire.owned_record.checked_div(4).expect("nonzero piece prefix"),
            archived_turn: jig_protocol_transcript::Limits::new(
                wire.owned_record.min(receiving.turn_carried_bytes),
                turn,
            ),
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
    fn valid(&self) -> bool {
        self.exchanges != 0
            && self.client.watches != 0
            && self.client.heartbeat != Duration::ZERO
            && crate::store::valid(self)
            && w::Authorize::encoded_bound(&self.wire).is_some()
            && w::AuthorizeAnswer::encoded_bound(&self.wire).is_some()
    }
}
#[expect(clippy::too_many_lines, reason = "the generated field bounds are assigned together at construction")]
fn wire(client: &jig_client_domain::Limits) -> Option<w::Limits> {
    let mut wire = w::CEILINGS;
    let document_bytes = u32::try_from(client.document_bytes).ok()?;
    let ask_bytes = u32::try_from(client.ask_bytes).ok()?;
    wire.owned_record = document_bytes;
    wire.document_text_bytes = client.text.max(HOLD_ACCOUNT_BYTES);
    wire.task_text_words = client.text;
    wire.placed_line_reason = client.text;
    wire.me_providers = wire.owned_record;
    wire.accounts_readiness = wire.owned_record;
    wire.deployment_providers = wire.owned_record;
    wire.deployment_readiness = wire.owned_record;
    wire.provider_offer_programs = client.rows;
    wire.model_set_names_names_item = client.text.min(w::CEILINGS.model_set_names_names_item);
    wire.chip_phase_waiting_dependencies = client.rows;
    wire.chip_details_fields_dependencies = client.rows;
    wire.task_dependency_facts = client.rows;
    wire.circle_tasks = client.rows;
    wire.refusal_circle_tasks = client.rows;
    wire.refusal_kept_pending = client.rows;
    wire.activity_kept_pending = client.rows;
    wire.waits_on_dependencies_pending = client.rows;

    wire.call_identity_id = client.text;
    wire.run_work_tool_tool = client.text;
    wire.recipient_roles_roles = client.rows;
    wire.taken_by_name_some = client.text;
    wire.party_name = client.text;
    wire.decision_rejected_reason = client.text;
    wire.decision_left_held_reason = client.text;
    wire.attempt_failure_reason = client.text;
    wire.part_key_path_value = client.segments;
    wire.part_key_path_value_item = client.text;
    wire.result_field_name = client.text;
    wire.result_field_value = client.text;
    wire.closing_fields_connectors_some = client.rows;
    wire.ending_failed_fields_reason_some = client.text;
    wire.ending_cancelled_fields_reason_some = client.text;
    wire.ending_withdrawn_fields_reason = client.text;
    wire.hold_reason_tries_fields_failures = client.rows;
    wire.hold_reason_stalled_fields_waiting = client.text;
    wire.chip_details_fields_resources = document_bytes;
    wire.chip_details_fields_closing_connectors_some = client.rows;
    wire.result_value_owner_verdict_fields_fields = client.rows;
    wire.result_value_verdict_fields_fields = client.rows;
    wire.result_value_verdict_fields_tasks = client.rows;
    wire.result_value_approved_fields_state = client.text;
    wire.chip_rows = document_bytes;
    wire.chip_phase_facts_rows = document_bytes;
    wire.closing_rows = document_bytes;
    wire.ending_rows = document_bytes;
    wire.held_runtime_rows = document_bytes;
    wire.hold_reason_rows = document_bytes;
    wire.chip_details_rows = document_bytes;
    wire.chip_ending_rows = document_bytes;
    wire.result_value_rows = document_bytes;
    wire.person_answer_choice_reason = client.text;
    wire.person_answer_approved_words_some = client.text;
    wire.person_answer_words_value = client.text;
    wire.person_answer_cannot_reason = client.text;
    wire.parameter_bytes_value = client.text;
    wire.create_tasks = document_bytes.min(w::CEILINGS.create_tasks);
    wire.new_task_named = document_bytes;
    wire.new_task_dependencies = client.rows;
    wire.contract_verdict_choices = client.rows;
    wire.specification_words = client.text;
    wire.specification_parameters = client.rows;
    wire.specification_inputs = client.rows;
    wire.pattern_end_exact_value = client.text;
    wire.pattern_end_open_value = client.text;
    wire.pattern_segments = client.segments;
    wire.pattern_segments_item = client.text;
    wire.delegation_kinds = client.authority;
    wire.authority_grants = document_bytes;
    wire.authority_note_resources = document_bytes;
    wire.amendment_dependencies_some = client.rows;
    wire.amendment_instructions_some = client.text;
    wire.policy_edit_requirements_requirements = document_bytes;
    wire.policy_edit_projections_grants = document_bytes;
    wire.policy_edit_permissions_mappings = client.rows;
    wire.policy_edit_templates_overrides = client.rows.min(w::CEILINGS.policy_edit_templates_overrides);
    wire.message_words = client.text;
    wire.answer_question_words = client.text;
    wire.amend_reason = client.text;
    wire.cancel_reason = client.text;
    wire.move_reason = client.text;
    wire.edit_note_description = client.text;
    wire.edit_note_body = client.text;
    wire.edit_note_references = client.rows;
    wire.set_roles_holdings = client.rows;
    wire.make_service_name = client.text;
    wire.made_tasks_tasks = client.rows;
    wire.read_notes_description_some = client.text;
    wire.spend_page_rows = client.rows;
    wire.word_result_approved_state = client.text;
    wire.word_result_verdict_fields = client.rows;
    wire.word_words = client.text;
    wire.activity_words_words = client.text;
    wire.field_rule_name = client.text;
    wire.verdict_choice_fields = client.rows;
    wire.task_contract_document_approval_state = client.text;
    wire.task_contract_document_verdict_choices = document_bytes;
    wire.result_details_delegates = document_bytes;
    wire.result_details_followups_some = document_bytes;
    wire.bounds_create_charters = client.rows;
    wire.bounds_create_procedures = client.rows;
    wire.bounds_amend_grants = document_bytes;
    wire.bounds_amend_implications = client.authority;
    wire.result_document_acts = document_bytes;
    wire.activity_page_rows = document_bytes;
    wire.hold_spend = client.rows;
    wire.document_specification_parameters = client.rows;
    wire.document_specification_inputs = client.rows;
    wire.proposed_task_dependencies = client.rows;
    wire.proposed_task_resources = document_bytes;
    wire.retained_amendment_dependencies_some = client.rows;
    wire.retained_action_batch_value = document_bytes;
    wire.amendment_state_dependencies = client.rows;
    wire.history_page_rows = document_bytes;
    wire.runtime_running_tool_some = client.text;
    wire.runtime_backing_off_reason = client.text;
    wire.note_tasks = client.rows;
    wire.note_acts = document_bytes;
    wire.note_page_rows = document_bytes;
    wire.object_page_rows = document_bytes;
    wire.ended_page_rows = document_bytes;
    wire.inbox_selection_goals = client.rows;
    wire.inbox_page_selection_some = document_bytes;
    wire.inbox_page_rows = document_bytes;
    wire.subject_chips_tasks = client.rows;
    wire.propose_settings_items = ask_bytes.min(w::CEILINGS.propose_settings_items);
    wire.settings_review_items = document_bytes;
    wire.proposal_action_settings_items = w::CEILINGS.proposal_action_settings_items;
    wire.retained_action_settings_items = w::CEILINGS.retained_action_settings_items;
    // Settings share the receiving text, rows and message envelopes.
    wire.charter_instructions = document_bytes.min(w::CEILINGS.charter_instructions);
    wire.change_settings_items = ask_bytes.min(w::CEILINGS.change_settings_items);
    wire.revision_document_items = document_bytes;
    wire.setting_model_name = client.text.min(w::CEILINGS.setting_model_name);
    wire.setting_model_program_some = client.text.min(w::CEILINGS.setting_model_program_some);
    wire.setting_model_effort_some = client.text.min(w::CEILINGS.setting_model_effort_some);
    wire.setting_model_efforts_item = client.text.min(w::CEILINGS.setting_model_efforts_item);
    wire.charter_documents = client.rows.min(w::CEILINGS.charter_documents);
    wire.charter_document_name = client.segments;
    wire.charter_document_name_item = client.text;
    wire.charter_alternatives = client.rows.min(w::CEILINGS.charter_alternatives);
    wire.setting_model_efforts = client.rows.min(w::CEILINGS.setting_model_efforts);
    wire.setting_policy_projections = client.rows;
    wire.setting_policy_roles = client.rows;
    wire.setting_policy_requirements = client.rows;
    wire.setting_policy_permissions = client.rows;
    wire.setting_policy_templates = client.rows.min(w::CEILINGS.setting_policy_templates);
    wire.rules_implies = client.rows;
    wire.rules_requirements = client.rows;
    // Settings are a whole inventory, independent of a page's row limit.
    wire.settings_document_settings = document_bytes.min(w::CEILINGS.settings_document_settings);
    wire.settings_document_held = client.rows;
    wire.settings_document_pending = client.rows;
    wire.revision_row_items = client.rows;
    wire.revisions_page_rows = client.rows;
    wire.refusal_full_tasks = client.rows;
    w::check_limits(&wire).ok()?;
    Some(wire)
}
// Receiving records share the client's text, row, path and document bounds.
fn receiving(mut wire: w::Limits, client: &jig_client_domain::Limits) -> Option<w::Limits> {
    wire.call_tool = client.text;
    wire.call_input = client.text;
    wire.call_outcome_some = client.text;
    wire.call_objects = client.rows;
    wire.call_parts = wire.owned_record;
    wire.current_call_tool = client.text;
    wire.current_call_input = client.text;
    wire.chip_work_tool_call = client.text;
    wire.chip_work_tool_tool = client.text;
    wire.deployment_hosts = wire.owned_record;
    wire.deployment_attempts = wire.owned_record;
    wire.deployment_accounts = wire.owned_record;
    wire.accounts_accounts = wire.owned_record;
    wire.accounts_signing = wire.owned_record;
    wire.account_line_sources = wire.owned_record;
    wire.account_sources = wire.owned_record;
    wire.account_line_uses_some = w::CEILINGS.account_line_uses_some;
    wire.account_uses_some = w::CEILINGS.account_uses_some;
    wire.deployment_projects = client.rows;
    wire.deployment_hello_refusals = client.rows;
    wire.deployment_person_room_refusals = client.rows;
    wire.deployment_task_room_refusals = client.rows;
    wire.deployment_connectors = client.rows;
    wire.project_acts = wire.owned_record;
    wire.project_resources = wire.owned_record;
    wire.project_members = client.rows;
    wire.project_pools = client.rows;
    wire.project_configured_pools = client.rows;
    wire.project_spend = wire.owned_record;
    wire.project_recurring = wire.owned_record;
    wire.project_policy_projections = wire.owned_record;
    wire.project_policy_roles = wire.owned_record;
    wire.project_policy_requirements = wire.owned_record;
    wire.project_policy_permissions_some = client.rows;
    wire.run_text_text = client.text;
    wire.subscriptions_entries = client.rows;
    wire.task_lineage_chips = wire.owned_record;
    wire.task_inputs = wire.owned_record;
    wire.task_lineage = client.rows;
    wire.task_dependencies = wire.owned_record;
    wire.task_references = wire.owned_record;
    wire.task_resources = wire.owned_record;
    wire.task_proposals = client.rows;
    wire.task_acts = wire.owned_record;
    wire.task_delegates = wire.owned_record;
    wire.task_context_parts = wire.owned_record;
    wire.task_context_spend = client.rows;
    wire.task_context_writer_slots = wire.owned_record;
    wire.tasks_rows = wire.owned_record;
    wire.task_page_live = wire.owned_record;
    wire.ended_rows_rows = wire.owned_record;
    wire.escalation_acts = wire.owned_record;
    wire.escalation_decision_reject_reason = client.text;
    wire.person_proposal_goal_dependencies = client.rows;
    wire.person_proposal_goal_holdings = wire.owned_record;
    wire.person_proposal_state_rejected_reason = client.text;
    wire.person_task_results = wire.owned_record;
    wire.person_task_acts = wire.owned_record;
    wire.proposal_funding = client.rows;
    wire.proposal_acts = wire.owned_record;
    wire.proposal_action_batch_value = wire.owned_record;
    wire.person_contract_choice_options = wire.owned_record;
    wire.question_acts = wire.owned_record;
    wire.chips_rows = wire.owned_record;
    wire.tree_rows = wire.owned_record;
    wire.tree_folded = client.rows;
    wire.tree_row_proposals = client.rows;
    wire.tree_row_dependencies = wire.owned_record;
    wire.inbox_needs = wire.owned_record;
    wire.inbox_updates = wire.owned_record;
    wire.inbox_selection_some = wire.owned_record;
    w::check_limits(&wire).ok()?;
    Some(wire)
}
