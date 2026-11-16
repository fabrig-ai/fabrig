//! One first-come-first-served outbox line (domain/forge/room.md, 2–4).
use crate::{Domain, Limits, Request};
use skein_lib::{Env, Queue, Set, Token};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Waiter {
    Change(u64),
    Making(Token),
    Release(u64),
}

#[derive(Hash, Debug)]
pub(crate) struct State {
    asked: Set<Waiter>,
    passed: Set<Waiter>,
    pub(crate) ready: bool,
}

impl State {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self {
            asked: Set::with_capacity(limits.changes.saturating_add(limits.holds).saturating_add(limits.tasks)),
            passed: Set::with_capacity(limits.changes.saturating_add(limits.holds).saturating_add(limits.tasks)),
            ready: false,
        }
    }
}

pub(crate) fn reserved(d: &Domain) -> u32 {
    d.room.asked.len()
}

pub(crate) fn free(d: &Domain, limits: &Limits) -> u32 {
    limits.entries.saturating_sub(d.entries.len()).saturating_sub(reserved(d))
}

fn place(d: &Domain, waiter: Waiter) -> Option<u64> {
    match waiter {
        Waiter::Change(task) => d.changes.get(&task)?.room,
        Waiter::Making(owner) => Some(d.makings.get(&owner)?.outbox_wait.as_ref()?.order),
        Waiter::Release(task) => d.releases.get(&task)?.room,
    }
}

fn first(d: &Domain) -> Option<Waiter> {
    let mut next: Option<(u64, Waiter)> = None;
    for (task, row) in &d.changes {
        if let Some(order) = row.room {
            select(d, &mut next, order, Waiter::Change(*task));
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
    if d.room.asked.contains(&waiter) || d.room.passed.contains(&waiter) {
        return;
    }
    if let Waiter::Change(task) = waiter
        && let Some(row) = d.changes.get(&task)
        && (row.frozen() || crate::domain::matches_held(&row.change.state))
    {
        return;
    }
    if match next {
        Some((before, _)) => order < *before,
        None => true,
    } {
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
    d.room.passed.remove(&Waiter::Release(task));
    for (owner, pending) in &d.makings {
        if pending.task == task {
            d.room.passed.remove(&Waiter::Making(*owner));
        }
    }
}

pub(crate) fn ask(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut stale = skein_lib::List::uncapped();
    for (owner, pending) in &d.makings {
        let live = match d.holds.get(&pending.name) {
            Some(hold) => hold.task == pending.task,
            None => false,
        };
        if pending.outbox_wait.is_some() && !live {
            stale.push(*owner).expect("making waits");
        }
    }
    for owner in stale.into_boxed() {
        d.makings.remove(&owner);
        leave(d, Waiter::Making(owner));
    }
    // A completed making may have left during its read's answer.
    let mut gone = skein_lib::List::uncapped();
    for waiter in &d.room.asked {
        if place(d, *waiter).is_none() {
            gone.push(*waiter).expect("asked waiters");
        }
    }
    for waiter in gone.into_boxed() {
        leave(d, waiter);
    }
    if !d.room.ready || d.stopping {
        return;
    }
    while free(d, &env.limits) > 0 {
        let Some(waiter) = first(d) else { break };
        d.room.asked.insert(waiter).expect("one ask per waiter");
        let request = match waiter {
            Waiter::Change(task) => {
                d.procedure_wakes.remove(&task);
                Request::WakeProcedure { task }
            }
            Waiter::Release(task) => Request::ContinueRelease { task },
            Waiter::Making(owner) => {
                let pending = d.makings.get(&owner).expect("selected making");
                Request::ContinueGoalBranch { task: pending.task }
            }
        };
        out.push(request);
    }
}
