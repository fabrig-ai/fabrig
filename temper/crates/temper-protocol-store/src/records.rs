//! Total translations between durable domain rows and generated records.
use crate::{Bounds, Undecodable, v25 as w};
use alloc::boxed::Box;
use skein_codec::Record;
use skein_lib::Reader;
use temper_engine_domain_forge_vocabulary as d;
pub mod by_hand;

/// Context for owner-shaped maps (skein/docs/design/data/codec/maps.md, 4.1).
/// Piece bounds, old-log acceptance and a posting's failing IDs stay the owner's.
/// Translations needing none of those use `UNBOUNDED`.
#[derive(Debug)]
pub struct Context {
    bounds: Option<Bounds>,
    follow_files: u32,
    answer_name_bytes: u32,
    logged_landing_tiers: u32,
    posted_failures: temper_engine_domain_forge_tiers::Failures,
}
impl Context {
    /// No extra owner bounds; used for encoding and context-free translations.
    pub const UNBOUNDED: Self = Self {
        bounds: None,
        follow_files: u32::MAX,
        answer_name_bytes: w::CEILINGS.answer_name_name,
        logged_landing_tiers: w::CEILINGS.logged_landing_tiers,
        posted_failures: temper_engine_domain_forge_tiers::Failures::Many,
    };

    fn bounded(bounds: &Bounds) -> Self {
        Self { bounds: Some(bounds.clone()), ..Self::UNBOUNDED }
    }

    fn bounds(&self) -> &Bounds {
        self.bounds.as_ref().expect("bounded store translation")
    }
}

mod answers;
mod api;
mod application;
mod away;
mod change;
mod client;
mod closer;
mod drafts;
mod findings;
mod follows;
mod reds;
#[cfg(test)]
pub(crate) use api::encode_error;
mod entries;
mod forges;
mod history;
mod issues;
mod landing;
mod landing_log;
mod limit_holds;
mod making;
mod milestones;
mod names;
mod settings;
mod status;
mod tiers;
mod topics;
mod verification;
const FITS: &str = "admitted fields fit the schema";

