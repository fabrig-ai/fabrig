//! One first-come-first-served outbox line (domain/forge/room.md, 2–4).
use crate::domain::writes;
use crate::{Domain, Limits, Name, Request};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::waits::{AtBound, Reported, ReportedBound, Thing, Unbounded, WaitKind, WaitObject};
use skein_lib::{Decimal, Duration, Env, List, Map, Queue, Set, Token, Wall, bytes};
use temper_client_protocol_wire::waits::{THING_ENTRY, THING_WRITE_ENTRY, THING_WRITE_RECORD};
use temper_engine_domain_forge_landing::Phase;
use temper_engine_domain_forge_vocabulary::writes::{Sender, WRITES};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Waiter {
    Change(u64),
    Landing(u64),
    Making(Token),
    Release(u64),
}

#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) places: Map<Waiter, Place>,
    pub(crate) asked: Map<Waiter, Asked>,
    passed: Set<Waiter>,
    write_blocks: Map<Waiter, Name>,
    pub(crate) ready: bool,
    pub(crate) deadline: Option<Wall>,
    // Release cleanup can keep new entries after its task has begun to close.
    pub(crate) closing: Map<u64, Wall>,
    restored_owners_known: bool,
}

impl State {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self {
            places: Map::uncapped(),
            write_blocks: Map::uncapped(),
            asked: Map::with_capacity(limits.entries),
            passed: Set::with_capacity(
                limits
                    .changes
                    .saturating_add(limits.holds)
                    .saturating_add(limits.landings)
                    .saturating_add(limits.tasks),
            ),
            ready: false,
            deadline: None,
            closing: Map::with_capacity(limits.tasks),
            restored_owners_known: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct Asked {
    task: u64,
    place: u64,
}

#[derive(Debug, Hash)]
pub(crate) struct Place {
    order: u64,
    since: Wall,
}

const NAMED_ENTRIES: usize = 4;

#[derive(PartialEq, Eq, Debug, Hash)]
pub(crate) struct Holders {
    entries: [Option<Thing>; NAMED_ENTRIES],
    behind: Option<u64>,
    latest_end: Option<Wall>,
    // Used only if every holder ended and there is no entry to name.
    oldest_reservation: Option<u64>,
}

pub(crate) fn holders(d: &Domain) -> Holders {
    let mut entries = [None; NAMED_ENTRIES];
    let mut behind = None;
    let mut latest_end = None;
    for (index, (number, row)) in d.entries.iter().enumerate() {
        if let Some(slot) = entries.get_mut(index) {
            *slot = Some(Thing { what: THING_ENTRY, number: *number, owner: Some(row.entry.task) });
        } else if behind.is_some() {
            break;
        }
        if let Some(ended) = row.owner_ended {
            latest_end = latest_end.max(Some(ended));
        } else if behind.is_none() {
            behind = Some(row.entry.task);
        }
    }
    let mut oldest_reservation = None;
    if behind.is_none() {
        let (live, ended, oldest) = crate::domain::full::reservation_owners(d);
        behind = live;
        latest_end = latest_end.max(ended);
        oldest_reservation = oldest;
    }
    // Ends do not affect any record while a live holder remains.
    if behind.is_some() {
        latest_end = None;
        oldest_reservation = None;
    } else if !d.entries.is_empty() {
        oldest_reservation = None;
    }
    Holders { entries, behind, latest_end, oldest_reservation }
}

impl Holders {
    fn bound(&self, since: Wall, figure: Duration) -> Option<ReportedBound> {
        if let Some(task) = self.behind {
            return Some(ReportedBound::None(Unbounded::Behind(task)));
        }
        let origin = since.max(self.latest_end?);
        let at = Wall::from_nanos(origin.as_nanos().saturating_add(figure.as_nanos()));
        Some(ReportedBound::At { at, then: AtBound::Hold })
    }

    fn oldest_words(&self) -> Option<(u64, Box<[u8]>)> {
        if let [Some(first), ..] = self.entries {
            Some((
                first.number,
                bytes::concat(&[
                    b"entry ",
                    Decimal::of(first.number).as_bytes(),
                    b" of ended task ",
                    Decimal::of(first.owner?).as_bytes(),
                ]),
            ))
        } else {
            Some((
                0,
                bytes::concat(&[b"a reservation for ended task ", Decimal::of(self.oldest_reservation?).as_bytes()]),
            ))
        }
    }

