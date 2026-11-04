//! A goal's taking makes its branch; its release deletes it once a change landed it, and keeps it otherwise
//! (temper/docs/design/domain/forge.md, 2, 11.1 and 16).
use super::{
    BranchHead, Domain, Env, Limits, LostReadState, Name, Request, Role, Stored, Writer, child, emit, keep_entry,
    save_hold, to_client,
};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Error, Read, Write};
use skein_forge_client::{Condition, Effect, Entry, Outcome};
use skein_lib::{List, Queue, Token};

#[derive(Debug)]
pub(super) struct Making {
    task: u64,
    name: Name,
    entry: Option<u64>,
    default: bool,
    state: LostReadState,
}

/// A taking retains its outbox number through branch reads and read retries.
pub(super) fn reserved_before(d: &Domain, number: u64) -> bool {
    for (_, pending) in &d.makings {
        if let Some(entry) = pending.entry
            && entry < number
        {
            return true;
        }
    }
    false
}

pub(super) fn is_goal(d: &Domain, name: &Name) -> bool {
    let provider = skein_forge::api::Repository { forge: name.forge, repository: name.repository };
    match d.repositories.get(&provider) {
        Some(repository) => crate::items::goal_branch(name, &repository.prefix).is_some(),
        None => false,
    }
}

pub(super) fn landed(d: &Domain, name: &Name) -> bool {
    let Some(tip) = d.branch_head(name) else { return false };
    for (_, row) in &d.changes {
        if row.repository.forge == name.forge
            && row.repository.repository == name.repository
            && crate::items::branch_matches(name, &row.branch)
            && super::landed_commit(&row.change.state).is_some()
            && row.result.head == Some(tip)
        {
            return true;
        }
    }
    false
}

pub(super) fn waiting(d: &Domain, name: &Name) -> bool {
    if reading(d, name) {
        return true;
    }
    for (_, row) in &d.entries {
        if makes(&row.entry, name) {
            return true;
        }
    }
    false
}

fn makes(entry: &Entry, name: &Name) -> bool {
    if entry.repository.forge != name.forge || entry.repository.repository != name.repository {
        return false;
    }
    match &entry.effect.write {
        Write::CreateBranch { branch, .. } => crate::items::branch_matches(name, branch),
        Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Merge { .. }
        | Write::Update { .. }
        | Write::Status { .. }
        | Write::Dispatch { .. }
        | Write::Rerun { .. }
        | Write::DeleteBranch { .. } => false,
    }
}

pub(super) fn reading(d: &Domain, name: &Name) -> bool {
    for (_, pending) in &d.makings {
        if &pending.name == name {
            return true;
        }
    }
    false
}

pub(super) fn base_wait(d: &Domain, repository: skein_forge::api::Repository, base: &[u8]) -> bool {
    for (name, _) in &d.holds {
        if name.forge == repository.forge
            && name.repository == repository.repository
            && crate::items::branch_matches(name, base)
            && is_goal(d, name)
            && waiting(d, name)
        {
            return true;
        }
    }
    false
}

pub(super) fn start(d: &mut Domain, env: &Env<Limits>, name: Name, entry: Option<u64>, out: &mut Queue<Request>) {
    if !is_goal(d, &name)
        || reading(d, &name)
        || (entry.is_some() && waiting(d, &name))
        || d.stopping
        || d.forge_ended(name.forge)
    {
        return;
    }
    let Some(hold) = d.holds.get(&name) else { return };
    let task = hold.task;
    let Some(resource) = to_client(&name, &env.limits) else { return };
    let branch = match resource.what {
        skein_forge_client::What::Branch(branch) => branch,
        skein_forge_client::What::Repository
        | skein_forge_client::What::Pull(_)
        | skein_forge_client::What::Issue(_) => return,
    };
    let Some(sequence) = d.sequence.checked_add(1) else {
        failed(task, name, b"goal branch read sequence exhausted", out);
        return;
    };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 59));
    let pending = Making { task, name, entry, default: false, state: LostReadState::Asked };
    if let Err((_, pending)) = d.makings.insert(owner, pending) {
        failed(pending.task, pending.name, b"goal branch read capacity exhausted", out);
        return;
    }
    child(
        d,
        env,
        skein_forge_client::Event::Read { owner, repository: resource.repository, read: Read::Branch { branch } },
        out,
    );
}