/// Encode every durable family admitted by the domain.
#[must_use]
pub fn encode(value: &d::Stored) -> Box<[u8]> {
    let value = match value {
        d::Stored::HistoryProject(row) => w::Value::HistoryProject(history::encode_project(row)),
        d::Stored::HistoryEvent(row) => w::Value::HistoryEvent(history::encode_event(row)),
        d::Stored::HistoryChange(row) => w::Value::HistoryChange(history::encode_change_index(row)),
        d::Stored::HistoryLanding(row) => w::Value::HistoryLanding(history::encode_landing_index(row)),
        d::Stored::HistoryLast(row) => w::Value::HistoryLast(history::encode_last(row)),

        d::Stored::Follow(row) => w::Value::Follow(Box::new(follows::encode(row))),
        d::Stored::Closer(row) => w::Value::Closer(closer::encode(row)),
        d::Stored::PersonBrief(row) => w::Value::PersonBrief(status::encode_person_brief(row)),
        d::Stored::ProjectFacts(row) => w::Value::ProjectFacts(status::encode_facts(row)),
        d::Stored::Device(row) => w::Value::Device(away::encode_device(row)),
        d::Stored::LandedDraft(row) => w::Value::LandedDraft(drafts::encode(row)),
        d::Stored::Away(row) => w::Value::Away(away::encode_away(row)),
        d::Stored::RedPass(row) => w::Value::RedPass(reds::encode_pass(row)),
        d::Stored::Red(row) => w::Value::Red(reds::encode(row)),
        d::Stored::RedNumber { last } => w::Value::RedNumber(w::RedNumber { last: *last }),
        d::Stored::Finding(row) => w::Value::Finding(findings::encode(row)),
        d::Stored::Decided(row) => w::Value::Decided(away::encode_decided(row)),
        d::Stored::Looked(row) => w::Value::Looked(status::encode_looked(row)),
        d::Stored::Making(row) => w::Value::Making(making::encode(row)),
        d::Stored::RefusedSetting(_) => unreachable!("a refused setting is never saved"),
        d::Stored::Setting(row) => w::Value::Setting(settings::encode(row)),
        d::Stored::Tier(row) => w::Value::Tier(Box::new(tiers::encode(row))),
        d::Stored::Milestone(row) => w::Value::Milestone(milestones::encode(row)),
        d::Stored::LandingLog(row) => w::Value::LandingLog(landing_log::encode(row)),
        d::Stored::Verification(row) => w::Value::Verification(verification::encode(row)),
        d::Stored::Forge(row) => w::Value::Forge(forges::encode(row)),
        d::Stored::ForgeNumber { last } => w::Value::ForgeNumber(w::ForgeNumber { last: *last }),
        d::Stored::Playground(row) => w::Value::Playground(forges::encode_playground(row)),
        d::Stored::Landing(row) => w::Value::Landing(Box::new(landing::encode(row))),
        d::Stored::Receipt { key, receipt } => w::Value::Receipt(landing::encode_receipt(key, receipt)),
        d::Stored::Application { token, until, outcome } => {
            w::Value::Application(application::encode(*token, *until, outcome))
        }
        d::Stored::Repository(value) => w::Value::Repository(names::encode_adopted(value)),
        d::Stored::Hold(value) => w::Value::Hold(names::encode_hold(value)),
        d::Stored::LimitHold(row) => w::Value::LimitHold(limit_holds::encode(row)),
        d::Stored::Names { task, resources } => w::Value::Names(topics::encode_names(*task, resources)),
        d::Stored::Subscription(value) => w::Value::Subscription(topics::encode_subscriber(value)),
        d::Stored::BranchHead(value) => w::Value::BranchHead(topics::encode_branch_head(value)),
        d::Stored::PullState(value) => w::Value::PullState(topics::encode_pull_state(value)),
        d::Stored::Ci(value) => w::Value::Ci(topics::encode_ci_state(value)),
        d::Stored::Landed { commit, task } => w::Value::Landed(topics::encode_landed(*commit, *task)),
        d::Stored::Release(value) => w::Value::Release(topics::encode_release(value)),
        d::Stored::ProposedEffect(value) => w::Value::ProposedEffect(entries::encode_proposed(value)),
        d::Stored::Entry { entry, held } => w::Value::Entry(entries::encode_entry_row(entry, *held)),
        d::Stored::Call { key, answer } => w::Value::Call(entries::encode_call(key, answer)),
        d::Stored::Dismissed { party, what } => {
            let what = crate::generated::store::person_fact_to_record(what);
            w::Value::Dismissed(w::Dismissed { party: *party, what })
        }
        d::Stored::Binding { address, subject } => {
            w::Value::Binding(w::Binding { address: address.clone(), subject: subject.clone() })
        }
        d::Stored::Flight { task, entry } => w::Value::Flight(w::Flight { task: *task, entry: *entry }),
        d::Stored::Writer(value) => w::Value::Writer(crate::generated::store::forge_writer_to_record(value)),
        d::Stored::Client(value) => w::Value::Client(client::encode(value)),
        d::Stored::ChangeCold(value) => w::Value::ChangeCold(Box::new(change::encode_change_cold(value))),
        d::Stored::ChangeHot(value) => w::Value::ChangeHot(Box::new(change::encode_change_hot(value))),
        d::Stored::Change(_) => unreachable!("whole changes are saved as two parts"),
        d::Stored::Issue(value) => w::Value::Issue(Box::new(issues::encode(value))),
    };
    let row = w::Row { value };
    row.encode(&w::CEILINGS).expect("admitted row fits the schema")
}

