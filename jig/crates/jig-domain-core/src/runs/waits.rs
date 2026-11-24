//! Task waits and their readings (jig/docs/design/domain/engine/waits.md, 4).
pub use jig_domain_core_vocabulary::runs::waits::*;

mod own;

use crate::{Bound, Core, Limits, Nearest, Wait, WaitKind, WaitObject, Waits, documents::PhaseCounts};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Token, Wall};

impl Core {
    /// A live task's waits. An unknown or ended task has none.
    #[must_use]
    pub fn waits(&self, task: u64) -> Waits {
        let Some(row) = self.tasks.task(task) else { return empty() };
        read(self, &self.wait_limits, row).waits
    }

    /// The compatibility summary of the waits in a live subtree.
    #[must_use]
    pub fn waits_on(&self, limits: &Limits, task: u64) -> WaitsOn {
        let Some(row) = self.tasks.task(task) else { return WaitsOn::Nothing };
        from_row(self, limits, row)
    }
}

pub(crate) fn from_row(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> WaitsOn {
    read(core, limits, row).on
}

pub(crate) struct Projection {
    pub(crate) waits: Waits,
    pub(crate) on: WaitsOn,
    pub(crate) phases: PhaseCounts,
    pub(crate) due_wait: Option<crate::documents::TaskRunWait>,
}

fn empty() -> Waits {
    Waits { first: None, nearest: None, own: List::with_capacity(0), count: 0 }
}

/// Fold phases and eligible party entries in the same bounded walk. Only the
/// subject needs its full set of records; below it only the oldest entry can
/// become a record on this line.
pub(crate) fn read(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> Projection {
    if let tasks::Phase::Ended(_) = row.phase {
        return Projection { waits: empty(), on: WaitsOn::Nothing, phases: PhaseCounts::ZERO, due_wait: None };
    }
    let mut phases = PhaseCounts::ZERO;
    let mut pending = List::with_capacity(limits.tasks.tree_tasks);
    pending.push(row.number).expect("validated tree bound");
    let mut oldest: Option<(people::Entry, &tasks::TaskRecord)> = None;
    let mut entries = 0_u32;
    let hears = crate::admission::escalation::wait_holder(core, limits, row, None);
    let mut own = own::Own::new(hears);
    for _ in 0..limits.tasks.tree_tasks {
        let Some(index) = pending.len().checked_sub(1) else { break };
        let number = *pending.get(index).expect("pending live task");
        pending.truncate(index);
        let candidate = if number == row.number {
            row
        } else {
            let Some(candidate) = core.tasks.task(number) else { continue };
            candidate
        };
        phases.add(&candidate.phase);
        for entry in core.waiting_entries(limits, candidate).as_ref() {
            if !party_entry(candidate, entry.kind) {
                continue;
            }
            entries = entries.saturating_add(1);
            let earlier = match oldest {
                Some((previous, _)) => (entry.at, entry.task) < (previous.at, previous.task),
                None => true,
            };
            if earlier {
                oldest = Some((*entry, candidate));
            }
            if number == row.number {
                own.insert(own::party(core, limits, row, entry, hears));
            }
        }
        for delegate in candidate.delegates.iter().rev() {
            pending.push(*delegate).expect("live subtree within tree_tasks");
        }
    }
    let due_wait = match row.phase {
        tasks::Phase::Active(tasks::Active::Due) => crate::documents::run_wait(core, row),
        tasks::Phase::Active(_)
        | tasks::Phase::Waiting
        | tasks::Phase::Held { .. }
        | tasks::Phase::Closing(_)
        | tasks::Phase::Ended(_) => None,
    };
    let (working, parked) = activity(core, row);
    let own = own.finish(core, limits, row, working, parked, due_wait);
    let party = match oldest {
        Some((entry, candidate)) => {
            let holder = if candidate.number == row.number {
                hears
            } else {
                crate::admission::escalation::wait_holder(core, limits, candidate, None)
            };
            Some(own::party(core, limits, candidate, &entry, holder))
        }
        None => None,
    };
    let on = summary(core, limits, row, party.as_ref(), entries, working, &own);
    let first = match party {
        Some(_) => party,
        None if working => None,
        None => match own.first_key_value() {
            Some((_, wait)) => Some(wait.clone()),
            None => None,
        },
    };
    let mut nearest = None;
    if let Some(wait) = &first {
        approach(&mut nearest, wait);
    }
    for wait in own.values() {
        approach(&mut nearest, wait);
    }
    let count = u32::try_from(own.len()).expect("bounded waits");
    let mut lines = List::with_capacity(limits.documents.lines);
    for wait in own.into_values().take(skein_lib::usize_of(limits.documents.lines)) {
        lines.push(wait).expect("document wait lines");
    }
    Projection { waits: Waits { first, nearest, own: lines, count }, on, phases, due_wait }
}

fn party_entry(row: &tasks::TaskRecord, kind: people::EntryKind) -> bool {
    match kind {
        people::EntryKind::Question { .. }
        | people::EntryKind::Proposal { .. }
        | people::EntryKind::Escalation { .. } => true,
        people::EntryKind::PersonTask => row.waiting_on.is_empty(),
        people::EntryKind::ChatReply { .. } | people::EntryKind::Reply { .. } => false,
    }
}

fn activity(core: &Core, row: &tasks::TaskRecord) -> (bool, Option<Wall>) {
    match row.phase {
        tasks::Phase::Active(tasks::Active::Claimed { attempt } | tasks::Active::Running { attempt }) => {
            if let Some(value) = core.fleet.observation(Token::new(row.number), Token::new(attempt))
                && value.activity == fleet::Activity::Waiting
            {
                (false, Some(value.since))
            } else {
                (true, None)
            }
        }
        tasks::Phase::Active(tasks::Active::Idle) => (false, Some(row.phase_since)),
        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Preparing | tasks::Active::BackingOff { .. })
        | tasks::Phase::Waiting
        | tasks::Phase::Held { .. }
        | tasks::Phase::Closing(_)
        | tasks::Phase::Ended(_) => (false, None),
    }
}

fn approach(nearest: &mut Option<Nearest>, wait: &Wait) {
    if let Bound::At { at, then } = wait.bound {
        if let Some(previous) = nearest
            && previous.at <= at
        {
            return;
        }
        *nearest = Some(Nearest { at, then, kind: wait.kind });
    }
}

fn summary(
    core: &Core,
    limits: &Limits,
    row: &tasks::TaskRecord,
    party: Option<&Wait>,
    entries: u32,
    working: bool,
    own: &own::Records,
) -> WaitsOn {
    if let Some(wait) = party
        && let WaitObject::Entry { kind, at } = wait.object
    {
        let whom = match at {
            tasks::EscalationHolder::Person(person) => people::Whom::Person(person),
            tasks::EscalationHolder::Role { project, role } => people::Whom::Role { project, role },
            tasks::EscalationHolder::Task(_) => unreachable!("the line's oldest entry is in a party inbox"),
        };
        return WaitsOn::Party { whom, entries, task: wait.task, kind };
    }
    if working {
        return WaitsOn::Nothing;
    }
    if let tasks::Phase::Active(_) = row.phase {
        // The legacy keep names the lowest question number, not the earliest wall reading.
        let mut question = match row.requester {
            tasks::Party::Task(_) => None,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => tasks::own_question(row),
        };
        for wait in own.values() {
            if let WaitObject::Question { number, .. } = wait.object {
                question = Some(question.unwrap_or(number).min(number));
            }
        }
        if let Some(number) = question {
            return WaitsOn::Question { number };
        }
        let pending = dependencies(core, limits, own);
        if let WaitsOn::Dependencies { count, .. } = pending
            && count != 0
        {
            return pending;
        }
    }
    for wait in own.values() {
        match wait.kind {
            WaitKind::Delegate => return WaitsOn::Delegates,
            WaitKind::News => return WaitsOn::News,
            WaitKind::Party
            | WaitKind::Question
            | WaitKind::Dependency
            | WaitKind::Resource
            | WaitKind::Account
            | WaitKind::Host
            | WaitKind::Step
            | WaitKind::Room
            | WaitKind::Turn
            | WaitKind::SignOff => {}
        }
    }
    // Closing has no records of its own, but the compatibility view still names live delegates.
    if let tasks::Phase::Closing(_) = row.phase
        && !row.delegates.is_empty()
    {
        return WaitsOn::Delegates;
    }
    WaitsOn::Nothing
}

fn dependencies(core: &Core, limits: &Limits, own: &own::Records) -> WaitsOn {
    let mut order = List::with_capacity(u32::try_from(own.len()).expect("bounded waits"));
    for wait in own.values() {
        if wait.kind == WaitKind::Dependency
            && let WaitObject::Task(number) = wait.object
        {
            // WaitsOn orders by the dependency's creation, not by when it was added here.
            let made = match core.tasks.task(number) {
                Some(row) => row.created_at,
                None => Wall::EPOCH,
            };
            order.push((made, number)).expect("bounded dependency count");
        }
    }
    let count = order.len();
    let mut order = order.into_boxed();
    order.sort_unstable();
    let mut pending = List::with_capacity(limits.documents.lines);
    for (_, number) in order.into_iter().take(skein_lib::usize_of(limits.documents.lines)) {
        pending.push(number).expect("bounded pending dependencies");
    }
    WaitsOn::Dependencies { pending, count }
}
