//! Index changes belong to the deciding step (jig/docs/design/domain/people.md, 13.7).
use super::{Change, CoreTasks};
use crate::{Core, Limits, PublicIndexLimits};
use jig_domain_tasks as tasks;

pub(crate) fn restored(core: &mut Core, project: u32) {
    if core.public_project(project) {
        core.public_index.pending(project);
    }
}

pub(crate) fn publish(core: &mut Core, limits: PublicIndexLimits, project: u32) {
    if core.public_project(project) {
        let tasks = CoreTasks { tasks: &core.tasks, names: &core.names, changed: None };
        core.public_index.publish(&tasks, &limits, project);
    }
}

pub(crate) fn follow(core: &mut Core, limits: &Limits, row: &tasks::TaskRecord) {
    if !core.public_project(row.project) {
        return;
    }
    let previous = match core.public_index.projects.get(&row.project) {
        Some(Some(project)) => project.line(row.number),
        Some(None) | None => return,
    };
    let change = match row.phase {
        tasks::Phase::Ended(_) => Change::Ended { task: row.number },
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            classify(previous, row)
        }
    };
    let tasks = CoreTasks { tasks: &core.tasks, names: &core.names, changed: Some(row) };
    core.public_index.follow(&tasks, &limits.public, change);
}

fn classify(previous: Option<&crate::PublicLine>, row: &tasks::TaskRecord) -> Change {
    let Some(previous) = previous else {
        return if row.tracked.is_some() {
            Change::Tracked { task: row.number }
        } else {
            Change::Made { task: row.number }
        };
    };
    let parent = match row.requester {
        tasks::Party::Task(parent) => Some(parent),
        tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
    };
    if previous.parent.is_some() && previous.parent != parent {
        return Change::Moved { task: row.number };
    }
    if previous.priority.is_some() && row.tracked.is_none() {
        return Change::Untracked { task: row.number };
    }
    if previous.priority.is_none() && row.tracked.is_some() {
        return Change::Tracked { task: row.number };
    }
    if previous.priority != row.tracked {
        return Change::Priority { task: row.number };
    }
    Change::Made { task: row.number }
}
