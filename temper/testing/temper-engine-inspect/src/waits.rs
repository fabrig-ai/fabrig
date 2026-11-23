use crate::{View, landing_ended, task_name};
use jig_domain_tasks as tasks;
use skein_lib::Time;
use skein_world::inspect::{Inspection, Wait, Wake};
use temper_engine_domain_forge::{LandingRow, Operation};
use temper_engine_domain_forge_change::State;
use temper_engine_domain_forge_landing::Awaited;

pub(super) fn inspect(view: &View<'_>, into: &mut Inspection) {
    for (_, row) in view.forge.inspection_landings() {
        if let Some(wait) = landing_wait(view, row) {
            into.wait(wait);
        }
    }
    for (_, row) in view.forge.inspection_changes() {
        if let Some(wait) = change_wait(view, row) {
            into.wait(wait);
        }
    }
    for task in view.forge.inspection_making_waiters() {
        if !view.ended(task) {
            let mut wakes = room(view, task);
            if view.forge.room_asked(task) {
                wakes.push(Wake::Party("forge: room".into()));
            }
            into.wait(Wait { waiter: task_name(task), kind: "making: room".into(), wakes, bound: None });
        }
    }
}

fn landing_wait(view: &View<'_>, row: &LandingRow) -> Option<Wait> {
    let task = row.landing.task;
    if landing_ended(&row.landing.phase) || view.ended(task) {
        return None;
    }
    // The prepare effect leaves its job in flight before a later landing step
    // records an explicit Wait decision. Its phase and job still name that wait.
    let awaited = row.waiting.or_else(|| {
        (row.landing.phase == temper_engine_domain_forge_landing::Phase::Preparing && row.job.is_some())
            .then_some(Awaited::Prepare)
    });
    let (kind, mut wakes) =
        if row.room.is_some() { ("landing: room".to_owned(), room(view, task)) } else { landing(view, row, awaited?) };
    if row.room.is_none() {
        person_release(view, task, &mut wakes);
        wakes.extend(view.read_wakes(task));
        if let Some(due) = view.forge.inspection_landing_due(task) {
            wakes.push(Wake::Timer(Time::from_nanos(due.as_nanos())));
        }
    }
    Some(Wait { waiter: task_name(task), kind, wakes, bound: None })
}

fn change_wait(view: &View<'_>, row: &temper_engine_domain_forge::ChangeRow) -> Option<Wait> {
    let task = row.task;
    // Once its ending is handed to the root, the procedure no longer waits;
    // the task can still be closing its holds in jig. Those entries are read
    // by the table inspection, not invented as another procedure step.
    if view.ended(task) || row.ending.is_some() {
        return None;
    }
    let kind = change_kind(&row.change.state)?;
    let (kind, mut wakes) = if row.room.is_some() {
        ("change: room", room(view, task))
    } else {
        let mut wakes = change_wakes(view, row);
        person_release(view, task, &mut wakes);
        // Containment detaches a merge from the change before its withdrawal
        // settles. The release still waits for that outbox entry's answer.
        for entry in view.forge.inspection_settling_entries(task) {
            let wake = Wake::Party(format!("forge: entry {entry}"));
            if !wakes.contains(&wake) {
                wakes.push(wake);
            }
        }
        wakes.extend(view.read_wakes(task));
        if let Some(due) = view.forge.inspection_change_due(task) {
            wakes.push(Wake::Timer(Time::from_nanos(due.as_nanos())));
        }
        if let Some(due) = view.forge.inspection_step_retry(task) {
            wakes.push(Wake::Timer(due));
        }
        if view.root.is_none() && wakes.is_empty() && root_turn(row) {
            wakes.push(Wake::Party("root".into()));
        }
        (kind, wakes)
    };
    admission(view, task, &mut wakes);
    Some(Wait { waiter: task_name(task), kind: kind.into(), wakes, bound: bound(view, row) })
}

