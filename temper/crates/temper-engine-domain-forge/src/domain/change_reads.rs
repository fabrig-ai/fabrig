//! One change's reads through the write record (domain/forge/change/drift.md, 1).
use super::{Domain, Env, Limits, PendingStep, Request, change, writes};
use crate::{Actor, BehindRead, ChangeRow, ReadBehind, Seen};
use skein_forge::api::{Commit, Pull, State};
use skein_lib::{Queue, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct Drift {
    pub(super) why: change::Cause,
    pub(super) nearest: Option<u64>,
}

pub(super) enum Judgement {
    Read,
    Owed,
    Behind(ReadBehind),
    Drift(Drift),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Recovery {
    Taking,
    Recreating,
    Reopening,
    Opening,
    None,
}

fn recovery(state: &change::State) -> Recovery {
    use change::State;
    match state {
        State::Taking { .. } => Recovery::Taking,
        State::Recreating { .. } => Recovery::Recreating,
        State::Reopening { .. } => Recovery::Reopening,
        State::Opening { .. } => Recovery::Opening,
        State::Producing { .. }
        | State::Checking { .. }
        | State::Gating { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::CancellingGates { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::NoChange { .. }
        | State::Ended
        | State::Held { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::Judging { .. } => Recovery::None,
    }
}

pub(super) fn branch(d: &mut Domain, env: &Env<Limits>, pending: &PendingStep, out: &mut Queue<Request>) -> Judgement {
    if !pending.branch_fresh || pending.heard.cancelled {
        return Judgement::Read;
    }
    let row = d.changes.get(&pending.task).expect("live step");
    let Some(name) = writes::branch(&env.limits, row.repository, &row.branch) else { return Judgement::Read };
    let pin = row.change.state.pinned_head().or(row.change.last_head).or(row.kept);
    let expected = writes::record_head(d, &name).or(pin);
    let actor = actor(d, row, pending.branch, BehindRead::Branch);
    let accept = super::takes_released_head(&row.change);
    let recovery = recovery(&row.change.state);
    let creating = recovery == Recovery::Taking || recovery == Recovery::Recreating;
    let legacy = if let Some(head) = d.heads.get(&name)
        && head.owned
        && row.drift.is_none()
    {
        Some(head.commit)
    } else {
        pin
    };
    let held_new_head = super::change_requester_writing(d, row);
    let seen = d.observe(&name, pending.branch, actor, out);
    if accept {
        return Judgement::Read;
    }
    match seen {
        Seen::Current if !d.writes.contains_key(&name) => {
            // Older procedure rows can predate the write record. Their last
            // confirmed head is the baseline until they have a record.
            let missing = !creating && pending.branch.is_none() && (pin.is_some() || recovery == Recovery::Opening);
            let moved = pending.branch.is_some()
                && pending.branch != pin
                && (actor == Actor::Other || pin.is_some() && pending.branch != legacy && !held_new_head);
            if missing || moved { drifted_branch(pending.branch, None) } else { Judgement::Read }
        }
        Seen::Current | Seen::Explained { .. } => {
            // No branch yet is normal while producing and during recreation.
            // If the record already has a first push, wait for it to appear.
            if pending.branch.is_none() && pin.is_none() && expected.is_some() {
                behind(BehindRead::Branch, expected, None)
            } else {
                Judgement::Read
            }
        }
        Seen::Owed => Judgement::Owed,
        Seen::Behind { .. } => behind(BehindRead::Branch, expected, pending.branch),
        Seen::Drift { nearest } => {
            if pending.branch.is_none() && (pin.is_none() || creating) {
                if expected.is_some() && !creating {
                    behind(BehindRead::Branch, expected, None)
                } else {
                    Judgement::Read
                }
            } else if pending.branch.is_some() && pending.branch == pin {
                if pending.branch == expected {
                    Judgement::Read
                } else {
                    behind(BehindRead::Branch, expected, pending.branch)
                }
            } else {
                drifted_branch(pending.branch, nearest)
            }
        }
    }
}

fn drifted_branch(found: Option<Commit>, nearest: Option<u64>) -> Judgement {
    let why = if found.is_some() { change::Cause::BranchMoved } else { change::Cause::BranchMissing };
    Judgement::Drift(Drift { why, nearest })
}

pub(super) fn update_head(d: &Domain, env: &Env<Limits>, row: &ChangeRow, found: Option<Commit>) -> bool {
    let Some(base) = super::updating_base(&row.change.state) else { return false };
    let Some(head) = row.change.state.pinned_head() else { return false };
    let Some(pull) = row.pull else { return false };
    let Some(name) = writes::branch(&env.limits, row.repository, &row.branch) else { return false };
    let Some(writes) = d.writes.get(&name) else {
        // Legacy updates can have a kept result before any write record.
        return true;
    };
    let Some(found) = found else { return false };
    let operation = crate::writes::Operation::Update { pull, before: head, base, after: Some(found) };
    for entry in &writes.entries {
        if entry.state == crate::WriteState::Confirmed && entry.operation == operation {
            match entry.sender {
                crate::Sender::Entry { task, .. } if task == row.task => return true,
                crate::Sender::Entry { .. } | crate::Sender::Attempt { .. } | crate::Sender::Previous => {}
            }
        }
    }
    false
}

pub(super) fn base(
    d: &mut Domain,
    env: &Env<Limits>,
    pending: &PendingStep,
    commit: Commit,
    out: &mut Queue<Request>,
) -> Judgement {
    if pending.heard.cancelled {
        return Judgement::Read;
    }
    let row = d.changes.get(&pending.task).expect("live step");
    let Some(name) = writes::branch(&env.limits, row.repository, &row.base) else { return Judgement::Read };
    let actor = actor(d, row, Some(commit), BehindRead::Base);
    match d.observe(&name, Some(commit), actor, out) {
        Seen::Behind { .. } => behind(BehindRead::Base, writes::record_head(d, &name), Some(commit)),
        Seen::Owed => Judgement::Owed,
        // An unexplained move of the *base branch* is not a change's drift.
        Seen::Current | Seen::Explained { .. } | Seen::Drift { .. } => Judgement::Read,
    }
}

pub(super) fn pull(d: &mut Domain, env: &Env<Limits>, pending: &PendingStep, out: &mut Queue<Request>) -> Judgement {
    let row = d.changes.get(&pending.task).expect("live step");
    let Some(name) = writes::branch(&env.limits, row.repository, &row.branch) else { return Judgement::Read };
    let recovery = recovery(&row.change.state);
    let reopening = recovery == Recovery::Reopening || recovery == Recovery::Recreating;
    let Some(pull) = &pending.pull else {
        // Temper's open can assign the number while a search for the pull
        // by branches is out. Its absence is not a read of the new pull.
        if recovery == Recovery::Opening
            && row.pull.is_some()
            && let Some((_, skein_forge::api::Read::PullFor { .. })) = &pending.read
        {
            return behind(BehindRead::PullState, None, None);
        }
        return if row.pull.is_some() && !reopening && !pending.heard.cancelled {
            Judgement::Drift(Drift { why: change::Cause::PullClosed, nearest: writes::nearest_order(d, &name) })
        } else {
            Judgement::Read
        };
    };
    let target = row.base.clone();
    let target_changed = pull.base != target;
    let number = row.pull.unwrap_or(pull.number);
    let closed = pull.state == State::Closed;
    let read = d.observe_pull_state(&name, number, pull, Some(&target), out);
    // The forge's report of a merge is decisive even if its head or base
    // commit view has not caught up. Cancellation also settles, never holds.
    if pull.merged.is_some() || pending.heard.cancelled {
        return Judgement::Read;
    }
    match read.state {
        Seen::Drift { nearest } => {
            if reopening {
                return Judgement::Read;
            }
            let why = if target_changed { change::Cause::Retargeted } else { change::Cause::PullClosed };
            return Judgement::Drift(Drift { why, nearest });
        }
        Seen::Behind { .. } => return behind(BehindRead::PullState, None, None),
        Seen::Owed => return Judgement::Owed,
        Seen::Current | Seen::Explained { .. } => {}
    }
    if read.head == Seen::Owed || read.base == Seen::Owed {
        return Judgement::Owed;
    }
    if !closed {
        return pull_commits(pending, pull, read);
    }
    Judgement::Read
}

fn pull_commits(pending: &PendingStep, pull: &Pull, read: writes::PullSeen) -> Judgement {
    // The branch read is authoritative for BranchMoved. A pull carrying a
    // different commit cannot supply the found head for a drift hold.
    if pending.branch.is_some() && pending.branch != Some(pull.commit) {
        return behind(BehindRead::PullHead, pending.branch, Some(pull.commit));
    }
    // A different base commit can be newer than the branch read. Only the
    // record can say it is behind; otherwise both remain dependencies.
    match read.head {
        Seen::Behind { .. } => return behind(BehindRead::PullHead, pending.branch, Some(pull.commit)),
        Seen::Current | Seen::Explained { .. } | Seen::Drift { .. } | Seen::Owed => {}
    }
    match read.base {
        Seen::Behind { .. } => behind(BehindRead::PullBase, pending.base_read, pull.base_commit),
        Seen::Current | Seen::Explained { .. } | Seen::Drift { .. } | Seen::Owed => Judgement::Read,
    }
}

fn behind(read: BehindRead, expected: Option<Commit>, found: Option<Commit>) -> Judgement {
    Judgement::Behind(ReadBehind { read, expected, found })
}

fn actor(d: &Domain, row: &ChangeRow, found: Option<Commit>, read: BehindRead) -> Actor {
    let hint = match d.settling.get(&row.task) {
        Some(settle) => match read {
            BehindRead::Branch => settle.branch_push,
            BehindRead::Base => settle.base_push,
            BehindRead::PullHead | BehindRead::PullBase | BehindRead::PullState => None,
        },
        None => None,
    };
    if let Some(push) = hint
        && found == Some(push.head)
        && let Some(writer) = d.writers.get(&row.repository.forge)
    {
        if *writer == push.actor { Actor::Writer } else { Actor::Other }
    } else {
        Actor::Unknown
    }
}

pub(super) fn keep(
    d: &mut Domain,
    env: &Env<Limits>,
    pending: &mut PendingStep,
    judgement: Judgement,
    out: &mut Queue<Request>,
) -> bool {
    // A hint identifying another writer, or a missing branch, is not made
    // ambiguous by a later view of the base or of the pull.
    if pending.drift.is_some() {
        return true;
    }
    match judgement {
        Judgement::Read => true,
        Judgement::Drift(drift) => {
            pending.drift = Some(drift);
            true
        }
        Judgement::Behind(behind) => {
            pending.behind = Some(behind);
            true
        }
        Judgement::Owed => {
            super::read_session::owe_readback(d, pending.task);
            super::branch_due(d, pending.task);
            super::wait_for_forge(d, env, pending, Some(retry(env)), out);
            false
        }
    }
}

pub(super) fn wait(d: &mut Domain, env: &Env<Limits>, pending: PendingStep, out: &mut Queue<Request>) {
    // No ordinary phase progress, write or gate may use these partial facts.
    super::branch_due(d, pending.task);
    super::wait_for_forge(d, env, &pending, Some(retry(env)), out);
}

fn retry(env: &Env<Limits>) -> Wall {
    Wall::from_nanos(env.wall.as_nanos().saturating_add(1_000_000_000))
}
