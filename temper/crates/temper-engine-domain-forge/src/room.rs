//! One first-come-first-served outbox line (domain/forge/room.md, 2–4).
use crate::{Domain, Limits, Request};
use jig_domain_core_vocabulary::waits::{Reported, ReportedBound, Thing, Unbounded, WaitKind, WaitObject};
use skein_lib::{Env, List, Map, Queue, Set, Token, Wall};
use temper_client_protocol_wire::waits::THING_ENTRY;
use temper_engine_domain_forge_landing::Phase;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Waiter {
    Change(u64),
    Landing(u64),
    Making(Token),
    Release(u64),
}

#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) places: Map<Waiter, Place>,
    pub(crate) asked: Set<Waiter>,
    passed: Set<Waiter>,
    pub(crate) ready: bool,
}

impl State {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self {
            places: Map::uncapped(),
            asked: Set::with_capacity(
                limits
                    .changes
                    .saturating_add(limits.holds)
                    .saturating_add(limits.landings)
                    .saturating_add(limits.tasks),
            ),
            passed: Set::with_capacity(
                limits
                    .changes
                    .saturating_add(limits.holds)
                    .saturating_add(limits.landings)
                    .saturating_add(limits.tasks),
            ),
            ready: false,
        }
    }
}

#[derive(Debug, Hash)]
pub(crate) struct Place {
    order: u64,
    since: Wall,
}

const NAMED_ENTRIES: usize = 4;

#[derive(PartialEq, Eq, Debug, Hash)]
pub(crate) struct Holders {
    entries: [Option<Thing>; NAMED_ENTRIES],
    // Reservations do not have an entry number yet.
    reservation: Option<u64>,
}

pub(crate) fn holders(d: &Domain) -> Holders {
    let mut entries = [None; NAMED_ENTRIES];
    for (slot, (number, row)) in entries.iter_mut().zip(d.entries.iter()) {
        *slot = Some(Thing { what: THING_ENTRY, number: *number, owner: Some(row.entry.task) });
    }
    let reservation = if d.entries.is_empty() { crate::domain::full::oldest_reservation(d) } else { None };
    Holders { entries, reservation }
}

/// Keep a place's join time alongside its durable order, without storing a second timestamp.
pub(crate) fn observe(d: &mut Domain, task: u64, since: Wall) {
    let procedure = d.changes.contains_key(&task) || d.landing.rows.contains_key(&task) || d.closer(task).is_some();
    for waiter in [Waiter::Change(task), Waiter::Landing(task), Waiter::Release(task)] {
        if procedure && let Some(order) = place(d, waiter) {
            let prior = d.room.places.get(&waiter);
            if let Some(prior) = prior
                && prior.order == order
            {
                continue;
            }
            d.room.places.insert(waiter, Place { order, since }).expect("one join per place");
        } else {
            d.room.places.remove(&waiter);
        }
    }
}

/// The room record for a procedure with a place in the line (waits.md, 7).
pub(crate) fn waited(d: &Domain, _env: &Env<Limits>, task: u64, step: u32) -> Option<Reported> {
    let mut waiting = None;
    for waiter in [Waiter::Release(task), Waiter::Change(task), Waiter::Landing(task)] {
        if let Some(place) = d.room.places.get(&waiter) {
            waiting = Some((waiter, place));
            break;
        }
    }
    let (waiter, place) = waiting?;
    let holders = holders(d);
    let [first, ..] = holders.entries;
    let behind = if let Some(entry) = first {
        entry.owner?
    } else {
        let reservation = holders.reservation?;
        if reservation == task || d.room.asked.contains(&waiter) {
            return None;
        }
        reservation
    };
    let mut things = List::with_capacity(u32::try_from(NAMED_ENTRIES).expect("wait object bound"));
    for entry in holders.entries.into_iter().flatten() {
        things.push(entry).expect("at most four entries");
    }
    Some(Reported {
        kind: WaitKind::Room,
        object: WaitObject::Procedure { step, things },
        since: place.since,
        bound: ReportedBound::None(Unbounded::Behind(behind)),
    })
}