// Admission can park the root before a change gets its delegate. Only a kept
// limit account or jig's recorded room wait names these release paths.
fn admission(view: &View<'_>, task: u64, wakes: &mut Vec<Wake>) {
    let Some(root) = view.root else { return };
    if let Some(hold) = root.inspection_limit_hold(task) {
        wakes.push(Wake::Party(format!("operator limits: {}", String::from_utf8_lossy(&hold.member))));
        wakes.push(Wake::Party("operator restart".into()));
    }
    let Some(wait) = root.tasks().task(task).and_then(|row| row.room_wait) else { return };
    for row in root.tasks().view_tasks() {
        let serves = match wait.room {
            tasks::LiveRoom::Deployment => true,
            tasks::LiveRoom::Project(project) => row.project == project,
        };
        if row.number != task && serves && !view.ended(row.number) {
            wakes.push(view.waiter(row.number));
        }
    }
}

// A person's hold can stop jig without changing the forge procedure's step.
// Resource and circle holds need their own wake; a person is not their fallback.
fn person_release(view: &View<'_>, task: u64, wakes: &mut Vec<Wake>) {
    let Some(row) = view.root.and_then(|root| root.tasks().task(task)) else { return };
    let tasks::Phase::Held { why, .. } = row.phase else { return };
    let released = match why {
        tasks::Hold::Exceeded(_)
        | tasks::Hold::Failures(_)
        | tasks::Hold::Stalled
        | tasks::Hold::Stopped
        | tasks::Hold::Placed { .. }
        | tasks::Hold::Drift
        | tasks::Hold::Effects
        | tasks::Hold::EffectFailed
        | tasks::Hold::Uncertain { .. }
        | tasks::Hold::Budget
        | tasks::Hold::Deadline
        | tasks::Hold::Procedure => true,
        tasks::Hold::Dependency(_)
        | tasks::Hold::HoldsWaited
        | tasks::Hold::ResourceUnavailable
        | tasks::Hold::ResourceAccess
        | tasks::Hold::PoolLost { .. }
        | tasks::Hold::Circle => false,
    };
    let wake = Wake::Party("person".into());
    if released && !wakes.contains(&wake) {
        wakes.push(wake);
    }
}

fn change_wakes(view: &View<'_>, row: &temper_engine_domain_forge::ChangeRow) -> Vec<Wake> {
    let mut wakes = Vec::new();
    match row.change.state {
        State::Held { .. } => wakes.push(Wake::Party("person".into())),
        State::Ready { .. } | State::Preparing { .. } => {
            if let Some(task) = row.change.landing {
                wakes.push(view.waiter(task));
            } else if matches!(row.change.state, State::Ready { .. }) {
                if view.forge.landing_held() {
                    wakes.push(Wake::Party("operator restart".into()));
                }
                if let Some(due) = view.root.and_then(|root| root.inspection_forming_retry(row.task)) {
                    wakes.push(Wake::Timer(Time::from_nanos(due.as_nanos())));
                }
                let full = view.forge.inspection_landings().len() >= view.forge.inspection_landings().capacity();
                for (task, landing) in view.forge.inspection_landings() {
                    let same_train = landing.landing.branch == row.base
                        && landing.landing.members.iter().any(|member| {
                            member.repository.forge == row.repository.forge
                                && member.repository.repository == row.repository.repository
                        });
                    if !landing_ended(&landing.landing.phase) && (full || same_train) {
                        wakes.push(view.waiter(*task));
                    }
                }
            }
            if matches!(row.change.state, State::Preparing { .. }) {
                pending_write(view, row, &mut wakes);
            }
            // A terminal landing may still occupy a table slot while its writes
            // settle. With no live landing wake, read the train's due member.
            if matches!(row.change.state, State::Ready { .. }) && row.change.landing.is_none() && wakes.is_empty() {
                forming(view, row, &mut wakes);
            }
        }
        _ => {
            if let Some((delegate, _)) = row.delegate {
                // At the forge alone, the delegate's standing is its lifecycle projection.
                if view.root.is_some() || row.delegate_status == temper_engine_domain_forge_change::Status::Unknown {
                    wakes.push(view.waiter(delegate));
                }
            }
            pending_write(view, row, &mut wakes);
            if !row.questions.open.is_empty() {
                wakes.push(view.answerer(row.task));
            }
            change_turn(view, row.task, &mut wakes);
            if matches!(row.change.state, State::Producing { requested: false }) {
                follow_restart(view, row.task, &mut wakes);
            }
        }
    }
    if matches!(row.change.state, State::WaitingProducer { .. })
        || (matches!(row.change.state, State::Ready { .. }) && row.change.landing.is_none())
    {
        for child in pushing_children(view, row.task) {
            let wake = view.waiter(child);
            if !wakes.contains(&wake) {
                wakes.push(wake);
            }
        }
    }
    wakes
}

