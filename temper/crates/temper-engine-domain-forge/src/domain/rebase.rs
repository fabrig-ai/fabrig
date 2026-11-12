//! A prepare's in-flight rewrite is its own (domain/landing.md, 5.2).
//! Keeps compare with the original head; prepare instead needs the requested main.
use super::{Domain, child, drift, from_client, record_change_head};
use crate::{DriftChange, Limits, Request};
use skein_forge::api::{Answer, Change, Commit, Error, Hint, Read, UpdateStyle, Write};
use skein_forge_client::{Condition, Event, Made, Outcome, Resource, What};
use skein_lib::{Env, List, Queue, Token};
use temper_engine_domain_forge_change as change;

#[derive(Clone, Copy, Hash, Debug)]
pub(super) struct Push {
    entry: u64,
    head: Commit,
    actor: u64,
}

#[derive(Clone, Hash, Debug)]
pub(super) struct Observation {
    entry: u64,
    resource: Resource,
    expected: Commit,
    head: Commit,
    read: Option<Token>,
    owned: bool,
}

pub(super) fn pending(d: &Domain, task: u64) -> Option<(u64, Commit, Commit)> {
    let row = d.changes.get(&task)?;
    let change::State::Preparing { from, main, .. } = row.change.state else { return None };
    let number = row.pending?;
    let entry = d.entries.get(&number)?;
    if entry.held || entry.entry.task != task || entry.entry.repository != row.repository {
        return None;
    }
    match entry.entry.effect.write {
        Write::Update { style: UpdateStyle::Rebase, .. } => {}
        Write::Update { style: UpdateStyle::Merge, .. }
        | Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Merge { .. }
        | Write::Status { .. }
        | Write::Dispatch { .. }
        | Write::Rerun { .. }
        | Write::CreateBranch { .. }
        | Write::DeleteBranch { .. } => return None,
    }
    if entry.entry.effect.condition != (Condition::Update { head: from, base: main }) {
        return None;
    }
    Some((number, from, main))
}

pub(super) fn hint(d: &mut Domain, hint: &Hint) {
    let Change::BranchMoved { branch, head, actor } = &hint.change else { return };
    for (task, row) in &d.changes {
        if row.repository == hint.repository
            && row.branch == *branch
            && let Some((entry, _, _)) = pending(d, *task)
        {
            d.rebase_hints.insert(*task, Push { entry, head: *head, actor: *actor }).expect("one push per change");
        }
    }
}

pub(super) fn observe(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: &Resource,
    movement: &DriftChange,
    out: &mut Queue<Request>,
) -> bool {
    let DriftChange::Branch { expected, observed: Some(head) } = movement else { return false };
    let Some(name) = from_client(resource, &env.limits) else { return false };
    let mut found = None;
    for (task, row) in &d.changes {
        if row.repository == resource.repository
            && crate::items::branch_matches(&name, &row.branch)
            && row.drift.is_none()
            && let Some((entry, from, main)) = pending(d, *task)
            && from == *expected
            && from != *head
        {
            found = Some((*task, entry, main));
        }
    }
    let Some((task, entry, main)) = found else { return false };
    let actor = match d.rebase_hints.get(&task) {
        Some(push) if push.entry == entry && push.head == *head => Some(push.actor),
        Some(_) | None => None,
    };
    if let Some(actor) = actor
        && d.writers.get(&resource.repository.forge) != Some(&actor)
    {
        return false;
    }
    if let Some(known) = d.rebases.get(&task) {
        return known.entry == entry && known.head == *head;
    }
    let read = if actor.is_some() {
        let Some(sequence) = d.sequence.checked_add(1) else { return false };
        d.sequence = sequence;
        Some(Token::new(sequence | (1_u64 << 63)))
    } else {
        None
    };
    d.rebases
        .insert(
            task,
            Observation { entry, resource: resource.clone(), expected: *expected, head: *head, read, owned: false },
        )
        .expect("one observation per change");
    if let Some(owner) = read {
        child(
            d,
            env,
            Event::Read {
                owner,
                repository: resource.repository,
                read: Read::Commits { before: *head, after: main, page: 1 },
            },
            out,
        );
    }
    true
}

pub(super) fn reading(d: &Domain, owner: Token) -> bool {
    for (_, observation) in &d.rebases {
        if observation.read == Some(owner) {
            return true;
        }
    }
    false
}

pub(super) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let mut task = None;
    for (number, observation) in &d.rebases {
        if observation.read == Some(owner) {
            task = Some(*number);
        }
    }
    let Some(task) = task else { return };
    let status = super::ancestry_status(result);
    let observation = d.rebases.get_mut(&task).expect("read owns observation");
    observation.read = None;
    let contains = match status {
        change::Status::Passed => true,
        change::Status::Failed => false,
        change::Status::Unknown | change::Status::Pending => return,
    };
    observation.owned = contains;
    let observation = observation.clone();
    if contains {
        let row = d.changes.get(&task).expect("pending change").clone();
        if row.drift.is_some() {
            return;
        }
        record_change_head(d, env, &row, observation.head, out);
    } else {
        judge(d, env, &observation, out);
    }
}

pub(super) fn outcome(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
    outcome: Outcome,
    out: &mut Queue<Request>,
) {
    let made = match outcome {
        Outcome::Made { made: Made::Updated(head), .. } => Some(head),
        Outcome::Queued | Outcome::Uncertain => return,
        Outcome::Made { .. } | Outcome::Failed(_) | Outcome::Raced { .. } | Outcome::Withdrawn | Outcome::Held => None,
    };
    d.rebase_hints.remove(&task);
    let Some(observation) = d.rebases.remove(&task) else { return };
    if observation.entry == entry && made != Some(observation.head) && (!observation.owned || made.is_some()) {
        judge(d, env, &observation, out);
    }
}

fn judge(d: &mut Domain, env: &Env<Limits>, observation: &Observation, out: &mut Queue<Request>) {
    drift(
        d,
        env,
        observation.resource.clone(),
        DriftChange::Branch { expected: observation.expected, observed: Some(observation.head) },
        out,
    );
}

/// Rebuild a deferral from the retained entry and the saved keep observation.
/// Without retained identity, only the entry's exact completion can confirm it.
pub(super) fn read_afresh(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut branches = List::with_capacity(env.limits.changes);
    for (task, row) in &d.changes {
        if let Some((_, from, _)) = pending(d, *task) {
            let resource = Resource { repository: row.repository, what: What::Branch(row.branch.clone()) };
            if let Some(cached) = d.client.cached(&resource)
                && let Some(head) = cached.tip
                && head != from
            {
                branches
                    .push((resource, DriftChange::Branch { expected: from, observed: Some(head) }))
                    .expect("one branch per change");
            }
        }
    }
    for (resource, movement) in branches.into_boxed() {
        if !observe(d, env, &resource, &movement, out) {
            drift(d, env, resource, movement, out);
        }
    }
}
