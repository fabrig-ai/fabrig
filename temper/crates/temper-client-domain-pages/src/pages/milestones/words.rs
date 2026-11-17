//! The milestone's pure vocabulary (temper/docs/design/client/pages/milestone.md, 2).
use jig_client::contract as c;
use skein_lib::{List, Map};
use temper_client_protocol_wire::v1 as w;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Closed,
    Complete,
    Ended,
    Done,
    Needs,
    Paused,
    Running,
    Waits,
    Open,
}

#[must_use]
pub fn state(line: &c::TaskLine, goals: &[c::TaskLine], totals: Option<&w::MilestoneTotals>, needs: bool) -> State {
    if line.chip.phase == c::Phase::Ended {
        return match &line.chip.details.details.ending {
            Some(c::ChipEnding::Cancelled { .. }) => State::Closed,
            Some(c::ChipEnding::Done(_) | c::ChipEnding::Failed(_)) | None => match totals {
                Some(totals) if totals.failed() == 0 => State::Complete,
                Some(_) => State::Ended,
                None => State::Done,
            },
        };
    }
    if needs {
        return State::Needs;
    }
    if let c::ChipPhase::Held { .. } = line.chip.details.phase
        && let Some(hold) = &line.chip.details.details.hold
        && let Some(placed) = &hold.placed
        && placed.mode == c::HoldMode::Stop
        && placed.scope == (c::HoldScope::Subtree { task: line.chip.task })
        && !placed.lifted
    {
        return State::Paused;
    }
    let mut underway = false;
    for goal in goals {
        if goal.chip.phase != c::Phase::Ended {
            underway = true;
            if started(&goal.chip) {
                return State::Running;
            }
        }
    }
    if underway { State::Waits } else { State::Open }
}
fn started(chip: &c::Chip) -> bool {
    match chip.details.phase {
        c::ChipPhase::Waiting { .. } | c::ChipPhase::Ended => false,
        c::ChipPhase::Active(_)
        | c::ChipPhase::Held { .. }
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_) => true,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Count {
    pub done: u64,
    pub total: u64,
}
#[must_use]
pub fn count(goals: &[c::TaskLine], totals: Option<&w::MilestoneTotals>) -> Count {
    if let Some(totals) = totals {
        let done = u64::from(totals.done());
        return Count {
            done,
            total: done
                .checked_add(u64::from(totals.failed()))
                .expect("two u32 counts fit u64")
                .checked_add(u64::from(totals.live()))
                .expect("three u32 counts fit u64"),
        };
    }
    let mut count = Count { done: 0, total: 0 };
    for goal in goals {
        if let Some(mark) = mark(goal, &[]) {
            count.total = count.total.checked_add(1).expect("held goals fit");
            if mark == Mark::Done {
                count.done = count.done.checked_add(1).expect("held goals fit");
            }
        }
    }
    count
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Done,
    Needs,
    Live,
    Waits,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    pub task: u64,
    pub mark: Mark,
}
fn mark(goal: &c::TaskLine, needing: &[u64]) -> Option<Mark> {
    if goal.chip.phase == c::Phase::Ended {
        return match &goal.chip.details.details.ending {
            Some(c::ChipEnding::Done(_)) => Some(Mark::Done),
            Some(c::ChipEnding::Failed(_)) => Some(Mark::Failed),
            Some(c::ChipEnding::Cancelled { .. }) | None => None,
        };
    }
    Some(if needing.contains(&goal.chip.task) {
        Mark::Needs
    } else if started(&goal.chip) {
        Mark::Live
    } else {
        Mark::Waits
    })
}
#[must_use]
pub fn strip(goals: &[c::TaskLine], needing: &[u64]) -> List<Segment> {
    let mut ordered = Map::uncapped();
    for goal in goals {
        if let Some(mark) = mark(goal, needing) {
            ordered.insert(goal.chip.task, mark).expect("held goals fit");
        }
    }
    let mut segments = List::with_capacity(ordered.len());
    for (task, mark) in &ordered {
        segments.push(Segment { task: *task, mark: *mark }).expect("same count");
    }
    segments
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Headline {
    Count(Count),
    Complete(Count),
    Closed(Count),
    Open(Count),
    Ended { count: Count, failed: u32, first: Option<u64> },
    Running { count: Count, goal: u64, more: u32 },
}
#[must_use]
pub fn headline(
    line: &c::TaskLine,
    goals: &[c::TaskLine],
    totals: Option<&w::MilestoneTotals>,
    needs: bool,
) -> Headline {
    let count = count(goals, totals);
    match state(line, goals, totals, needs) {
        State::Closed => return Headline::Closed(count),
        State::Complete => return Headline::Complete(count),
        State::Ended => {
            let failed = totals.expect("ended has totals").failed();
            let mut first = None;
            for goal in goals {
                if mark(goal, &[]) == Some(Mark::Failed)
                    && match first {
                        Some(task) => goal.chip.task < task,
                        None => true,
                    }
                {
                    first = Some(goal.chip.task);
                }
            }
            return Headline::Ended { count, failed, first };
        }
        State::Done => return Headline::Count(count),
        State::Open => return Headline::Open(count),
        State::Needs | State::Paused | State::Running | State::Waits => {}
    }
    let mut newest: Option<&c::TaskLine> = None;
    let mut running = 0_u32;
    for goal in goals {
        if goal.chip.phase != c::Phase::Ended && started(&goal.chip) {
            running = running.checked_add(1).expect("held goals fit");
            if match newest {
                Some(old) => (goal.last_activity, goal.chip.task) > (old.last_activity, old.chip.task),
                None => true,
            } {
                newest = Some(goal);
            }
        }
    }
    match newest {
        Some(goal) => {
            Headline::Running { count, goal: goal.chip.task, more: running.checked_sub(1).expect("one started goal") }
        }
        None => Headline::Count(count),
    }
}