/// Decode one supported row with the checked startup bounds.
pub fn decode(bytes: &[u8], bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    let row = match w::Row::decode(bounds.limits(), &mut Reader::new(bytes)) {
        Ok(row) => row,
        Err(_) => {
            // Earlier logs kept up to the schema's tier ceiling. The domain's
            // restore has the repository settings and retires removed entries.
            let mut earlier = *bounds.limits();
            earlier.logged_landing_tiers = w::CEILINGS.logged_landing_tiers;
            w::Row::decode(&earlier, &mut Reader::new(bytes)).or(Err(Undecodable::Malformed))?
        }
    };
    Ok(match row.value {
        w::Value::RedPass(row) => d::Stored::RedPass(reds::decode_pass(row, bounds)?),
        w::Value::Red(row) => d::Stored::Red(reds::decode(row, bounds)?),
        w::Value::RedNumber(row) => d::Stored::RedNumber { last: row.last },
        w::Value::HistoryProject(row) => d::Stored::HistoryProject(history::decode_project(row)),
        w::Value::HistoryEvent(row) => d::Stored::HistoryEvent(history::decode_event(row)),
        w::Value::HistoryChange(row) => d::Stored::HistoryChange(history::decode_change_index(row)),
        w::Value::HistoryLanding(row) => d::Stored::HistoryLanding(history::decode_landing_index(row)),
        w::Value::HistoryLast(row) => d::Stored::HistoryLast(history::decode_last(row)),

        w::Value::Follow(row) => d::Stored::Follow(follows::decode(*row, bounds)?),
        w::Value::Closer(row) => d::Stored::Closer(closer::decode(row)),
        w::Value::PersonBrief(row) => d::Stored::PersonBrief(status::decode_person_brief(row)),
        w::Value::ProjectFacts(row) => d::Stored::ProjectFacts(status::decode_facts(row)),
        w::Value::Device(row) => d::Stored::Device(away::decode_device(row)?),
        w::Value::LandedDraft(row) => d::Stored::LandedDraft(drafts::decode(row)?),
        w::Value::Away(row) => d::Stored::Away(away::decode_away(row)),
        w::Value::Finding(row) => d::Stored::Finding(findings::decode(row)?),
        w::Value::Decided(row) => d::Stored::Decided(away::decode_decided(row)?),
        w::Value::Looked(row) => d::Stored::Looked(status::decode_looked(row)),
        w::Value::Making(row) => d::Stored::Making(making::decode(row)),
        w::Value::Setting(row) => d::Stored::Setting(settings::decode(row, bounds)?),
        w::Value::Tier(row) => d::Stored::Tier(tiers::decode(*row)?),
        w::Value::Milestone(row) => d::Stored::Milestone(milestones::decode(row)),
        w::Value::LandingLog(row) => {
            d::Stored::LandingLog(landing_log::decode(row, bounds.limits().logged_landing_tiers)?)
        }
        w::Value::Verification(row) => d::Stored::Verification(verification::decode(row)?),
        w::Value::Forge(row) => d::Stored::Forge(forges::decode(row)?),
        w::Value::ForgeNumber(row) => {
            let w::ForgeNumber { last } = row;
            d::Stored::ForgeNumber { last }
        }
        w::Value::Playground(row) => d::Stored::Playground(forges::decode_playground(row)),
        w::Value::Landing(row) => d::Stored::Landing(landing::decode(*row, bounds)?),
        w::Value::Receipt(row) => landing::decode_receipt(row)?,
        w::Value::Application(value) => application::decode(value, bounds)?,
        w::Value::Issue(value) => d::Stored::Issue(issues::decode(*value)?),
        w::Value::ChangeCold(value) => d::Stored::ChangeCold(Box::new(change::decode_change_cold(*value))),
        w::Value::ChangeHot(value) => d::Stored::ChangeHot(Box::new(change::decode_change_hot(*value)?)),
        w::Value::Dismissed(value) => {
            let w::Dismissed { party, what } = value;
            let what = crate::generated::store::person_fact_to_domain(what);
            d::Stored::Dismissed { party, what }
        }
        w::Value::Binding(value) => {
            let w::Binding { address, subject } = value;
            d::Stored::Binding { address, subject }
        }
        w::Value::Flight(row) => {
            let w::Flight { task, entry } = row;
            d::Stored::Flight { task, entry }
        }
        w::Value::Writer(value) => d::Stored::Writer(crate::generated::store::forge_writer_to_domain(value)),
        w::Value::Client(value) => d::Stored::Client(client::decode(value, bounds)?),
        w::Value::Call(value) => entries::decode_call(value, bounds)?,
        w::Value::ProposedEffect(value) => d::Stored::ProposedEffect(entries::decode_proposed(value, bounds)?),
        w::Value::Entry(value) => entries::decode_entry_row(value),
        w::Value::Repository(value) => d::Stored::Repository(names::decode_adopted(value, bounds)?),
        w::Value::Hold(value) => d::Stored::Hold(names::decode_hold(value, bounds)?),
        w::Value::LimitHold(row) => d::Stored::LimitHold(limit_holds::decode(row, bounds)?),
        w::Value::Names(value) => {
            let w::Names { task, resources } = value;
            d::Stored::Names { task, resources: topics::decode_names(resources, bounds)? }
        }
        w::Value::Subscription(value) => d::Stored::Subscription(topics::decode_subscriber(value)),
        w::Value::BranchHead(value) => d::Stored::BranchHead(topics::decode_branch_head(value, bounds)?),
        w::Value::PullState(value) => d::Stored::PullState(topics::decode_pull_state(value, bounds)?),
        w::Value::Ci(value) => d::Stored::Ci(topics::decode_ci_state(value)),
        w::Value::Landed(value) => {
            let w::Landed { commit, task } = value;
            d::Stored::Landed { commit, task }
        }
        w::Value::Release(value) => d::Stored::Release(topics::decode_release(value, bounds)?),
    })
}

