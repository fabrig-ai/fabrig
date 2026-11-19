//! Stock-readable rows past a resolved codec bound (configuration/limit-settings.md, 3).
use crate::{Bounds, Undecodable, bounds, records};
use jig_protocol_store::codec::Lowered;
use temper_engine_domain_forge_vocabulary::Limits;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Stock {
    pub forge: Limits,
    pub change_words: u32,
}

pub(crate) fn lowering(key: &[u8], bytes: &[u8], configured: &Bounds) -> Option<Lowered> {
    let stock = configured.stock.as_ref()?;
    let stock_bounds = bounds(&stock.forge, stock.change_words)?;
    if records::raw_row(key, bytes, &stock_bounds).is_err() {
        return None;
    }
    let mut forge = configured.forge;
    let mut words = configured.change_words;
    for name in SOURCES {
        let low = value(&configured.forge, configured.change_words, name)?;
        let high = value(&stock.forge, stock.change_words, name)?;
        if low >= high {
            continue;
        }
        set(&mut forge, &mut words, name, high)?;
        if !fits(key, bytes, &forge, words) {
            continue;
        }
        let mut refused = low;
        let mut accepted = high;
        for _ in 0_u32..64 {
            if accepted.saturating_sub(refused) <= 1 {
                break;
            }
            let middle = refused.checked_add(accepted.checked_sub(refused)?.checked_div(2)?)?;
            set(&mut forge, &mut words, name, middle)?;
            if fits(key, bytes, &forge, words) {
                accepted = middle;
            } else {
                refused = middle;
            }
        }
        return Some(Lowered { name, configured: u64::from(low), held: u64::from(accepted) });
    }
    None
}
fn fits(key: &[u8], bytes: &[u8], forge: &Limits, words: u32) -> bool {
    let Some(bounds) = bounds(forge, words) else { return false };
    match records::raw_row(key, bytes, &bounds) {
        Ok(_) => true,
        Err(Undecodable::Malformed | Undecodable::Setting(_) | Undecodable::Lowered(_)) => false,
    }
}
const SOURCES: &[&[u8]] = &[
    b"forge.tasks",
    b"forge.holds",
    b"forge.changes",
    b"forge.resources_per_task",
    b"forge.paths_per_subscription",
    b"forge.name_bytes",
    b"authority.segments",
    b"forge.entries",
    b"forge.judge_projects",
    b"forge.landing_members",
    b"forge.gates",
    b"forge.landings_kept",
    b"forge.red_key_bytes",
    b"authority.roles",
    b"forge.follow_paths",
    b"forge.follow_queue",
    b"forge.follow_merge",
    b"forge.follow_commits",
    b"forge.follow_files",
    b"forge.follow_message_bytes",
    b"forge.follow_skips",
    b"forge.follow_reason_bytes",
    b"forge.tiers.tiers",
    b"forge.tiers.failing_tests",
    b"forge.tiers.test_id_bytes",
    b"forge.issue_policy.body_bytes",
    b"forge.issue_policy.comment_bytes",
    b"forge.issue_policy.title_bytes",
    b"forge.issue_policy.plan_items",
    b"forge.issue_policy.milestones",
    b"forge.change_policy.gates",
    b"forge.change_policy.clean_heads",
    b"forge.change_policy.questions",
    b"forge.client.op_bytes",
    b"forge.client.answer_bytes",
    b"forge.client.rows",
    b"forge.client.inbox",
    b"tasks.spec_bytes",
];
fn value(forge: &Limits, words: u32, name: &[u8]) -> Option<u32> {
    Some(match name {
        b"forge.tasks" => forge.tasks,
        b"forge.holds" => forge.holds,
        b"forge.changes" => forge.changes,
        b"forge.resources_per_task" => forge.resources_per_task,
        b"forge.paths_per_subscription" => forge.paths_per_subscription,
        b"forge.name_bytes" => forge.name_bytes,
        b"authority.segments" => forge.authority_segments,
        b"forge.entries" => forge.entries,
        b"forge.judge_projects" => forge.judge_projects,
        b"forge.landing_members" => forge.landing_members,
        b"forge.gates" => forge.gates,
        b"forge.landings_kept" => forge.landings_kept,
        b"forge.red_key_bytes" => forge.red_key_bytes,
        b"authority.roles" => forge.roles,
        b"forge.follow_paths" => forge.follow_paths,
        b"forge.follow_queue" => forge.follow_queue,
        b"forge.follow_merge" => forge.follow_merge,
        b"forge.follow_commits" => forge.follow_commits,
        b"forge.follow_files" => forge.follow_files,
        b"forge.follow_message_bytes" => forge.follow_message_bytes,
        b"forge.follow_skips" => forge.follow_skips,
        b"forge.follow_reason_bytes" => forge.follow_reason_bytes,
        b"forge.tiers.tiers" => forge.tiers.tiers,
        b"forge.tiers.failing_tests" => forge.tiers.failing_tests,
        b"forge.tiers.test_id_bytes" => forge.tiers.test_id_bytes,
        b"forge.issue_policy.body_bytes" => forge.issue_policy.body_bytes,
        b"forge.issue_policy.comment_bytes" => forge.issue_policy.comment_bytes,
        b"forge.issue_policy.title_bytes" => forge.issue_policy.title_bytes,
        b"forge.issue_policy.plan_items" => forge.issue_policy.plan_items,
        b"forge.issue_policy.milestones" => forge.issue_policy.milestones,
        b"forge.change_policy.gates" => forge.change_policy.gates,
        b"forge.change_policy.clean_heads" => forge.change_policy.clean_heads,
        b"forge.change_policy.questions" => forge.change_policy.questions,
        b"forge.client.op_bytes" => forge.client.op_bytes,
        b"forge.client.answer_bytes" => forge.client.answer_bytes,
        b"forge.client.rows" => forge.client.rows,
        b"forge.client.inbox" => forge.client.inbox,
        b"tasks.spec_bytes" => words,
        _ => return None,
    })
}
fn set(forge: &mut Limits, words: &mut u32, name: &[u8], value: u32) -> Option<()> {
    match name {
        b"forge.tasks" => forge.tasks = value,
        b"forge.holds" => forge.holds = value,
        b"forge.changes" => forge.changes = value,
        b"forge.resources_per_task" => forge.resources_per_task = value,
        b"forge.paths_per_subscription" => forge.paths_per_subscription = value,
        b"forge.name_bytes" => forge.name_bytes = value,
        b"authority.segments" => forge.authority_segments = value,
        b"forge.entries" => forge.entries = value,
        b"forge.judge_projects" => forge.judge_projects = value,
        b"forge.landing_members" => forge.landing_members = value,
        b"forge.gates" => forge.gates = value,
        b"forge.landings_kept" => forge.landings_kept = value,
        b"forge.red_key_bytes" => forge.red_key_bytes = value,
        b"authority.roles" => forge.roles = value,
        b"forge.follow_paths" => forge.follow_paths = value,
        b"forge.follow_queue" => forge.follow_queue = value,
        b"forge.follow_merge" => forge.follow_merge = value,
        b"forge.follow_commits" => forge.follow_commits = value,
        b"forge.follow_files" => forge.follow_files = value,
        b"forge.follow_message_bytes" => forge.follow_message_bytes = value,
        b"forge.follow_skips" => forge.follow_skips = value,
        b"forge.follow_reason_bytes" => forge.follow_reason_bytes = value,
        b"forge.tiers.tiers" => forge.tiers.tiers = value,
        b"forge.tiers.failing_tests" => forge.tiers.failing_tests = value,
        b"forge.tiers.test_id_bytes" => forge.tiers.test_id_bytes = value,
        b"forge.issue_policy.body_bytes" => forge.issue_policy.body_bytes = value,
        b"forge.issue_policy.comment_bytes" => forge.issue_policy.comment_bytes = value,
        b"forge.issue_policy.title_bytes" => forge.issue_policy.title_bytes = value,
        b"forge.issue_policy.plan_items" => forge.issue_policy.plan_items = value,
        b"forge.issue_policy.milestones" => forge.issue_policy.milestones = value,
        b"forge.change_policy.gates" => forge.change_policy.gates = value,
        b"forge.change_policy.clean_heads" => forge.change_policy.clean_heads = value,
        b"forge.change_policy.questions" => forge.change_policy.questions = value,
        b"forge.client.op_bytes" => forge.client.op_bytes = value,
        b"forge.client.answer_bytes" => forge.client.answer_bytes = value,
        b"forge.client.rows" => forge.client.rows = value,
        b"forge.client.inbox" => forge.client.inbox = value,
        b"tasks.spec_bytes" => *words = value,
        _ => return None,
    }
    Some(())
}
