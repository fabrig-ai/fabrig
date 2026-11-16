//! Shared domain fixtures and translation limits (protocol/store/crates.md, 4.3).
pub(crate) use jig_protocol_store_kit::{
    CORE_LIMITS, DEADLINE, NOTES_LIMITS, TASKS_LIMITS, calls, cold_task, configured_model_names, core_rows, holds,
    identity, model_names, note_entry, note_line, people_amendment, people_asks, people_authority, people_member,
    people_outcomes, people_policy, task_amendment, task_authority, task_contract, task_creation, task_funding,
    task_history, task_name, task_person_proposal, task_phase, task_proposal, task_record, task_spec, task_template,
    task_word, task_writer,
};

pub fn codec_limits() -> crate::codec::View {
    crate::codec::limits(&CORE_LIMITS).expect("fixture codec limits")
}
