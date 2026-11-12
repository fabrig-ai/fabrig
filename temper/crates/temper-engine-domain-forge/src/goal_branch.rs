//! A held goal branch is made across restarts and releases of a failed making.
//! Closing deletes it once a change landed it, and keeps it otherwise
//! (temper/docs/design/domain/forge/README.md, 2, and temper/docs/design/domain/forge/branches.md, 1, and temper/docs/design/domain/forge/README.md, 9).
use super::{
    BranchHead, Domain, Env, Limits, LostReadState, Name, Request, Role, Stored, Writer, child, emit, keep_entry,
    save_hold, to_client,
};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Error, Read, Write};
use skein_forge_client::{Condition, Effect, Entry, Outcome};
use skein_lib::{Duration, List, Queue, Time, Token};

#[derive(Hash, Debug)]
pub(super) struct Making {
    task: u64,
    name: Name,
    entry: Option<u64>,
    default: bool,
    state: LostReadState,
    retry_at: Option<Time>,
    backoff: Duration,
    outbox_wait: Option<RoomWait>,
}

/// Outbox room waits reserve no number; fresh numbers follow wait order
/// (temper/docs/design/domain/forge/branches.md, 1).
#[derive(Hash, Debug)]
struct RoomWait {
    order: u64,
    asked: bool,
}

/// A taking retains its outbox number through branch reads and read retries,
/// but gives it up while waiting for outbox room (domain/forge/branches.md, 1).
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

/// A released drift is read before the goal can write design again (domain/forge/branches.md, 3).
pub(super) fn accept_release(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let mut names = List::with_capacity(env.limits.holds);
    for (name, hold) in &d.holds {
        if hold.task == task && hold.drift.is_some() && is_goal(d, name) {
            names.push(name.clone()).expect("goal branches bounded by holds");
        }
    }
    for name in &names {
        let hold = d.holds.get_mut(name).expect("collected goal hold");
        hold.drift = None;
        save_hold(out, hold.clone());
        start(d, env, name.clone(), None, out);
    }
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
    let pending = Making {
        task,
        name,
        entry,
        default: false,
        state: LostReadState::Asked,
        retry_at: None,
        backoff: env.limits.client.backoff,
        outbox_wait: None,
    };
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
    if let Err(Error::Unavailable | Error::Timeout) = result {
        pending.retry_at = Some(env.now.saturating_add(pending.backoff));
        pending.backoff = pending.backoff.saturating_add(pending.backoff).min(env.limits.client.slow);
        d.makings.insert(owner, pending).expect("making keeps its read slot");
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
    pending.backoff = env.limits.client.backoff;
    read_result(d, env, owner, pending, result, out);
}

fn read_result(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    pending: Making,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    match result {
        Ok(Answer::Commit(commit)) if pending.default => make(d, env, owner, pending, commit, out),
        Ok(Answer::Commit(commit)) => {
            if let Some(entry) = held_entry(d, &pending.name) {
                let expected = match entry.effect.write {
                    Write::CreateBranch { commit, .. } => commit,
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
                    | Write::DeleteBranch { .. } => unreachable!("held goal branch creation"),
                };
                let outcome = if expected == commit {
                    Outcome::Made { made: skein_forge_client::Made::Branch(commit), found: true }
                } else {
                    Outcome::Failed(Error::Exists)
                };
                d.client_out.push(skein_forge_client::Request::Outcome {
                    entry: entry.number,
                    task: entry.task,
                    outcome,
                });
                super::drain(d, env, out);
                return;
            }
            let head = BranchHead { name: pending.name.clone(), commit, owned: true };
            let concerning = super::heads::concerning(d, &pending.name, Some(pending.task));
            if super::heads::record(d, env, head.clone(), concerning, out) {
                emit(out, Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head) });
            }
            if let Some(resource) = to_client(&pending.name, &env.limits) {
                child(d, env, skein_forge_client::Event::Pushed { resource, commit }, out);
            }
            wake(d, &pending.name);
            released(d, pending.task, out);
        }
        Err(Error::Missing) if !pending.default => missing(d, env, owner, pending, out),
        Err(error) => {
            wake(d, &pending.name);
            failed(pending.task, pending.name, &crate::effect_words::error_words(error), out);
            released(d, pending.task, out);
        }
        Ok(_) => {
            wake(d, &pending.name);
            failed(pending.task, pending.name, b"forge answered the branch read without a commit", out);
            released(d, pending.task, out);
        }
    }
}

