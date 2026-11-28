//! The set stamped at task creation, and the live agents that hold it
//! (jig/docs/design/domain/engine/project-charters.md, 3 and 4).

use crate::{Domain, Executor, Request, TaskRecord};
use skein_lib::{Map, Queue};

impl Domain {
    /// Size the parent-supplied newest-set index at construction.
    #[must_use]
    pub fn with_project_charter_capacity(mut self, projects: u32) -> Self {
        assert!(self.project_charters.is_empty(), "set the project bound before supplying sets");
        self.project_charters = Map::with_capacity(projects);
        self
    }

    /// Install the parent's newest revision, or forget an erased project.
    pub fn set_project_charters(&mut self, project: u32, revision: u64) {
        assert!(project != 0, "a set belongs to a project");
        if revision == 0 {
            self.project_charters.remove(&project);
        } else {
            let _previous = self.project_charters.insert(project, revision).expect("bounded project membership");
        }
    }

    pub(crate) fn newest_project_charters(&self, project: u32) -> u64 {
        self.project_charters.get(&project).copied().unwrap_or(0)
    }

    /// Whether an agent task still records the named revision.
    #[must_use]
    pub fn holds_project_charters(&self, project: u32, revision: u64) -> bool {
        self.project_charter_holders.contains_key(&(project, revision))
    }
}

fn held_by(task: &TaskRecord) -> Option<(u32, u64)> {
    if task.charters == 0 {
        return None;
    }
    match task.executor {
        Executor::Agent { .. } => Some((task.project, task.charters)),
        Executor::Person(_) | Executor::Procedure { .. } | Executor::Job { .. } => None,
    }
}

pub(crate) fn hold(domain: &mut Domain, task: &TaskRecord) {
    let Some(key) = held_by(task) else { return };
    if let Some(count) = domain.project_charter_holders.get_mut(&key) {
        *count = count.checked_add(1).expect("agent count");
    } else {
        let _previous = domain.project_charter_holders.insert(key, 1).expect("at most one set per live agent");
    }
}

pub(crate) fn release(domain: &mut Domain, task: &TaskRecord, out: &mut Queue<Request>) {
    let Some((project, revision)) = held_by(task) else { return };
    let key = (project, revision);
    let count = domain.project_charter_holders.get_mut(&key).expect("set held by live agent");
    *count = count.checked_sub(1).expect("counted live agent");
    if *count == 0 {
        domain.project_charter_holders.remove(&key);
        out.push(Request::ReleasedProjectCharters { project, revision });
    }
}
