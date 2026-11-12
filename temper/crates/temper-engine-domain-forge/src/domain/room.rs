//! A change's room and task-owned accounts (domain/forge/change/README.md, 7).
#[cfg(test)]
mod tests;
use super::{ChangeRow, Domain, Env, Key, Limits, List, Queue, Request, change, connection_wait, emit};

impl Domain {
    /// The forge's settled, unpinned predicate; the root also checks whether its goal lives.
    #[must_use]
    pub fn droppable(&self, task: u64) -> bool {
        let Some(row) = self.changes.get(&task) else { return false };
        match row.change.state {
            change::State::Ended | change::State::Cleared { .. } | change::State::NoChange { .. } => {}
            change::State::Producing { .. }
            | change::State::Opening { .. }
            | change::State::Recreating { .. }
            | change::State::Reopening { .. }
            | change::State::Checking { .. }
            | change::State::Judging { .. }
            | change::State::Gating { .. }
            | change::State::CancellingGates { .. }
            | change::State::Ready { .. }
            | change::State::Preparing { .. }
            | change::State::Queued { .. }
            | change::State::First { .. }
            | change::State::Updating { .. }
            | change::State::WaitingProducer { .. }
            | change::State::Resolving { .. }
            | change::State::Repairing { .. }
            | change::State::Landing { .. }
            | change::State::Landed { .. }
            | change::State::Red { .. }
            | change::State::Uncleared { .. }
            | change::State::Held { .. } => return false,
        }
        self.finished_unpinned(task, row)
    }

    /// Ended verifying changes give room only after every droppable settled change.
    #[must_use]
    pub fn droppable_last(&self, task: u64) -> bool {
        let Some(row) = self.changes.get(&task) else { return false };
        match row.change.state {
            change::State::Landed { .. } | change::State::Red { .. } | change::State::Uncleared { .. } => {}
            change::State::Ended
            | change::State::Cleared { .. }
            | change::State::NoChange { .. }
            | change::State::Producing { .. }
            | change::State::Opening { .. }
            | change::State::Recreating { .. }
            | change::State::Reopening { .. }
            | change::State::Checking { .. }
            | change::State::Judging { .. }
            | change::State::Gating { .. }
            | change::State::CancellingGates { .. }
            | change::State::Ready { .. }
            | change::State::Preparing { .. }
            | change::State::Queued { .. }
            | change::State::First { .. }
            | change::State::Updating { .. }
            | change::State::WaitingProducer { .. }
            | change::State::Resolving { .. }
            | change::State::Repairing { .. }
            | change::State::Landing { .. }
            | change::State::Held { .. } => return false,
        }
        self.finished_unpinned(task, row)
    }

    fn finished_unpinned(&self, task: u64, row: &ChangeRow) -> bool {
        if row.ending.is_none() || self.releases.contains_key(&task) {
            return false;
        }
        for (_, landing) in &self.landing.rows {
            for member in &landing.landing.members {
                if member.change == task {
                    return false;
                }
            }
        }
        true
    }
}

pub(super) fn drop_change(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    if d.changes.len() != env.limits.changes {
        return;
    }
    if !d.droppable(task) {
        if !d.droppable_last(task) {
            return;
        }
        for (number, _) in &d.changes {
            if d.droppable(*number) {
                return;
            }
        }
    }
    let row = d.changes.remove(&task).expect("droppable row exists");
    emit(out, Request::Erase { key: Key::Change(task) });
    d.panel_ci.remove(&task);
    d.settling.remove(&task);
    d.start_conflicts.remove(&task);
    d.procedure_wakes.remove(&task);
    d.change_due.remove(&task);
    d.goal_files.remove(&task);
    d.review_facts.remove(&task);
    d.review_since.remove(&task);
    d.review_requests.remove(&task);
    connection_wait::take(d, task);
    let mut steps = List::with_capacity(d.steps.len());
    for (owner, pending) in &d.steps {
        if pending.task == task {
            steps.push(*owner).expect("bounded steps");
        }
    }
    for owner in steps.into_boxed() {
        d.steps.remove(&owner);
    }
    let mut reads = List::with_capacity(d.goal_file_reads.len());
    for (owner, pending) in &d.goal_file_reads {
        if pending.task == task {
            reads.push(*owner).expect("bounded goal file reads");
        }
    }
    for owner in reads.into_boxed() {
        d.goal_file_reads.remove(&owner);
    }
    let mut commits = List::with_capacity(d.landed.len());
    for (commit, owner) in &d.landed {
        if *owner == task || !d.changes.contains_key(owner) {
            commits.push(*commit).expect("bounded landed commits");
        }
    }
    for commit in commits.into_boxed() {
        d.landed.remove(&commit);
        emit(out, Request::Erase { key: Key::Landed(commit) });
    }
    let mut tasks = List::with_capacity(crate::MARKED);
    tasks.push(task).expect("own panel subject");
    if let Some(landing) = row.change.landing {
        tasks.push(landing).expect("member landing subject");
    }
    if let Some(subjects) = crate::views::changes_mark(d, row.repository) {
        for subject in subjects {
            tasks.push(subject).expect("project changes and status subjects");
        }
    }
    emit(out, Request::Marked { tasks: tasks.into_boxed() });
}

pub(super) fn refused(out: &mut Queue<Request>, row: &ChangeRow, cause: crate::RefusalCause) {
    emit(
        out,
        Request::ResourceRefused {
            task: row.task,
            operation: b"register change",
            resource: Some(crate::Name {
                forge: row.repository.forge,
                repository: row.repository.repository,
                what: crate::What::Repository,
            }),
            cause,
        },
    );
}
