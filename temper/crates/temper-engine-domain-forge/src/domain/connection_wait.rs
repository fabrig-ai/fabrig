//! A change waits for access without holding, polling or spending its stall
//! (temper/docs/design/domain/forge/adoption.md, 1). Wait starts are volatile;
//! the connector's change limit bounds one per change.
use super::{Domain, Env, Limits, Request, branch_due, change, emit, save_change};
use crate::Connection;
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue, Wall};

pub(super) fn contains(d: &Domain, task: u64) -> bool {
    match &d.change_connections {
        Some(waiting) => waiting.contains_key(&task),
        None => false,
    }
}

pub(super) fn take(d: &mut Domain, task: u64) -> Option<Wall> {
    let waiting = d.change_connections.as_mut()?;
    let start = waiting.remove(&task);
    if waiting.is_empty() {
        d.change_connections = None;
    }
    start
}

pub(super) fn retire(d: &mut Domain, forge: u16) {
    let Some(waits) = &d.change_connections else { return };
    let mut tasks = List::with_capacity(waits.len());
    for (task, _) in waits.as_ref() {
        if let Some(row) = d.changes.get(task)
            && row.repository.forge == forge
        {
            tasks.push(*task).expect("bounded connection waits");
        }
    }
    for task in tasks.into_boxed() {
        take(d, task);
    }
}

pub(super) fn waiting(d: &Domain, task: u64, forge: u16) -> bool {
    if contains(d, task) {
        return true;
    }
    match d.connection(forge) {
        Connection::Connected { .. } | Connection::Connecting => false,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => true,
    }
}

pub(super) fn wait(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    // Passive restored owners allocate no wait bookkeeping.
    if d.change_connections.is_none() {
        d.change_connections = Some(Box::new(Map::with_capacity(env.limits.changes)));
    }
    let waiting = d.change_connections.as_mut().expect("active connection waits");
    if !waiting.contains_key(&task) {
        waiting.insert(task, env.wall).expect("one connection wait per bounded change");
    }
    d.change_due.remove(&task);
    emit(
        out,
        Request::ChangeDecision { task, decision: change::Decision::Wait { until: None }, entry: None, evidence: None },
    );
}

pub(super) fn connected(d: &mut Domain, env: &Env<Limits>, forge: u16, out: &mut Queue<Request>) {
    let Some(waits) = &d.change_connections else { return };
    let mut waiting = List::with_capacity(waits.len());
    for (task, _) in waits.as_ref() {
        if let Some(row) = d.changes.get(task)
            && row.repository.forge == forge
        {
            waiting.push(*task).expect("bounded connection waits");
        }
    }
    for task in waiting.into_boxed() {
        let start = take(d, task).expect("collected wait");
        let row = d.changes.get_mut(&task).expect("waiting change");
        if row.terminal() {
            continue;
        }
        // As with an outage, exclude only the time after this stall began.
        let start = start.max(row.change.since);
        let elapsed = env.wall.as_nanos().saturating_sub(start.as_nanos());
        row.change.since = Wall::from_nanos(row.change.since.as_nanos().saturating_add(elapsed));
        let saved = row.clone();
        save_change(d, out, saved);
        branch_due(d, task);
        d.procedure_wakes.insert(task, true).expect("one wake per bounded change");
    }
}