pub(super) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.makings.remove(&owner) else { return };
    if d.stopping
        || match d.holds.get(&pending.name) {
            Some(hold) => hold.task != pending.task,
            None => true,
        }
    {
        return;
    }
    let read_state = match result {
        Err(Error::Busy) => Some(LostReadState::WaitingForRoom),
        Err(Error::NotConnected) => Some(LostReadState::WaitingForConnection),
        Ok(_) | Err(_) => None,
    };
    if let Some(state) = read_state {
        pending.state = state;
        d.makings.insert(owner, pending).expect("making keeps its read slot");
        return;
    }
    match result {
        Ok(Answer::Commit(commit)) if pending.default => make(d, env, pending, commit, out),
        Ok(Answer::Commit(commit)) => {
            let head = BranchHead { name: pending.name.clone(), commit, owned: true };
            if d.heads.insert(pending.name.clone(), head.clone()).is_ok() {
                emit(out, Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head) });
            }
            wake(d, &pending.name);
            released(d, pending.task, out);
        }
        Err(Error::Missing) if !pending.default => {
            if pending.entry.is_none() {
                if waiting(d, &pending.name) {
                    return;
                }
                let hold = d.holds.get_mut(&pending.name).expect("restored goal hold");
                match hold.writer {
                    Some(Writer::Entry(_)) => {
                        hold.writer = None;
                        save_hold(out, hold.clone());
                    }
                    Some(Writer::Run { .. } | Writer::Lost { .. }) | None => {}
                }
                failed(
                    pending.task,
                    pending.name,
                    b"branch missing after restart; forge has no restored outbox entry allocator",
                    out,
                );
                return;
            }
            let resource = to_client(&pending.name, &env.limits).expect("validated held branch");
            let repository = d.repositories.get(&resource.repository).expect("held repository");
            let branch = repository.settings.default_branch.clone();
            // A known default tip is already part of this decision's working set.
            let mut commit = None;
            for (name, head) in &d.heads {
                if name.forge == pending.name.forge
                    && name.repository == pending.name.repository
                    && crate::items::branch_matches(name, &branch)
                {
                    commit = Some(head.commit);
                }
            }
            if let Some(commit) = commit {
                make(d, env, pending, commit, out);
            } else {
                pending.default = true;
                d.makings.insert(owner, pending).expect("replaces making read");
                child(
                    d,
                    env,
                    skein_forge_client::Event::Read {
                        owner,
                        repository: resource.repository,
                        read: Read::Branch { branch },
                    },
                    out,
                );
            }
        }
        Err(error) => {
            wake(d, &pending.name);
            failed(pending.task, pending.name, crate::effect_words::error_words(error), out);
            released(d, pending.task, out);
        }
        Ok(_) => {
            wake(d, &pending.name);
            failed(pending.task, pending.name, b"forge answered the branch read without a commit", out);
            released(d, pending.task, out);
        }
    }
}