pub(crate) fn reserved(d: &Domain) -> u32 {
    d.room.asked.len().saturating_add(d.call_room.len()).saturating_add(d.projection_room.len())
}

pub(crate) fn free(d: &Domain, limits: &Limits) -> u32 {
    limits.entries.saturating_sub(d.entries.len()).saturating_sub(reserved(d))
}

pub(crate) fn place(d: &Domain, waiter: Waiter) -> Option<u64> {
    match waiter {
        Waiter::Change(task) => d.changes.get(&task)?.room,
        Waiter::Landing(task) => d.landing.rows.get(&task)?.room,
        Waiter::Making(owner) => Some(d.makings.get(&owner)?.outbox_wait.as_ref()?.order),
        Waiter::Release(task) => d.releases.get(&task)?.room,
    }
}

fn first(d: &Domain) -> Option<Waiter> {
    let mut next: Option<(u64, Waiter)> = None;
    for (task, row) in &d.changes {
        if let Some(order) = row.room
            && !row.frozen()
            && !crate::domain::matches_held(&row.change.state)
        {
            select(d, &mut next, order, Waiter::Change(*task));
        }
    }
    for (task, row) in &d.landing.rows {
        if let Some(order) = row.room
            && !d.landing.held
        {
            match row.landing.phase {
                Phase::Held { .. } => {}
                Phase::Waiting
                | Phase::Preparing
                | Phase::Proving
                | Phase::Proved { .. }
                | Phase::Turn { .. }
                | Phase::Landed
                | Phase::Cancelled => select(d, &mut next, order, Waiter::Landing(*task)),
            }
        }
    }
    for (owner, pending) in &d.makings {
        if let Some(wait) = &pending.outbox_wait {
            select(d, &mut next, wait.order, Waiter::Making(*owner));
        }
    }
    for (task, row) in &d.releases {
        if let Some(order) = row.room {
            select(d, &mut next, order, Waiter::Release(*task));
        }
    }
    match next {
        Some((_, waiter)) => Some(waiter),
        None => None,
    }
}

fn select(d: &Domain, next: &mut Option<(u64, Waiter)>, order: u64, waiter: Waiter) {
    if match next {
        Some((before, _)) => order >= *before,
        None => false,
    } {
        return;
    }
    if !d.room.asked.contains(&waiter) && !d.room.passed.contains(&waiter) {
        *next = Some((order, waiter));
    }
}

pub(crate) fn check(d: &Domain, limits: &Limits, waiter: Option<Waiter>) -> bool {
    if let Some(waiter) = waiter
        && d.room.asked.contains(&waiter)
    {
        return d.entries.len() < limits.entries;
    }
    if free(d, limits) == 0 {
        return false;
    }
    let Some(first) = first(d) else { return true };
    match waiter {
        Some(waiter) => match place(d, waiter) {
            Some(order) => order <= place(d, first).expect("selected place"),
            None => false,
        },
        None => false,
    }
}

pub(crate) fn next_place(d: &Domain) -> u64 {
    let mut last = 0;
    for (_, row) in &d.changes {
        last = last.max(row.room.unwrap_or(0));
    }
    for (_, row) in &d.landing.rows {
        last = last.max(row.room.unwrap_or(0));
    }
    for (_, pending) in &d.makings {
        if let Some(wait) = &pending.outbox_wait {
            last = last.max(wait.order);
        }
    }
    for (_, row) in &d.releases {
        last = last.max(row.room.unwrap_or(0));
    }
    last.checked_add(1).expect("outbox places fit")
}

pub(crate) fn waiting(d: &Domain) -> bool {
    d.room.ready && d.entries.len().saturating_add(reserved(d)) < d.entries.capacity() && first(d).is_some()
}

pub(crate) fn asked(d: &Domain, waiter: Waiter) -> bool {
    d.room.asked.contains(&waiter)
}

