//! Whole task-tree feeds and closing retention (`domain/connectors.md`, 8;
//! `domain/engine/README.md`, 4.4 and 5.4). The core retains only the feed's task
//! facts and unread history milestones. Connector digests and writes stay
//! with their owners. Roots call `finish_decision` after routing ordinary
//! work, route its asks synchronously, and call again until it emits no work.
//!
//! | State | Event | Next state | Emits |
//! | --- | --- | --- | --- |
//! | Absent | Tracked goal made | Changed | Bounded goal and plan |
//! | Open | Subtree save/history | Changed | Updated plan and pending milestones |
//! | Changed | Decision finishing | Open or closing | Save and one feed per connector |
//! | Closing | Last connector settles | Absent | Erase and forget asks |

pub use crate::below::effects::projections::*;

use crate::{Ask, Core, CoreKey, CoreRecord, Limits, Record, Request, Write};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue};

/// Durable feed state retained until every connector settles the closing feed.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Projection {
    pub goal: ProjectionGoal,
    pub plan: Box<[ProjectionTask]>,
    pub milestones: Box<[ProjectionMilestone]>,
    pub pending: Box<[u16]>,
    pub changed: bool,
    pub fed_at: Option<u64>,
}

fn header(state: &Projection) -> ProjectionRecord {
    ProjectionRecord::Goal {
        goal: state.goal.clone(),
        pending: state.pending.clone(),
        changed: state.changed,
        fed_at: state.fed_at,
    }
}

fn write(out: &mut Queue<Request>, record: ProjectionRecord) {
    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Projection(record)))));
}

fn goal(row: &tasks::TaskRecord) -> ProjectionGoal {
    ProjectionGoal {
        number: row.number,
        project: row.project,
        priority: row.tracked.expect("tracked goal"),
        words: row.spec.words.clone(),
        phase: row.phase.clone(),
    }
}

pub(crate) fn save(core: &mut Core, limits: &Limits, row: &tasks::TaskRecord, out: &mut Queue<Request>) {
    if row.tracked.is_some() && !core.projections.contains_key(&row.number) {
        core.projections
            .insert(
                row.number,
                Box::new(Projection {
                    goal: goal(row),
                    plan: Box::new([]),
                    milestones: Box::new([]),
                    pending: Box::new([]),
                    changed: true,
                    fed_at: None,
                }),
            )
            .expect("tracked projection room admitted");
    }
    let number = if core.projections.contains_key(&row.number) {
        row.number
    } else {
        let Some(number) = core.goal_of(row) else { return };
        number
    };
    let Some(state) = core.projections.get_mut(&number) else { return };
    if row.number == number {
        let mut next = state.goal.clone();
        next.words.clone_from(&row.spec.words);
        next.phase.clone_from(&row.phase);
        if let Some(priority) = row.tracked {
            next.priority = priority;
        }
        state.changed |= state.goal != next;
        state.goal = next;
    }
    let next = ProjectionTask {
        number: row.number,
        requester: row.requester,
        words: row.spec.words.clone(),
        phase: row.phase.clone(),
    };
    let mut found = false;
    for old in &mut state.plan {
        if old.number == row.number {
            state.changed |= *old != next;
            *old = next.clone();
            found = true;
            break;
        }
    }
    if !found {
        let mut plan = List::with_capacity(limits.tasks.tree_tasks);
        for old in core::mem::replace(&mut state.plan, Box::new([])) {
            plan.push(old).expect("bounded tree plan");
        }
        plan.push(next).expect("lifetime tree task room admitted");
        state.plan = plan.into_boxed();
        state.changed = true;
    }
    write(
        out,
        ProjectionRecord::Task {
            goal: number,
            task: ProjectionTask {
                number: row.number,
                requester: row.requester,
                words: row.spec.words.clone(),
                phase: row.phase.clone(),
            },
        },
    );
    write(out, header(state));
}

pub(crate) fn history(
    core: &mut Core,
    limits: &Limits,
    task: u64,
    milestone: ProjectionMilestone,
    out: &mut Queue<Request>,
) {
    let root = match core.tasks.task(task) {
        Some(row) => {
            let Some(goal) = core.goal_of(row) else { return };
            goal
        }
        None => {
            let mut root = None;
            for (number, projection) in &core.projections {
                for row in &projection.plan {
                    if row.number == task {
                        root = Some(*number);
                    }
                }
            }
            let Some(root) = root else { return };
            root
        }
    };
    if !core.projections.contains_key(&root)
        && let Some(goal) = core.tasks.task(root)
        && goal.tracked.is_some()
    {
        let goal = goal.clone();
        save(core, limits, &goal, out);
    }
    let Some(state) = core.projections.get_mut(&root) else { return };
    let saved = milestone.clone();
    let count = u32::try_from(state.milestones.len()).expect("milestone count fits u32");
    let mut milestones = List::with_capacity(count.checked_add(1).expect("one more milestone"));
    for old in &state.milestones {
        if old.identity == milestone.identity {
            return;
        }
        milestones.push(old.clone()).expect("pending history bounded by decision");
    }
    assert!(count < core.routes, "pending history bounded by decision");
    milestones.push(milestone).expect("decision history room");
    state.milestones = milestones.into_boxed();
    state.changed = true;
    write(out, ProjectionRecord::Milestone { goal: root, milestone: saved });
    write(out, header(state));
}

fn eligible(core: &Core, number: u64, state: &Projection) -> bool {
    let untracked = match core.tasks.task(number) {
        Some(task) => task.tracked.is_none(),
        None => false,
    };
    if untracked && !state.pending.is_empty() {
        return false;
    }
    untracked
        || state.changed
        || (!state.milestones.is_empty() && state.fed_at != Some(core.counters.deployment().commits))
}

