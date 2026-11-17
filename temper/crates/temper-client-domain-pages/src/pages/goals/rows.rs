//! Grouped rows (temper/docs/design/client/pages/home.md, 1; work.md, 1, Its milestones).
use crate::pages::milestones::Milestones;
use core::cmp::Reverse;
use jig_client::contract as c;
use skein_lib::{List, Map, Set, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupRow {
    Goal(u64),
    Milestone(u64),
}

/// Home sorts and limits rows after replacing goals with their milestone.
#[must_use]
pub fn home_rows(
    live: &[c::TaskLine],
    ended: &[c::TaskLine],
    milestones: Option<&Milestones>,
    wall: Wall,
) -> List<GroupRow> {
    let mut ordered = Map::uncapped();
    let mut seen = Set::with_capacity(u32::MAX);
    if let Some(milestones) = milestones {
        for milestone in milestones.drawn() {
            let task = milestone.task;
            for line in milestones.lines() {
                if line.chip.task != task {
                    continue;
                }
                let mut activity = line.last_activity;
                for goal in &milestone.goals {
                    activity = activity.max(goal.last_activity);
                }
                for goal in live.iter().chain(ended) {
                    if milestones.of(goal) == Some(task) {
                        activity = activity.max(goal.last_activity);
                    }
                }
                let is_ended = line.ended_at.is_some() || line.chip.phase == c::Phase::Ended;
                let at = if is_ended { line.ended_at.unwrap_or(line.last_activity) } else { activity };
                ordered.insert((is_ended, Reverse(at), task), GroupRow::Milestone(task)).expect("held rows fit");
            }
        }
    }
    for line in live.iter().chain(ended) {
        if !seen.insert(line.chip.task).expect("held goals fit") {
            continue;
        }
        if let Some(milestones) = milestones
            && let Some(task) = milestones.of(line)
        {
            if milestones.milestone(task).is_some() {
                continue;
            }
            // An eligible listed milestone beyond DRAWN keeps its goals' rows.
            // Older or unlisted milestones and their goals have no Home row.
            let mut eligible = false;
            for listed in milestones.lines() {
                if listed.chip.task == task {
                    eligible = listed.chip.phase != c::Phase::Ended;
                    if let Some(at) = listed.ended_at {
                        eligible |= at <= wall && wall.as_nanos().saturating_sub(at.as_nanos()) <= 604_800_000_000_000;
                    }
                }
            }
            if !eligible {
                continue;
            }
        }
        let is_ended = line.ended_at.is_some() || line.chip.phase == c::Phase::Ended;
        let at = if is_ended { line.ended_at.unwrap_or(line.last_activity) } else { line.last_activity };
        ordered.insert((is_ended, Reverse(at), line.chip.task), GroupRow::Goal(line.chip.task)).expect("held rows fit");
    }
    let mut live_count = 0_u32;
    for ((ended, _, _), _) in &ordered {
        if !ended {
            live_count = live_count.saturating_add(1);
        }
    }
    let mut rows = List::uncapped();
    for (_, row) in &ordered {
        if rows.len() == 5_u32.max(live_count) {
            break;
        }
        rows.push(*row).expect("held rows fit");
    }
    rows
}

/// Filtered goals keep their order; each group stands at its first shown goal.
#[must_use]
pub fn groups(shown: &[c::TaskLine], milestones: Option<&Milestones>) -> List<GroupRow> {
    let mut rows = List::uncapped();
    let mut seen = Set::with_capacity(u32::MAX);
    for line in shown {
        let mut row = GroupRow::Goal(line.chip.task);
        if let Some(milestones) = milestones
            && let Some(task) = milestones.of(line)
            && milestones.milestone(task).is_some()
        {
            row = GroupRow::Milestone(task);
        }
        let task = match row {
            GroupRow::Goal(task) | GroupRow::Milestone(task) => task,
        };
        if seen.insert(task).expect("held rows fit") {
            rows.push(row).expect("held rows fit");
        }
    }
    rows
}

/// A row resolves its number without copying a held goal.
#[must_use]
#[expect(clippy::manual_find, reason = "step code uses no closures")]
pub fn goal<'a>(live: &'a [c::TaskLine], ended: &'a [c::TaskLine], task: u64) -> Option<&'a c::TaskLine> {
    for line in live.iter().chain(ended) {
        if line.chip.task == task {
            return Some(line);
        }
    }
    None
}
