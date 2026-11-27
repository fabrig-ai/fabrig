//! Restart waiters judge only their repositories (domain/forge/catching-up.md, 2 and 4).
use super::{
    Domain, Env, FreshStep, Limits, List, Name, Queue, Request, claim, continue_release_once, decide_step, emit,
    finish_release_outcome, fresh_reads_in, repository_fresh, step_verified, take_entry, wake_once,
};
use skein_forge::api::Repository;

pub(super) fn repository(name: &Name) -> Repository {
    Repository { forge: name.forge, repository: name.repository }
}

pub(super) fn all_fresh(d: &Domain, repositories: &[Repository]) -> bool {
    // The stall clock's session can span reads whose repositories are not retained here.
    if repositories.is_empty() {
        return fresh_reads_in(d);
    }
    for repository in repositories {
        if !repository_fresh(d, *repository) {
            return false;
        }
    }
    true
}

pub(super) fn names_fresh(d: &Domain, names: &[Name]) -> bool {
    for name in names {
        if !repository_fresh(d, repository(name)) {
            return false;
        }
    }
    true
}

pub(super) fn claim_fresh(d: &Domain, writes: &[Name], designs: &[Name]) -> bool {
    names_fresh(d, writes) && names_fresh(d, designs)
}

pub(crate) fn change_fresh(d: &Domain, task: u64) -> bool {
    match d.changes.get(&task) {
        Some(row) => repository_fresh(d, row.repository),
        None => true,
    }
}

pub(super) fn projection_fresh(d: &Domain, goal: u64) -> bool {
    match d.issues.get(&goal) {
        Some(row) => repository_fresh(d, row.repository),
        None => true,
    }
}

fn add_repository(repositories: &mut List<Repository>, repository: Repository) {
    if !repositories.as_slice().contains(&repository) {
        repositories.push(repository).expect("release repositories are adopted");
    }
}

fn release_repositories(d: &Domain, task: u64) -> List<Repository> {
    let mut repositories = List::with_capacity(d.repositories.capacity());
    if let Some(waiting) = d.release_waits.get(&task) {
        for repository in waiting {
            add_repository(&mut repositories, *repository);
        }
    }
    for names in [d.names.get(&task), d.kept.get(&task)].into_iter().flatten() {
        for name in names {
            add_repository(&mut repositories, repository(name));
        }
    }
    if let Some(row) = d.changes.get(&task) {
        add_repository(&mut repositories, row.repository);
    }
    if let Some(row) = d.issues.get(&task) {
        add_repository(&mut repositories, row.repository);
    }
    for (name, hold) in &d.holds {
        let mut uses = hold.task == task;
        if let Some(writer) = hold.writer {
            uses |= match writer {
                crate::Writer::Run { task: writer, .. } | crate::Writer::Lost { task: writer, .. } => writer == task,
                crate::Writer::Entry(_) => false,
            };
        }
        for writer in &hold.design_writers {
            uses |= writer.task == task;
        }
        if uses {
            add_repository(&mut repositories, repository(name));
        }
    }
    for (_, row) in &d.entries {
        if row.entry.task == task {
            add_repository(&mut repositories, row.entry.repository);
        }
    }
    repositories
}

pub(super) fn release_fresh(d: &Domain, task: u64) -> bool {
    for repository in release_repositories(d, task).into_boxed() {
        if !repository_fresh(d, repository) {
            return false;
        }
    }
    true
}

pub(super) fn park_release_in(d: &mut Domain, task: u64, repository: Repository) {
    let mut repositories = release_repositories(d, task);
    add_repository(&mut repositories, repository);
    d.release_waits.insert(task, repositories.into_boxed()).expect("one wait per release");
}

pub(super) fn park_release(d: &mut Domain, task: u64) {
    let repositories = release_repositories(d, task).into_boxed();
    d.release_waits.insert(task, repositories).expect("one wait per release");
}

/// All scans and temporary selections are bounded by the parked tables' capacities.
pub(super) fn wake(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut repositories = List::with_capacity(d.repositories.capacity());
    for (repository, _) in &d.repositories {
        if !d.fresh_repositories.contains(repository) && repository_fresh(d, *repository) {
            repositories.push(*repository).expect("adopted repositories");
        }
    }
    for repository in repositories.into_boxed() {
        d.fresh_repositories.insert(repository).expect("adopted repositories");
        crate::tiers::fresh(d, env, repository, out);
        crate::follows::fresh(d, repository, out);
    }

    let mut steps = List::with_capacity(d.fresh_steps.capacity());
    for (task, _) in &d.fresh_steps {
        if change_fresh(d, *task) {
            steps.push(*task).expect("parked changes");
        }
    }
    for task in steps.into_boxed() {
        let step = d.fresh_steps.remove(&task).expect("collected parked step");
        d.fresh_waits.remove(&task);
        if !d.stopping {
            match step {
                FreshStep::Gather(pending) => decide_step(d, env, *pending, out),
                FreshStep::Verified { merge, heard } => step_verified(d, env, task, merge, *heard, out),
            }
        }
    }
    let mut waiting = List::with_capacity(d.fresh_waits.capacity());
    for (task, repositories) in &d.fresh_waits {
        if all_fresh(d, repositories) {
            waiting.push(*task).expect("parked procedures");
        }
    }
    for task in waiting.into_boxed() {
        d.fresh_waits.remove(&task);
        wake_once(out, task);
    }

    wake_outcomes(d, env, out);
    let mut claims = List::with_capacity(d.waiting_claims.capacity());
    for (task, (_, writes, designs, _)) in &d.waiting_claims {
        if claim_fresh(d, writes, designs) {
            claims.push(*task).expect("parked claims");
        }
    }
    for task in claims.into_boxed() {
        let (attempt, writes, designs, holders) = d.waiting_claims.remove(&task).expect("collected claim");
        claim(d, env, task, attempt, &writes, &designs, &holders, out);
        wake_once(out, task);
    }
    let mut releases = List::with_capacity(d.release_waits.capacity());
    for (task, _) in &d.release_waits {
        if release_fresh(d, *task) {
            releases.push(*task).expect("parked releases");
        }
    }
    for task in releases.into_boxed() {
        d.release_waits.remove(&task);
        if let Some(row) = d.releases.get(&task)
            && !row.failed
        {
            continue_release_once(out, task);
        }
    }
    let mut projections = List::with_capacity(d.projection_waits.capacity());
    for goal in &d.projection_waits {
        if projection_fresh(d, *goal) {
            projections.push(*goal).expect("parked projections");
        }
    }
    for goal in projections.into_boxed() {
        d.projection_waits.remove(&goal);
        d.projection_due.remove(&goal);
        emit(out, Request::ProjectAfter { goal, when: None });
    }
}

fn wake_outcomes(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut outcomes = List::with_capacity(d.waiting_release_outcomes.capacity());
    for (entry, _) in &d.waiting_release_outcomes {
        for (task, row) in &d.releases {
            if let Some((pending, _)) = &row.pending
                && *pending == *entry
                && release_fresh(d, *task)
            {
                outcomes.push((*entry, *task)).expect("one owner per release outcome");
            }
        }
    }
    for (entry, task) in outcomes.into_boxed() {
        let (outcome, account) = d.waiting_release_outcomes.remove(&entry).expect("collected release outcome");
        finish_release_outcome(d, env, task, entry, outcome, account, out);
        // Keep the release and entry linked across a crash until this judgement is saved.
        if outcome != skein_forge_client::Outcome::Held && d.entries.contains_key(&entry) {
            take_entry(d, entry, task);
            emit(out, Request::Erase { key: crate::Key::Entry(entry) });
        }
    }
}
