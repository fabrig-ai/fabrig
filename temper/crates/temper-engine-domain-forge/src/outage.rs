//! Connector-owned fresh reads and never-sent writes ride a forge away
//! (temper/docs/design/domain/forge/keeping-up.md, 5). All waits and reachability
//! marks are volatile; a restart asks afresh (section 16).
use super::read_line::{self, Waiter};
use super::{Domain, Env, Handoff, Limits, Request, Topic, erase_flight, keep_entry};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Error, Op, Read, Repository};
use skein_forge_client::{Event, Outcome};
use skein_lib::{Duration, Map, Queue, Set, Time, Token, Wall};

/// One re-ask's growing wait, reset by retiring it on an answer.
#[derive(Hash, Debug)]
struct Wait {
    delay: Duration,
    at: Option<Time>,
}
impl Wait {
    fn new(limits: &Limits) -> Self {
        Self { delay: limits.client.backoff, at: None }
    }
    fn again(&mut self, env: &Env<Limits>, grow: bool) {
        self.at = Some(env.now.saturating_add(self.delay));
        if grow {
            self.delay = self.delay.saturating_add(self.delay).min(env.limits.client.slow);
        }
    }
}
#[derive(Hash, Debug)]
struct Reading {
    repository: Repository,
    read: Read,
    asked: Wall,
    wait: Wait,
    outage: bool,
    retry: bool,
}
#[derive(Hash, Debug)]
pub(super) struct State {
    // Their owners bound these maps: one row per pending read or outbox entry.
    reads: Map<Token, Reading>,
    writes: Map<u64, Wait>,
    calls: Map<Token, (Repository, bool)>,
    away: Set<u16>,
    back: Set<u16>,
}
impl State {
    pub(super) fn new(limits: &Limits) -> Self {
        Self {
            reads: Map::uncapped(),
            writes: Map::with_capacity(limits.entries),
            calls: Map::with_capacity(limits.client.calls),
            away: Set::with_capacity(limits.forges),
            back: Set::with_capacity(limits.forges),
        }
    }
    pub(super) fn raise(&mut self, entries: u32) {
        self.writes.raise(entries);
    }
    pub(super) fn deadline(&self) -> Option<Time> {
        let mut next = None;
        for (_, reading) in &self.reads {
            if let Some(at) = reading.wait.at {
                next = Some(match next {
                    Some(old) => at.min(old),
                    None => at,
                });
            }
        }
        for (_, wait) in &self.writes {
            if let Some(at) = wait.at {
                next = Some(match next {
                    Some(old) => at.min(old),
                    None => at,
                });
            }
        }
        next
    }
    pub(super) fn forget_entry(&mut self, entry: u64) {
        self.writes.remove(&entry);
    }
    pub(super) fn recommitted(&self, entry: u64) -> bool {
        self.writes.contains_key(&entry)
    }
    pub(super) fn called(&mut self, call: Token, repository: Repository, op: &Op) {
        let write = match op {
            Op::Write(_) => true,
            Op::Read(_) => false,
        };
        self.calls.insert(call, (repository, write)).expect("one row per client call");
    }
    pub(super) fn observed(&mut self, forge: u16, outage: bool) {
        if outage {
            self.away.insert(forge).expect("one mark per forge");
        } else if self.away.remove(&forge) {
            self.back.insert(forge).expect("one recovery per forge");
        }
    }
}

