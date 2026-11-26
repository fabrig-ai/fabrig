//! A milestone's totals (temper/docs/design/domain/milestones.md, 7 and 8).
use crate::domain::Domain;
use jig_domain_tasks as tasks;
use temper_engine_domain_forge::MilestoneTotals;

pub(super) fn read(domain: &Domain, project: u32, task: u64) -> Option<MilestoneTotals> {
    let row = domain.forge.milestone(task);
    if let Some(row) = row {
        if row.project != project {
            return None;
        }
        if let Some(end) = &row.ended {
            return Some(MilestoneTotals {
                task,
                done: end.done,
                failed: end.failed,
                closed: end.closed,
                live: 0,
                ended: Some(end.at),
                last_landing: row.last_landing.clone(),
                spent: end.spent,
                reds: domain.forge.reds_owed(row.project, task),
            });
        }
    }
    let record = domain.engine.core().tasks.task(task)?;
    if record.project != project || !tasks::gathering::executor(record.executor) {
        return None;
    }
    let mut spent = record.numbers.spent.checked_add(record.numbers.spent_below).expect("task accounting fits");
    for goal in &record.delegates {
        let goal = domain.engine.core().tasks.task(*goal).expect("live delegate has a task record");
        spent = spent.checked_add(goal.numbers.spent).expect("task accounting fits");
        spent = spent.checked_add(goal.numbers.spent_below).expect("task accounting fits");
    }
    Some(MilestoneTotals {
        task,
        done: record.ended_delegates.done,
        failed: record.ended_delegates.failed,
        closed: record.ended_delegates.cancelled,
        live: u32::try_from(record.delegates.len()).expect("bounded delegates"),
        ended: record.ended_at,
        last_landing: match row {
            Some(row) => row.last_landing.clone(),
            None => None,
        },
        spent,
        reds: domain.forge.reds_owed(record.project, task),
    })
}
