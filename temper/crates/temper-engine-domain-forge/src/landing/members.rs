//! The live membership index and the change's ask queue (temper/docs/design/domain/landing-members.md, 2.1 and 3).
use super::{State, commit_hex, concat, matches_terminal, policy};
use crate::{ChangeRow, Domain, LandingRow, RecordKind, Request, RestoreFit, RestoreSet};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Map, Queue};
use temper_engine_domain_forge_change as change;

pub(crate) const ASKS: u32 = 16;

impl Domain {
    /// The oldest live landing whose row holds this change as a member that has not left.
    #[must_use]
    pub fn landing_of(&self, change: u64) -> Option<u64> {
        self.landing.memberships.get(&change).copied()
    }
}

pub(super) fn holds(row: &LandingRow, change: u64) -> bool {
    if matches_terminal(&row.landing.phase) {
        return false;
    }
    for member in &row.landing.members {
        if member.change == change && member.left.is_none() {
            return true;
        }
    }
    false
}

fn still_holds(row: Option<&LandingRow>, change: u64) -> bool {
    if let Some(row) = row { holds(row, change) } else { false }
}

pub(super) fn same_members(prior: &LandingRow, row: &LandingRow) -> bool {
    let ended = matches_terminal(&row.landing.phase);
    if matches_terminal(&prior.landing.phase) != ended {
        return false;
    }
    if ended {
        return true;
    }
    if prior.landing.members.len() != row.landing.members.len() {
        return false;
    }
    for (before, after) in prior.landing.members.iter().zip(&row.landing.members) {
        if before.change != after.change || before.left.is_none() != after.left.is_none() {
            return false;
        }
    }
    true
}

impl State {
    pub(crate) fn restore_row(&mut self, row: LandingRow) {
        self.rows.insert(row.landing.task, row).expect("restored landing row was admitted");
        self.memberships_unbuilt = true;
    }

    /// Check claims in task order before publishing the index (domain/landing-members.md, 2.2).
    pub(crate) fn restore_memberships(&mut self) -> Result<(), RestoreFit> {
        if !self.memberships_unbuilt {
            return Ok(());
        }
        let mut memberships = Map::with_capacity(self.memberships.capacity());
        let mut claims = Map::with_capacity(self.claims.capacity());
        for (task, row) in &self.rows {
            if matches_terminal(&row.landing.phase) {
                continue;
            }
            for member in &row.landing.members {
                if member.left.is_some() {
                    continue;
                }
                match memberships.get(&member.change) {
                    Some(older) if older < task => {
                        if let policy::Phase::Held { why: policy::Hold::Claimed { .. }, .. } = row.landing.phase {
                            continue;
                        }
                        if !claims.contains_key(task) {
                            claims.insert(*task, member.change).expect("one claim mark per admitted row");
                        }
                    }
                    Some(_) => {}
                    None => record(&mut memberships, member.change, *task)?,
                }
            }
        }
        self.claims = claims;
        self.memberships = memberships;
        self.memberships_unbuilt = false;
        Ok(())
    }

    /// Replace a row and its projection together.
    pub(crate) fn remember(&mut self, row: LandingRow) -> Result<Option<LandingRow>, RestoreFit> {
        let task = row.landing.task;
        let prior = self.rows.insert(task, row).expect("landing row was admitted");
        if let Some(old) = &prior {
            if same_members(old, self.rows.get(&task).expect("inserted landing")) {
                return Ok(prior);
            }
            self.departed(old);
        }
        let row = self.rows.get(&task).expect("inserted landing");
        if matches_terminal(&row.landing.phase) {
            self.claims.remove(&task);
        } else {
            for member in &row.landing.members {
                let change = member.change;
                if member.left.is_none() && self.memberships.get(&change).copied().unwrap_or(u64::MAX) >= task {
                    record(&mut self.memberships, change, task)?;
                }
            }
        }
        Ok(prior)
    }

    pub(crate) fn erase(&mut self, task: u64) -> Option<LandingRow> {
        let row = self.rows.remove(&task)?;
        self.claims.remove(&task);
        self.unmade.remove(&task);
        self.departed(&row);
        Some(row)
    }

    fn departed(&mut self, prior: &LandingRow) {
        let task = prior.landing.task;
        let row = self.rows.get(&task);
        for member in &prior.landing.members {
            let change = member.change;
            if self.memberships.get(&change) != Some(&task) || still_holds(row, change) {
                continue;
            }
            // A conflicting restored claim may outlive its older owner.
            let mut oldest = None;
            for (other, saved) in &self.rows {
                if holds(saved, change) {
                    oldest = Some(*other);
                    break;
                }
            }
            if let Some(other) = oldest {
                self.memberships.insert(change, other).expect("replaces the older claim");
            } else {
                self.memberships.remove(&change);
            }
        }
    }
}

fn record(memberships: &mut Map<u64, u64>, change: u64, task: u64) -> Result<(), RestoreFit> {
    memberships.insert(change, task).or(Err(RestoreFit {
        family: Some(RecordKind::Landing),
        set: RestoreSet::LandingMemberships,
        held: u64::from(memberships.len()),
        additional: 1,
        limit: u64::from(memberships.capacity()),
    }))?;
    Ok(())
}

pub(super) fn older(d: &Domain, task: u64, change: u64) -> Option<u64> {
    let owner = d.landing_of(change)?;
    if owner < task { Some(owner) } else { None }
}