    fn account(&self, step: &[u8], figure: Duration) -> Option<crate::EffectAccount> {
        let (entry, holder) = self.oldest_words()?;
        let words = bytes::concat(&[
            b"no progress at ",
            step,
            b" for ",
            Decimal::of(figure.as_nanos() / 60_000_000_000).as_bytes(),
            b" minutes: waiting for room in the forge's outbox, held by ",
            &holder,
        ]);
        Some(crate::EffectAccount { entry, words })
    }
}

impl Domain {
    /// At catch-up, the root's task arena identifies which restored writers are still live.
    pub fn prepare_room_catch_up(&mut self, writers: &Set<u64>) {
        self.room.restored_owners_known = true;
        let Some(started) = self.restore_started else { return };
        let mut ended = List::with_capacity(self.entries.len());
        for (number, row) in &self.entries {
            if !writers.contains(&row.entry.task) {
                ended.push(*number).expect("counted entries");
            }
        }
        mark_restored(self, &ended, started);
    }

    /// A room wait's finite bound and whether it has an armed timer (engine-checks.md, 2.2).
    #[must_use]
    pub fn inspection_room_wait(&self, task: u64) -> Option<(Wall, bool)> {
        for waiter in [Waiter::Change(task), Waiter::Landing(task), Waiter::Release(task)] {
            if let Some(wait) = inspection(self, waiter) {
                return Some(wait);
            }
        }
        for (owner, pending) in &self.makings {
            if pending.task == task
                && let Some(wait) = inspection(self, Waiter::Making(*owner))
            {
                return Some(wait);
            }
        }
        None
    }

