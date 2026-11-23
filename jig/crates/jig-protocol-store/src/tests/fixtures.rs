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
        let count =
            crate::codec::component::Count { family: Box::from(family.as_bytes()), before: *before, after: *after };
        counts.push(count).expect("fixture count");
    }
    counts
}

pub(crate) use jig_protocol_store_kit::{
    ALICE, CHILD, CORE_LIMITS, DEPLOYMENT, ENDED, FIRST_MANIFEST_COUNTS, GOAL, GRANDCHILD, INSTALLATION, KEY_LIMIT,
    LIVE, NOTES_LIMITS, OTHER_DEPLOYMENT, PROJECT, TASKS_LIMITS, accounts_key, activity_line, answer_line, attempt_row,
    call, cold_task, configured_model_names, core_key, core_rows, delegate, deployment_record, ended_delegate,
    ended_of, ended_task, escalation_decision, fixture_turn, header_record, history, history_with_proposal, hold_row,
    key, ledger_period, live_task, note_entry, notes_entry, notes_entry_in, notes_key, notes_line, notified_line,
    number, pattern, pattern_key, people_key, people_policy, person, person_proposal, proposal_decision, question_line,
    read, result_index, run_proof, task_record, tasks_key, terminal, turn, update, window_turn, with_hold, with_links,
    with_proposal_rows, with_question_lines, within,
};

pub use store::floor_put;
pub use store::limits as store_limits;
pub use store::{
    Answer, ROOT, answer, below, commit, committed, committed_with_places, directory, env, events, format_put, get_kv,
    header, header_put, installation_put, opened, opened_with_budget, opened_with_transcripts, over, plant, reopen,
    run, save, script, sign_in_secret, stores, written,
};

/// A shell's prepared binary texts, with any kept fixture texts the test supplies.
#[expect(clippy::disallowed_methods, reason = "simulate a shell holding immutable schemas for each fixture")]
pub fn prepared(sets: crate::OwnerSets, extra: &[&str]) -> &'static [skein_codec::schema::Schema] {
    let mut schemas = skein_lib::List::uncapped();
    for schema in sets.prepare() {
        schemas.push(schema).expect("uncapped fixture schemas");
    }
    for text in extra {
        let schema = skein_codec::schema::parse(text.as_bytes()).expect("kept fixture text parses");
        schemas.push(schema).expect("uncapped fixture schemas");
    }
    Box::leak(schemas.into_boxed())
}

pub fn codec_limits() -> crate::codec::View {
    crate::codec::limits(&CORE_LIMITS).expect("fixture codec limits")
}

/// Component fixture room follows the effective settings capacities.
pub fn kv_limits() -> skein_kv::Limits {
    let mut limits = jig_protocol_store_kit::kv_limits();
    limits.value = limits.value.max(crate::sizes::settings_value(&CORE_LIMITS).expect("settings history room"));
    limits.page.bytes = limits.page.bytes.max(limits.key.checked_add(limits.value).expect("largest row"));
    limits.chunk = skein_kv::chunk_bytes(limits.key, limits.value).expect("largest row chunk");
    limits
}
