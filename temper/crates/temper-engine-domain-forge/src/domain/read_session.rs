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
        let mut session = Session::new(env, since.min(env.wall));
        session.exempt = !super::fresh_reads_in(d);
        d.read_sessions.insert(task, session).expect("one clock per change");
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
    super::wait_fresh(d, pending.task);
    let Some(row) = d.changes.get(&pending.task) else { return };
    let mut exempt = !super::fresh_reads_in(d)
        || row.room.is_some()
        || pending.heard.delegate_live
        || waits_for_dependencies(row, pending)
        || connection_wait::waiting(d, row.task, row.repository.forge);
    if let Some(owner) = step_owner(d, row.task) {
        exempt |= outage::reading(d, owner);
    }
    if !d.read_sessions.contains_key(&pending.task) {
        let since = if let change::State::Taking { .. } = row.change.state { row.change.since } else { env.wall };
        restart(d, env, pending.task, since);
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
    if !super::fresh_reads_in(d) {
        return None;
    }
    let session = d.read_sessions.get(&task)?;
    if session.exempt { None } else { Some(session.until) }
}

pub(super) fn deadline(d: &Domain) -> Option<Wall> {
    if !super::fresh_reads_in(d) {
        return None;
    }
    let mut next: Option<Wall> = None;
    for (task, session) in &d.read_sessions {
        let Some(row) = d.changes.get(task) else { continue };
        if !session.started || session.exempt || row.frozen() || row.room.is_some() {
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
    if !super::fresh_reads_in(d) {
        return;
    }
    let mut due = List::with_capacity(env.limits.changes);
    for (task, session) in &d.read_sessions {
        if session.started && !session.exempt && session.until <= env.wall {
            due.push(*task).expect("bounded read clocks");
        }
    }
    for task in due.into_boxed() {
        let Some(row) = d.changes.get(&task) else { continue };
        if row.frozen() {
            d.read_sessions.remove(&task);
            if let Some(owner) = step_owner(d, task) {
                d.steps.remove(&owner);
            }
            continue;
        }
        if let change::State::Held { .. } = row.change.state {
            continue;
        }
        if row.room.is_some() || connection_wait::waiting(d, task, row.repository.forge) {
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

/// A restart's reads spend none of a session's stall allowance (forge/catching-up.md, 4).
pub(super) fn fresh(d: &mut Domain, env: &Env<Limits>) {
    let mut sessions = skein_lib::Map::with_capacity(d.read_sessions.capacity());
    for (task, prior) in &d.read_sessions {
        let mut session = Session::new(env, env.wall);
        session.started = prior.started;
        sessions.insert(*task, session).expect("existing read clocks");
    }
    // A restored taking can get its first procedure step after the fresh pass.
    // Its stored since predates that pass just as an already-started session's does.
    for (task, row) in &d.changes {
        if let change::State::Taking { .. } = row.change.state
            && !sessions.contains_key(task)
        {
            sessions.insert(*task, Session::new(env, env.wall)).expect("one clock per change");
        }
    }
    d.read_sessions = sessions;
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
