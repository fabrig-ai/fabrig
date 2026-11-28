//! Whole-task transcript drops at the fake store's durable boundary
//! (jig/docs/design/protocol/store/growth.md, 1.1; jig/docs/design/protocol/store/transcripts.md, 5, jig/docs/design/protocol/store/growth.md, 1.1).
use super::{core, root};
use jig_conformance::Durable;
use jig_domain_core::engine::store as engine;
use jig_domain_tasks as tasks;
use skein_lib::Token;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

type Rows = jig_fake_store::Store<root::Key, root::Record>;

/// Ops's typed transcript erasures around the opaque row store.
#[derive(Clone, Debug, Default)]
pub struct Store {
    inner: Rows,
    drops: BTreeMap<u64, Vec<u64>>,
}

impl Deref for Store {
    type Target = Rows;
    fn deref(&self) -> &Rows {
        &self.inner
    }
}

impl DerefMut for Store {
    fn deref_mut(&mut self) -> &mut Rows {
        &mut self.inner
    }
}

impl Durable for Store {
    type Write = root::Write;

    fn submit(&mut self, number: u64, writes: Box<[root::Write]>, delay: u32) {
        let mut rows = Vec::new();
        let mut drops = Vec::new();
        for write in writes {
            match write {
                root::Write::DropTranscript { task } => drops.push(task),
                root::Write::Save(row) => rows.push(jig_fake_store::Write::Save { key: row.key(), row }),
                root::Write::Erase(key) => rows.push(jig_fake_store::Write::Erase(key)),
                root::Write::DropHistory { .. } | root::Write::Wires => {}
                root::Write::CopySettingsRevision(_) => {
                    unreachable!("this scripted ops world restarts with identical defaults");
                }
            }
        }
        if !drops.is_empty() {
            self.drops.insert(number, drops);
        }
        self.inner.submit(number, rows.into_boxed_slice(), delay);
    }

    fn tick(&mut self, hold: bool) -> Result<Option<u64>, u64> {
        let before = self.inner.applied;
        let result = self.inner.tick(hold);
        if self.inner.applied != before
            && let Some(tasks) = self.drops.remove(&self.inner.applied)
        {
            self.inner.rows.retain(|key, _| {
                if let root::Key::Core(core::Key::Core(core::CoreKey::Turn { task, .. })) = key {
                    !tasks.contains(task)
                } else {
                    true
                }
            });
        }
        if result.is_err() {
            self.drops.clear();
        }
        result
    }

    fn release_held(&mut self) -> Option<u64> {
        self.inner.release_held()
    }

    fn crash(&mut self, _: u64) -> Result<(), String> {
        self.inner.crash();
        self.drops.clear();
        Ok(())
    }

    fn fork(&self) -> Result<Self, String> {
        Ok(self.clone())
    }

    fn applied(&self) -> u64 {
        self.inner.applied
    }

    fn idle(&self) -> bool {
        self.inner.pending.is_empty() && !self.inner.loading()
    }

    fn holding(&self) -> bool {
        !self.inner.held.is_empty()
    }
}

/// Each in-memory walk fits in one page. Intervals keep the engine's order,
/// and rows within an interval keep the store's order.
pub(crate) fn start_page(
    rows: &BTreeMap<root::Key, root::Record>,
    load: Token,
    ranges: &[core::RestoreRange],
    after: Option<&engine::Cursor>,
) -> root::Event {
    assert!(after.is_none(), "the in-memory store has no later page");
    let mut items = Vec::new();
    for range in ranges {
        let admitted = rows.iter().filter_map(|(key, row)| match (key, row) {
            (root::Key::Core(key), root::Record::Core(row)) if range.admits(key) => Some((key, row)),
            _ => None,
        });
        if let core::RestoreRange::Census(census) = range {
            let mut ordered: Vec<_> = admitted.collect();
            ordered.sort_by_key(|(key, _)| census.interval(key));
            for (key, row) in ordered {
                census_owner(rows, key, row, &mut items);
            }
        } else {
            items.extend(admitted.map(|(_, row)| engine::Item::Record(row.clone())));
        }
    }
    loaded(load, engine::Answer::Rows(engine::Rows::Start { items: items.into_boxed_slice(), next: None }))
}

fn census_owner(
    rows: &BTreeMap<root::Key, root::Record>,
    key: &core::Key,
    row: &core::Record,
    items: &mut Vec<engine::Item>,
) {
    let task = match row {
        core::Record::Tasks(tasks::Stored::Result(row)) => row.task,
        core::Record::People(jig_domain_people::Stored::EndedRead { task, .. }) => {
            items.push(engine::Item::Record(row.clone()));
            *task
        }
        _ => return,
    };
    let target = core::Key::Tasks(tasks::Key::Ended(task));
    items.push(match rows.get(&root::Key::Core(target.clone())) {
        Some(root::Record::Core(owner)) => engine::Item::Record(owner.clone()),
        _ => engine::Item::Missing { record: key.clone(), target },
    });
}