pub(crate) fn leave(d: &mut Domain, waiter: Waiter) {
    if let Waiter::Change(task) | Waiter::Landing(task) | Waiter::Release(task) = waiter {
        crate::waits::changed(d, task);
    }
    d.room.asked.remove(&waiter);
    d.room.passed.remove(&waiter);
}

pub(crate) fn pass(d: &mut Domain, task: u64) {
    let release = Waiter::Release(task);
    d.room.asked.remove(&release);
    if place(d, release).is_some() {
        d.room.passed.insert(release).expect("one pass per waiter");
    }
    let waiter = Waiter::Change(task);
    d.room.asked.remove(&waiter);
    if place(d, waiter).is_some() {
        d.room.passed.insert(waiter).expect("one pass per waiter");
    }
    let waiter = Waiter::Landing(task);
    d.room.asked.remove(&waiter);
    if place(d, waiter).is_some() {
        d.room.passed.insert(waiter).expect("one pass per waiter");
    }
    for (owner, pending) in &d.makings {
        if pending.task == task && pending.outbox_wait.is_some() {
            let waiter = Waiter::Making(*owner);
            d.room.asked.remove(&waiter);
            d.room.passed.insert(waiter).expect("one pass per waiter");
        }
    }
}

pub(crate) fn stepping(d: &mut Domain, task: u64) {
    d.room.passed.remove(&Waiter::Change(task));
    d.room.passed.remove(&Waiter::Landing(task));
    d.room.passed.remove(&Waiter::Release(task));
    for (owner, pending) in &d.makings {
        if pending.task == task {
            d.room.passed.remove(&Waiter::Making(*owner));
        }
    }
}

pub(crate) fn ask(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    crate::domain::full::update(d, env, out);
    let mut stale = List::uncapped();
    for (owner, pending) in &d.makings {
        let live = match d.holds.get(&pending.name) {
            Some(hold) => hold.task == pending.task,
            None => false,
        };
        if pending.outbox_wait.is_some() && !live {
            stale.push(*owner).expect("making waits");
        }
    }
    let mut pruned = !stale.is_empty();
    for owner in stale.into_boxed() {
        d.makings.remove(&owner);
        leave(d, Waiter::Making(owner));
    }
    // A completed making may have left during its read's answer.
    let mut gone = List::uncapped();
    for waiter in &d.room.asked {
        if place(d, *waiter).is_none() {
            gone.push(*waiter).expect("asked waiters");
        }
    }
    pruned |= !gone.is_empty();
    for waiter in gone.into_boxed() {
        leave(d, waiter);
    }
    // Pruning can end a full interval before the next admission fills it again.
    // At the clock's maximum a saturated report deadline remains due.
    let saturated = env.now.as_nanos() == u64::MAX;
    if pruned || saturated {
        crate::domain::full::update(d, env, out);
    }
    if !d.room.ready || d.stopping {
        crate::waits::flush(d, env, out);
        return;
    }
    let mut admitted = false;
    while free(d, &env.limits) > 0 {
        let Some(waiter) = first(d) else { break };
        admitted = true;
        d.room.asked.insert(waiter).expect("one ask per waiter");
        let task = match waiter {
            Waiter::Change(task) | Waiter::Landing(task) | Waiter::Release(task) => task,
            Waiter::Making(owner) => d.makings.get(&owner).expect("selected making").task,
        };
        d.full.reserve(crate::domain::full::Reservation::Asked(waiter), task);
        let request = match waiter {
            Waiter::Change(task) => {
                d.procedure_wakes.remove(&task);
                Request::WakeProcedure { task }
            }
            Waiter::Landing(task) => Request::WakeProcedure { task },
            Waiter::Release(task) => Request::ContinueRelease { task },
            Waiter::Making(owner) => {
                let pending = d.makings.get(&owner).expect("selected making");
                Request::ContinueGoalBranch { task: pending.task }
            }
        };
        out.push(request);
    }
    // With no admission, neither holders nor report inputs changed since the last update.
    if admitted || saturated {
        crate::domain::full::update(d, env, out);
    }
    crate::waits::flush(d, env, out);
}