pub(crate) fn owed(core: &Core) -> bool {
    for (number, state) in &core.projections {
        if eligible(core, *number, state) {
            return true;
        }
    }
    false
}

/// Complete a composed root decision's projection handoffs. Repeated calls
/// retain subsequent changes but never feed a goal twice under one commit.
pub fn finish_decision(core: &mut Core, env: &Env<Limits>) -> Queue<Request> {
    core.start_runtime_input();
    if !owed(core) {
        return crate::routing::finish_watches(core, env);
    }
    let mut out = Queue::uncapped();
    let commit = core.counters.deployment().commits;
    let mut goals = List::with_capacity(core.projections.len());
    for (number, _) in &core.projections {
        goals.push(*number).expect("bounded tracked goals");
    }
    for number in &goals {
        if !eligible(core, *number, core.projections.get(number).expect("retained projection")) {
            continue;
        }
        let untracked = match core.tasks.task(*number) {
            Some(task) => task.tracked.is_none(),
            None => false,
        };
        let state = core.projections.get_mut(number).expect("retained feed state");
        let feed = if state.fed_at == Some(commit) {
            None
        } else {
            state.fed_at = Some(commit);
            let closing = match state.goal.phase {
                tasks::Phase::Ended(_) => true,
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { .. } => false,
            } || untracked;
            if closing && state.pending.is_empty() {
                state.pending.clone_from(&core.connectors);
            }
            let feed = ProjectionFeed {
                goal: state.goal.clone(),
                plan: state.plan.clone(),
                milestones: state.milestones.clone(),
                closing,
            };
            for milestone in &state.milestones {
                out.push(Request::Write(Write::Erase(crate::Key::Core(CoreKey::Projection(milestone_key(
                    state.goal.number,
                    milestone.identity,
                ))))));
            }
            state.milestones = Box::new([]);
            Some(feed)
        };
        state.changed = false;
        write(&mut out, header(state));
        if let Some(feed) = feed {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::ProjectGoal { feed: Box::new(feed.clone()) } });
            }
        }
        let ended = match state.goal.phase {
            tasks::Phase::Ended(_) => true,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                false
            }
        };
        if (ended || untracked) && core.connectors.is_empty() {
            forget(core, *number, &mut out);
        }
    }
    if out.is_empty() {
        let mut watches = crate::routing::finish_watches(core, env);
        for _ in 0..watches.len() {
            let request = watches.pop().expect("watch decision outputs");
            out.push(request);
        }
    }
    out
}

pub(crate) fn settled(core: &mut Core, goal: u64, connector: u16, out: &mut Queue<Request>) {
    let Some(state) = core.projections.get_mut(&goal) else { return };
    if !state.pending.contains(&connector) {
        return;
    }
    let mut pending = List::with_capacity(u32::try_from(state.pending.len()).expect("bounded connectors"));
    for &number in &state.pending {
        if number != connector {
            pending.push(number).expect("connector subset");
        }
    }
    state.pending = pending.into_boxed();
    if state.pending.is_empty() {
        forget(core, goal, out);
    } else {
        write(out, header(state));
    }
}

fn forget(core: &mut Core, goal: u64, out: &mut Queue<Request>) {
    let state = core.projections.remove(&goal).expect("closing feed state retained");
    for row in &state.plan {
        out.push(Request::Write(Write::Erase(crate::Key::Core(CoreKey::Projection(ProjectionKey::Task {
            goal,
            task: row.number,
        })))));
    }
    for milestone in &state.milestones {
        out.push(Request::Write(Write::Erase(crate::Key::Core(CoreKey::Projection(milestone_key(
            goal,
            milestone.identity,
        ))))));
    }
    out.push(Request::Write(Write::Erase(crate::Key::Core(CoreKey::Projection(ProjectionKey::Goal(goal))))));
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask { connector, ask: Ask::ForgetProjection { goal } });
    }
}

/// Checked maximum owned bytes of one retained projection feed cache.
#[must_use]
pub fn projection_bytes(limits: &Limits) -> Option<u64> {
    let phase = u64::from(limits.tasks.result_bytes).checked_mul(2)?;
    let goal = u64::try_from(size_of::<ProjectionGoal>())
        .ok()?
        .checked_add(u64::from(limits.tasks.spec_bytes))?
        .checked_add(phase)?;
    let task = u64::try_from(size_of::<ProjectionTask>())
        .ok()?
        .checked_add(u64::from(limits.tasks.spec_bytes))?
        .checked_add(phase)?;
    let milestone = u64::try_from(size_of::<ProjectionMilestone>())
        .ok()?
        .checked_add(u64::from(limits.tasks.message_bytes.max(limits.tasks.result_bytes)))?
        .checked_add(phase)?;
    u64::try_from(size_of::<Projection>())
        .ok()?
        .checked_add(goal)?
        .checked_add(u64::from(limits.tasks.tree_tasks).checked_mul(task)?)?
        .checked_add(u64::from(limits.connectors).checked_mul(2)?)?
        .checked_add(u64::from(limits.routes).checked_mul(milestone)?)
}

pub(crate) fn phase_within(phase: &tasks::Phase, limits: &Limits) -> bool {
    match tasks::phase_bytes(phase) {
        Some(bytes) => bytes <= u64::from(limits.tasks.result_bytes) * 2,
        None => false,
    }
}

pub(crate) fn restore(core: &mut Core, limits: &Limits, row: ProjectionRecord) -> bool {
    crate::start::projections::start_restore(core, limits, row)
}
