//! Total translations between durable domain rows and generated records.
use crate::{Bounds, Undecodable, v11 as w};
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};
use temper_engine_domain_forge as d;
mod answers;
mod api;
mod application;
mod away;
mod change;
mod client;
mod closer;
mod drafts;
mod follows;
#[cfg(test)]
pub(crate) use api::encode_error;
mod entries;
mod forges;
mod issues;
mod landing;
mod landing_log;
mod making;
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
        d::Stored::Follow(row) => w::Value::Follow(Box::new(follows::encode(row))),
        d::Stored::Closer(row) => w::Value::Closer(closer::encode(row)),
        d::Stored::ProjectFacts(row) => w::Value::ProjectFacts(status::encode_facts(row)),
        d::Stored::LandedDraft(row) => w::Value::LandedDraft(drafts::encode(row)),
        d::Stored::Away(row) => w::Value::Away(away::encode_away(row)),
        d::Stored::Decided(row) => w::Value::Decided(away::encode_decided(row)),
        d::Stored::Looked(row) => w::Value::Looked(status::encode_looked(row)),
        d::Stored::Making(row) => w::Value::Making(making::encode(row)),
        d::Stored::RefusedSetting(_) => unreachable!("a refused setting is never saved"),
        d::Stored::Setting(row) => w::Value::Setting(settings::encode(row)),
        d::Stored::Tier(row) => w::Value::Tier(Box::new(tiers::encode(row))),
        d::Stored::LandingLog(row) => w::Value::LandingLog(landing_log::encode(row)),
        d::Stored::Verification(row) => w::Value::Verification(verification::encode(row)),
        d::Stored::Forge(row) => w::Value::Forge(forges::encode(row)),
        d::Stored::ForgeNumber { last } => {
            w::Value::ForgeNumber(w::ForgeNumber::new(&w::CEILINGS, w::ForgeNumberParts { last: *last }).expect(FITS))
        }
        d::Stored::Playground(row) => w::Value::Playground(forges::encode_playground(row)),
        d::Stored::Landing(row) => w::Value::Landing(landing::encode(row)),
        d::Stored::Receipt { key, receipt } => w::Value::Receipt(landing::encode_receipt(key, receipt)),
        d::Stored::Application { token, until, outcome } => {
            w::Value::Application(application::encode(*token, *until, outcome))
        }
        d::Stored::Repository(value) => w::Value::Repository(names::encode_adopted(value)),
        d::Stored::Hold(value) => w::Value::Hold(names::encode_hold(value)),
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
            let what = match what {
                d::PersonFact::KeepAsking { project, subject, at } => w::PersonFact::KeepAsking(
                    w::KeepAsking::new(
                        &w::CEILINGS,
                        w::KeepAskingParts { project: *project, subject: encode_subject(*subject), at: *at },
                    )
                    .expect(FITS),
                ),
                d::PersonFact::ToBegin { project } => w::PersonFact::ToBegin(
                    w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: *project }).expect(FITS),
                ),
                d::PersonFact::GoalFrom { project, chat, goal, at } => w::PersonFact::GoalFrom(
                    w::GoalFrom::new(
                        &w::CEILINGS,
                        w::GoalFromParts { project: *project, chat: *chat, goal: *goal, at: *at },
                    )
                    .expect(FITS),
                ),
            };
            w::Value::Dismissed(w::Dismissed::new(&w::CEILINGS, w::DismissedParts { party: *party, what }).expect(FITS))
        }
        d::Stored::Binding { address, subject } => w::Value::Binding(
            w::Binding::new(&w::CEILINGS, w::BindingParts { address: address.clone(), subject: subject.clone() })
                .expect(FITS),
        ),
        d::Stored::Flight { task, entry } => {
            w::Value::Flight(w::Flight::new(&w::CEILINGS, w::FlightParts { task: *task, entry: *entry }).expect(FITS))
        }
        d::Stored::Writer(value) => w::Value::Writer(
            w::ForgeWriter::new(&w::CEILINGS, w::ForgeWriterParts { forge: value.forge, author: value.author })
                .expect(FITS),
        ),
        d::Stored::Client(value) => w::Value::Client(client::encode(value)),
        d::Stored::Change(value) => w::Value::Change(Box::new(change::encode_change_row(value))),
        d::Stored::Issue(value) => w::Value::Issue(Box::new(issues::encode(value))),
    };
    let row = w::Row::new(&w::CEILINGS, w::RowParts { value }).expect("admitted row fits the schema");
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("row length fits"));
    row.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Decode one supported row with the checked startup bounds.
