//! Compact ended rows (protocol/store/retention.md, 3–4).
use super::{index, phase, value};
use crate::tasks_v13 as w;
use jig_domain_tasks as d;
use skein_lib::Wall;

pub(super) fn stage(stage: d::RetentionStage) -> w::RetentionStage {
    match stage {
        d::RetentionStage::Turns => w::RetentionStage::Turns,
        d::RetentionStage::Record => w::RetentionStage::Record,
        d::RetentionStage::Summary => w::RetentionStage::Summary,
        d::RetentionStage::Forgotten => w::RetentionStage::Forgotten,
        d::RetentionStage::Proposal => w::RetentionStage::Proposal,
    }
}
pub(super) fn read_stage(stage: w::RetentionStage) -> d::RetentionStage {
    match stage {
        w::RetentionStage::Turns => d::RetentionStage::Turns,
        w::RetentionStage::Record => d::RetentionStage::Record,
        w::RetentionStage::Summary => d::RetentionStage::Summary,
        w::RetentionStage::Forgotten => d::RetentionStage::Forgotten,
        w::RetentionStage::Proposal => d::RetentionStage::Proposal,
    }
}
pub(super) fn retention(row: d::RetentionRow, limits: &w::Limits) -> Result<w::RetentionRow, w::Problem> {
    w::RetentionRow::new(limits, w::RetentionRowParts { stage: stage(row.stage), since: row.since.as_nanos(), number: row.number })
}
pub(super) fn read_retention(row: w::RetentionRow) -> d::RetentionRow {
    let w::RetentionRowParts { stage, since, number } = row.into_parts();
    d::RetentionRow { stage: read_stage(stage), since: Wall::from_nanos(since), number }
}
fn ending(ending: d::SummaryEnding, limits: &w::Limits) -> Result<w::SummaryEnding, w::Problem> {
    Ok(match ending {
        d::SummaryEnding::Done => w::SummaryEnding::Done,
        d::SummaryEnding::Failed => w::SummaryEnding::Failed,
        d::SummaryEnding::Cancelled { by, at } => w::SummaryEnding::Cancelled(w::SummaryCancelled::new(
            limits, w::SummaryCancelledParts { by: value::party(&by, limits)?, at: at.as_nanos() },
        )?),
    })
}
fn read_ending(ending: w::SummaryEnding) -> d::SummaryEnding {
    match ending {
        w::SummaryEnding::Done => d::SummaryEnding::Done,
        w::SummaryEnding::Failed => d::SummaryEnding::Failed,
        w::SummaryEnding::Cancelled(row) => {
            let w::SummaryCancelledParts { by, at } = row.into_parts();
            d::SummaryEnding::Cancelled { by: value::read_party(by), at: Wall::from_nanos(at) }
        }
    }
}
fn result(row: &d::SummaryResult, limits: &w::Limits) -> Result<w::SummaryResult, w::Problem> {
    let kind = match row.kind {
        d::SummaryResultKind::Job => w::SummaryResultKind::Job,
        d::SummaryResultKind::Approved => w::SummaryResultKind::Approved,
        d::SummaryResultKind::Report => w::SummaryResultKind::Report,
        d::SummaryResultKind::Verdict { code } => w::SummaryResultKind::Verdict(w::SummaryVerdict::new(limits, w::SummaryVerdictParts { code })?),
        d::SummaryResultKind::Change { connector, kind, resource } => w::SummaryResultKind::Change(w::SummaryChange::new(limits, w::SummaryChangeParts { connector, kind, resource })?),
        d::SummaryResultKind::NoChange => w::SummaryResultKind::NoChange,
        d::SummaryResultKind::Failed => w::SummaryResultKind::Failed,
    };
    w::SummaryResult::new(limits, w::SummaryResultParts { kind, words: row.words.clone(), words_cut: row.words_cut })
}
fn read_result(row: w::SummaryResult) -> d::SummaryResult {
    let w::SummaryResultParts { kind, words, words_cut } = row.into_parts();
    let kind = match kind {
        w::SummaryResultKind::Job => d::SummaryResultKind::Job,
        w::SummaryResultKind::Approved => d::SummaryResultKind::Approved,
        w::SummaryResultKind::Report => d::SummaryResultKind::Report,
        w::SummaryResultKind::Verdict(row) => d::SummaryResultKind::Verdict { code: row.into_parts().code },
        w::SummaryResultKind::Change(row) => {
            let w::SummaryChangeParts { connector, kind, resource } = row.into_parts();
            d::SummaryResultKind::Change { connector, kind, resource }
        }
        w::SummaryResultKind::NoChange => d::SummaryResultKind::NoChange,
        w::SummaryResultKind::Failed => d::SummaryResultKind::Failed,
    };
    d::SummaryResult { kind, words, words_cut }
}
pub(super) fn encode(row: &d::TaskSummary, limits: &w::Limits) -> Result<w::TaskSummary, w::Problem> {
    let result = match &row.result { Some(row) => Some(result(row, limits)?), None => None };
    w::TaskSummary::new(limits, w::TaskSummaryParts {
        number: row.number, root: row.root, depth: row.depth,
        requester: value::party(&row.requester, limits)?, project: row.project, label: row.label,
        flags: phase::flags(row.flags, limits)?, tracked: row.tracked, executor: value::executor(row.executor, limits)?,
        title: row.title.clone(), title_cut: row.title_cut, ending: ending(row.ending, limits)?,
        reason: row.reason.clone(), reason_cut: row.reason_cut, result,
        created_at: row.created_at.as_nanos(), changed: row.changed.as_nanos(), last_activity: row.last_activity.as_nanos(), ended_at: row.ended_at.as_nanos(),
        numbers: phase::numbers(&row.numbers, limits)?,
        budget: w::Budget::new(limits, w::BudgetParts { spend: row.budget.spend, deadline: value::wall(row.budget.deadline) })?,
        funder: value::funder(row.funder, limits)?, ended_delegates: index::endings(&row.ended_delegates, limits)?,
        attempt: row.attempt, activity: row.activity, result_position: row.result_position, last_reply: row.last_reply,
    })
}
pub(super) fn decode(row: w::TaskSummary) -> Result<d::TaskSummary, w::Problem> {
    let w::TaskSummaryParts { number, root, depth, requester, project, label, flags, tracked, executor,
        title, title_cut, ending, reason, reason_cut, result, created_at, changed, last_activity, ended_at,
        numbers, budget, funder, ended_delegates, attempt, activity, result_position, last_reply } = row.into_parts();
    let w::BudgetParts { spend, deadline } = budget.into_parts();
    let result = match result { Some(row) => Some(read_result(row)), None => None };
    Ok(d::TaskSummary {
        number, root, depth, requester: value::read_party(requester), project, label,
        flags: phase::read_flags(flags), tracked, executor: value::read_executor(executor),
        title, title_cut, ending: read_ending(ending), reason, reason_cut, result,
        created_at: Wall::from_nanos(created_at), changed: Wall::from_nanos(changed),
        last_activity: Wall::from_nanos(last_activity), ended_at: Wall::from_nanos(ended_at),
        numbers: phase::read_numbers(numbers), budget: d::Budget { spend, deadline: value::read_wall(deadline) },
        funder: value::read_funder(funder), ended_delegates: index::read_endings(ended_delegates)?,
        attempt, activity, result_position, last_reply,
    })
}