pub(super) fn conflict(d: &Domain, row: &LandingRow) -> Option<u64> {
    for member in &row.landing.members {
        if member.left.is_none() && older(d, row.landing.task, member.change).is_some() {
            return Some(member.change);
        }
    }
    None
}

pub(crate) fn head(row: &ChangeRow) -> Option<&change::Handed> {
    for handed in &row.asks {
        if let change::Ask::Landed { .. } = handed.ask {
            return Some(handed);
        }
    }
    row.asks.first()
}

pub(super) fn queued(row: &ChangeRow, landing: u64) -> bool {
    for handed in &row.asks {
        if handed.landing == landing {
            return true;
        }
    }
    false
}

pub(super) fn asked(row: &ChangeRow, landing: &LandingRow) -> bool {
    if queued(row, landing.landing.task) {
        return true;
    }
    match &row.change.state {
        change::State::Preparing { .. } => true,
        change::State::Judging { target, .. } => {
            target.landing == landing.landing.task && target.round == landing.landing.preparation
        }
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Ready { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Held { .. }
        | change::State::Landed { .. }
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Kind {
    Prepare,
    Repair,
    Landed,
    Resolve,
    Judge,
}

impl Kind {
    pub(crate) fn words(self) -> &'static str {
        match self {
            Self::Prepare => "prepare",
            Self::Repair => "repair",
            Self::Landed => "landed",
            Self::Resolve => "resolve",
            Self::Judge => "judge",
        }
    }
}

pub(crate) fn kind(ask: &change::Ask) -> Kind {
    match ask {
        change::Ask::Prepare { .. } => Kind::Prepare,
        change::Ask::Repair { .. } => Kind::Repair,
        change::Ask::Landed { .. } => Kind::Landed,
        change::Ask::Resolve { .. } => Kind::Resolve,
        change::Ask::Judge(_) => Kind::Judge,
    }
}

pub(crate) fn hand(d: &mut Domain, row: &LandingRow, member: u16, ask: change::Ask, out: &mut Queue<Request>) {
    let task = row.landing.members.get(usize::from(member)).expect("member").change;
    let Some(change) = d.changes.get_mut(&task) else { return };
    let landing = row.landing.task;
    let round = row.landing.preparation;
    let mut asks = List::with_capacity(ASKS);
    for old in &change.asks {
        if old.landing != landing || kind(&old.ask) != kind(&ask) {
            asks.push(old.clone()).expect("bounded asks");
        }
    }
    let number = change.handed.checked_add(1).expect("change ask numbers fit");
    asks.push(change::Handed { number, landing, round, ask }).expect("one ask of each kind per landing");
    change.handed = number;
    change.asks = asks.into_boxed();
    let saved = change.clone();
    crate::domain::save_change(d, out, saved);
    out.push(Request::WakeProcedure { task });
}

/// Remove exactly the number returned by the policy, not the current head.
pub(crate) fn take(row: &mut ChangeRow, took: change::Took, out: &mut Queue<Request>) {
    let number = match took {
        change::Took::Taken(number) | change::Took::Dropped(number) => number,
    };
    let mut asks = List::with_capacity(ASKS);
    for ask in &row.asks {
        if ask.number != number {
            asks.push(ask.clone()).expect("bounded asks");
        } else if let change::Took::Dropped(_) = took {
            let words = dropped_words(row, &ask.ask);
            out.push(Request::AskDropped { change: row.task, words });
        }
    }
    row.asks = asks.into_boxed();
}

fn dropped_words(row: &ChangeRow, ask: &change::Ask) -> Box<[u8]> {
    let task = Decimal::of(row.task);
    match ask {
        change::Ask::Prepare { head: expected, .. } => {
            let change::State::Ready { head, .. } = row.change.state else {
                unreachable!("a prepare is dropped only at a ready head");
            };
            concat(&[
                b"change ",
                task.as_bytes(),
                b": prepare for ",
                &commit_hex(*expected),
                b" dropped: the head is now ",
                &commit_hex(head),
            ])
        }
        change::Ask::Judge(target) => concat(&[
            b"change ",
            task.as_bytes(),
            b": judgement of landing ",
            Decimal::of(target.landing).as_bytes(),
            b", round ",
            Decimal::of(u64::from(target.round)).as_bytes(),
            b", dropped: the change is in that landing no longer",
        ]),
        change::Ask::Repair { .. } | change::Ask::Resolve { .. } | change::Ask::Landed { .. } => {
            unreachable!("the policy never drops a repair, resolve or landed ask")
        }
    }
}

pub(crate) fn end(d: &mut Domain, row: &LandingRow, out: &mut Queue<Request>) {
    for member in &row.landing.members {
        let Some(change) = d.changes.get_mut(&member.change) else { continue };
        let mut asks = List::with_capacity(ASKS);
        for ask in &change.asks {
            if ask.landing == row.landing.task
                && let change::Ask::Prepare { .. } | change::Ask::Judge(_) = ask.ask
            {
                continue;
            }
            asks.push(ask.clone()).expect("bounded asks");
        }
        if asks.as_slice() != change.asks.as_ref() {
            change.asks = asks.into_boxed();
            let saved = change.clone();
            crate::domain::save_change(d, out, saved);
        }
    }
}
