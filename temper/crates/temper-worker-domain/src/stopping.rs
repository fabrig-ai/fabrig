//! Root-owned stop deadlines (domain/worker/README.md, section 2;
//! jig's domain/hosts/hub.md, sections 6 and 7).
use crate::{Domain, Limits, Request, domain, host, route, workspace};
use skein_lib::{Env, List, Queue, Time, Token};
use smith_host_domain as agent;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ContactStop {
    at: Time,
    cut: bool,
}

pub(crate) fn shutdown(domain: &mut Domain, env: &Env<Limits>, drain: bool) {
    if domain.stopping {
        return;
    }
    domain.stopping = true;
    domain.stop_at = Some(env.now.saturating_add(env.limits.stop_bound));
    let event = if drain {
        domain.drain_at = Some(env.now.saturating_add(env.limits.drain));
        host::Event::Drain
    } else {
        host::Event::CancelAll { reason: host::Reason::Shutdown }
    };
    route::host_step(domain, env, event);
}

/// Only the attempts cancelled by this loss are subject to its deadline.
/// A later hello can admit new attempts while these finish their saves.
pub(crate) fn contact(domain: &mut Domain, env: &Env<Limits>) {
    let mut stale = List::with_capacity(domain.contact_stops.len());
    for (owner, _) in &domain.contact_stops {
        if domain.host.hosting(*owner).is_none() {
            stale.push(*owner).expect("one entry per contact stop");
        }
    }
    for owner in &stale {
        domain.contact_stops.remove(owner);
    }
    let at = env.now.saturating_add(env.limits.after_grace);
    let mut owners = skein_lib::Set::with_capacity(env.limits.host.slots);
    for (_, workspace) in &domain.workspaces {
        owners.insert(workspace.run).expect("one owner per hosted slot");
    }
    for (owner, _) in &domain.agents {
        owners.insert(*owner).expect("one owner per hosted slot");
    }
    for (owner, _) in &domain.jobs {
        owners.insert(*owner).expect("one owner per hosted slot");
    }
    for pending in &domain.pending {
        owners.insert(pending.owner()).expect("one owner per hosted slot");
    }
    for owner in &owners {
        if domain.contact_stops.get(owner).is_none() {
            domain.contact_stops.insert(*owner, ContactStop { at, cut: false }).expect("one stop per hosted slot");
        }
    }
}

pub(crate) fn cut(domain: &Domain, owner: Token) -> bool {
    match domain.contact_stops.get(&owner) {
        Some(stop) => stop.cut,
        None => false,
    }
}

pub(crate) fn next(domain: &Domain) -> Option<Time> {
    let mut next = skein_lib::earliest(domain.drain_at, domain.stop_at);
    for (_, stop) in &domain.contact_stops {
        if !stop.cut {
            next = skein_lib::earliest(next, Some(stop.at));
        }
    }
    next
}

pub(crate) fn due(domain: &Domain, now: Time) -> bool {
    match next(domain) {
        Some(at) => at <= now,
        None => false,
    }
}

pub(crate) fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if passed(domain.stop_at, env.now) {
        domain.stop_at = None;
        domain.drain_at = None;
        domain.stopping = false;
        domain::give_up(domain);
        out.push(Request::Abandon);
        return;
    }
    if passed(domain.drain_at, env.now) {
        domain.drain_at = None;
        route::host_step(domain, env, host::Event::CancelAll { reason: host::Reason::Shutdown });
    }
    let mut owners = List::with_capacity(domain.contact_stops.len());
    for (owner, stop) in &domain.contact_stops {
        if !stop.cut && stop.at <= env.now {
            owners.push(*owner).expect("one owner per contact stop");
        }
    }
    for owner in &owners {
        domain.contact_stops.get_mut(owner).expect("collected stop").cut = true;
        workspace::cut(domain, env, *owner);
        if let Some(bound) = domain.agents.get(owner) {
            let agent = bound.agent;
            route::agent_step(domain, env, agent::Input::Parent(agent::parent::Event::Drop { agent }));
        }
    }
}

fn passed(at: Option<Time>, now: Time) -> bool {
    match at { Some(at) => at <= now, None => false }
}
