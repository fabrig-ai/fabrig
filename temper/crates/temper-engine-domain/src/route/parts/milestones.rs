//! Compose live lines, kept ended lines and exact totals (temper/docs/design/client/forge/parts.md, 3).
use super::{
    Box, CoreRange, Domain, Ended, Env, Limits, List, Making, Map, Phase, Range, TaskLine, core, d, encode, encoded,
    note_load, tasks, unreadable,
};

pub(super) fn milestone(domain: &mut Domain, env: &Env<Limits>, party: u64, row: &TaskLine) -> Making {
    let task = row.chip.number;
    let record = domain.core.tasks.task(task);
    if let Some(live) = record
        && (live.project != row.project || !tasks::gathering::executor(live.executor))
    {
        return unreadable();
    }
    let totals = super::super::milestones::read(domain, row.project, task);
    let count = match &totals {
        Some(totals) => totals.done.saturating_add(totals.failed).saturating_add(totals.closed),
        None => 0,
    };
    let kept = domain.parts.entry(row.project, task).ended.clone();
    let kept = match kept {
        Some(kept) if (record.is_some() || kept.totals.is_some()) && (totals.is_none() || kept.count == count) => kept,
        Some(_) | None => {
            let Some(live) = domain.core.tasks.task(task) else {
                return Making::Load(
                    Phase::Milestone { project: row.project, task },
                    Range::Core(CoreRange::TaskResult { task }),
                );
            };
            let ended = Ended {
                count,
                loaded: count == 0,
                whole: match live.concluded {
                    Some(concluded) => Some(concluded.at),
                    None => None,
                },
                totals: None,
                lines: List::with_capacity(16),
            };
            if count != 0 {
                return delegates(env, party, row.project, task, ended, 0, None);
            }
            domain.parts.entry(row.project, task).ended = Some(ended.clone());
            ended
        }
    };
    if !kept.loaded {
        return delegates(env, party, row.project, task, kept, 0, None);
    }
    let Some(mut totals) = totals.or(kept.totals) else { return unreadable() };
    totals.reds = domain.forge.reds_owed(row.project, task);
    let mut ordered = Map::uncapped();
    if let Some(live) = domain.core.tasks.task(task) {
        for goal in &live.delegates {
            let goal = domain.core.tasks.task(*goal).expect("live delegate");
            let Ok(line) = d::task_line(&domain.core, &crate::limits::environment_core(env).limits, party, goal, None)
            else {
                return unreadable();
            };
            ordered.insert((goal.tracked.is_none(), goal.tracked.unwrap_or(0), goal.number), line).expect("live goals");
        }
    }
    let mut lines = List::with_capacity(16);
    for (_, line) in &ordered {
        if lines.len() == 16 {
            break;
        }
        lines.push(line.clone()).expect("goal prefix");
    }
    for line in &kept.lines {
        if lines.len() == 16 {
            break;
        }
        let mut present = false;
        for live in &lines {
            if live.chip.number == line.chip.number {
                present = true;
                break;
            }
        }
        if !present {
            lines.push(line.clone()).expect("goal prefix");
        }
    }
    let mut goals = List::with_capacity(lines.len());
    for line in lines.into_boxed() {
        let Some(notes) = domain.parts.entry(row.project, line.chip.number).notes.clone() else {
            return note_load(domain, row.project, line.chip.number);
        };
        let headline = match notes.report {
            Some((headline, _)) => Some(headline),
            None => None,
        };
        goals.push((line, headline)).expect("goal prefix");
    }
    let whole = match domain.core.tasks.task(task) {
        Some(live) => match live.concluded {
            Some(concluded) => Some(concluded.at),
            None => None,
        },
        None => kept.whole,
    };
    encoded(encode::milestone(whole, &totals, goals.as_slice()))
}

pub(super) fn delegates(
    env: &Env<Limits>,
    party: u64,
    project: u32,
    task: u64,
    ended: Ended,
    page: u32,
    after: Option<core::reads::Position>,
) -> Making {
    Making::Joined(
        Phase::Delegates { project, task, ended, page },
        Box::new(core::reads::Range {
            party,
            scope: core::reads::Scope::Tasks {
                query: d::TaskQuery {
                    project: Some(project),
                    requester: None,
                    root: None,
                    parent: Some(task),
                    label: None,
                    phase: Some(d::PhaseClass::Ended),
                    executor: None,
                    tracked: None,
                    order: d::QueryOrder::Ended,
                    parts: List::with_capacity(0),
                },
            },
            after,
            backwards: true,
            rows: env.limits.documents.rows,
            bytes: env.limits.loads.read_bytes,
        }),
    )
}
