//! The keep supplies observations; only the write record judges them
//! (temper/docs/design/domain/forge/writes.md, 4–5).
use super::{Actor, Domain, Limits, Name, Queue, Request, Seen};
use skein_forge::api::Commit;
use skein_forge_client::{Event, Resource};
use skein_lib::{Env, List};

#[derive(Clone, Copy, Hash, Debug)]
pub(in crate::domain) struct Movement {
    expected: Commit,
    observed: Option<Commit>,
    actor: Actor,
}

pub(in crate::domain) fn moved(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: &Resource,
    expected: Commit,
    observed: Option<Commit>,
    actor: Option<u64>,
    out: &mut Queue<Request>,
) {
    let Some(name) = super::super::from_client(resource, &env.limits) else { return };
    let actor = match actor {
        Some(author) if d.writers.get(&resource.repository.forge) == Some(&author) => Actor::Writer,
        Some(_) => Actor::Other,
        None => Actor::Unknown,
    };
    if let Some(prior) = d.kept_moves.remove(&name)
        && prior.observed == observed
        && actor == Actor::Unknown
    {
        // A repeated read without a hint does not undo an attribution we
        // are still checking for this observation.
        judge(d, env, name, Movement { expected, observed, actor: prior.actor }, out);
    } else {
        judge(d, env, name, Movement { expected, observed, actor }, out);
    }
}

fn judge(d: &mut Domain, env: &Env<Limits>, name: Name, movement: Movement, out: &mut Queue<Request>) {
    let seen = if d.writes.contains_key(&name) {
        d.observe(&name, movement.observed, movement.actor, out)
    } else {
        // Previous-layout watches can still have an expected head without
        // a record. They have no entries that can explain a move.
        Seen::Drift { nearest: None }
    };
    match seen {
        Seen::Current | Seen::Explained { .. } | Seen::Behind { .. } => {}
        Seen::Owed => {
            d.kept_moves.insert(name, movement).expect("one observation per watched branch");
        }
        Seen::Drift { .. } => {
            let Some(resource) = super::super::to_client(&name, &env.limits) else { return };
            let expected = super::known_head(d, &name).unwrap_or(movement.expected);
            let change = crate::DriftChange::Branch { expected, observed: movement.observed };
            super::super::drift(d, env, resource, change, out);
        }
    }
}

pub(in crate::domain) fn rejudge(d: &mut Domain, env: &Env<Limits>, name: &Name, out: &mut Queue<Request>) {
    if let Some(movement) = d.kept_moves.remove(name) {
        judge(d, env, name.clone(), movement, out);
    }
}

pub(in crate::domain) fn rejudge_effect(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    // An outcome can settle or remove the very intent a kept read is waiting for.
    let mut names = List::with_capacity(d.kept_moves.len());
    for (name, _) in &d.kept_moves {
        names.push(name.clone()).expect("one name per pending observation");
    }
    for name in &names {
        rejudge(d, env, name, out);
    }
}

pub(in crate::domain) fn fence(d: &mut Domain, env: &Env<Limits>, resource: &Resource) {
    if let Some(name) = super::super::from_client(resource, &env.limits) {
        d.kept_moves.remove(&name);
    }
}

/// A new watch or a restored watch inherits the record's head.
pub(in crate::domain) fn watches(d: &mut Domain, env: &Env<Limits>) {
    for resource in d.client.live_keys() {
        if let Some(name) = super::super::from_client(&resource, &env.limits)
            && d.writes.contains_key(&name)
        {
            d.write_heads.insert(name).expect("uncapped head changes");
        }
    }
    let mut gone = List::with_capacity(d.kept_moves.len());
    for (name, _) in &d.kept_moves {
        if let Some(resource) = super::super::to_client(name, &env.limits)
            && d.client.cached(&resource).is_none()
        {
            gone.push(name.clone()).expect("one name per pending observation");
        }
    }
    for name in &gone {
        d.kept_moves.remove(name);
    }
}

pub(in crate::domain) fn sync_heads(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    while let Some(name) = d.write_heads.pop_first() {
        let Some(record) = d.writes.get(&name) else { continue };
        let Some(commit) = record.head else { continue };
        let previous = d.branch_head(&name);
        let owned = d.holds.contains_key(&name) || super::super::owned_head(d, &name);
        if previous != Some(commit) && owned {
            super::super::record_owned_head(d, env, &name, commit, None, out);
        }
        let Some(resource) = super::super::to_client(&name, &env.limits) else { continue };
        if let Some(cached) = d.client.cached(&resource)
            && (owned || cached.tip == Some(commit))
            && d.client.expected(&resource) != Some(commit)
        {
            // Pushed also sets the client's cached tip. On a shared landing
            // branch, let the keep report the move before setting its expectation.
            skein_forge_client::step(
                &mut d.client,
                &super::super::child_env(env),
                Event::Pushed { resource, commit },
                &mut d.client_out,
            );
        }
    }
}
