//! The gather's volatile clock is separate from policy wait and settling
//! origins (temper/docs/design/domain/forge/change/repairs.md, Stalls).
use super::{
    Domain, Env, Limits, PendingStep, Request, change, connection_wait, outage, step_owner, wait_for_forge,
    waits_for_dependencies,
};
use skein_lib::{List, Queue, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Session {
    since: Wall,
    until: Wall,
    exempt: bool,
    ended: bool,
    started: bool,
}

impl Session {
    fn new(env: &Env<Limits>, since: Wall) -> Self {
        Self {
            since,
            until: Wall::from_nanos(since.as_nanos().saturating_add(env.limits.change_policy.stall.as_nanos())),
            exempt: false,
            ended: false,
            started: false,
        }
    }
}

pub(crate) fn restart(d: &mut Domain, env: &Env<Limits>, task: u64, since: Wall) {
    if d.changes.contains_key(&task) {
        d.read_sessions.insert(task, Session::new(env, since.min(env.wall))).expect("one clock per change");
    }
}

pub(crate) fn ended(d: &mut Domain, env: &Env<Limits>, task: u64, at: Wall) {
    let (exempt, started) = match d.read_sessions.get(&task) {
        Some(session) => (session.exempt, session.started),
        None => (false, false),
    };
    restart(d, env, task, at);
    if let Some(session) = d.read_sessions.get_mut(&task) {
        session.ended = true;
        session.exempt = exempt;
        session.started = started;
    }
}

pub(crate) fn pause(d: &mut Domain, task: u64) {
    if let Some(session) = d.read_sessions.get_mut(&task) {
        session.exempt = true;
    }
}

pub(super) fn refresh(d: &mut Domain, env: &Env<Limits>, pending: &PendingStep) {
    let Some(row) = d.changes.get(&pending.task) else { return };
    let mut exempt = pending.heard.delegate_live
        || waits_for_dependencies(row, pending)
        || connection_wait::waiting(d, row.task, row.repository.forge);
    if let Some(owner) = step_owner(d, row.task) {
        exempt |= outage::reading(d, owner);
    }
    if !d.read_sessions.contains_key(&pending.task) {
        restart(d, env, pending.task, env.wall);
    }
    let session = d.read_sessions.get_mut(&pending.task).expect("known gather clock");
    if session.exempt && !exempt && !session.ended {
        *session = Session::new(env, env.wall);
    }
    session.exempt = exempt;
    session.started = true;
    if !exempt {
        session.ended = false;
    }
}

pub(super) fn until(d: &Domain, task: u64) -> Option<Wall> {
    let session = d.read_sessions.get(&task)?;
    if session.exempt { None } else { Some(session.until) }
}

pub(super) fn deadline(d: &Domain) -> Option<Wall> {
    let mut next: Option<Wall> = None;
    for (task, session) in &d.read_sessions {
        let Some(row) = d.changes.get(task) else { continue };
        if !session.started || session.exempt || row.terminal() {
            continue;
        }
        if let change::State::Held { .. } = row.change.state {
            continue;
        }
        next = Some(match next {
            Some(previous) => previous.min(session.until),
            None => session.until,
        });
    }
    next
}

pub(super) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut due = List::with_capacity(env.limits.changes);
    for (task, session) in &d.read_sessions {
        if session.started && !session.exempt && session.until <= env.wall {
            due.push(*task).expect("bounded read clocks");
        }
    }
    for task in due.into_boxed() {
        let Some(row) = d.changes.get(&task) else { continue };
        if row.terminal() {
            d.read_sessions.remove(&task);
            if let Some(owner) = step_owner(d, task) {
                d.steps.remove(&owner);
            }
            continue;
        }
        if let change::State::Held { .. } = row.change.state {
            continue;
        }
        if connection_wait::waiting(d, task, row.repository.forge) {
            pause(d, task);
            continue;
        }
        let Some(owner) = step_owner(d, task) else {
            super::hold_read_session(d, env, task, None, out);
            continue;
        };
        if outage::reading(d, owner) {
            pause(d, task);
            continue;
        }
        let pending = d.steps.remove(&owner).expect("due gather");
        refresh(d, env, &pending);
        let expired = match until(d, task) {
            Some(until) => until <= env.wall,
            None => false,
        };
        if expired {
            wait_for_forge(d, env, &pending, None, out);
        } else {
            d.steps.insert(owner, pending).expect("gather keeps its room");
        }
    }
}

pub(super) fn connection(d: &mut Domain, env: &Env<Limits>, forge: u16, connected: bool) {
    let mut tasks = List::with_capacity(env.limits.changes);
    for (task, _) in &d.read_sessions {
        if let Some(row) = d.changes.get(task)
            && row.repository.forge == forge
        {
            tasks.push(*task).expect("bounded connection clocks");
        }
    }
    for task in tasks.into_boxed() {
        if connected {
            ended(d, env, task, env.wall);
            d.read_sessions.get_mut(&task).expect("resumed clock").exempt = false;
        } else {
            pause(d, task);
        }
    }
}

#[cfg(test)]
pub(crate) fn client_refusals(d: &mut Domain) -> u32 {
    let mut refused = 0_u32;
    while let Some(fact) = d.client.pop_fact() {
        if let skein_forge_client::Fact::Refused = fact {
            refused = refused.saturating_add(1);
        }
    }
    refused
}
