//! Sealed, indivisible units selected in queue order (domain/landing.md, 4.1, 9).
use crate::{ChangeDependencies, Domain};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_change as change;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct WaitingUnit {
    /// The first queue candidate, which owns a refusal hold.
    pub task: u64,
    pub changes: Box<[u64]>,
    pub alone: bool,
    pub refused: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Selection {
    /// Positions in the supplied units, in their queue order.
    pub take: Box<[u32]>,
    pub hold: Box<[u32]>,
}
/// Take a prefix of admitted units, passing refusals, never splitting a unit.
/// The first may pass batch, but no unit or landing may pass `landing_members`.
#[must_use]
pub fn select_units(units: &[WaitingUnit], batch: u16, landing_members: u32) -> Selection {
    let capacity = u32::try_from(units.len()).expect("bounded waiting units");
    let mut take = List::with_capacity(capacity);
    let mut hold = List::with_capacity(capacity);
    let mut count = 0_u32;
    for (index, unit) in units.iter().enumerate() {
        let size = u32::try_from(unit.changes.len()).expect("bounded unit");
        let index = u32::try_from(index).expect("bounded units");
        if unit.refused || size > landing_members {
            hold.push(index).expect("one position per unit");
            continue;
        }
        let next = count.saturating_add(size);
        if count != 0 && (unit.alone || next > u32::from(batch) || next > landing_members) {
            break;
        }
        take.push(index).expect("one position per unit");
        count = next;
        if unit.alone || count >= u32::from(batch) {
            break;
        }
    }
    Selection { take: take.into_boxed(), hold: hold.into_boxed() }
}
impl Domain {
    /// Waiting closures ordered by their first queue candidate. Every member must wait;
    /// task holds and pending task work are supplied by the root, not stored here.
    #[must_use]
    pub fn waiting_units(
        &self,
        repository: skein_forge::api::Repository,
        branch: &[u8],
        kept: &[u64],
        graph: &[ChangeDependencies],
        now: Wall,
    ) -> Box<[WaitingUnit]> {
        let mut ready = List::from_boxed(self.waiting_changes(repository, branch));
        let mut seen = List::with_capacity(self.landing.limits.changes);
        let mut units = List::with_capacity(self.landing.limits.changes);
        let Some(repo) = self.repository(repository) else { return units.into_boxed() };
        while let Some(task) = change::first(ready.as_slice(), self.landing.limits.queue_window, now) {
            let mut remaining = List::with_capacity(self.landing.limits.changes);
            for candidate in &ready {
                if candidate.task != task {
                    remaining.push(*candidate).expect("bounded changes");
                }
            }
            ready = remaining;
            if seen.as_slice().contains(&task) {
                continue;
            }
            let joined = self.joined_changes(repo.project, task, graph);
            let mut eligible = true;
            let mut alone = false;
            let mut refused = false;
            for member in &joined {
                seen.push(*member).expect("disjoint closures");
                let Some(row) = self.change(*member) else {
                    eligible = false;
                    continue;
                };
                eligible &= super::landing::waiting(row) && !kept.contains(member);
                refused |= row.repository != repository;
                alone |= row.change.alone;
            }
            if eligible {
                units.push(WaitingUnit { task, changes: joined, alone, refused }).expect("bounded units");
            }
        }
        units.into_boxed()
    }
    /// The forming decision for a free train with room for a landing.
    #[must_use]
    pub fn select_waiting(
        &self,
        repository: skein_forge::api::Repository,
        branch: &[u8],
        units: &[WaitingUnit],
    ) -> Selection {
        let selection = select_units(units, self.train_batch(repository, branch), self.landing.limits.landing_members);
        if self.landing.limits.landing_members == 0 || !self.train_free(repository, branch) {
            return Selection { take: Box::new([]), hold: Box::new([]) };
        }
        let Some(index) = selection.take.first() else { return selection };
        let Some(unit) = units.get(skein_lib::usize_of(*index)) else { return selection };
        let Some(first) = unit.changes.first() else { return selection };
        let Some(repo) = self.repository(repository) else {
            return Selection { take: Box::new([]), hold: Box::new([]) };
        };
        if self.landing_admits(repo.project, &[*first]).is_err() {
            return Selection { take: Box::new([]), hold: Box::new([]) };
        }
        selection
    }
}
