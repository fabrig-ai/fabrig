use crate::{View, landing_ended, task_name};
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
}

fn landing_wait(view: &View<'_>, row: &LandingRow) -> Option<Wait> {
    let task = row.landing.task;
    if landing_ended(&row.landing.phase) || view.ended(task) {
        return None;
    }
    let (kind, mut wakes) =
        if row.room.is_some() { ("landing: room".to_owned(), room(view)) } else { landing(view, row, row.waiting?) };
    if row.room.is_none() {
        wakes.extend(view.read_wakes(task));
        if let Some(due) = view.forge.inspection_landing_due(task) {
            wakes.push(Wake::Timer(Time::from_nanos(due.as_nanos())));
        }
    }
    Some(Wait { waiter: task_name(task), kind, wakes, bound: None })
}

fn change_wait(view: &View<'_>, row: &temper_engine_domain_forge::ChangeRow) -> Option<Wait> {
    let task = row.task;
    if view.ended(task) {
        return None;
    }
    let kind = change_kind(&row.change.state)?;
    let (kind, wakes) = if row.room.is_some() {
        ("change: room", room(view))
    } else {
        let mut wakes = change_wakes(view, row);
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
    Some(Wait { waiter: task_name(task), kind: kind.into(), wakes, bound: bound(view, row) })
}

fn change_wakes(view: &View<'_>, row: &temper_engine_domain_forge::ChangeRow) -> Vec<Wake> {
    let mut wakes = Vec::new();
    match row.change.state {
        State::Held { .. } => wakes.push(Wake::Party("person".into())),
        State::Ready { .. } | State::Preparing { .. } => {
            if let Some(task) = row.change.landing {
                wakes.push(view.waiter(task));
            } else if matches!(row.change.state, State::Ready { .. })
                && view.forge.inspection_landings().len() >= view.forge.inspection_landings().capacity()
            {
                for (task, landing) in view.forge.inspection_landings() {
                    if !landing_ended(&landing.landing.phase) {
                        wakes.push(view.waiter(*task));
                    }
                }
            }
        }
        _ => {
            if let Some((delegate, _)) = row.delegate {
                // At the forge alone, the delegate's standing is its lifecycle projection.
                if view.root.is_some() || row.delegate_status == temper_engine_domain_forge_change::Status::Unknown {
                    wakes.push(view.waiter(delegate));
                }
            }
            if let Some(entry) = row.pending
                && view.forge.inspection_entry(entry).is_some()
            {
                wakes.push(Wake::Party(format!("forge: entry {entry}")));
            }
            if !row.questions.open.is_empty() {
                wakes.push(view.answerer(row.task));
            }
        }
    }
    wakes
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
    if matches!(row.change.state, State::Producing { .. })
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
        _ => row.change.since.as_nanos().saturating_add(limits.stall.as_nanos()),
    };
    Some(Time::from_nanos(nanos))
}

fn room(view: &View<'_>) -> Vec<Wake> {
    view.forge
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
        .collect()
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
            wakes.push(view.prepare(row.landing.task));
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