impl Domain {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_retry_readers(&self) -> Box<[Token]> {
        let Some(state) = &self.outage else { return Box::new([]) };
        let mut readers = skein_lib::List::with_capacity(state.reads.len());
        for (owner, _) in &state.reads {
            readers.push(*owner).expect("counted read retries");
        }
        readers.into_boxed()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_read_retry(&self, owner: Token) -> Option<Time> {
        self.outage.as_ref()?.reads.get(&owner)?.wait.at
    }
}

// No outage bookkeeping is allocated for a passive restored owner. Keeping
// this subtree boxed also leaves the connector's inline restore frame small.
fn activate(d: &mut Domain, limits: &Limits) {
    if d.outage.is_none() {
        d.outage = Some(Box::new(State::new(limits)));
    }
}
fn state(d: &mut Domain) -> &mut State {
    d.outage.as_mut().expect("active outage bookkeeping")
}
pub(super) fn deadline(d: &Domain) -> Option<Time> {
    match &d.outage {
        Some(state) => state.deadline(),
        None => None,
    }
}
/// An explicit unavailable answer exempts this read until it answers again.
pub(crate) fn reading(d: &Domain, owner: Token) -> bool {
    let Some(state) = &d.outage else { return false };
    match state.reads.get(&owner) {
        Some(reading) => reading.outage,
        None => false,
    }
}

pub(super) fn recommitted(d: &Domain, entry: u64) -> bool {
    match &d.outage {
        Some(state) => state.recommitted(entry),
        None => false,
    }
}
pub(crate) fn forget_read(d: &mut Domain, owner: Token) {
    if let Some(active) = &mut d.outage {
        active.reads.remove(&owner);
    }
}

pub(super) fn forget_entry(d: &mut Domain, entry: u64) {
    if let Some(state) = &mut d.outage {
        state.forget_entry(entry);
    }
}
pub(super) fn called(d: &mut Domain, env: &Env<Limits>, call: Token, repository: Repository, op: &Op) {
    activate(d, &env.limits);
    state(d).called(call, repository, op);
}
pub(super) fn observed(d: &mut Domain, env: &Env<Limits>, forge: u16, unavailable: bool) {
    activate(d, &env.limits);
    state(d).observed(forge, unavailable);
}

fn riding(d: &Domain, owner: Token) -> bool {
    // Taking spends its own stall even during an outage (change/opened.md, 4).
    if let Some(pending) = d.steps.get(&owner)
        && let Some(row) = d.changes.get(&pending.task)
        && super::taking::active(&row.change.state)
    {
        return false;
    }
    d.steps.contains_key(&owner)
        || d.lost.contains_key(&owner)
        || d.pending_ci.contains_key(&owner)
        || d.landing.pending.contains_key(&owner)
        || d.tier_ops.reads.contains_key(&owner)
        || d.closing.selection_reads.contains_key(&owner)
        || d.rebase_fresh.contains_key(&owner)
        || crate::unmade::reader(d, owner).is_some()
}
/// Remember the exact fresh read, not a reconstruction from facts that may move.
pub(super) fn track(d: &mut Domain, env: &Env<Limits>, event: &Event) {
    match event {
        Event::Read { owner, repository, read } => {
            activate(d, &env.limits);
            let retry = riding(d, *owner);
            state(d)
                .reads
                .insert(
                    *owner,
                    Reading {
                        repository: *repository,
                        read: read.clone(),
                        asked: env.wall,
                        wait: Wait::new(&env.limits),
                        outage: false,
                        retry,
                    },
                )
                .expect("one row per caller-owned read");
        }
        Event::Answered { call, result, .. } => {
            let Some(state) = &mut d.outage else { return };
            let Some((repository, write)) = state.calls.remove(call) else { return };
            if write && *result == Err(Error::Timeout) {
                // A sent write's Timeout is an uncertain ending, not a blind retry.
                observed(d, env, repository.forge, true);
            }
            match result {
                Err(Error::Unavailable | Error::Timeout | Error::NotConnected | Error::NotInstalled | Error::Busy) => {}
                Ok(_)
                | Err(
                    Error::RateLimited { .. }
                    | Error::Forbidden
                    | Error::Missing
                    | Error::MissingJob
                    | Error::TooLarge(_)
                    | Error::Empty
                    | Error::Full
                    | Error::Exists
                    | Error::NothingToMerge
                    | Error::Closed
                    | Error::Stale
                    | Error::BaseMoved { .. }
                    | Error::Conflict
                    | Error::Protected
                    | Error::Refused
                    | Error::InvalidAnswer,
                ) => observed(d, env, repository.forge, false),
            }
        }
        Event::Settle
        | Event::Keep { .. }
        | Event::Hint { .. }
        | Event::Writer { .. }
        | Event::Pushed { .. }
        | Event::Fence { .. }
        | Event::Make { .. }
        | Event::Remake { .. }
        | Event::Withdraw { .. }
        | Event::Restore { .. }
        | Event::Restored { .. }
        | Event::ReadAfresh
        | Event::SettleOutbox => {}
    }
}
/// An outage has no terminal at the procedure boundary. Its pending row stays put.
pub(super) fn read_ended(d: &mut Domain, env: &Env<Limits>, owner: Token, result: &Result<Answer, Error>) -> bool {
    let Some(active) = &mut d.outage else { return false };
    let Some(mut reading) = active.reads.remove(&owner) else { return false };
    let outage = match result {
        Err(Error::Unavailable | Error::Timeout) => true,
        Ok(_)
        | Err(
            Error::RateLimited { .. }
            | Error::Forbidden
            | Error::Missing
            | Error::MissingJob
            | Error::TooLarge(_)
            | Error::Empty
            | Error::Full
            | Error::Exists
            | Error::NothingToMerge
            | Error::Closed
            | Error::Stale
            | Error::BaseMoved { .. }
            | Error::Conflict
            | Error::Protected
            | Error::Refused
            | Error::InvalidAnswer
            | Error::NotConnected
            | Error::NotInstalled
            | Error::Busy,
        ) => false,
    };
    if outage {
        observed(d, env, reading.repository.forge, true);
    }
    if *result == Err(Error::Busy) && reading.outage && line_waiter(d, owner).is_some() && !d.stopping {
        // The read is still at the head. Preserve its outage backoff, but
        // leave the next admission attempt to the read line.
        state(d).reads.insert(owner, reading).expect("retains its pending read");
        return false;
    }
    let capacity = (*result == Err(Error::Busy)
        && (reading.outage || d.pending_ci.contains_key(&owner) || d.rebase_fresh.contains_key(&owner)))
        || (*result == Err(Error::NotConnected) && d.rebase_fresh.contains_key(&owner));
    if !d.stopping && reading.retry && riding(d, owner) && (outage || capacity) {
        if let Some(pending) = d.steps.get(&owner) {
            let task = pending.task;
            super::read_session::pause(d, task);
            crate::waits::changed(d, task);
        }
        reading.outage |= outage;
        reading.wait.again(env, outage);
        state(d).reads.insert(owner, reading).expect("retains its pending read");
        return true;
    }
    if reading.outage
        && let Some(pending) = d.steps.get(&owner)
    {
        let task = pending.task;
        super::read_session::ended(d, env, task, env.wall);
        crate::waits::changed(d, task);
    }
    if reading.outage
        && let Some(pending) = d.steps.get(&owner)
        && let Some(row) = d.changes.get_mut(&pending.task)
    {
        // since is stored, but this extension is in memory only. Only the
        // unavailable part of this read chain is excluded; prior answered
        // waiting still counts, and a later ordinary step can save its since.
        let start = reading.asked.max(row.change.since);
        let elapsed = env.wall.as_nanos().saturating_sub(start.as_nanos());
        row.change.since = Wall::from_nanos(row.change.since.as_nanos().saturating_add(elapsed));
    }
    false
}

/// Only Failed(Unavailable) is known never sent. Uncertain and Timeout keep
/// the client's find; the existing entry number and effect identity do not move.
pub(super) fn outcome(
    d: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    outcome: Outcome,
    out: &mut Queue<Request>,
) -> bool {
    if let Some(entry) = d.entries.get(&number)
        && let Some(row) = d.changes.get(&entry.entry.task)
        && super::taking::active(&row.change.state)
    {
        return false;
    }
    if outcome == Outcome::Failed(Error::Unavailable) || outcome == Outcome::Failed(Error::Timeout) {
        activate(d, &env.limits);
    }
    let Some(active) = &mut d.outage else { return false };
    // Owners may veto or cancel an unsent entry without a client outcome.
    let mut retired = skein_lib::List::with_capacity(active.writes.len());
    for (entry, _) in &active.writes {
        if !d.entries.contains_key(entry) {
            retired.push(*entry).expect("one wait per entry");
        }
    }
    for entry in retired.into_boxed() {
        active.writes.remove(&entry);
    }
    let Some(row) = d.entries.get(&number) else { return false };
    if outcome == Outcome::Failed(Error::Timeout) {
        active.observed(row.entry.repository.forge, true);
    }
    if outcome != Outcome::Failed(Error::Unavailable) || d.stopping {
        if outcome != Outcome::Uncertain && outcome != Outcome::Queued {
            active.writes.remove(&number);
        }
        return false;
    }
    let mut entry = row.entry.clone();
    active.observed(entry.repository.forge, true);
    entry.start = None;
    entry.attempt = None;
    entry.failures = 0;
    entry.queued = false;
    if !active.writes.contains_key(&number) {
        active.writes.insert(number, Wait::new(&env.limits)).expect("one wait per retained entry");
    }
    active.writes.get_mut(&number).expect("retained write wait").again(env, true);
    erase_flight(d, entry.task, number, out);
    keep_entry(d, entry, None, out);
    true
}

pub(super) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(active) = &mut d.outage else { return };
    let mut due = None;
    for (token, reading) in &active.reads {
        if let Some(at) = reading.wait.at
            && at <= env.now
        {
            due = Some(*token);
            break;
        }
    }
    if let Some(owner) = due {
        if !riding(d, owner) {
            state(d).reads.remove(&owner);
        } else if let Some(waiter) = line_waiter(d, owner) {
            state(d).reads.get_mut(&owner).expect("due pending read").wait.at = None;
            match waiter {
                Waiter::Lost(owner) => {
                    d.lost.get_mut(&owner).expect("due lost read").state = super::LostReadState::WaitingForRoom;
                    read_line::join(d, waiter, env.now, out);
                }
                Waiter::Landing(owner) => crate::landing::wait_in_line(d, owner, env.now, out),
                Waiter::Making(_) | Waiter::Brief(_) | Waiter::Document(_) => unreachable!("outage-owned readers"),
            }
            d.read_line.wake();
        } else {
            ask(d, env, owner, out);
        }
    }
    let Some(active) = &mut d.outage else { return };
    let mut entry = None;
    for (number, wait) in &active.writes {
        if let Some(at) = wait.at
            && at <= env.now
        {
            entry = Some(*number);
            break;
        }
    }
    if let Some(number) = entry {
        active.writes.get_mut(&number).expect("due write").at = None;
        if let Some(row) = d.entries.get_mut(&number) {
            row.handoff = Handoff::Uncommitted;
            let entry = row.entry.clone();
            // Ordinary durable fence: only Committed may hand it to Remake.
            keep_entry(d, entry, Some(number), out);
        } else {
            active.writes.remove(&number);
        }
    }
}

