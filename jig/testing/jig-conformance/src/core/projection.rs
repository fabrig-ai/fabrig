//! Keyed accounting of a commit's rows (`jig/docs/design/domain/testing.md`, 10).
use super::{Commit, Write, core, r};
use std::collections::{BTreeMap, BTreeSet};
mod joins;

/// Current per-row evidence and the submitted keys that can change it.
/// Observations remain whole snapshots, emitted once per commit. Cold rows and
/// bulk operations are projected whole; neither is inferred from old submissions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Projection {
    applied: Option<u64>,
    pending: BTreeMap<u64, Option<BTreeSet<core::Key>>>,
    rows: BTreeMap<core::Key, Row>,
    joins: joins::Joins,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    naming: Vec<r::Naming>,
    receipts: BTreeSet<r::Receipt>,
    facts: Box<[Fact]>,
}

/// Retain only facts the row contributes, rather than an empty collection for
/// every other kind of fact in a whole committed snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Fact {
    Newest(u32),
    Task(u64, Box<r::Task>),
    PlacedHold(u64, r::PlacedHold),
    ReadPosition((u64, u64), u64),
    Writer(r::Name, u64, u64),
    Pool(r::Name, u32),
    PartyProposal(u64, u64),
    Batch(Vec<u64>),
    Balance((u64, u64, u64, u64)),
    Deleting(u32),
    PersonSpend(u64, u64),
}

impl Fact {
    fn append(&self, snapshot: &mut r::Snapshot) {
        match self {
            Self::Newest(value) => snapshot.newest = snapshot.newest.max(*value),
            Self::Task(key, value) => {
                snapshot.tasks.insert(*key, (**value).clone());
            }
            Self::PlacedHold(key, value) => {
                snapshot.placed_holds.insert(*key, *value);
            }
            Self::ReadPosition(key, value) => {
                snapshot.read_positions.insert(*key, *value);
            }
            Self::Writer(name, task, attempt) => snapshot.writers.push((name.clone(), *task, *attempt)),
            Self::Pool(key, value) => {
                snapshot.pools.insert(key.clone(), *value);
            }
            Self::PartyProposal(key, value) => {
                snapshot.party_proposals.insert(*key, *value);
            }
            Self::Batch(value) => snapshot.batches.push(value.clone()),
            Self::Balance(value) => snapshot.balances.push(*value),
            Self::Deleting(value) => {
                snapshot.deleting.insert(*value);
            }
            Self::PersonSpend(person, spent) => *snapshot.person_spend.entry(*person).or_default() += spent,
        }
    }
}

impl Row {
    fn project(key: &core::Key, record: &core::Record) -> Option<Self> {
        let snapshot = super::snapshot::project(&BTreeMap::from([(key.clone(), record)]));
        let mut facts = Vec::new();
        if snapshot.newest != 0 {
            facts.push(Fact::Newest(snapshot.newest));
        }
        facts.extend(snapshot.tasks.into_iter().map(|(key, value)| Fact::Task(key, Box::new(value))));
        facts.extend(snapshot.placed_holds.into_iter().map(|(key, value)| Fact::PlacedHold(key, value)));
        facts.extend(snapshot.read_positions.into_iter().map(|(key, value)| Fact::ReadPosition(key, value)));
        facts.extend(snapshot.writers.into_iter().map(|(name, task, attempt)| Fact::Writer(name, task, attempt)));
        facts.extend(snapshot.pools.into_iter().map(|(key, value)| Fact::Pool(key, value)));
        facts.extend(snapshot.party_proposals.into_iter().map(|(key, value)| Fact::PartyProposal(key, value)));
        facts.extend(snapshot.batches.into_iter().map(Fact::Batch));
        facts.extend(snapshot.balances.into_iter().map(Fact::Balance));
        facts.extend(snapshot.deleting.into_iter().map(Fact::Deleting));
        facts.extend(snapshot.person_spend.into_iter().map(|(person, spent)| Fact::PersonSpend(person, spent)));
        // Acceptances and projection ownership are resolved against all current
        // rows, after keyed replacements and erasures have completed.
        (!snapshot.naming.is_empty() || !snapshot.receipts.is_empty() || !facts.is_empty()).then_some(Self {
            naming: snapshot.naming,
            receipts: snapshot.receipts,
            facts: facts.into_boxed_slice(),
        })
    }

    fn append(&self, snapshot: &mut r::Snapshot) {
        snapshot.naming.extend(self.naming.iter().cloned());
        snapshot.receipts.extend(self.receipts.iter().cloned());
        for fact in &self.facts {
            fact.append(snapshot);
        }
    }
}

