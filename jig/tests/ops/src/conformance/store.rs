//! Ops's whole-transcript mutation at its fake store boundary (protocol/store.md, 8.5).
use jig_conformance::Durable;
use jig_domain_core as core;
use jig_ops_domain as root;
use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

type Rows = jig_fake_store::Store<root::Key, root::Record>;
type Write = jig_fake_store::Write<root::Key, root::Record>;

#[derive(Clone, Debug, Default)]
pub struct Store(Rows);

impl Store {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn drop_keys(&self, task: u64, translated: &[Write]) -> BTreeSet<root::Key> {
        let mut keys = BTreeSet::new();
        for key in self.0.rows.keys() {
            if matches!(key, root::Key::Core(core::Key::Core(core::CoreKey::Turn { task: found, .. })) if *found == task)
            {
                keys.insert(key.clone());
            }
        }
        for write in self.0.pending.iter().flat_map(|pending| pending.writes.iter()).chain(translated) {
            if let Write::Save { key, .. } = write
                && matches!(key, root::Key::Core(core::Key::Core(core::CoreKey::Turn { task: found, .. })) if *found == task)
            {
                keys.insert(key.clone());
            }
        }
        keys
    }
}

impl Deref for Store {
    type Target = Rows;
    fn deref(&self) -> &Rows {
        &self.0
    }
}
impl DerefMut for Store {
    fn deref_mut(&mut self) -> &mut Rows {
        &mut self.0
    }
}

impl Durable for Store {
    type Write = root::Write;
    fn submit(&mut self, number: u64, writes: Box<[root::Write]>, delay: u32) {
        let mut translated = Vec::new();
        for write in writes {
            match write {
                root::Write::Wires => {}
                root::Write::CopySettingsRevision(_) => unreachable!("scripted ops worlds keep identical defaults"),
                root::Write::Save(row) => translated.push(Write::Save { key: row.key(), row }),
                root::Write::Erase(key) => translated.push(Write::Erase(key)),
                root::Write::DropTranscript(task) => {
                    for key in self.drop_keys(task, &translated) {
                        translated.push(Write::Erase(key));
                    }
                }
            }
        }
        self.0.submit(number, translated.into_boxed_slice(), delay);
    }
    fn tick(&mut self, hold: bool) -> Result<Option<u64>, u64> {
        self.0.tick(hold)
    }
    fn release_held(&mut self) -> Option<u64> {
        self.0.release_held()
    }
    fn crash(&mut self, _: u64) {
        self.0.crash();
    }
    fn fork(&self) -> Self {
        self.clone()
    }
    fn applied(&self) -> u64 {
        self.0.applied
    }
    fn idle(&self) -> bool {
        self.0.pending.is_empty() && !self.0.loading()
    }
    fn holding(&self) -> bool {
        !self.0.held.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(task: u64, turn: u32) -> root::Write {
        root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Turn(core::TurnRecord {
            opens_window: true,
            task,
            attempt: 1,
            turn,
            spent: 0,
            read: None,
            at: skein_lib::Wall::from_nanos(0),
            transcript: Box::new([]),
        }))))
    }

    #[test]
    fn a_transcript_drop_erases_turns_in_earlier_pending_commits_too() {
        const TASK: u64 = 8;
        const NEIGHBOUR: u64 = 9;
        let mut store = Store::new();
        Durable::submit(&mut store, 1, Box::new([turn(TASK, 1), turn(NEIGHBOUR, 1)]), 0);
        Durable::submit(&mut store, 2, Box::new([turn(TASK, 2), root::Write::DropTranscript(TASK)]), 0);
        assert_eq!(Durable::tick(&mut store, false), Ok(Some(1)));
        assert_eq!(Durable::tick(&mut store, false), Ok(Some(2)));
        let key = root::Key::Core(core::Key::Core(core::CoreKey::Turn { task: NEIGHBOUR, attempt: 1, turn: 1 }));
        assert_eq!(store.rows.keys().cloned().collect::<Vec<_>>(), vec![key]);
    }
}