// A gate can displace the current delegate without ending a pushing child.
// The root uses every such child's terminal to release forming or resolution.
fn pushing_children(view: &View<'_>, task: u64) -> Vec<u64> {
    let Some(root) = view.root else { return Vec::new() };
    let Some(owner) = root.tasks().task(task) else { return Vec::new() };
    owner
        .delegates
        .iter()
        .copied()
        .filter(|child| root.inspection_pushing_child(*child) && !view.ended(*child))
        .collect()
}

// A change can prepare its own rebase before it belongs to a landing.
fn pending_write(view: &View<'_>, row: &temper_engine_domain_forge::ChangeRow, wakes: &mut Vec<Wake>) {
    if let Some(entry) = row.pending
        && view.forge.inspection_entry(entry).is_some()
    {
        wakes.push(Wake::Party(format!("forge: entry {entry}")));
    }
}

// The forge does not expose the spent one-step forming hold. Name the due
// members gating at their head in this train (landing-turn.md, 2). The forge
// alone cannot read which task is due, so it gives no forming wake.
fn forming(view: &View<'_>, row: &temper_engine_domain_forge::ChangeRow, wakes: &mut Vec<Wake>) {
    let Some(root) = view.root else { return };
    if row.change.way != temper_engine_domain_forge_change::Way::Turn {
        return;
    }
    for (task, member) in view.forge.inspection_changes() {
        if member.repository != row.repository
            || member.base != row.base
            || member.change.way != temper_engine_domain_forge_change::Way::Turn
            || member.change.landing.is_some()
            || member.terminal()
            || root.tasks().procedure_due(*task).is_none()
        {
            continue;
        }
        if let State::Gating { head, .. } = member.change.state
            && member.change.last_head == Some(head)
        {
            wakes.push(view.waiter(*task));
        }
    }
}

// Dependencies and a follow owner's word keep an unstarted change waiting
// without a delegate. Neither is a fallback for an otherwise unwakeable step.
fn change_turn(view: &View<'_>, task: u64, wakes: &mut Vec<Wake>) {
    let mut dependencies = view.forge.inspection_step_dependencies(task).into_vec();
    if let Some(row) = view.root.and_then(|root| root.tasks().task(task)) {
        dependencies.extend(row.waiting_on.iter().copied());
    }
    if let Some((owner, state)) = view.forge.inspection_follow_wait(task) {
        use temper_engine_domain_forge::FollowState;
        match state {
            FollowState::WaitsWord | FollowState::HeldByOwner(_) => {
                let wake = follow_owner(view, owner);
                if !wakes.contains(&wake) {
                    wakes.push(wake);
                }
            }
            FollowState::WaitsFor(dependency) => dependencies.push(*dependency),
            FollowState::NotMade | FollowState::Producing | FollowState::Held(_) | FollowState::Ended(_) => {}
        }
    }
    for dependency in dependencies {
        let wake = view.waiter(dependency);
        if !wakes.contains(&wake) {
            wakes.push(wake);
        }
    }
}

fn follow_owner(view: &View<'_>, owner: temper_engine_domain_forge::settings::FollowOwner) -> Wake {
    use temper_engine_domain_forge::settings::FollowOwner;
    match owner {
        FollowOwner::Person(_) => Wake::Party("person".into()),
        FollowOwner::Task(owner) => view.waiter(owner),
    }
}

// Jig clears waiting_on when a failed dependency holds the task. Only an
// owner's start or skip of the queued ended entry can remake this held tail.
fn follow_restart(view: &View<'_>, task: u64, wakes: &mut Vec<Wake>) {
    let Some(root) = view.root else { return };
    let mut seen = vec![task];
    let mut current = task;
    while let Some(row) = root.tasks().task(current) {
        let tasks::Phase::Held { why: tasks::Hold::Dependency(dependency), .. } = row.phase else { return };
        if seen.contains(&dependency) {
            return;
        }
        if let Some(owner) = view.forge.inspection_follow_restart_owner(task, dependency) {
            let wake = follow_owner(view, owner);
            if !wakes.contains(&wake) {
                wakes.push(wake);
            }
            return;
        }
        seen.push(dependency);
        current = dependency;
    }
}

