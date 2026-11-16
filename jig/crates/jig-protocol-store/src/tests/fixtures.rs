//! Store fixtures and shared domain values (protocol/store/crates.md, 4.2).
use alloc::boxed::Box;

mod keys;
pub use keys::own;

pub const APPLICATION: crate::sizes::Application = crate::sizes::Application { version: 1, key: 64, value: 128 };

pub mod disk;
mod store;
/// Manifest count rows through the codec's checked constructor.
pub fn manifest_counts(rows: &[(&str, u64, u64)]) -> skein_lib::List<crate::codec::component::Count> {
    let mut counts = skein_lib::List::with_capacity(u32::try_from(rows.len()).expect("fixture count"));
    for (family, before, after) in rows {
        let count = crate::codec::component::Count::new(
            &crate::component_v3::CEILINGS,
            crate::codec::component::CountParts {
                family: Box::from(family.as_bytes()),
                before: *before,
                after: *after,
            },
        )
        .expect("fixture family");
        counts.push(count).expect("fixture count");
    }
    counts
}

pub(crate) use crate::store_kit::{
    ALICE, CHILD, CORE_LIMITS, DEPLOYMENT, ENDED, FIRST_MANIFEST_COUNTS, GOAL, GRANDCHILD, INSTALLATION, KEY_LIMIT,
    LIVE, NOTES_LIMITS, OTHER_DEPLOYMENT, PROJECT, TASKS_LIMITS, accounts_key, activity_line, answer_line, attempt_row,
    call, cold_task, configured_model_names, core_key, core_rows, delegate, deployment_record, ended_delegate,
    ended_of, ended_task, escalation_decision, fixture_turn, header_record, history, history_with_proposal, hold_row,
    key, kv_limits, ledger_period, live_task, note_entry, notes_entry, notes_entry_in, notes_key, notes_line,
    notified_line, number, pattern, pattern_key, people_key, people_policy, person, person_proposal, proposal_decision,
    question_line, read, result_index, run_proof, task_record, tasks_key, terminal, turn, update, window_turn,
    with_hold, with_links, with_proposal_rows, with_question_lines, within,
};

pub use store::limits as store_limits;
pub use store::{
    Answer, ROOT, answer, below, commit, committed, committed_with_places, directory, env, events, format_put, get_kv,
    header, header_put, installation_put, layouts_put, opened, opened_with_budget, opened_with_transcripts, over,
    plant, run, save, script, sign_in_secret, stores, written,
};

pub fn codec_limits() -> crate::codec::View {
    crate::codec::limits(&CORE_LIMITS).expect("fixture codec limits")
}