fn line_waiter(d: &Domain, owner: Token) -> Option<Waiter> {
    if d.lost.contains_key(&owner) {
        Some(Waiter::Lost(owner))
    } else if crate::landing::gather_read(d, owner) {
        Some(Waiter::Landing(owner))
    } else {
        None
    }
}

pub(super) fn ask_line(d: &mut Domain, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) -> bool {
    if !reading(d, owner) || !riding(d, owner) {
        return false;
    }
    if let Some(lost) = d.lost.get_mut(&owner) {
        lost.state = super::LostReadState::Asked;
    } else {
        crate::landing::asking(d, owner);
    }
    ask(d, env, owner, out);
    true
}

fn ask(d: &mut Domain, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) {
    let reading = state(d).reads.get_mut(&owner).expect("due pending read");
    reading.wait.at = None;
    let event = Event::Read { owner, repository: reading.repository, read: reading.read.clone() };
    // Do not track again: the same read keeps its wait and outage span.
    super::client_step(d, env, event, out);
}

pub(super) fn refresh(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if d.stopping {
        return;
    }
    let Some(active) = &mut d.outage else { return };
    let back = core::mem::replace(&mut active.back, Set::with_capacity(env.limits.forges));
    let mut heads = Set::with_capacity(env.limits.subscriptions);
    for (_, subscriber) in &d.subscriptions {
        match &subscriber.topic {
            Topic::Ci { repository, head } if back.contains(&repository.forge) => {
                heads.insert((*repository, *head)).expect("one head per CI subscription");
            }
            Topic::Ci { .. }
            | Topic::Landings { .. }
            | Topic::Pull { .. }
            | Topic::Participation { .. }
            | Topic::Findings { .. } => {}
        }
    }
    for (repository, head) in &heads {
        let mut older = None;
        for (owner, pending) in &d.pending_ci {
            if pending.repository == *repository && pending.head == *head {
                older = Some(*owner);
                break;
            }
        }
        if let Some(owner) = older {
            // A pre-recovery read may carry an older snapshot. Let it finish,
            // then ask afresh rather than accepting it as the recovery pass.
            d.pending_ci.get_mut(&owner).expect("pending CI read").refresh = true;
        } else {
            super::ci_refresh(d, env, *repository, *head, out);
        }
    }
}

pub(super) fn retire(d: &mut Domain, forge: u16) {
    let Some(active) = &mut d.outage else { return };
    let mut reads = skein_lib::List::with_capacity(active.reads.len());
    for (owner, reading) in &active.reads {
        if reading.repository.forge == forge {
            reads.push(*owner).expect("retained read count");
        }
    }
    for owner in reads.into_boxed() {
        active.reads.remove(&owner);
    }
    active.away.remove(&forge);
    active.back.remove(&forge);
}

pub(super) fn release_repository(d: &mut Domain, repository: Repository) {
    let Some(active) = &mut d.outage else { return };
    let mut keys = skein_lib::List::with_capacity(active.reads.len());
    for (key, row) in &active.reads {
        if row.repository == repository {
            keys.push(*key).expect("bounded reads");
        }
    }
    for key in keys.into_boxed() {
        let _row = active.reads.remove(&key).expect("collected reads");
    }
    let mut calls = skein_lib::List::with_capacity(active.calls.len());
    for (call, (owner, _)) in &active.calls {
        if *owner == repository {
            calls.push(*call).expect("bounded outage calls");
        }
    }
    for call in calls.into_boxed() {
        active.calls.remove(&call);
    }
}