// These procedure states keep an act already requested of the parent, not a
// missing external event. Writes and room deliberately have no such escape.
fn root_turn(row: &temper_engine_domain_forge::ChangeRow) -> bool {
    match &row.change.state {
        State::Producing { requested } | State::Resolving { requested, .. } | State::Repairing { requested, .. } => {
            *requested && row.delegate.is_none()
        }
        State::Gating { asked, .. } | State::Judging { asked, .. } => !asked.is_empty() && row.delegate.is_none(),
        State::Ready { .. } => row.change.landing.is_none(),
        _ => false,
    }
}

fn bound(view: &View<'_>, row: &temper_engine_domain_forge::ChangeRow) -> Option<Time> {
    // A stored start belongs to a stall only while the procedure is waiting;
    // root turns, room, holds, and a live non-gate agent are not stall time.
    view.forge.inspection_change_due(row.task)?;
    if row.room.is_some() || view.forge.inspection_access_wait(row.task).is_some() || !row.questions.open.is_empty() {
        return None;
    }
    if !matches!(
        view.forge.connection(row.repository.forge),
        temper_engine_domain_forge::Connection::Connected { .. } | temper_engine_domain_forge::Connection::Connecting
    ) {
        return None;
    }
    if let Some((delegate, kind)) = row.delegate
        && !matches!(kind, temper_engine_domain_forge_change::Delegate::Gate { .. })
        && !view.ended(delegate)
    {
        return None;
    }
    if matches!(row.change.state, State::WaitingProducer { .. }) {
        return view.forge.inspection_change_read_bound(row.task).map(|at| Time::from_nanos(at.as_nanos()));
    }
    if matches!(row.change.state, State::Producing { .. })
        && (!view.forge.inspection_step_dependencies(row.task).is_empty()
            || view.root.and_then(|root| root.tasks().task(row.task)).is_some_and(|task| !task.waiting_on.is_empty()))
    {
        return None;
    }
    // A settled turn change kept by dependencies has finished gating: its
    // poll waits for readiness, not for a forge gate to answer.
    if view.forge.inspection_gates_settled(row.task)
        && (!view.forge.inspection_step_dependencies(row.task).is_empty()
            || view.root.and_then(|root| root.tasks().task(row.task)).is_some_and(|task| !task.waiting_on.is_empty()))
    {
        return None;
    }
    let limits = &view.forge.inspection_limits().change_policy;
    let nanos = match row.change.state {
        State::Held { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::Queued { .. }
        | State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::Ended
        | State::NoChange { .. } => return None,
        State::Checking { .. } => {
            let clock = row.change.clock?;
            let counted = clock
                .ran
                .as_nanos()
                .saturating_add(clock.unknown.as_nanos().saturating_sub(limits.unknown_grace.as_nanos()));
            let remaining = match clock.seen {
                temper_engine_domain_forge_change::Execution::Running => {
                    limits.stall.as_nanos().saturating_sub(counted)
                }
                temper_engine_domain_forge_change::Execution::Waiting => {
                    limits.runner_wait.as_nanos().saturating_sub(clock.waited.as_nanos())
                }
                temper_engine_domain_forge_change::Execution::Unknown => limits
                    .unknown_grace
                    .as_nanos()
                    .saturating_sub(clock.unknown.as_nanos())
                    .saturating_add(limits.stall.as_nanos().saturating_sub(counted)),
            };
            clock.at.as_nanos().saturating_add(remaining)
        }
        State::Gating { .. } => {
            // The workflow's completing read can end after the last policy step.
            let since = view.forge.inspection_follow_check_ended(row.task).unwrap_or(row.change.since);
            since.max(row.change.since).as_nanos().saturating_add(limits.stall.as_nanos())
        }
        _ => row.change.since.as_nanos().saturating_add(limits.stall.as_nanos()),
    };
    Some(Time::from_nanos(nanos))
}

fn room(view: &View<'_>, task: u64) -> Vec<Wake> {
    let mut wakes: Vec<Wake> = view
        .forge
        .inspection_entries()
        .iter()
        .filter_map(|number| {
            let entry = view.forge.inspection_entry(*number)?;
            Some(if view.ended(entry.task) {
                Wake::Ended(format!("entry {number} of task {}", entry.task))
            } else {
                Wake::Waiter(task_name(entry.task))
            })
        })
        .collect();
    // The place remains while an asked waiter reads afresh, before its next
    // decision keeps an entry or leaves the line (domain/forge/room.md, 3;
    // testing/engine-checks.md, 2.2). Those reads still wake their owner.
    wakes.extend(view.read_wakes(task));
    person_release(view, task, &mut wakes);
    wakes
}

fn landing(view: &View<'_>, row: &LandingRow, awaited: Awaited) -> (String, Vec<Wake>) {
    let mut wakes = Vec::new();
    let kind = match awaited {
        Awaited::Release => {
            wakes.push(Wake::Party("person".into()));
            "release"
        }
        Awaited::Ready { member } | Awaited::Judgement { member } => {
            if let Some(member) = row.landing.members.get(usize::from(member)) {
                wakes.push(view.waiter(member.change));
            }
            if matches!(awaited, Awaited::Ready { .. }) { "ready" } else { "judgement" }
        }
        Awaited::Turn => {
            for (task, holder) in view.forge.inspection_landings() {
                if *task != row.landing.task
                    && holder.landing.project == row.landing.project
                    && matches!(holder.landing.phase, temper_engine_domain_forge_landing::Phase::Turn { .. })
                {
                    wakes.push(view.waiter(*task));
                }
            }
            "turn"
        }
        Awaited::Prepare => {
            wakes.extend(view.prepare(row));
            "prepare"
        }
        Awaited::Open => {
            write(view, row, "open", Operation::Open, None, None, &mut wakes);
            "open"
        }
        Awaited::Close => {
            write(view, row, "close", Operation::Close, None, None, &mut wakes);
            "close"
        }
        Awaited::Delete { member } => {
            write(view, row, "delete", Operation::Delete, Some(member), None, &mut wakes);
            "delete"
        }
        Awaited::Post { member, gate } => {
            write(view, row, "post", Operation::Pending, Some(member), Some(gate), &mut wakes);
            "post"
        }
        Awaited::Merge => {
            write(view, row, "merge", Operation::Merge, None, None, &mut wakes);
            "merge"
        }
        Awaited::Record { member } => {
            write(view, row, "record", Operation::Record, Some(member), None, &mut wakes);
            "record"
        }
        Awaited::Prefixes => "prefixes",
        Awaited::Run { .. } => "run",
        Awaited::Receipt { .. } => "receipt",
        Awaited::Ancestry { .. } => "ancestry",
        Awaited::Read => "read",
    };
    (format!("landing: {kind}"), wakes)
}

fn write(
    view: &View<'_>,
    row: &LandingRow,
    kind: &str,
    op: Operation,
    member: Option<u16>,
    gate: Option<u16>,
    wakes: &mut Vec<Wake>,
) {
    for entry in &row.entries {
        if (entry.operation == op || (op == Operation::Pending && entry.operation == Operation::Verdict))
            && member.is_none_or(|member| member == entry.member)
            && gate.is_none_or(|gate| gate == entry.gate)
            && entry.preparation.is_none_or(|round| round == row.landing.preparation)
            && view.forge.inspection_entry(entry.entry).is_some()
        {
            wakes.push(Wake::Party(format!("forge: entry {}", entry.entry)));
        }
    }
    if wakes.is_empty() {
        wakes.push(Wake::Ended(format!("{kind} write")));
    }
}

fn change_kind(state: &State) -> Option<&'static str> {
    Some(match state {
        State::Taking { .. } | State::Producing { .. } => "change: producing",
        State::Opening { .. } | State::Recreating { .. } | State::Reopening { .. } => "change: opening",
        State::Checking { .. } => "change: checking",
        State::Gating { .. } => "change: gating",
        State::Queued { .. } => "change: queued",
        State::First { .. } => "change: first",
        State::Landing { .. }
        | State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. } => "change: landing",
        State::Ready { .. } => "change: ready",
        State::Preparing { .. } => "change: preparing",
        State::Judging { .. } => "change: judging",
        State::Updating { .. } => "change: updating",
        State::WaitingProducer { .. } | State::Resolving { .. } => "change: resolving",
        State::Repairing { .. } => "change: repairing",
        State::CancellingGates { .. } => "change: cancelling gates",
        State::Held { .. } => "change: held",
        State::Ended => return None,
    })
}