pub(crate) fn connector_rows(rows: &BTreeMap<root::Key, root::Record>, range: &root::Range) -> Vec<root::Event> {
    let mut events = Vec::new();
    for row in rows.values() {
        match (range, row) {
            (root::Range::Infrastructure, root::Record::Infrastructure(record)) => {
                events.push(root::Event::Infrastructure(super::infra::Event::Restore { record: record.clone() }));
            }
            (root::Range::Observability, root::Record::Observability(record)) => {
                events.push(root::Event::Observability(super::obs::Event::Restore { record: record.clone() }));
            }
            _ => {}
        }
    }
    events.push(match range {
        root::Range::Infrastructure => root::Event::Infrastructure(super::infra::Event::RestoreDone),
        root::Range::Observability => root::Event::Observability(super::obs::Event::RestoreDone),
    });
    events
}

pub(crate) fn loaded(load: Token, result: engine::Answer) -> root::Event {
    root::Event::Store(root::Store::Engine(engine::In::Loaded { load, result }))
}

pub(crate) fn switched() -> root::Event {
    root::Event::Store(root::Store::Engine(engine::In::Switched))
}

pub(crate) fn account_rows(providers: &jig_domain_accounts::Providers) -> BTreeMap<root::Key, root::Record> {
    jig_test_profile::fixtures::account_rows(providers)
        .into_iter()
        .map(|row| {
            let jig_test_domain::Record::Core(row) = row else { panic!("account fixture row") };
            (root::Key::Core(row.key()), root::Record::Core(row))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_fake_store::Fault;
    use skein_lib::Wall;

    fn turn(task: u64, attempt: u64) -> root::Record {
        root::Record::Core(core::Record::Core(core::CoreRecord::Turn(core::TurnRecord {
            task,
            attempt,
            turn: 1,
            opens_window: true,
            spent: 0,
            read: None,
            at: Wall::EPOCH,
            transcript: Box::new([1]),
        })))
    }

    fn seeded() -> Store {
        let mut store = Store::default();
        for row in [turn(7, 1), turn(7, 2), turn(8, 1)] {
            store.inner.rows.insert(row.key(), row);
        }
        store
    }

    #[test]
    fn a_durable_drop_erases_every_attempt_and_preserves_other_tasks() {
        let mut store = seeded();
        for number in [1, 2] {
            Durable::submit(&mut store, number, Box::new([root::Write::DropTranscript { task: 7 }]), 0);
            assert_eq!(Durable::tick(&mut store, false), Ok(Some(number)));
            assert_eq!(store.inner.rows.values().cloned().collect::<Vec<_>>(), [turn(8, 1)]);
        }
    }

    #[test]
    fn a_drop_is_durable_while_its_acknowledgement_is_held() {
        let mut store = seeded();
        store.fault(Fault::Hold { commits: 1 });
        Durable::submit(&mut store, 1, Box::new([root::Write::DropTranscript { task: 7 }]), 0);
        assert_eq!(Durable::tick(&mut store, false), Ok(None));
        assert_eq!(store.inner.rows.values().cloned().collect::<Vec<_>>(), [turn(8, 1)]);
        assert_eq!(Durable::release_held(&mut store), Some(1));
    }

    #[test]
    fn a_failed_drop_leaves_the_rows_and_a_crash_discards_it() {
        let mut store = seeded();
        let rows = store.inner.rows.clone();
        store.fault(Fault::Fail { commit: 1 });
        Durable::submit(&mut store, 1, Box::new([root::Write::DropTranscript { task: 7 }]), 0);
        assert_eq!(Durable::tick(&mut store, false), Err(1));
        assert_eq!(store.inner.rows, rows);
        Durable::crash(&mut store, 0).expect("crash readback");
        Durable::submit(&mut store, 1, Box::new([]), 0);
        assert_eq!(Durable::tick(&mut store, false), Ok(Some(1)));
        assert_eq!(store.inner.rows, rows);
    }

    #[test]
    fn a_drop_waits_for_earlier_turns_and_is_retained_by_a_fork() {
        let mut store = Store::default();
        Durable::submit(&mut store, 1, Box::new([root::Write::Save(turn(7, 1))]), 0);
        Durable::submit(&mut store, 2, Box::new([root::Write::DropTranscript { task: 7 }]), 0);
        let mut fork = Durable::fork(&store).expect("fork readback");
        for current in [&mut store, &mut fork] {
            assert_eq!(Durable::tick(current, false), Ok(Some(1)));
            assert_eq!(current.inner.rows.values().cloned().collect::<Vec<_>>(), [turn(7, 1)]);
            assert_eq!(Durable::tick(current, false), Ok(Some(2)));
            assert!(current.inner.rows.is_empty());
        }
    }
}