fn missing(d: &mut Domain, env: &Env<Limits>, owner: Token, mut pending: Making, out: &mut Queue<Request>) {
    if let Some(entry) = held_entry(d, &pending.name) {
        super::resolve(d, env, entry.number, true, out);
        return;
    }
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
        emit(out, Request::ContinueGoalBranch { task: pending.task });
        d.makings.insert(owner, pending).expect("making waits for its entry");
        return;
    }
    let resource = to_client(&pending.name, &env.limits).expect("validated held branch");
    let repository = d.repositories.get(&resource.repository).expect("held repository");
    let branch = repository.settings.default_branch.clone();
    pending.default = true;
    d.makings.insert(owner, pending).expect("replaces making read");
    child(
        d,
        env,
        skein_forge_client::Event::Read { owner, repository: resource.repository, read: Read::Branch { branch } },
        out,
    );
}

fn make(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
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
    let reason = match d.repositories.get(&resource.repository) {
        None => Some(b"the repository is not adopted".as_slice()),
        Some(repository) if repository.role == Role::Context => {
            Some(b"the repository is adopted to be read, not written".as_slice())
        }
        Some(repository) if !repository.kinds.branch => {
            Some(b"the repository's adoption takes no branch writes".as_slice())
        }
        Some(_) => None,
    };
    if let Some(reason) = reason {
        wake(d, &pending.name);
        failed(pending.task, pending.name, reason, out);
        released(d, pending.task, out);
        return;
    }
    if d.entries.contains_key(&number) {
        wake(d, &pending.name);
        failed(pending.task, pending.name, &entry_in_use(number), out);
        released(d, pending.task, out);
        return;
    }
    if d.entries.len() == env.limits.entries {
        wait_for_room(d, owner, pending, out);
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

fn entry_in_use(number: u64) -> Box<[u8]> {
    let digits = skein_lib::Decimal::of(number);
    let parts = [b"its outbox entry ".as_slice(), digits.as_bytes(), b" is already in use"];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("entry number words fit");
    }
    let mut words = skein_lib::Writer::new(length);
    for part in parts {
        words.put(part).expect("measured entry number words");
    }
    words.finish()
}

/// A wait gives up its reservation without giving up its position
/// (temper/docs/design/domain/forge/branches.md, 1).
fn wait_for_room(d: &mut Domain, owner: Token, mut pending: Making, out: &mut Queue<Request>) {
    if let Some(wait) = &mut pending.outbox_wait {
        wait.asked = false;
    } else {
        let Some(order) = d.sequence.checked_add(1) else {
            failed(pending.task, pending.name, b"goal branch wait sequence exhausted", out);
            return;
        };
        d.sequence = order;
        pending.outbox_wait = Some(RoomWait { order, asked: false });
    }
    pending.entry = None;
    pending.default = false;
    let hold = d.holds.get_mut(&pending.name).expect("making's hold");
    match hold.writer {
        Some(Writer::Entry(_)) => {
            hold.writer = None;
            save_hold(out, hold.clone());
        }
        Some(Writer::Run { .. } | Writer::Lost { .. }) | None => {}
    }
    d.makings.insert(owner, pending).expect("making keeps its room wait slot");
}

fn held_entry(d: &Domain, name: &Name) -> Option<Entry> {
    let hold = d.holds.get(name)?;
    for (_, row) in &d.entries {
        if row.held && row.entry.task == hold.task && makes(&row.entry, name) {
            return Some(row.entry.clone());
        }
    }
    None
}

/// The client's unrecoverable branch creation is conditional for a goal (11.1).
pub(super) fn remake_held(d: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) -> bool {
    let Some(row) = d.entries.get(&number) else { return false };
    let mut name = None;
    for (resource, hold) in &d.holds {
        if row.entry.task == hold.task && makes(&row.entry, resource) && is_goal(d, resource) {
            name = Some(resource.clone());
            break;
        }
    }
    let Some(name) = name else { return false };
    let row = d.entries.get_mut(&number).expect("goal making entry");
    row.held = true;
    emit(
        out,
        Request::Save {
            read_afresh: None,
            release: None,
            record: Stored::Entry { entry: row.entry.clone(), held: true },
        },
    );
    let task = row.entry.task;
    super::erase_flight(d, task, number, out);
    start(d, env, name, None, out);
    true
}

// The root allocates numbers and queues these replies synchronously, in ask order.
pub(super) fn continue_making(d: &mut Domain, env: &Env<Limits>, task: u64, entry: u64, out: &mut Queue<Request>) {
    let mut selected = None;
    for (owner, pending) in &d.makings {
        let requested = match &pending.outbox_wait {
            Some(wait) => wait.asked,
            None => true,
        };
        if pending.task == task && pending.entry.is_none() && !pending.default && requested {
            selected = Some(*owner);
            break;
        }
    }
    let Some(owner) = selected else { return };
    let mut pending = d.makings.remove(&owner).expect("making waiting for its entry");
    pending.entry = Some(entry);
    missing(d, env, owner, pending, out);
}

pub(super) fn retry_task(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let mut names = List::with_capacity(env.limits.holds);
    for (name, hold) in &d.holds {
        if hold.task == task && is_goal(d, name) {
            names.push(name.clone()).expect("bounded goal holds");
        }
    }
    for name in names.into_boxed() {
        start(d, env, name, None, out);
    }
}

pub(super) fn deadline(d: &Domain) -> Option<Time> {
    let mut next: Option<Time> = None;
    for (_, pending) in &d.makings {
        if let Some(at) = pending.retry_at {
            next = Some(match next {
                Some(previous) => previous.min(at),
                None => at,
            });
        }
    }
    next
}

pub(super) fn fire(d: &mut Domain, now: Time) {
    let mut due = List::with_capacity(d.makings.capacity());
    for (owner, pending) in &d.makings {
        if let Some(at) = pending.retry_at
            && at <= now
        {
            due.push(*owner).expect("bounded making reads");
        }
    }
    for owner in due.into_boxed() {
        let pending = d.makings.get_mut(&owner).expect("due making read");
        pending.retry_at = None;
        pending.state = LostReadState::WaitingForRoom;
    }
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
        Outcome::Held => Some(Box::from(b"uncertain: the forge cannot safely retry the branch making".as_slice())),
        Outcome::Withdrawn => Some(Box::from(b"branch making withdrawn before completion".as_slice())),
        Outcome::Uncertain | Outcome::Queued => unreachable!("unsettled returned above"),
    };
    if let Some(reason) = reason {
        failed(task, name.clone(), &reason, out);
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
    for (name, hold) in &d.holds {
        // A design writer's confirmed branch keeps its drift baseline until
        // that attempt's fresh read (domain/forge/branches.md, 3).
        if is_goal(d, name) && hold.design_writers.is_empty() {
            names.push(name.clone()).expect("bounded holds");
        }
    }
    for name in names.into_boxed() {
        start(d, env, name, None, out);
    }
}

/// A freed entry admits one waiter, including its number request and fresh read,
/// before another waiter may use that room (domain/forge/branches.md, 1).
fn next_room_waiter(d: &Domain) -> Option<Token> {
    let mut occupied = d.entries.len();
    let mut selected: Option<(u64, Token)> = None;
    for (owner, pending) in &d.makings {
        if let Some(wait) = &pending.outbox_wait {
            if wait.asked {
                occupied = occupied.saturating_add(1);
            } else if match selected {
                Some((order, _)) => wait.order < order,
                None => true,
            } {
                selected = Some((wait.order, *owner));
            }
        }
    }
    if occupied >= d.entries.capacity() {
        return None;
    }
    match selected {
        Some((_, owner)) => Some(owner),
        None => None,
    }
}

pub(super) fn room_waiting(d: &Domain) -> bool {
    next_room_waiter(d).is_some()
}

pub(super) fn resume_room(d: &mut Domain, out: &mut Queue<Request>) -> bool {
    let Some(owner) = next_room_waiter(d) else { return false };
    let pending = d.makings.get_mut(&owner).expect("selected room waiter");
    let live = match d.holds.get(&pending.name) {
        Some(hold) => hold.task == pending.task,
        None => false,
    };
    if live {
        pending.outbox_wait.as_mut().expect("selected room wait").asked = true;
        emit(out, Request::ContinueGoalBranch { task: pending.task });
    } else {
        d.makings.remove(&owner);
    }
    true
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
