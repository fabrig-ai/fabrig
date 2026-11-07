//! Keyed accounting of a commit's rows (`jig/docs/design/domain/testing.md`, 10).
use super::{Commit, Write, core, r};
use std::collections::{BTreeMap, BTreeSet};

/// Current per-row evidence and the submitted keys that can change it.
/// Observations remain whole snapshots, emitted once per commit. Cold rows and
/// bulk operations are projected whole; neither is inferred from old submissions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Projection {
    applied: Option<u64>,
    pending: BTreeMap<u64, Option<BTreeSet<core::Key>>>,
    rows: BTreeMap<core::Key, Row>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_domain_people as people;
    use jig_domain_tasks as tasks;

    fn ledger(period: u64, spent: u64) -> core::Record {
        core::Record::Tasks(tasks::Stored::Ledger(tasks::FundingRecord {
            period: None,
            funder: tasks::Funder::Pool { project: 1, person: 2, period },
            parent: None,
            numbers: tasks::Numbers { budget: 100, spent, spent_below: 0, reserved: 0 },
            made: 0,
            closed: false,
        }))
    }

    fn check(projection: &mut Projection, applied: u64, rows: &BTreeMap<core::Key, core::Record>) -> r::Snapshot {
        let rows = rows.iter().map(|(key, row)| (key.clone(), row)).collect();
        let snapshot = projection.snapshot(applied, &rows);
        assert_eq!(snapshot, super::super::snapshot(&rows), "commit {applied}");
        snapshot
    }

    #[test]
    fn keyed_rows_replace_erase_and_rebuild_without_double_charging_spend() {
        let first = ledger(3, 7);
        let second = ledger(4, 11);
        let sign_in = core::Record::People(people::Stored::SignIn { number: 5, person: 2, expires: skein_lib::Wall::EPOCH });
        let mut rows = BTreeMap::from([(first.key(), first.clone()), (second.key(), second), (sign_in.key(), sign_in)]);
        let mut projection = Projection::default();
        assert_eq!(check(&mut projection, 1, &rows).person_spend.get(&2), Some(&18));
        let replacement = ledger(3, 13);
        projection.submitted(&Commit { number: 2, writes: vec![Write::Save(&replacement)] });
        rows.insert(replacement.key(), replacement);
        assert_eq!(check(&mut projection, 2, &rows).person_spend.get(&2), Some(&24));
        projection.submitted(&Commit { number: 3, writes: vec![Write::Erase(first.key())] });
        rows.remove(&first.key());
        assert_eq!(check(&mut projection, 3, &rows).person_spend.get(&2), Some(&11));
        // A restore's Got can change rows without changing applied.
        rows.clear();
        projection.invalidate();
        check(&mut projection, 3, &rows);
        rows.insert(first.key(), first.clone());
        projection.submitted(&Commit { number: 4, writes: vec![Write::Refresh] });
        check(&mut projection, 4, &rows);
        // An unobserved/failed submission must not poison a reused commit number.
        projection.submitted(&Commit { number: 5, writes: vec![Write::Erase(first.key())] });
        projection.invalidate();
        check(&mut projection, 4, &rows);
        projection.submitted(&Commit { number: 5, writes: vec![] });
        check(&mut projection, 5, &rows);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    naming: Vec<r::Naming>,
    receipts: BTreeSet<r::Receipt>,
    facts: Option<r::Snapshot>,
}

impl Row {
    fn project(key: &core::Key, record: &core::Record) -> Option<Self> {
        let mut facts = super::snapshot(&BTreeMap::from([(key.clone(), record)]));
        let naming = std::mem::take(&mut facts.naming);
        let receipts = std::mem::take(&mut facts.receipts);
        // Acceptances and projection ownership are resolved against all current
        // rows, after keyed replacements and erasures have completed.
        facts.accepted_tasks.clear();
        let facts = (facts != r::Snapshot::default()).then_some(facts);
        (!naming.is_empty() || !receipts.is_empty() || facts.is_some()).then_some(Self { naming, receipts, facts })
    }

    fn append(&self, snapshot: &mut r::Snapshot) {
        snapshot.naming.extend(self.naming.iter().cloned());
        snapshot.receipts.extend(self.receipts.iter().cloned());
        let Some(facts) = &self.facts else { return };
        snapshot.newest = snapshot.newest.max(facts.newest);
        snapshot.tasks.extend(facts.tasks.iter().map(|(key, value)| (*key, value.clone())));
        snapshot.placed_holds.extend(facts.placed_holds.iter().map(|(key, value)| (*key, value.clone())));
        snapshot.read_positions.extend(facts.read_positions.iter().map(|(key, value)| (*key, *value)));
        snapshot.writers.extend(facts.writers.iter().cloned());
        snapshot.pools.extend(facts.pools.iter().map(|(key, value)| (key.clone(), *value)));
        snapshot.party_proposals.extend(facts.party_proposals.iter().map(|(key, value)| (*key, *value)));
        snapshot.batches.extend(facts.batches.iter().cloned());
        snapshot.balances.extend(facts.balances.iter().copied());
        snapshot.deleting.extend(facts.deleting.iter().copied());
        for (person, spent) in &facts.person_spend {
            *snapshot.person_spend.entry(*person).or_default() += spent;
        }
    }
}

impl Projection {
    /// Retain keys, not submitted values: the projection reads what became durable.
    pub fn submitted(&mut self, commit: &Commit<'_>) {
        let mut keys = BTreeSet::new();
        for write in &commit.writes {
            match write {
                Write::Save(row) => { keys.insert(row.key()); }
                Write::Erase(key) => { keys.insert(key.clone()); }
                Write::Other => {}
                Write::Refresh => {
                    self.pending.insert(commit.number, None);
                    return;
                }
            }
        }
        self.pending.insert(commit.number, Some(keys));
    }

    /// Restore can supply rows outside commits, even without advancing applied.
    pub fn invalidate(&mut self) {
        *self = Self::default();
    }

    /// Assemble the same ordered snapshot as whole-store projection. Each row
    /// retains its own receipts and spend, so erasing one contributor cannot
    /// erase another contributor's receipt or leave its spend charged twice.
    #[must_use]
    pub fn snapshot(&mut self, applied: u64, records: &BTreeMap<core::Key, &core::Record>) -> r::Snapshot {
        if self.applied != Some(applied) {
            let incremental = self.applied.is_some_and(|previous| {
                previous < applied && (previous + 1..=applied)
                    .all(|number| self.pending.get(&number).is_some_and(Option::is_some))
            });
            if incremental {
                let previous = self.applied.expect("incremental projection has a base");
                for number in previous + 1..=applied {
                    for key in self.pending.remove(&number).flatten().expect("complete commit feed") {
                        self.rows.remove(&key);
                        if let Some(record) = records.get(&key)
                            && let Some(row) = Row::project(&key, record)
                        {
                            self.rows.insert(key, row);
                        }
                    }
                }
            } else {
                self.rows = records.iter().filter_map(|(key, record)| {
                    Row::project(key, record).map(|row| (key.clone(), row))
                }).collect();
            }
            self.pending.retain(|number, _| *number > applied);
            self.applied = Some(applied);
        }
        let mut snapshot = r::Snapshot::default();
        for row in self.rows.values() {
            row.append(&mut snapshot);
        }
        super::snapshot::resolve(&mut snapshot, records);
        snapshot
    }
}
