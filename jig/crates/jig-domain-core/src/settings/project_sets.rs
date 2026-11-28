//! Project charter retention (jig/docs/design/domain/engine/project-charters.md, 2–4 and 8).

use crate::{
    Core, CoreKey, CoreRecord, Key, Limits, Now, ProjectCharters, ProjectChartersFault, Record, Request, Write,
};
use alloc::boxed::Box;
use skein_lib::{List, Queue};

type SetKey = (u32, u64);

impl Core {
    /// The project's newest set, or zero where it has not been given one.
    #[must_use]
    pub fn project_charters(&self, project: u32) -> u64 {
        match self.project_charter_sets.range((project, 0)..=(project, u64::MAX)).next_back() {
            Some((key, _)) => key.1,
            None => 0,
        }
    }

    /// A set kept for the project or for its live agent tasks.
    #[must_use]
    pub fn project_charter_set(&self, project: u32, revision: u64) -> Option<&ProjectCharters> {
        self.project_charter_sets.get(&(project, revision))
    }

    pub(crate) fn take_project_charters(&mut self, limits: &Limits, set: ProjectCharters) -> Queue<Request> {
        let project = set.project;
        let revision = set.revision;
        let previous = self.project_charters(project);
        if project == 0 || !self.open_project(project) {
            return refused(project, revision, ProjectChartersFault::Unknown);
        }
        if revision <= previous {
            return refused(project, revision, ProjectChartersFault::Stale);
        }
        if let Err(fault) = set.check(limits) {
            return refused(project, revision, fault);
        }
        let mut out = Queue::uncapped();
        // Free an unheld newest slot before inserting its replacement. If it
        // has a holder, the new row takes the project's slot, not the task's.
        if previous != 0 && !self.tasks.holds_project_charters(project, previous) {
            self.erase_project_charters((project, previous), &mut out);
        }
        self.tasks.set_project_charters(project, revision);
        out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Settings(super::Record::ProjectCharters(
            set.clone(),
        ))))));
        assert!(self.project_charter_sets.insert((project, revision), set) == Ok(None), "project and task slots");
        out
    }

    pub(crate) fn restore_project_charters(&mut self, limits: &Limits, set: ProjectCharters) -> bool {
        let project = set.project;
        let revision = set.revision;
        let key = (project, revision);
        if project == 0 || revision == 0 || set.check(limits).is_err() {
            let _refused = self.start_refuse(crate::start::Failure::Accounting { key: record_key(key) });
            return false;
        }
        // The project policies and task holders have not been read yet. Publish
        // the newest stamp only once those rows have established membership.
        assert!(!self.project_charter_sets.contains_key(&key), "duplicate restored charter set");
        if self.project_charter_sets.insert(key, set).is_err() {
            let limit = self.project_charter_sets.capacity();
            let _refused = self.start_refuse(crate::start::Failure::LiveSet {
                set: crate::start::LiveSet::ProjectCharters,
                key: record_key(key),
                held: u64::from(limit).checked_add(1).expect("row beyond capacity"),
                limit: u64::from(limit),
            });
            return false;
        }
        true
    }

    pub(crate) fn restore_project_charter_stamps(&mut self) {
        for &project in &self.projects {
            let revision = self.project_charters(project);
            if revision != 0 {
                self.tasks.set_project_charters(project, revision);
            }
        }
    }

    pub(crate) fn release_project_charters(&mut self, project: u32, revision: u64, out: &mut Queue<Request>) {
        if revision != self.project_charters(project) && !self.tasks.holds_project_charters(project, revision) {
            self.erase_project_charters((project, revision), out);
        }
    }

    pub(crate) fn prune_project_charters(&mut self, out: &mut Queue<Request>) {
        let mut unused = List::with_capacity(self.project_charter_sets.len());
        for (&(project, revision), _) in &self.project_charter_sets {
            let newest = self.authority.policy(project).is_some() && revision == self.project_charters(project);
            if !newest && !self.tasks.holds_project_charters(project, revision) {
                unused.push((project, revision)).expect("subset of restored sets");
            }
        }
        for &key in &unused {
            self.erase_project_charters(key, out);
        }
    }

    pub(crate) fn erase_project_charter_sets(&mut self, project: u32, out: &mut Queue<Request>) {
        self.tasks.set_project_charters(project, 0);
        let range = (project, 0)..=(project, u64::MAX);
        for _ in 0..self.project_charter_sets.len() {
            let Some((&key, _)) = self.project_charter_sets.range(range.clone()).next() else { break };
            assert!(!self.tasks.holds_project_charters(key.0, key.1), "project's tasks ended before its rows erase");
            self.erase_project_charters(key, out);
        }
    }

    fn erase_project_charters(&mut self, key: SetKey, out: &mut Queue<Request>) {
        if self.project_charter_sets.remove(&key).is_some() {
            out.push(Request::Write(Write::Erase(record_key(key))));
        }
    }
}

fn record_key((project, revision): SetKey) -> Key {
    Key::Core(CoreKey::Settings(super::Key::ProjectCharters { project, revision }))
}

pub(crate) fn refused(project: u32, revision: u64, fault: ProjectChartersFault) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(Now::ProjectChartersRefused { project, revision, fault })));
    out
}
