//! Budgeted ordered map. The fixed charge per entry includes B-tree nodes.

#![expect(clippy::disallowed_types, reason = "returned pages are bounded by Page")]
#![expect(clippy::disallowed_methods, reason = "pure lookups and bounded admission scans have no hidden effects")]

use crate::{Census, CensusRule, Limits, Op, Page, Range, Refusal, Row};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Bound;
use skein_lib::{Map, usize_of};

/// A conservative charge for tree nodes, key and value boxes and allocator
/// metadata. Payload lengths are charged separately.
pub(crate) const ENTRY: u64 = 128;

#[derive(Debug)]
pub(crate) struct PageRows {
    pub(crate) rows: Box<[Row]>,
    pub(crate) next: Option<Box<[u8]>>,
}

#[derive(Debug)]
pub(crate) struct KvMap {
    entries: Map<Box<[u8]>, Box<[u8]>>,
    used: u64,
    live: u64,
    pub(crate) census: Census,
}

/// A key's charge once the commits touching it apply, and the last of those commits.
#[derive(Debug)]
struct Charge {
    bytes: Option<u64>,
    number: u64,
}

/// What the commits in flight and queued leave behind: each key they touch, with its charge
/// once they apply, and the map's used bytes and entries then (skein/docs/design/data/kv.md, 7).
#[derive(Debug)]
pub(crate) struct Queued {
    charges: Map<Box<[u8]>, Charge>,
    used: u64,
    count: u32,
}

/// What the admitted commits and one more leave: the map's used bytes and entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Projected {
    used: u64,
    count: u32,
}

impl Queued {
    pub(crate) fn new() -> Queued {
        Queued { charges: Map::uncapped(), used: 0, count: 0 }
    }

    #[cfg(test)]
    pub(crate) fn keys(&self) -> u32 {
        self.charges.len()
    }

    /// Admit `ops`, which `check` projected: they are commit `number`.
    pub(crate) fn reserve(&mut self, ops: &[Op], projected: Projected, number: u64) {
        for (index, op) in ops.iter().enumerate() {
            if has_later(ops, index, op.key()) {
                continue;
            }
            let charge = op_charge(op).expect("the last op's charge passed admission");
            let previous = self.charges.insert(Box::from(op.key()), Charge { bytes: charge, number });
            assert!(previous.is_ok(), "projected keys are bounded by admitted commits");
        }
        self.used = projected.used;
        self.count = projected.count;
    }

    /// Check admission without reserving charges when the byte queue would refuse it (data/kv.md, 4).
    pub(crate) fn check(&self, map: &KvMap, ops: &[Op], limits: &Limits) -> Result<Projected, Refusal> {
        let (used, count) = if self.charges.is_empty() { (map.used, map.len()) } else { (self.used, self.count) };
        let mut after = i128::from(used);
        let mut count = i128::from(count);
        for (index, op) in ops.iter().enumerate() {
            let key = op.key();
            if key.len() > usize_of(limits.key) {
                return Err(Refusal::TooLarge);
            }
            match op {
                Op::Put { value, .. } => {
                    if value.len() > usize_of(limits.value) {
                        return Err(Refusal::TooLarge);
                    }
                }
                Op::Erase { .. } => {}
            }
            if has_later(ops, index, key) {
                continue;
            }
            let old = match self.charges.get(key) {
                Some(charge) => charge.bytes,
                None => match map.entries.get(key) {
                    Some(value) => Some(charge(key, value)?),
                    None => None,
                },
            };
            if let Some(old) = old {
                after = after.checked_sub(i128::from(old)).expect("bounded charges fit i128");
                count = count.checked_sub(1).expect("bounded entry counts fit i128");
            }
            if let Some(charge) = op_charge(op)? {
                after = after.checked_add(i128::from(charge)).expect("bounded charges fit i128");
                count = count.checked_add(1).expect("bounded entry counts fit i128");
            }
        }
        if after < 0 || after > i128::from(limits.budget) || count > i128::from(map.entries.capacity()) {
            return Err(Refusal::Full);
        }
        let used = u64::try_from(after).expect("projected bytes fit the budget");
        let count = u32::try_from(count).expect("projected entries fit the map capacity");
        Ok(Projected { used, count })
    }

    /// Commit `number` is in the map: forget the keys no later commit touches.
    pub(crate) fn applied(&mut self, ops: &[Op], number: u64) {
        for op in ops {
            if let Some(charge) = self.charges.get(op.key())
                && charge.number <= number
            {
                let _charge = self.charges.remove(op.key());
            }
        }
    }

    /// The commits failed: forget their projection before recovering the map.
    pub(crate) fn clear(&mut self) {
        *self = Queued::new();
    }
}