    /// When an outbox writer closed or ended, as learned by the forge.
    #[must_use]
    pub fn inspection_entry_owner_ended(&self, number: u64) -> Option<Wall> {
        self.entries.get(&number)?.owner_ended
    }
}

fn inspection(d: &Domain, waiter: Waiter) -> Option<(Wall, bool)> {
    match bound(d, d.inspection_limits(), waiter)? {
        ReportedBound::At { at, .. } => Some((at, d.room.deadline.is_some() && eligible(d, waiter))),
        ReportedBound::Figure(_) | ReportedBound::None(_) => None,
    }
}

pub(crate) fn catch_up(d: &mut Domain, now: Wall) {
    let started = d.restore_started.unwrap_or(now);
    if d.restore_started.is_some() && !d.room.restored_owners_known {
        // Standalone worlds have no task arena; use the writer rows they drive.
        let mut ended = List::with_capacity(d.entries.len());
        for (number, row) in &d.entries {
            let task = row.entry.task;
            let live = standalone_writer(d, task);
            if !live {
                ended.push(*number).expect("counted entries");
            }
        }
        mark_restored(d, &ended, started);
        d.room.restored_owners_known = true;
    }
    let mut releases = List::with_capacity(d.releases.len());
    for (task, row) in &d.releases {
        if !d.room.closing.contains_key(task) {
            d.room.closing.insert(*task, started).expect("counted releases");
        }
        if row.room.is_some() {
            releases.push(*task).expect("counted releases");
        }
    }
    for task in &releases {
        observe(d, *task, started);
    }
}

fn standalone_writer(d: &Domain, task: u64) -> bool {
    if d.releases.contains_key(&task) {
        return false;
    }
    if let Some(row) = d.changes.get(&task) {
        return !row.frozen();
    }
    if let Some(row) = d.landing.rows.get(&task) {
        return match row.landing.phase {
            Phase::Cancelled | Phase::Landed => false,
            Phase::Waiting
            | Phase::Preparing
            | Phase::Proving
            | Phase::Proved { .. }
            | Phase::Turn { .. }
            | Phase::Held { .. } => true,
        };
    }
    if let Some(row) = d.issues.get(&task)
        && let Some(goal) = &row.desired
    {
        return goal.finished.is_none();
    }
    for (_, hold) in &d.holds {
        if hold.task == task {
            return true;
        }
    }
    false
}

fn mark_restored(d: &mut Domain, ended: &List<u64>, started: Wall) {
    for number in ended {
        d.entries.get_mut(number).expect("restored entry").owner_ended.get_or_insert(started);
    }
}

pub(crate) fn release_started(d: &mut Domain, task: u64, now: Wall) {
    if !d.room.closing.contains_key(&task) {
        d.room.closing.insert(task, now).expect("one clock per release row");
        crate::domain::goal_branch::end_room_waits(d, task);
        owner_ended(d, task, now);
        crate::waits::changed(d, task);
    }
}

pub(crate) fn owner_ended(d: &mut Domain, task: u64, now: Wall) {
    let mut ended = List::uncapped();
    for (number, row) in &d.entries {
        if row.entry.task == task && row.owner_ended.is_none() {
            ended.push(*number).expect("counted entries");
        }
    }
    for number in &ended {
        d.entries.get_mut(number).expect("ending writer's entry").owner_ended = Some(now);
    }
    d.full.owner_ended(task, now);
}

/// Keep a place's join time alongside its durable order, without storing a second timestamp.
pub(crate) fn observe(d: &mut Domain, task: u64, since: Wall) {
    let procedure = d.changes.contains_key(&task) || d.landing.rows.contains_key(&task) || d.closer(task).is_some();
    for waiter in [Waiter::Change(task), Waiter::Landing(task), Waiter::Release(task)] {
        if (procedure || waiter == Waiter::Release(task))
            && let Some(order) = place(d, waiter)
        {
            let prior = d.room.places.get(&waiter);
            if let Some(prior) = prior
                && prior.order == order
            {
                continue;
            }
            d.room.places.insert(waiter, Place { order, since }).expect("one join per place");
        } else {
            d.room.places.remove(&waiter);
        }
    }
}

/// Keep only the reason a place cannot yet be asked. The procedure's next
/// turn re-reads and decides the write again, just as for an outbox wait.
pub(crate) fn block_write(d: &mut Domain, waiter: Waiter, name: Option<Name>) {
    if let Some(name) = name {
        // A blocked write keeps its place, not a reservation it cannot use.
        give_back_ask(d, waiter);
        d.room.write_blocks.insert(waiter, name).expect("one branch record per waiting write");
    } else {
        d.room.write_blocks.remove(&waiter);
    }
    if let Waiter::Change(task) | Waiter::Landing(task) | Waiter::Release(task) = waiter {
        crate::waits::changed(d, task);
    }
}

fn write_block(d: &Domain, waiter: Waiter) -> Option<&Name> {
    let name = d.room.write_blocks.get(&waiter)?;
    if writes::full(d, name) { Some(name) } else { None }
}

fn write_holders(d: &Domain, name: &Name, since: Wall) -> Holders {
    let mut entries = [None; NAMED_ENTRIES];
    let mut things = entries.iter_mut();
    *things.next().expect("the record is the first thing") =
        Some(Thing { what: THING_WRITE_RECORD, number: u64::try_from(WRITES).expect("write limit"), owner: None });
    let mut behind = None;
    let mut latest_end = Some(since);
    for entry in &d.writes.get(name).expect("full record").entries {
        if !entry.state.unsettled() {
            continue;
        }
        let (task, ended) = match entry.sender {
            Sender::Entry { task, entry } => {
                let ended = match d.entries.get(&entry) {
                    Some(row) => row.owner_ended,
                    None => d.room.closing.get(&task).copied(),
                };
                (Some(task), ended)
            }
            Sender::Attempt { task, .. } => (Some(task), d.room.closing.get(&task).copied()),
            Sender::Previous => (None, None),
        };
        if let Some(place) = things.next() {
            *place = Some(Thing { what: THING_WRITE_ENTRY, number: entry.order, owner: task });
        }
        if let Some(at) = ended {
            latest_end = latest_end.max(Some(at));
        } else if behind.is_none() {
            behind = task;
        }
    }
    Holders { entries, behind, latest_end, oldest_reservation: None }
}

fn waiting_holders(d: &Domain, waiter: Waiter, since: Wall) -> Holders {
    match write_block(d, waiter) {
        Some(name) => write_holders(d, name, since),
        None => holders(d),
    }
}

fn blocked_since(d: &Domain, limits: &Limits, waiter: Waiter) -> Option<Wall> {
    if asked(d, waiter) || (free(d, limits) != 0 && write_block(d, waiter).is_none()) {
        return None;
    }
    match waiter {
        Waiter::Making(owner) => Some(d.makings.get(&owner)?.outbox_wait.as_ref()?.since),
        Waiter::Change(_) | Waiter::Landing(_) | Waiter::Release(_) => Some(d.room.places.get(&waiter)?.since),
    }
}

/// The room wait's bound, shared by wait reports, holds and inspection.
pub(crate) fn bound(d: &Domain, limits: &Limits, waiter: Waiter) -> Option<ReportedBound> {
    let since = blocked_since(d, limits, waiter)?;
    waiting_holders(d, waiter, since).bound(since, limits.waits.room)
}

/// Recheck the deadline on the landing's admitted step, after any freeing of room.
pub(crate) fn expired_landing(d: &Domain, env: &Env<Limits>, task: u64) -> Option<Box<[u8]>> {
    let waiter = Waiter::Landing(task);
    let since = blocked_since(d, &env.limits, waiter)?;
    let holders = waiting_holders(d, waiter, since);
    if let Some(ReportedBound::At { at, .. }) = holders.bound(since, env.limits.waits.room)
        && at <= env.wall
    {
        if let Some(name) = write_block(d, waiter) {
            return Some(writes::room_account(d, &env.limits, name));
        }
        let (_, holder) = holders.oldest_words()?;
        return Some(bytes::concat(&[b"held by ", &holder]));
    }
    None
}

/// The room record for a procedure with a place in the line (waits.md, 7).
pub(crate) fn waited(d: &Domain, limits: &Limits, task: u64, step: u32) -> Option<Reported> {
    let mut since = None;
    for waiter in [Waiter::Change(task), Waiter::Landing(task)] {
        if let Some(joined) = blocked_since(d, limits, waiter) {
            since = Some((waiter, joined));
            break;
        }
    }
    let (waiter, since) = since?;
    let holders = waiting_holders(d, waiter, since);
    let bound = holders.bound(since, limits.waits.room)?;
    let mut things = List::with_capacity(u32::try_from(NAMED_ENTRIES).expect("wait object bound"));
    for entry in holders.entries.into_iter().flatten() {
        things.push(entry).expect("at most four entries");
    }
    Some(Reported { kind: WaitKind::Room, object: WaitObject::Procedure { step, things }, since, bound })
}

pub(crate) fn refresh_deadline(d: &mut Domain, limits: &Limits) {
    d.room.deadline = if d.stopping || !d.room.ready || (d.room.places.is_empty() && d.makings.is_empty()) {
        None
    } else {
        match next(d, limits) {
            Some((at, _)) => Some(at),
            None => None,
        }
    };
}

fn next(d: &Domain, limits: &Limits) -> Option<(Wall, Waiter)> {
    let mut first: Option<(Wall, Waiter)> = None;
    for (waiter, _) in &d.room.places {
        select_due(d, limits, *waiter, &mut first);
    }
    for (owner, pending) in &d.makings {
        if pending.outbox_wait.is_some() {
            select_due(d, limits, Waiter::Making(*owner), &mut first);
        }
    }
    first
}

fn eligible(d: &Domain, waiter: Waiter) -> bool {
    if d.room.passed.contains(&waiter) {
        return false;
    }
    match waiter {
        Waiter::Change(task) => match d.changes.get(&task) {
            Some(row) => {
                !row.frozen() && !d.releases.contains_key(&task) && !crate::domain::matches_held(&row.change.state)
            }
            None => false,
        },
        Waiter::Release(task) => match d.releases.get(&task) {
            Some(row) => row.releasing && !row.failed && row.pending.is_none(),
            None => false,
        },
        Waiter::Landing(task) => {
            if d.landing_held() {
                return false;
            }
            let Some(row) = d.landing.rows.get(&task) else { return false };
            match row.landing.phase {
                Phase::Held { .. } | Phase::Landed | Phase::Cancelled => false,
                Phase::Waiting | Phase::Preparing | Phase::Proving | Phase::Proved { .. } | Phase::Turn { .. } => true,
            }
        }
        Waiter::Making(owner) => match d.makings.get(&owner) {
            Some(pending) => pending.state == crate::domain::LostReadState::WaitingForOutbox && making_live(d, pending),
            None => false,
        },
    }
}

fn select_due(d: &Domain, limits: &Limits, waiter: Waiter, first: &mut Option<(Wall, Waiter)>) {
    if eligible(d, waiter)
        && let Some(ReportedBound::At { at, .. }) = bound(d, limits, waiter)
    {
        let due = (at, waiter);
        *first = Some(match *first {
            Some(prior) => prior.min(due),
            None => due,
        });
    }
}

/// Expire one room wait, after the line has taken any room freed by this step.
pub(crate) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
    if !match d.room.deadline {
        Some(at) => at <= env.wall,
        None => false,
    } {
        return false;
    }
    let (_, waiter) = next(d, &env.limits).expect("armed room wait");
    if let Waiter::Landing(task) = waiter {
        // The core must make a step due before a landing can commit its hold.
        d.room.passed.insert(waiter).expect("one expiry per waiter");
        out.push(Request::WakeProcedure { task });
        return true;
    }
    let step = match waiter {
        Waiter::Change(task) => d.changes.get(&task).expect("waiting change").change.state.step_name(),
        Waiter::Release(_) => b"release",
        Waiter::Making(_) => b"making",
        Waiter::Landing(_) => unreachable!("landing's room hold is handled by the landing"),
    };
    let account = match write_block(d, waiter) {
        Some(name) => crate::EffectAccount { entry: 0, words: writes::room_account(d, &env.limits, name) },
        None => holders(d).account(step, env.limits.waits.room).expect("expired wait has ended holders"),
    };
    match waiter {
        Waiter::Change(task) => crate::domain::hold_room_change(d, env, task, account, out),
        Waiter::Release(task) => crate::domain::room_release_failed(d, task, account.words, out),
        Waiter::Making(owner) => crate::domain::goal_branch::room_failed(d, owner, &account.words, out),
        Waiter::Landing(_) => unreachable!("landing's room hold is handled by the landing"),
    }
    true
}

/// A held change keeps its number but begins a fresh room wait on release.
pub(crate) fn released(d: &mut Domain, task: u64, at: Wall) {
    release_place(d, Waiter::Change(task), at);
}

pub(crate) fn release_place(d: &mut Domain, waiter: Waiter, at: Wall) {
    if let Some(place) = d.room.places.get_mut(&waiter) {
        place.since = at;
    }
}

pub(crate) fn reserved(d: &Domain) -> u64 {
    u64::from(d.room.asked.len())
        .saturating_add(u64::from(d.call_room.len()))
        .saturating_add(u64::from(d.projection_room.len()))
}

pub(crate) fn taken(d: &Domain) -> u64 {
    u64::from(d.entries.len()).saturating_add(reserved(d))
}

pub(crate) fn free(d: &Domain, limits: &Limits) -> u64 {
    free_at(d, limits.entries)
}

fn free_at(d: &Domain, limit: u32) -> u64 {
    u64::from(limit).saturating_sub(taken(d))
}

pub(crate) fn place(d: &Domain, waiter: Waiter) -> Option<u64> {
    match waiter {
        Waiter::Change(task) => d.changes.get(&task)?.room,
        Waiter::Landing(task) => d.landing.rows.get(&task)?.room,
        Waiter::Making(owner) => Some(d.makings.get(&owner)?.outbox_wait.as_ref()?.order),
        Waiter::Release(task) => d.releases.get(&task)?.room,
    }
}

fn first(d: &Domain) -> Option<Waiter> {
    let mut next: Option<(u64, Waiter)> = None;
    for (task, row) in &d.changes {
        if let Some(order) = row.room {
            select(d, &mut next, order, Waiter::Change(*task));
        }
    }
    for (task, row) in &d.landing.rows {
        if let Some(order) = row.room {
            select(d, &mut next, order, Waiter::Landing(*task));
        }
    }
    for (owner, pending) in &d.makings {
        if let Some(wait) = &pending.outbox_wait {
            select(d, &mut next, wait.order, Waiter::Making(*owner));
        }
    }
    for (task, row) in &d.releases {
        if let Some(order) = row.room {
            select(d, &mut next, order, Waiter::Release(*task));
        }
    }
    match next {
        Some((_, waiter)) => Some(waiter),
        None => None,
    }
}

fn select(d: &Domain, next: &mut Option<(u64, Waiter)>, order: u64, waiter: Waiter) {
    if match next {
        Some((before, _)) => order >= *before,
        None => false,
    } {
        return;
    }
    if !asked(d, waiter) && eligible(d, waiter) && write_block(d, waiter).is_none() {
        *next = Some((order, waiter));
    }
}

pub(crate) fn check(d: &Domain, limits: &Limits, waiter: Option<Waiter>) -> bool {
    if let Some(waiter) = waiter {
        if asked(d, waiter) {
            assert_asked(d, waiter);
            return true;
        }
        if d.room.passed.contains(&waiter) {
            return false;
        }
    }
    if free(d, limits) == 0 {
        return false;
    }
    let Some(first) = first(d) else { return true };
    match waiter {
        Some(waiter) => match place(d, waiter) {
            Some(order) => order <= place(d, first).expect("selected place"),
            None => false,
        },
        None => false,
    }
}

pub(crate) fn next_place(d: &Domain) -> u64 {
    let mut last = 0;
    for (_, row) in &d.changes {
        last = last.max(row.room.unwrap_or(0));
    }
    for (_, row) in &d.landing.rows {
        last = last.max(row.room.unwrap_or(0));
    }
    for (_, pending) in &d.makings {
        if let Some(wait) = &pending.outbox_wait {
            last = last.max(wait.order);
        }
    }
    for (_, row) in &d.releases {
        last = last.max(row.room.unwrap_or(0));
    }
    last.checked_add(1).expect("outbox places fit")
}

pub(crate) fn waiting(d: &Domain) -> bool {
    d.room.ready && free_at(d, d.entries.capacity()) > 0 && first(d).is_some()
}

pub(crate) fn asked(d: &Domain, waiter: Waiter) -> bool {
    d.room.asked.contains_key(&waiter)
}

fn waiter_task(d: &Domain, waiter: Waiter) -> Option<u64> {
    match waiter {
        Waiter::Change(task) | Waiter::Landing(task) | Waiter::Release(task) => Some(task),
        Waiter::Making(owner) => Some(d.makings.get(&owner)?.task),
    }
}

fn assert_place(d: &Domain, waiter: Waiter, ask: Asked) {
    let Asked { task, place: order } = ask;
    assert_eq!(waiter_task(d, waiter), Some(task), "room ask for {waiter:?} belongs to task {task}");
    assert_eq!(place(d, waiter), Some(order), "room ask for {waiter:?} at place {order} belongs to task {task}");
}

pub(crate) fn assert_asked(d: &Domain, waiter: Waiter) {
    let task = waiter_task(d, waiter);
    let Some(ask) = d.room.asked.get(&waiter).copied() else {
        unreachable!("waiter {waiter:?} of task {task:?} holds its outbox reservation");
    };
    assert_place(d, waiter, ask);
    let live = match waiter {
        // The making keeps its promise while it reads the default branch afresh.
        Waiter::Making(owner) => match d.makings.get(&owner) {
            Some(pending) => making_live(d, pending) && !d.room.passed.contains(&waiter),
            None => false,
        },
        Waiter::Change(_) | Waiter::Landing(_) | Waiter::Release(_) => eligible(d, waiter),
    };
    assert!(live, "asked waiter {waiter:?} of task {task:?} is neither held nor ended when taking room");
}

fn making_live(d: &Domain, pending: &crate::domain::goal_branch::Making) -> bool {
    crate::domain::goal_branch::held(d, pending) && !d.room.closing.contains_key(&pending.task)
}

pub(crate) fn reserve_ask(d: &mut Domain, waiter: Waiter, task: u64) {
    assert!(!asked(d, waiter), "one room reservation for asked waiter {waiter:?} of task {task}");
    let Some(order) = place(d, waiter) else {
        unreachable!("asked waiter {waiter:?} of task {task} has a place in the outbox line");
    };
    let ask = Asked { task, place: order };
    assert_place(d, waiter, ask);
    assert!(eligible(d, waiter), "waiter {waiter:?} of task {task} is eligible for its outbox ask");
    assert!(
        taken(d) < u64::from(d.entries.capacity()),
        "outbox reservation for waiter {waiter:?} of task {task} has room within forge.entries"
    );
    let inserted = d.room.asked.insert(waiter, ask);
    assert!(inserted.is_ok(), "outbox reservation for waiter {waiter:?} of task {task} fits its room");
    let ended = d.room.closing.get(&task).copied();
    d.full.reserve(crate::domain::full::Reservation::Asked(waiter), task, ended);
}

/// An owner may leave without ever having been asked (including on restart).
pub(crate) fn leave(d: &mut Domain, waiter: Waiter) {
    give_back_ask(d, waiter);
    d.room.write_blocks.remove(&waiter);
    if let Waiter::Change(task) | Waiter::Landing(task) | Waiter::Release(task) = waiter {
        crate::waits::changed(d, task);
    }
    d.room.passed.remove(&waiter);
}

fn give_back_ask(d: &mut Domain, waiter: Waiter) {
    if let Some(ask) = d.room.asked.get(&waiter).copied() {
        // Passing or ending a turn can itself be the reason its owner is no longer eligible.
        assert_place(d, waiter, ask);
        assert!(
            !d.room.passed.contains(&waiter),
            "asked outbox waiter {waiter:?} for task {} has not already been passed over",
            ask.task
        );
        d.full.assert_owner(crate::domain::full::Reservation::Asked(waiter), ask.task);
        let removed = d.room.asked.remove(&waiter);
        assert!(removed.is_some(), "return the held outbox reservation of {waiter:?} for task {}", ask.task);
    }
}

fn pass_waiter(d: &mut Domain, waiter: Waiter, task: u64) {
    give_back_ask(d, waiter);
    if place(d, waiter).is_some() {
        let inserted = d.room.passed.insert(waiter);
        assert!(inserted.is_ok(), "pass over outbox waiter {waiter:?} for task {task} in its own place");
    }
}

pub(crate) fn pass(d: &mut Domain, task: u64) {
    for waiter in [Waiter::Release(task), Waiter::Change(task), Waiter::Landing(task)] {
        pass_waiter(d, waiter, task);
    }
    let mut makings = List::with_capacity(d.makings.len());
    for (owner, pending) in &d.makings {
        if pending.task == task && pending.outbox_wait.is_some() {
            let pushed = makings.push(*owner);
            assert!(pushed.is_ok(), "bounded making {owner:?} for task {task} to pass in the outbox line");
        }
    }
    for owner in &makings {
        pass_waiter(d, Waiter::Making(*owner), task);
    }
}

pub(crate) fn stepping(d: &mut Domain, task: u64) {
    d.room.passed.remove(&Waiter::Change(task));
    d.room.passed.remove(&Waiter::Landing(task));
    d.room.passed.remove(&Waiter::Release(task));
    for (owner, pending) in &d.makings {
        if pending.task == task {
            d.room.passed.remove(&Waiter::Making(*owner));
        }
    }
}

pub(crate) fn ask(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    crate::domain::full::update(d, env, out);
    let mut stale = List::uncapped();
    for (owner, pending) in &d.makings {
        let live = match d.holds.get(&pending.name) {
            Some(hold) => hold.task == pending.task,
            None => false,
        };
        if pending.outbox_wait.is_some() && !live {
            stale.push(*owner).expect("making waits");
        }
    }
    let pruned = !stale.is_empty();
    for owner in stale.into_boxed() {
        crate::domain::goal_branch::drop_making(d, owner);
    }
    // Pruning can end a full interval before the next admission fills it again.
    // At the clock's maximum a saturated report deadline remains due.
    let saturated = env.now.as_nanos() == u64::MAX;
    if pruned || saturated {
        crate::domain::full::update(d, env, out);
    }
    if !d.room.ready || d.stopping {
        crate::waits::flush(d, env, out);
        return;
    }
    let mut admitted = false;
    while free(d, &env.limits) > 0 {
        let Some(waiter) = first(d) else { break };
        admitted = true;
        let Some(task) = waiter_task(d, waiter) else {
            unreachable!("selected outbox waiter {waiter:?} has its owner task");
        };
        reserve_ask(d, waiter, task);
        crate::waits::changed(d, task);
        let request = match waiter {
            Waiter::Change(task) => {
                d.procedure_wakes.remove(&task);
                Request::WakeProcedure { task }
            }
            Waiter::Landing(task) => Request::WakeProcedure { task },
            Waiter::Release(task) => Request::ContinueRelease { task },
            Waiter::Making(_) => Request::ContinueGoalBranch { task },
        };
        out.push(request);
    }
    // With no admission, neither holders nor report inputs changed since the last update.
    if admitted || saturated {
        crate::domain::full::update(d, env, out);
    }
    crate::waits::flush(d, env, out);
}
