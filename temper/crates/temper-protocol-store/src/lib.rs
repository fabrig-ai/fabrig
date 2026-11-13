//! Temper's ordered families under jig's application tag (protocol/store.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;

mod patch_set;
#[path = "../patches/mod.rs"]
pub mod patches;
pub use patch_set::PATCH_SET;

use alloc::boxed::Box;
pub mod keys;
pub mod records;
pub use temper_protocol_store_codec::v21;
#[cfg(test)]
mod tests;

/// One version for every durable family (protocol/store.md, section 4).
pub const STORE_VERSION: u32 = 21;

/// The least ceiling on pending, run and blamed landings (protocol/store.md, section 4).
#[must_use]
pub fn tier_landings_ceiling() -> u32 {
    v21::CEILINGS.tier_row_pending.min(v21::CEILINGS.tier_run_landings).min(v21::CEILINGS.tier_finding_blamed)
}

/// The most `forge.changes` the tier rows hold at two landings per change plus one.
#[must_use]
pub fn forge_changes_bound() -> Option<u32> {
    tier_landings_ceiling().checked_sub(1)?.checked_div(2)
}

/// Largest encodings under the forge's and task spec's bounds, before jig's key tag.
#[must_use]
pub fn application(
    limits: &temper_engine_domain_forge_vocabulary::Limits,
    change_words: u32,
) -> Option<jig_protocol_store::sizes::Application> {
    let bounds = bounds(limits, change_words)?;
    Some(jig_protocol_store::sizes::Application {
        version: STORE_VERSION,
        key: keys::largest(limits)?,
        value: u32::try_from(v21::Row::encoded_bound(bounds.limits())?).ok()?,
    })
}

/// A malformed row, which the routing reports before stopping the engine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Undecodable {
    Malformed,
    /// The setting's identity survives refusal of a retired stored value.
    Setting(temper_engine_domain_forge_vocabulary::settings::SettingKey),
}

/// Schema limits checked once alongside the forge's key bounds.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Bounds {
    // Keep schema limits out of the store's by-value construction frames.
    limits: Box<v21::Limits>,
    forge: temper_engine_domain_forge_vocabulary::Limits,
}
impl Bounds {
    /// The generated decoder's limits.
    #[must_use]
    pub const fn limits(&self) -> &v21::Limits {
        &self.limits
    }
    /// The domain limits for names and key decoding.
    #[must_use]
    pub const fn forge(&self) -> &temper_engine_domain_forge_vocabulary::Limits {
        &self.forge
    }
}

/// Make schema bounds from the forge and task spec bytes, refusing any ceiling overflow.
#[must_use]
pub fn bounds(forge: &temper_engine_domain_forge_vocabulary::Limits, change_words: u32) -> Option<Bounds> {
    let limits = v21::Limits {
        closer_selections: forge.tasks,
        closing_question_words: forge.client.op_bytes.max(change_words),
        closing_asked_answer_some: forge.client.op_bytes.max(change_words),
        // A change's notices/accounts fit 512 bytes; `Change ` + u64 + `: ` adds 29.
        closing_message_words: jig_domain_tasks::HOLD_ACCOUNT_BYTES.checked_add(29)?,
        status_brief_sentence: 200,
        status_brief_lede: 600,
        effect_account_words: temper_engine_domain_forge_vocabulary::EFFECT_ACCOUNT_BYTES,
        // Explicit titles fit the forge; the body retains the task words.
        change_row_title: 256,
        change_row_body: change_words,
        follow_question_words: forge.client.op_bytes.max(change_words),
        follow_question_answer_some: forge.client.op_bytes.max(change_words),
        change_failure_line: change_words,
        change_row_report: forge.client.op_bytes,
        held_answer: forge.client.op_bytes,
        open_words: forge.client.op_bytes,
        dispatch_input_name: 255,
        dispatch_input_value: forge.client.op_bytes,
        dispatch_write_workflow: 255,
        dispatch_write_branch: forge.name_bytes,
        adopted_workflows: u32::try_from(temper_engine_domain_forge_vocabulary::WORKFLOWS).ok()?,
        adopted_workflows_item: u32::try_from(temper_engine_domain_forge_vocabulary::WORKFLOW_BYTES).ok()?,
        dispatch_write_inputs: u32::try_from(skein_forge::api::INPUTS).ok()?,
        set_status_description: 140,
        set_status_link: 2048,
        create_issue_key: forge.client.op_bytes,
        create_issue_title: forge.client.op_bytes,
        create_issue_body: forge.client.op_bytes,
        open_pull_title: forge.client.op_bytes,
        open_pull_body: forge.client.op_bytes,
        open_pull_head: forge.client.op_bytes,
        open_pull_base: forge.client.op_bytes,
        release_row_comment_some: forge.client.op_bytes,
        post_key: forge.client.op_bytes,
        post_body: forge.client.op_bytes,
        post_review_key: forge.client.op_bytes,
        post_review_body: forge.client.op_bytes,
        edit_title_some: forge.client.op_bytes,
        edit_body_some: forge.client.op_bytes,
        set_reviewers_reviewers: forge.client.rows.min(forge.client.op_bytes.checked_div(8)?),
        set_status_context: forge.client.op_bytes,
        create_branch_branch: forge.client.op_bytes,
        delete_branch_branch: forge.client.op_bytes,
        merge_condition_base: forge.client.op_bytes,
        segment_name: forge.name_bytes,
        segments_segments: forge.name_bytes.checked_mul(8)?,
        protection_branch: forge.client.answer_bytes,
        protection_contexts: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(8)?)?,
        settings_default_branch: forge.client.answer_bytes,
        permission_value_login: forge.name_bytes,
        adopted_host: forge.name_bytes,
        adopted_owner: forge.name_bytes,
        adopted_name: forge.name_bytes,
        adopted_prefix: forge.name_bytes,
        adopted_checks: forge.change_policy.gates,
        adopted_contexts: forge.judge_projects,
        retargeted_before: forge.name_bytes,
        retargeted_after: forge.name_bytes,
        names_resources: forge.resources_per_task,
        hold_design_writers: forge.holds,
        landings_topic_branch: forge.name_bytes,
        subscriber_goal_tasks_some: forge.issue_policy.plan_items,
        subscriber_paths: forge.paths_per_subscription,
        subscriber_paths_item: forge.name_bytes,
        ..records::limits(forge)?
    };
    v21::check_limits(&limits).ok()?;
    Some(Bounds { limits: Box::new(limits), forge: *forge })
}

/// Restart families below byte 128, relative to jig's application tag.
#[must_use]
pub fn interval() -> skein_kv::Range {
    skein_kv::Range { start: Box::new([]), end: Some(Box::new([128])) }
}
