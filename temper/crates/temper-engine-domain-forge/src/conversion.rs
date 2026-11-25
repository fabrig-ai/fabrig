//! One written conversion in Start's Link phase (temper/docs/design/domain/landing-members.md, 4.2).
use crate::landing::members::{ASKS, Kind, kind};
use crate::{ChangeRow, Domain, LandingRow, RecordKind, Request, RestoreFit, RestoreSet, Stored};
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;

// The map's order is also the numbering order: landed first, then oldest landing first.
type Order = (bool, u64, Kind);
struct AskAt {
    round: u16,
    ask: change::Ask,
}
struct Conversion {
    newest: (u64, u16),
    asks: Map<Order, AskAt>,
}

impl Domain {
    pub(crate) fn convert_previous(&mut self, out: &mut Queue<Request>) -> Result<(), RestoreFit> {
        let mut changes = Map::with_capacity(self.changes.len());
        // A row can contribute at most one of each kind, plus one handed-over ask on the change.
        let candidates = self.landing.rows.len().saturating_mul(5).saturating_add(1);
        for (task, row) in &self.changes {
            if row.previous.is_some() {
                changes.insert(*task, Conversion::new(candidates)).ok().expect("one conversion per change");
            }
        }
        let mut rows = List::with_capacity(self.landing.rows.len());
        for (task, row) in &self.landing.rows {
            // The previous build wrote a slot even for a member with no ask. An empty list
            // is how the current build marks that a row owes no conversion, including derived asks.
            let legacy = !row.previous_asks.is_empty();
            if legacy {
                rows.push(*task).expect("one conversion per old landing");
            } else if changes.is_empty() {
                continue;
            }
            let at = (*task, row.landing.preparation);
            for (position, member) in row.landing.members.iter().enumerate() {
                let number = member.change;
                if let Some(pending) = changes.get_mut(&number) {
                    pending.newest = at;
                }
                if !legacy {
                    continue;
                }
                let Some(change) = self.changes.get(&number) else { continue };
                let from_slot = match row.previous_asks.get(position) {
                    Some(slot) => slot.as_ref(),
                    None => None,
                };
                let derived = if from_slot.is_none() && self.landing_of(number) == Some(*task) {
                    repair(member, change.change.last_head)
                } else {
                    None
                };
                let prefix = single_prefix(row, member);
                if from_slot.is_none() && derived.is_none() && prefix.is_none() {
                    continue;
                }
                if !changes.contains_key(&number) {
                    changes.insert(number, Conversion::new(candidates)).ok().expect("one conversion per change");
                }
                let pending = changes.get_mut(&number).expect("selected change for conversion");
                pending.newest = at;
                if let Some(ask) = from_slot {
                    pending.add(at, ask.clone());
                }
                if let Some(ask) = derived {
                    pending.add(at, ask);
                }
                if let Some(prefix) = prefix {
                    pending.add(at, change::Ask::Landed { prefix });
                }
            }
        }
        for _ in 0..changes.len() {
            let task = *changes.first().expect("one remaining change").0;
            let mut pending = changes.remove(&task).expect("selected change for conversion");
            let landing = self.landing_of(task);
            let row = self.changes.get_mut(&task).expect("selected restored change");
            if let Some(previous) = row.previous.take() {
                if let Some(named) = previous.landing
                    && landing != Some(named)
                {
                    out.push(Request::LinkDisagreed { change: task, named, landing });
                }
                if let Some(ask) = previous.ask {
                    pending.handed_over(ask);
                }
            }
            pending.write(row, out)?;
            let saved = row.clone();
            crate::domain::save_change_parts(self, out, saved);
        }
        for task in rows.into_boxed() {
            let row = self.landing.rows.get_mut(&task).expect("selected restored landing");
            row.previous_asks = Box::new([]);
            out.push(Request::Save { read_afresh: None, release: None, record: Stored::Landing(row.clone()) });
        }
        Ok(())
    }
}

fn single_prefix(row: &LandingRow, member: &landing::Member) -> Option<skein_forge::api::Commit> {
    if row.landing.round.branch.is_empty() { member.landed } else { None }
}

fn repair(member: &landing::Member, head: Option<skein_forge::api::Commit>) -> Option<change::Ask> {
    let candidate = member.candidate?;
    if head != Some(candidate) {
        return None;
    }
    for (gate, proof) in member.gates.iter().enumerate() {
        if let landing::Proof::Failed { .. } = proof.proof {
            let gate = u16::try_from(gate).expect("restored gate index fits");
            return Some(change::Ask::Repair { gate, candidate, below: None });
        }
    }
    None
}

fn order(landing: u64, ask: &change::Ask) -> Order {
    let kind = kind(ask);
    (kind != Kind::Landed, landing, kind)
}

impl Conversion {
    fn new(candidates: u32) -> Self {
        Self { newest: (0, 0), asks: Map::with_capacity(candidates) }
    }

    fn add(&mut self, (landing, round): (u64, u16), ask: change::Ask) {
        self.asks.insert(order(landing, &ask), AskAt { round, ask }).ok().expect("bounded old asks");
    }

    fn handed_over(&mut self, ask: change::Ask) {
        let at = match &ask {
            change::Ask::Judge(target) => (target.landing, target.round),
            change::Ask::Prepare { .. }
            | change::Ask::Repair { .. }
            | change::Ask::Resolve { .. }
            | change::Ask::Landed { .. } => self.newest,
        };
        if !self.asks.contains_key(&order(at.0, &ask)) {
            self.add(at, ask);
        }
    }

    fn write(mut self, row: &mut ChangeRow, out: &mut Queue<Request>) -> Result<(), RestoreFit> {
        if self.asks.is_empty() {
            return Ok(());
        }
        let mut asks = List::with_capacity(ASKS);
        for prior in &row.asks {
            self.asks.remove(&order(prior.landing, &prior.ask));
            asks.push(prior.clone()).expect("bounded stored queue");
        }
        let room = ASKS.saturating_sub(asks.len());
        let mut excess = self.asks.len().saturating_sub(room);
        for (&(ordinary, landing, kind), at) in &self.asks {
            if ordinary && excess != 0 {
                out.push(Request::ConversionAskDropped { change: row.task, landing, kind: kind.words() });
                excess = excess.saturating_sub(1);
                continue;
            }
            let number = row.handed.checked_add(1).expect("change ask numbers fit");
            if asks.push(change::Handed { number, landing, round: at.round, ask: at.ask.clone() }).is_err() {
                return Err(RestoreFit {
                    family: Some(RecordKind::Change),
                    set: RestoreSet::ChangeLandedAsks,
                    held: u64::from(ASKS),
                    additional: 1,
                    limit: u64::from(ASKS),
                });
            }
            row.handed = number;
        }
        row.asks = asks.into_boxed();
        Ok(())
    }
}