/// Restore a decoded record only when its durable key agrees with the row.
pub fn row(key: &[u8], value: &[u8], bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    match raw_row(key, value, bounds) {
        Ok(row) => Ok(row),
        Err(Undecodable::Malformed) => match crate::lowering::lowering(key, value, bounds) {
            Some(lowered) => Err(Undecodable::Lowered(lowered)),
            None => Err(Undecodable::Malformed),
        },
        Err(why @ (Undecodable::Setting(_) | Undecodable::Lowered(_))) => Err(why),
    }
}

pub(crate) fn raw_row(key: &[u8], value: &[u8], bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    let row = match decode(value, bounds) {
        Ok(row) => row,
        Err(Undecodable::Setting(setting)) => {
            let encoded = crate::keys::encode(&d::Key::Setting(setting), u32::MAX).ok_or(Undecodable::Malformed)?;
            return if encoded.as_ref() == key {
                Err(Undecodable::Setting(setting))
            } else {
                Err(Undecodable::Malformed)
            };
        }
        Err(Undecodable::Malformed | Undecodable::Lowered(_)) => return Err(Undecodable::Malformed),
    };
    let encoded = crate::keys::encode(&d::stored_key(&row), u32::MAX).ok_or(Undecodable::Malformed)?;
    if encoded.as_ref() != key {
        return Err(Undecodable::Malformed);
    }
    Ok(row)
}