fn make(
    d: &mut Domain,
    env: &Env<Limits>,
    pending: Making,
    commit: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    let number = pending.entry.expect("a taking supplies its entry");
    let resource = to_client(&pending.name, &env.limits).expect("validated held branch");
    let branch = match resource.what {
        skein_forge_client::What::Branch(branch) => branch,
        skein_forge_client::What::Repository
        | skein_forge_client::What::Pull(_)
        | skein_forge_client::What::Issue(_) => unreachable!("goal branch"),
    };
    let permitted = match d.repositories.get(&resource.repository) {
        Some(repository) => repository.kinds.branch && repository.role != Role::Context,
        None => false,
    };
    if d.entries.len() == env.limits.entries || d.entries.contains_key(&number) || !permitted {
        wake(d, &pending.name);
        failed(pending.task, pending.name, b"goal branch making exceeds forge access or outbox bounds", out);
        released(d, pending.task, out);
        return;
    }
    let hold = d.holds.get_mut(&pending.name).expect("making's hold");
    hold.writer = Some(Writer::Entry(number));
    save_hold(out, hold.clone());
    let entry = Entry {
        number,
        task: pending.task,
        repository: resource.repository,
        effect: Effect { write: Write::CreateBranch { branch, commit }, condition: Condition::None },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    keep_entry(d, entry, Some(number), out);
}

pub(super) fn outcome(d: &mut Domain, number: u64, outcome: Outcome, out: &mut Queue<Request>) {
    match outcome {
        Outcome::Uncertain | Outcome::Queued => return,
        Outcome::Made { .. } | Outcome::Failed(_) | Outcome::Raced { .. } | Outcome::Held | Outcome::Withdrawn => {}
    }
    let mut name = None;
    for (resource, hold) in &d.holds {
        let making = match d.entries.get(&number) {
            Some(row) => row.entry.task == hold.task && makes(&row.entry, resource),
            None => false,
        };
        if making && is_goal(d, resource) {
            name = Some(resource.clone());
            break;
        }
    }
    let Some(name) = name else { return };
    let hold = d.holds.get_mut(&name).expect("making's hold");
    let task = hold.task;
    hold.writer = None;
    save_hold(out, hold.clone());
    let reason = match outcome {
        Outcome::Made { .. } => None,
        Outcome::Failed(error) | Outcome::Raced { why: error, .. } => Some(crate::effect_words::error_words(error)),
        Outcome::Held => Some(b"uncertain: the forge cannot safely retry the branch making".as_slice()),
        Outcome::Withdrawn => Some(b"branch making withdrawn before completion".as_slice()),
        Outcome::Uncertain | Outcome::Queued => unreachable!("unsettled returned above"),
    };
    if let Some(reason) = reason {
        failed(task, name.clone(), reason, out);
    }
    wake(d, &name);
}

fn wake(d: &mut Domain, name: &Name) {
    for (task, row) in &d.changes {
        if row.repository.forge == name.forge
            && row.repository.repository == name.repository
            && crate::items::branch_matches(name, &row.base)
        {
            d.procedure_wakes.insert(*task, true).expect("one pending wake per change");
        }
    }
}
fn failed(task: u64, resource: Name, reason: &[u8], out: &mut Queue<Request>) {
    emit(out, Request::MakeFailed { task, resource, reason: Box::from(reason) });
}
fn released(d: &Domain, task: u64, out: &mut Queue<Request>) {
    if d.releases.contains_key(&task) {
        emit(out, Request::ContinueRelease { task });
    }
}

pub(super) fn restart(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut names = List::with_capacity(env.limits.holds);
    for (name, _) in &d.holds {
        if is_goal(d, name) {
            names.push(name.clone()).expect("bounded holds");
        }
    }
    for name in names.into_boxed() {
        start(d, env, name, None, out);
    }
}

/// Admission and disconnected reads wait without occupying client call room.
pub(super) fn retry(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut selected = None;
    for (owner, pending) in &d.makings {
        if pending.state == LostReadState::WaitingForRoom {
            selected = Some(*owner);
            break;
        }
    }
    let Some(owner) = selected else { return };
    let pending = d.makings.get_mut(&owner).expect("selected making read");
    pending.state = LostReadState::Asked;
    let resource = to_client(&pending.name, &env.limits).expect("validated held branch");
    let branch = if pending.default {
        d.repositories.get(&resource.repository).expect("held repository").settings.default_branch.clone()
    } else {
        match resource.what {
            skein_forge_client::What::Branch(branch) => branch,
            skein_forge_client::What::Repository
            | skein_forge_client::What::Pull(_)
            | skein_forge_client::What::Issue(_) => unreachable!("goal branch"),
        }
    };
    child(
        d,
        env,
        skein_forge_client::Event::Read { owner, repository: resource.repository, read: Read::Branch { branch } },
        out,
    );
}

pub(super) fn connected(d: &mut Domain, forge: u16) {
    let mut owners = List::with_capacity(d.makings.capacity());
    for (owner, pending) in &d.makings {
        if pending.name.forge == forge && pending.state == LostReadState::WaitingForConnection {
            owners.push(*owner).expect("bounded making reads");
        }
    }
    for owner in owners.into_boxed() {
        d.makings.get_mut(&owner).expect("collected making read").state = LostReadState::WaitingForRoom;
    }
}

pub(super) fn retire(d: &mut Domain, forge: u16) {
    let mut owners = List::with_capacity(d.makings.capacity());
    for (owner, pending) in &d.makings {
        if pending.name.forge == forge {
            owners.push(*owner).expect("bounded making reads");
        }
    }
    for owner in owners.into_boxed() {
        d.makings.remove(&owner);
    }
}

pub(super) fn read_waiting(d: &Domain) -> bool {
    for (_, pending) in &d.makings {
        if pending.state == LostReadState::WaitingForRoom {
            return true;
        }
    }
    false
}