fn has_later(ops: &[Op], index: usize, key: &[u8]) -> bool {
    let next = index.checked_add(1).expect("an op's successor fits usize");
    for later in ops.get(next..).expect("index is within ops") {
        if later.key() == key {
            return true;
        }
    }
    false
}

fn op_charge(op: &Op) -> Result<Option<u64>, Refusal> {
    match op {
        Op::Put { key, value } => Ok(Some(charge(key, value)?)),
        Op::Erase { .. } => Ok(None),
    }
}

fn charge(key: &[u8], value: &[u8]) -> Result<u64, Refusal> {
    let size = ENTRY.checked_add(skein_lib::len_u64(key.len())).ok_or(Refusal::Full)?;
    size.checked_add(skein_lib::len_u64(value.len())).ok_or(Refusal::Full)
}

impl KvMap {
    pub(crate) fn new(limits: &Limits) -> KvMap {
        let possible = limits.budget / ENTRY;
        let capacity = u32::try_from(possible).unwrap_or(u32::MAX);
        KvMap { entries: Map::with_capacity(capacity), used: 0, live: 0, census: Census::new() }
    }

    #[must_use]
    pub(crate) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.entries.get(key).map(Box::as_ref)
    }

    #[must_use]
    pub(crate) fn used(&self) -> u64 {
        self.used
    }

    /// Each row's key and value and 8 bytes of lengths, as a snapshot's chunks hold them
    /// (skein/docs/design/data/kv.md, 5 and 7).
    #[must_use]
    pub(crate) fn live(&self) -> u64 {
        self.live
    }

    #[must_use]
    pub(crate) fn len(&self) -> u32 {
        self.entries.len()
    }

    pub(crate) fn after<'a>(
        &'a self,
        cursor: Option<&'a [u8]>,
    ) -> impl Iterator<Item = (&'a Box<[u8]>, &'a Box<[u8]>)> {
        let lower = match cursor {
            Some(key) => Bound::Excluded(key),
            None => Bound::Unbounded,
        };
        self.entries.range::<[u8], _>((lower, Bound::Unbounded))
    }

    pub(crate) fn check(&self, ops: &[Op], limits: &Limits) -> Result<u64, Refusal> {
        Queued::new().check(self, ops, limits).map(|projected| projected.used)
    }

    pub(crate) fn apply(&mut self, ops: &[Op], used: u64, rule: Option<&CensusRule>, writes: bool) {
        // Remove first so an atomic batch whose final map fits is never
        // refused by a transient extra entry. Only each key's last op wins.
        for (index, op) in ops.iter().enumerate() {
            if has_later(ops, index, op.key()) {
                continue;
            }
            let old = self.entries.remove(op.key());
            if let Some(rule) = rule {
                self.census.apply(rule, op, old.as_deref(), writes);
            }
            if let Some(old) = old {
                let size = row_bytes(op.key(), &old);
                self.live = self.live.checked_sub(size).expect("the removed row is counted");
            }
        }
        for (index, op) in ops.iter().enumerate() {
            if has_later(ops, index, op.key()) {
                continue;
            }
            if let Op::Put { key, value } = op {
                let old = self.entries.insert(key.clone(), value.clone());
                assert!(old.is_ok(), "a checked commit fits the map");
                self.live = self.live.checked_add(row_bytes(key, value)).expect("a checked map's encoded size fits");
            }
        }
        self.used = used;
    }

    pub(crate) fn page(&self, range: &Range, max: Page) -> Result<PageRows, Refusal> {
        let mut rows = Vec::new();
        let mut bytes: u32 = 0;
        let mut next = None;
        let end = match range.end.as_ref() {
            Some(key) => Bound::Excluded(key.as_ref()),
            None => Bound::Unbounded,
        };
        for (key, value) in self.entries.range::<[u8], _>((Bound::Included(range.start.as_ref()), end)) {
            let size = u32::try_from(key.len().saturating_add(value.len())).unwrap_or(u32::MAX);
            if rows.len() >= usize_of(max.rows) || bytes.checked_add(size).is_none_or(|total| total > max.bytes) {
                if rows.is_empty() {
                    return Err(Refusal::TooLarge);
                }
                next = Some(key.clone());
                break;
            }
            bytes = bytes.checked_add(size).expect("checked row size");
            rows.push(Row { key: key.clone(), value: value.clone() });
        }
        Ok(PageRows { rows: rows.into_boxed_slice(), next })
    }
}

fn row_bytes(key: &[u8], value: &[u8]) -> u64 {
    let size = 8_u64.checked_add(skein_lib::len_u64(key.len())).expect("a checked key's encoded size fits");
    size.checked_add(skein_lib::len_u64(value.len())).expect("a checked row's encoded size fits")
}
