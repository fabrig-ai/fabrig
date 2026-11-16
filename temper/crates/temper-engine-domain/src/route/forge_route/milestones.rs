//! Milestones project to rows, not issues (temper/docs/design/domain/milestones.md, 8).
use super::{Domain, Work, forge, tasks};
use alloc::boxed::Box;

fn name(words: &[u8]) -> Box<[u8]> {
    let mut end = 0usize;
    for byte in words {
        if *byte == b'\n' {
            break;
        }
        end = end.checked_add(1).expect("first line within task words");
    }
    let line = words.get(..end).expect("first line");
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    forge::text::fit(line, skein_lib::usize_of(forge::milestones::MILESTONE_NAME_BYTES), b"")
}

pub(super) fn of(domain: &Domain, goal: Option<u64>) -> Option<u64> {
    let goal = domain.core.tasks.task(goal?)?;
    let tasks::Party::Task(requester) = goal.requester else { return None };
    let parent = domain.core.tasks.task(requester)?;
    if tasks::gathering::executor(parent.executor) { Some(requester) } else { None }
}

pub(super) fn project(domain: &mut Domain, feed: &jig_domain_core::ProjectionFeed, decision: &super::Decision) -> bool {
    let record = match domain.core.tasks.task(feed.goal.number) {
        Some(row) => Some(row),
        None => decision.ended_task(feed.goal.number),
    };
    let is_milestone = match record {
        Some(row) => tasks::gathering::executor(row.executor),
        None => domain.forge.milestone(feed.goal.number).is_some(),
    };
    if !is_milestone {
        return false;
    }
    let ended = match record {
        Some(row) => end(row),
        None => None,
    };
    domain.work.push(Work::Forge(forge::Event::Milestone {
        row: forge::MilestoneRow {
            project: feed.goal.project,
            task: feed.goal.number,
            name: name(&feed.goal.words),
            last_landing: None,
            ended,
        },
    }));
    if feed.closing {
        domain.work.push(Work::Core(jig_domain_core::Event::ProjectionSettled {
            goal: feed.goal.number,
            connector: domain.config.forge_connector,
        }));
    }
    true
}
// The final task record carries counts, spend and wall that the feed omits.
fn end(row: &tasks::TaskRecord) -> Option<forge::MilestoneEnd> {
    let outcome = match row.phase {
        tasks::Phase::Ended(tasks::Ending::Done(_)) => {
            if row.ended_delegates.failed == 0 {
                forge::MilestoneOutcome::Complete
            } else {
                forge::MilestoneOutcome::Ended
            }
        }
        tasks::Phase::Ended(tasks::Ending::Failed { .. }) => forge::MilestoneOutcome::Ended,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => forge::MilestoneOutcome::Closed,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            return None;
        }
    };
    Some(forge::MilestoneEnd {
        at: row.ended_at.expect("ended task's wall"),
        outcome,
        done: row.ended_delegates.done,
        failed: row.ended_delegates.failed,
        closed: row.ended_delegates.cancelled,
        spent: row.numbers.spent.checked_add(row.numbers.spent_below).expect("task accounting fits"),
    })
}