pub(crate) fn limits(forge: &d::Limits) -> Option<w::Limits> {
    Some(w::Limits {
        adopted_refusals: forge.changes,
        train_refusal_branch: forge.name_bytes,
        jig_refusal_why: d::FORMING_REASON_BYTES,
        change_follows_commits: forge.follow_merge,
        follow_source_paths: forge.follow_paths,
        follow_source_branch: forge.name_bytes,
        follow_target_branch: forge.name_bytes,
        landing_rule_gates: forge.change_policy.gates,
        landing_rule_approvals: forge.roles,
        landing_rule_sets: 6,
        rule_pattern_segments: forge.authority_segments.checked_mul(forge.name_bytes.checked_add(8)?)?,
        effect_account_words: temper_engine_domain_forge_vocabulary::EFFECT_ACCOUNT_BYTES,
        answer_name_name: forge.client.answer_bytes,
        making_key_org: forge.name_bytes.min(1024),
        making_key_name: forge.name_bytes.min(1024),
        make_into_host: forge.name_bytes.min(1024),
        making_made_branch: forge.name_bytes.min(1024),
        binding_address: 254,
        binding_subject: 255,
        application_adopted_owner: forge.name_bytes,
        application_adopted_name: forge.name_bytes,
        application_adopted_landing: forge.name_bytes,
        plan_item_text: forge.issue_policy.body_bytes,
        milestone_text: forge.issue_policy.comment_bytes,
        goal_view_title: forge.issue_policy.title_bytes,
        goal_view_goal_text: forge.issue_policy.body_bytes,
        goal_view_plan: forge.issue_policy.plan_items,
        goal_view_milestones: forge.issue_policy.milestones,
        goal_view_finished_some: forge.issue_policy.comment_bytes,
        projected_milestones: forge.issue_policy.milestones,
        gating_asked: forge.change_policy.gates,
        change_gates: forge.change_policy.gates,
        change_clean: forge.change_policy.clean_heads,
        change_gate_failures: forge.change_policy.gates,
        questions_open: forge.change_policy.questions.checked_add(1)?,
        result_text_bytes: forge.client.answer_bytes,
        change_cold_branch: forge.name_bytes,
        change_cold_base: forge.name_bytes,
        change_hot_verdicts: forge.change_policy.gates,
        change_hot_gate_remarks: forge.change_policy.gates,
        gate_remark_words: forge.client.answer_bytes,
        live_branch_branch: forge.name_bytes,
        live_record_comments: forge.client.inbox,
        live_record_reviews: forge.client.inbox,
        live_record_echoes: forge.client.rows,
        status_context: forge.client.answer_bytes,
        status_description: forge.client.answer_bytes,
        status_url: forge.client.answer_bytes,
        remark_path: forge.client.answer_bytes,
        remark_body: forge.client.answer_bytes,
        file_path: forge.client.answer_bytes,
        file_before_some: forge.client.answer_bytes,
        file_after_some: forge.client.answer_bytes,
        statuses_answer_statuses: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        remarks_answer_remarks: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        pull_files_answer_files: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        compare_answer_files: forge.client.rows,
        compare_answer_files_item: forge.client.answer_bytes,
        checks_answer_statuses: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        collaborators_answer_collaborators: forge.client.rows,
        summary_labels: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(8)?)?,
        summary_key_some: forge.client.answer_bytes,
        summary_title: forge.client.answer_bytes,
        summary_body: forge.client.answer_bytes,
        comment_key_some: forge.client.answer_bytes,
        comment_body: forge.client.answer_bytes,
        review_key_some: forge.client.answer_bytes,
        review_body: forge.client.answer_bytes,
        items_answer_items: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        item_answer_comments: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        reviews_answer_reviews: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        pull_reviewers: forge.client.rows,
        pull_head: forge.client.answer_bytes,
        pull_base: forge.client.answer_bytes,
        step_name: forge.client.answer_bytes,
        job_answer_log: forge.client.answer_bytes,
        directory_entry_name: forge.client.answer_bytes,
        directory_answer_path: forge.client.answer_bytes,
        // A stored entry uses fewer bytes than its public fixed entry charge.
        directory_answer_entries: forge.client.answer_bytes,
        file_answer_path: forge.client.answer_bytes,
        file_answer_bytes: forge.client.answer_bytes,
        branches_answer_branches: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(8)?)?,
        ..tier_limits(forge)?
    })
}

#[cfg(test)]
#[path = "tests/holds.rs"]
mod holds;