pub fn decode(bytes: &[u8], bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    let row = w::Row::decode(bounds.limits(), &mut Reader::new(bytes)).or(Err(Undecodable::Malformed))?;
    Ok(match row.into_parts().value {
        w::Value::Follow(row) => d::Stored::Follow(follows::decode(*row, bounds)?),
        w::Value::Closer(row) => d::Stored::Closer(closer::decode(row)),
        w::Value::ProjectFacts(row) => d::Stored::ProjectFacts(status::decode_facts(row)),
        w::Value::LandedDraft(row) => d::Stored::LandedDraft(drafts::decode(row)?),
        w::Value::Away(row) => d::Stored::Away(away::decode_away(row)),
        w::Value::Decided(row) => d::Stored::Decided(away::decode_decided(row)?),
        w::Value::Looked(row) => d::Stored::Looked(status::decode_looked(row)),
        w::Value::Making(row) => d::Stored::Making(making::decode(row)),
        w::Value::Setting(row) => d::Stored::Setting(settings::decode(row, bounds)?),
        w::Value::Tier(row) => d::Stored::Tier(tiers::decode(*row)?),
        w::Value::LandingLog(row) => d::Stored::LandingLog(landing_log::decode(row)),
        w::Value::Verification(row) => d::Stored::Verification(verification::decode(row)?),
        w::Value::Forge(row) => d::Stored::Forge(forges::decode(row)?),
        w::Value::ForgeNumber(row) => {
            let w::ForgeNumberParts { last } = row.into_parts();
            d::Stored::ForgeNumber { last }
        }
        w::Value::Playground(row) => d::Stored::Playground(forges::decode_playground(row)),
        w::Value::Landing(row) => d::Stored::Landing(landing::decode(row)),
        w::Value::Receipt(row) => landing::decode_receipt(row)?,
        w::Value::Application(value) => application::decode(value, bounds)?,
        w::Value::Issue(value) => d::Stored::Issue(issues::decode(*value)),
        w::Value::Change(value) => d::Stored::Change(Box::new(change::decode_change_row(*value)?)),
        w::Value::Dismissed(value) => {
            let w::DismissedParts { party, what } = value.into_parts();
            let what = match what {
                w::PersonFact::KeepAsking(value) => d::PersonFact::KeepAsking {
                    project: value.project(),
                    subject: decode_subject(value.subject())?,
                    at: value.at(),
                },
                w::PersonFact::ToBegin(value) => d::PersonFact::ToBegin { project: value.into_parts().project },
                w::PersonFact::GoalFrom(value) => d::PersonFact::GoalFrom {
                    project: value.project(),
                    chat: value.chat(),
                    goal: value.goal(),
                    at: value.at(),
                },
            };
            d::Stored::Dismissed { party, what }
        }
        w::Value::Binding(value) => {
            let w::BindingParts { address, subject } = value.into_parts();
            d::Stored::Binding { address, subject }
        }
        w::Value::Flight(row) => {
            let w::FlightParts { task, entry } = row.into_parts();
            d::Stored::Flight { task, entry }
        }
        w::Value::Writer(value) => {
            let w::ForgeWriterParts { forge, author } = value.into_parts();
            d::Stored::Writer(skein_forge_client::Writer { forge, author })
        }
        w::Value::Client(value) => d::Stored::Client(client::decode(value, bounds)?),
        w::Value::Call(value) => entries::decode_call(value, bounds)?,
        w::Value::ProposedEffect(value) => d::Stored::ProposedEffect(entries::decode_proposed(value, bounds)?),
        w::Value::Entry(value) => entries::decode_entry_row(value),
        w::Value::Repository(value) => d::Stored::Repository(names::decode_adopted(value, bounds)?),
        w::Value::Hold(value) => d::Stored::Hold(names::decode_hold(value, bounds)?),
        w::Value::Names(value) => {
            let w::NamesParts { task, resources } = value.into_parts();
            d::Stored::Names { task, resources: topics::decode_names(resources, bounds)? }
        }
        w::Value::Subscription(value) => d::Stored::Subscription(topics::decode_subscriber(value)),
        w::Value::BranchHead(value) => d::Stored::BranchHead(topics::decode_branch_head(value, bounds)?),
        w::Value::PullState(value) => d::Stored::PullState(topics::decode_pull_state(value, bounds)?),
        w::Value::Ci(value) => d::Stored::Ci(topics::decode_ci_state(value)),
        w::Value::Landed(value) => {
            let w::LandedParts { commit, task } = value.into_parts();
            d::Stored::Landed { commit, task }
        }
        w::Value::Release(value) => d::Stored::Release(topics::decode_release(value, bounds)?),
    })
}

