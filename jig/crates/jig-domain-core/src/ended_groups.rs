//! Memory-only ended-group positions (jig/docs/design/domain/engine/views.md, 2).
//! At most `limits.tasks.tasks` keys survive; eviction advances every key's floor.

use crate::{
    Core,
    documents::TaskQuery,
    reads::ended::{EndedPart, ended_part},
};
use jig_domain_tasks as tasks;
use skein_lib::{List, Map};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Key {
    Party(u64, Option<u32>),
    Delegates(u64),
    Project(u32, Option<u32>),
    Parts,
}

#[derive(Hash, Debug)]
pub(crate) struct Marks {
    entries: Map<Key, u64>,
    floor: u64,
    pub(crate) changed: bool,
}

impl Marks {
    pub(crate) fn new(bound: u32) -> Self {
        Self { entries: Map::with_capacity(bound), floor: 1, changed: false }
    }

    pub(crate) fn restart(&mut self, commits: u64) {
        self.floor = commits.checked_add(1).expect("restart position admitted");
    }

    fn get(&self, key: Key) -> u64 {
        self.entries.get(&key).copied().unwrap_or(self.floor).max(self.floor)
    }

    pub(crate) fn query(&self, query: &TaskQuery) -> u64 {
        let key = match ended_part(query) {
            EndedPart::None => return 0,
            EndedPart::ByParty { party, label } => Key::Party(party, label),
            EndedPart::Delegates { requester } => Key::Delegates(requester),
            EndedPart::ByProject { project, label } => Key::Project(project, label),
        };
        self.get(key).max(self.get(Key::Parts))
    }

    fn put(&mut self, key: Key, position: u64) {
        self.changed = true;
        if self.entries.contains_key(&key) {
            let old = self.entries.get_mut(&key).expect("present mark");
            *old = (*old).max(position);
            return;
        }
        if self.entries.len() == self.entries.capacity() {
            let mut oldest = None;
            for (key, value) in &self.entries {
                if match oldest {
                    Some((_, old)) => *value < old,
                    None => true,
                } {
                    oldest = Some((*key, *value));
                }
            }
            if let Some((key, value)) = oldest {
                let _removed = self.entries.remove(&key);
                self.floor = self.floor.max(value);
            } else {
                self.floor = self.floor.max(position);
                return;
            }
        }
        self.entries.insert(key, position).expect("evicted mark room");
    }
}

impl Core {
    fn ended_key_changed(&mut self, key: Key) {
        let mut before = List::with_capacity(self.view_names.len());
        for (name, watched) in &self.view_names {
            if let crate::documents::Subject::Query { query } = &watched.subject {
                before.push((name, self.ended_groups.query(query))).expect("one mark per subject");
            }
        }
        let position = self.counters.deployment().commits.checked_add(1).expect("commit position admitted");
        self.ended_groups.put(key, position);
        for (name, old) in before.into_boxed() {
            let crate::documents::Subject::Query { query } = &self.view_names.get(name).expect("watched query").subject
            else {
                unreachable!("query mark")
            };
            if self.ended_groups.query(query) > old {
                let _new = self.changed_views.insert(name).expect("bounded watched marks");
            }
        }
    }

    pub(crate) fn ended_parts_changed(&mut self) {
        self.ended_key_changed(Key::Parts);
    }

    pub(crate) fn ended_index_changed(&mut self, requester: tasks::Party, project: u32, label: u32, listed: bool) {
        match requester {
            tasks::Party::Person(party) => {
                self.ended_key_changed(Key::Party(party, Some(label)));
                self.ended_key_changed(Key::Party(party, None));
            }
            tasks::Party::Task(task) => self.ended_key_changed(Key::Delegates(task)),
            tasks::Party::Deployment { .. } => {}
        }
        if listed {
            self.ended_key_changed(Key::Project(project, Some(label)));
            self.ended_key_changed(Key::Project(project, None));
        }
    }
}