fn tier_limits(forge: &d::Limits) -> Option<w::Limits> {
    Some(w::Limits {
        empty_intent_org: forge.name_bytes,
        empty_intent_repository: forge.name_bytes,
        landing_state_members: forge.landing_members,
        landing_state_branch: forge.name_bytes,
        landing_member_gates: forge.gates,
        landing_by_test_gates: forge.gates,
        landing_by_test_probed: forge.gates.checked_mul(forge.landing_members)?,
        landing_selection_tests: forge.tiers.failing_tests,
        landing_searched_changes: forge.landing_members,
        landing_last_prepare_suffix: forge.landing_members,
        landing_member_branch: forge.name_bytes,
        landing_row_asks: forge.landing_members,
        landing_row_entries: forge.entries,
        landing_follow_reruns: forge.client.rows,
        landing_row_flake_reruns: forge.entries,
        receipt_name_name: 32,
        receipt_run_url: 2048,
        receipt_test_key: forge.tiers.test_id_bytes.min(4096),
        receipt_tests_tests: forge.tiers.failing_tests,
        follow_row_branch: forge.name_bytes,
        follow_row_stopped_some: forge.follow_reason_bytes,
        follow_row_queue: forge.follow_queue,
        // Only the head change produces, and it relays one question at a time.
        follow_row_questions: 1,
        follow_question_words: forge.client.op_bytes,
        follow_question_answer_some: forge.client.op_bytes,
        follow_row_skips: forge.follow_skips,
        follow_entry_commits: forge.follow_commits,
        follow_files_files: forge.client.answer_bytes.checked_add(forge.follow_files.checked_mul(4)?)?,
        follow_commit_message: forge.follow_message_bytes,
        follow_skip_reason: forge.follow_reason_bytes,
        follow_reason_reason: forge.follow_reason_bytes,
        follow_reason_act_reason: forge.follow_reason_bytes,
        follow_merge_act_afters: forge.follow_merge,
        tier_row_observations: 8_192,
        tier_row_tier: 32,
        tier_row_goals: forge.holds,
        tier_row_pending: forge.changes.checked_mul(2)?.checked_add(1)?,
        tier_posting_landings: forge.changes.checked_mul(2)?.checked_add(1)?,
        tier_posting_comments: d::tiers::posting_comments(forge)?,
        tier_posted_tests_tests: forge.tiers.failing_tests,
        tier_posting_link: 2048,
        tier_posting_description: 140,
        tier_flight_pending: forge.client.rows,
        tier_probe_waiting: forge.changes.checked_mul(2)?.checked_add(1)?,
        tier_group_tests: forge.tiers.failing_tests,
        tier_hold_reason: 1024,
        tier_run_landings: forge.changes.checked_mul(2)?.checked_add(1)?,
        tier_finding_blamed: forge.changes.checked_mul(2)?.checked_add(1)?,
        tier_finding_flakes: forge.tiers.failing_tests,
        tier_record_flakes: forge.tiers.failing_tests,
        tier_flake_test: forge.tiers.test_id_bytes,
        tier_tests_tests: forge.tiers.failing_tests,
        many_tests_tests: forge.tiers.failing_tests.min(w::CEILINGS.many_tests_tests),
        shown_test_test_id: forge.tiers.test_id_bytes,
        logged_red_tests: forge.tiers.failing_tests.min(d::LOGGED_TESTS),
        landing_log_landings: forge.landings_kept.checked_add(forge.landings_pending)?,
        logged_landing_tiers: forge.tiers.tiers,
        logged_landing_changes: forge.landing_members,
        logged_landing_branch: forge.name_bytes,
        verification_changes: forge.changes,
        verification_tiers: forge.tiers.tiers,
        tier_verification_tier: 32,
        failed_tests_tests: forge.tiers.failing_tests,
        failing_test_test_id: forge.tiers.test_id_bytes,
        history_formed_members: forge.landing_members,
        history_proved_proofs: forge.gates,
        ..w::CEILINGS
    })
}