/// Restore a decoded record only when its durable key agrees with the row.
pub fn row(key: &[u8], value: &[u8], bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    let row = decode(value, bounds)?;
    let encoded = crate::keys::encode(&d::stored_key(&row), u32::MAX).ok_or(Undecodable::Malformed)?;
    if encoded.as_ref() != key {
        return Err(Undecodable::Malformed);
    }
    Ok(row)
}

pub(crate) fn limits(forge: &d::Limits) -> Option<w::Limits> {
    Some(w::Limits {
        change_follows_commits: forge.follow_merge,
        follow_source_paths: forge.follow_paths,
        follow_source_branch: forge.name_bytes,
        follow_target_branch: forge.name_bytes,
        landing_rule_gates: forge.change_policy.gates,
        landing_rule_approvals: forge.roles,
        landing_rule_sets: 6,
        rule_pattern_segments: forge.authority_segments.checked_mul(forge.name_bytes.checked_add(8)?)?,
        effect_account_words: temper_engine_domain_forge::EFFECT_ACCOUNT_BYTES,
        answer_name_name: forge.client.answer_bytes,
        making_key_org: forge.name_bytes.min(1024),
        making_key_name: forge.name_bytes.min(1024),
        make_into_host: forge.name_bytes.min(1024),
        making_made_branch: forge.name_bytes.min(1024),
        binding_address: 254,
        binding_subject: 255,
        application_adopted_owner: forge.name_bytes,
        application_adopted_name: forge.name_bytes,
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
        change_row_branch: forge.name_bytes,
        change_row_base: forge.name_bytes,
        change_row_verdicts: forge.change_policy.gates,
        change_row_gate_remarks: forge.change_policy.gates,
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
        landing_member_gates: forge.gates,
        landing_row_asks: forge.landing_members,
        landing_row_entries: forge.entries,
        receipt_name_name: 32,
        receipt_run_url: 2048,
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
        tier_flight_pending: forge.client.rows,
        tier_run_landings: forge.changes.checked_mul(2)?.checked_add(1)?,
        tier_finding_blamed: forge.changes.checked_mul(2)?.checked_add(1)?,
        tier_finding_flakes: forge.tiers.failing_tests,
        tier_record_flakes: forge.tiers.failing_tests,
        tier_flake_test: forge.tiers.test_id_bytes,
        tier_tests_tests: forge.tiers.failing_tests,
        landing_log_landings: forge.landings_kept,
        logged_landing_changes: forge.landing_members,
        logged_landing_branch: forge.name_bytes,
        verification_changes: forge.changes,
        verification_tiers: forge.tiers.tiers,
        tier_verification_tier: 32,
        failed_tests_tests: forge.tiers.failing_tests,
        failing_test_test_id: forge.tiers.test_id_bytes,
        ..w::CEILINGS
    })
}

fn encode_subject(subject: temper_protocol_reaches::Subject) -> w::ClassSubject {
    let record = temper_protocol_reaches::subject_record(subject);
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("subject measure"));
    record.encode(&mut writer).expect("measured subject room");
    w::ClassSubject::decode(&w::CEILINGS, &mut Reader::new(&writer.finish())).expect("matching subject records")
}
fn decode_subject(record: &w::ClassSubject) -> Result<temper_protocol_reaches::Subject, Undecodable> {
    use temper_client_protocol_wire::v1;
    let mut writer = Writer::new(usize::try_from(record.measure()).or(Err(Undecodable::Malformed))?);
    record.encode(&mut writer).or(Err(Undecodable::Malformed))?;
    let record = v1::ClassSubject::decode(&v1::CEILINGS, &mut Reader::new(&writer.finish())).or(Err(Undecodable::Malformed))?;
    Ok(temper_protocol_reaches::subject_from(&record))
}