impl Projection {
    /// Retain keys, not submitted values: the projection reads what became durable.
    pub fn submitted(&mut self, commit: &Commit<'_>) {
        let mut keys = BTreeSet::new();
        for write in &commit.writes {
            match write {
                Write::Save(row) => {
                    keys.insert(row.key());
                }
                Write::Erase(key) => {
                    keys.insert(key.clone());
                }
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
        self.snapshot_by(applied, records.iter().map(|(key, row)| (key.clone(), *row)), |key| records.get(key).copied())
    }

    /// Borrow unchanged storage directly. The whole iterator is consumed only
    /// for a cold rebuild; a complete commit feed needs just keyed lookups.
    #[must_use]
    pub fn snapshot_by<'a>(
        &mut self,
        applied: u64,
        records: impl IntoIterator<Item = (core::Key, &'a core::Record)>,
        get: impl Fn(&core::Key) -> Option<&'a core::Record>,
    ) -> r::Snapshot {
        if self.applied != Some(applied) {
            let incremental = self.applied.is_some_and(|previous| {
                previous < applied
                    && (previous + 1..=applied).all(|number| self.pending.get(&number).is_some_and(Option::is_some))
            });
            if incremental {
                let previous = self.applied.expect("incremental projection has a base");
                for number in previous + 1..=applied {
                    for key in self.pending.remove(&number).flatten().expect("complete commit feed") {
                        let row = get(&key);
                        self.replace(key, row);
                    }
                }
            } else {
                self.rows.clear();
                self.joins = joins::Joins::default();
                for (key, row) in records {
                    self.replace(key, Some(row));
                }
            }
            self.pending.retain(|number, _| *number > applied);
            self.applied = Some(applied);
        }
        let mut snapshot = r::Snapshot::default();
        for row in self.rows.values() {
            row.append(&mut snapshot);
        }
        for task in snapshot.tasks.values_mut() {
            task.tracked |= self
                .rows
                .contains_key(&core::Key::Core(core::CoreKey::Projection(core::ProjectionKey::Goal(task.number))));
        }
        self.joins.resolve(&mut snapshot);
        snapshot
    }

    fn replace(&mut self, key: core::Key, record: Option<&core::Record>) {
        self.joins.replace(&key, record);
        self.rows.remove(&key);
        if let Some(record) = record
            && let Some(row) = Row::project(&key, record)
        {
            self.rows.insert(key, row);
        }
    }

    /// Current accepted decisions, in durable key order, for connector attribution.
    #[must_use]
    pub fn proposal_decisions(&self) -> Vec<core::ProposalDecisionRecord> {
        self.joins.decisions()
    }
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
        let sign_in =
            core::Record::People(people::Stored::SignIn { number: 5, person: 2, expires: skein_lib::Wall::EPOCH });
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

    fn person_amendment(task: u64) -> core::Record {
        core::Record::Tasks(tasks::Stored::DecidedProposal(Box::new(tasks::PersonProposal {
            number: 9,
            proposer: 1,
            project: 1,
            action: tasks::PersonAction::Amend {
                task,
                revision: 0,
                amendment: Box::new(tasks::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    release: None,
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: None,
                    llm: None,
                    reason: Box::new([]),
                }),
            },
            reason: Box::new([]),
            state: tasks::PersonProposalState::Accepted { by: tasks::Party::Person(2) },
        })))
    }

    #[test]
    fn keyed_joins_follow_replacement_and_erasure_of_the_other_row() {
        let decision = core::Record::Core(core::CoreRecord::ProposalDecision(core::ProposalDecisionRecord {
            project: 1,
            proposer: people::Proposer::Party(1),
            proposal: 9,
            kind: tasks::ProposalKind::Amend,
            by: 2,
            at: skein_lib::Wall::from_nanos(1),
            choice: people::ProposalChoice::Accepted,
            created: None,
        }));
        let mut rows = BTreeMap::from([(decision.key(), decision)]);
        let mut projection = Projection::default();
        assert!(check(&mut projection, 1, &rows).accepted_tasks.is_empty());
        let proposal = person_amendment(5);
        projection.submitted(&Commit { number: 2, writes: vec![Write::Save(&proposal)] });
        rows.insert(proposal.key(), proposal);
        assert_eq!(
            check(&mut projection, 2, &rows).accepted_tasks.get(&5),
            Some(&r::Acceptance { proposal: 9, by: r::Source::Person(2) })
        );
        let replacement = person_amendment(6);
        projection.submitted(&Commit { number: 3, writes: vec![Write::Save(&replacement)] });
        rows.insert(replacement.key(), replacement.clone());
        let changed = check(&mut projection, 3, &rows);
        assert!(!changed.accepted_tasks.contains_key(&5));
        assert!(changed.accepted_tasks.contains_key(&6));
        projection.submitted(&Commit { number: 4, writes: vec![Write::Erase(replacement.key())] });
        rows.remove(&replacement.key());
        assert!(check(&mut projection, 4, &rows).accepted_tasks.is_empty());
    }
}
