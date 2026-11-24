//! Compact ended rows (protocol/store/retention.md, 3–4).
use super::value;
use crate::generated::tasks as mapped;
use crate::tasks_v13 as w;
use jig_domain_tasks as d;
use skein_lib::Wall;

fn result(row: &d::SummaryResult) -> w::SummaryResult {
    w::SummaryResult { kind: mapped::summary_result_kind_to_record(&row.kind), words: row.words.clone() }
}
fn read_result(row: w::SummaryResult, words_cut: bool) -> d::SummaryResult {
    let w::SummaryResult { kind, words } = row;
    d::SummaryResult { kind: mapped::summary_result_kind_to_domain(kind), words, words_cut }
}
pub(super) fn encode(row: &d::TaskSummary) -> w::TaskSummary {
    let words_cut = match &row.result {
        Some(row) => row.words_cut,
        None => false,
    };
    let cuts = u8::from(row.title_cut) | (u8::from(row.reason_cut) << 1_u8) | (u8::from(words_cut) << 2_u8);
    let result = match &row.result {
        Some(row) => Some(result(row)),
        None => None,
    };
    w::TaskSummary {
        number: row.number,
        root: row.root,
        depth: row.depth,
        requester: mapped::party_to_record(&row.requester),
        project: row.project,
        label: row.label,
        flags: mapped::flags_to_record(&row.flags),
        tracked: row.tracked,
        executor: mapped::executor_to_record(&row.executor),
        title: row.title.clone(),
        ending: mapped::summary_ending_to_record(&row.ending),
        reason: row.reason.clone(),
        result,
        cuts,
        created_at: row.created_at.as_nanos(),
        changed: row.changed.as_nanos(),
        last_activity: row.last_activity.as_nanos(),
        ended_at: row.ended_at.as_nanos(),
        numbers: mapped::numbers_to_record(&row.numbers),
        deadline: value::wall(row.deadline),
        funder: mapped::funder_to_record(&row.funder),
        ended_delegates: mapped::summary_endings_to_record(&row.ended_delegates),
        attempt: row.attempt,
        activity: row.activity,
        result_position: row.result_position,
        last_reply: row.last_reply,
    }
}
pub(super) fn decode(row: w::TaskSummary) -> Result<d::TaskSummary, w::Problem> {
    let w::TaskSummary {
        number,
        root,
        depth,
        requester,
        project,
        label,
        flags,
        tracked,
        executor,
        title,
        ending,
        reason,
        result,
        cuts,
        created_at,
        changed,
        last_activity,
        ended_at,
        numbers,
        deadline,
        funder,
        ended_delegates,
        attempt,
        activity,
        result_position,
        last_reply,
    } = row;
    if cuts & !7 != 0 {
        return Err(w::Problem { path: w::paths::TASK_SUMMARY_CUTS, reason: skein_codec::Reason::Bound });
    }
    let result = match result {
        Some(row) => Some(read_result(row, cuts & 4 != 0)),
        None => None,
    };
    Ok(d::TaskSummary {
        number,
        root,
        depth,
        requester: mapped::party_to_domain(requester),
        project,
        label,
        flags: mapped::flags_to_domain(flags),
        tracked,
        executor: mapped::executor_to_domain(executor),
        title,
        title_cut: cuts & 1 != 0,
        ending: mapped::summary_ending_to_domain(ending),
        reason,
        reason_cut: cuts & 2 != 0,
        result,
        created_at: Wall::from_nanos(created_at),
        changed: Wall::from_nanos(changed),
        last_activity: Wall::from_nanos(last_activity),
        ended_at: Wall::from_nanos(ended_at),
        numbers: mapped::numbers_to_domain(numbers),
        deadline: value::read_wall(deadline),
        funder: mapped::funder_to_domain(funder),
        ended_delegates: mapped::summary_endings_to_domain(ended_delegates),
        attempt,
        activity,
        result_position,
        last_reply,
    })
}
