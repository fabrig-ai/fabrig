//! Commit-only fresh reads and counted job ends (domain/forge/keeping-up.md, 2).
use super::{Domain, Duration, Env, Limits, List, Queue, Request, ci_refresh, hints};
use skein_forge::api::{Check, Ci, Commit, Repository as Provider, Status};
use skein_lib::Time;
#[cfg(test)]
#[path = "../tests/ci_news.rs"]
mod tests;

#[derive(Hash, Debug)]
pub(super) struct Watch {
    status: Ci,
    latest: Ci,
    concluded: u32,
    target: Option<u32>,
    end: Option<Time>,
    due: Option<Time>,
    delay: Duration,
    reading: bool,
}

fn due(at: Option<Time>, now: Time) -> bool {
    match at {
        Some(at) => at <= now,
        None => false,
    }
}
fn shown(target: Option<u32>, count: u32) -> bool {
    match target {
        Some(target) => count >= target,
        None => false,
    }
}
fn earlier(at: Option<Time>, next: Time) -> Time {
    match at {
        Some(at) => at.min(next),
        None => next,
    }
}

fn concluded(rows: &[Status]) -> u32 {
    let mut count = 0_u32;
    for row in rows {
        match row.check {
            Check::Passed | Check::Failed => count = count.saturating_add(1),
            Check::Pending => {}
        }
    }
    count
}

pub(super) fn observe(d: &mut Domain, env: &Env<Limits>, task: u64, head: Commit, rows: &[Status], status: Ci) {
    observed(d, env, task, head, Some(concluded(rows)), status);
}

pub(super) fn observe_verdict(d: &mut Domain, env: &Env<Limits>, task: u64, head: Commit, status: Ci) {
    observed(d, env, task, head, None, status);
}

pub(super) fn seen(d: &Domain, task: u64, head: Commit, status: Ci) -> bool {
    match d.ci_news.get(&(task, head)) {
        Some(watch) => watch.latest == status,
        None => false,
    }
}

fn observed(d: &mut Domain, env: &Env<Limits>, task: u64, head: Commit, count: Option<u32>, status: Ci) {
    let key = (task, head);
    if let Some(watch) = d.ci_news.get_mut(&key) {
        watch.status = status;
        watch.latest = status;
        if let Some(count) = count {
            watch.concluded = count;
        }
        if shown(watch.target, watch.concluded) {
            watch.target = None;
            watch.end = None;
            watch.due = None;
        }
        return;
    }
    // Keep only live dependencies, including facts from an in-flight gather.
    let mut old = List::with_capacity(d.ci_news.len());
    for ((owner, commit), _) in &d.ci_news {
        let stale = match d.changes.get(owner) {
            Some(row) => row.terminal() || (*owner == task && !hints::known_commit(d, row, *commit)),
            None => true,
        };
        if stale {
            old.push((*owner, *commit)).expect("sized from watches");
        }
    }
    for key in &old {
        d.ci_news.remove(key);
    }
    d.ci_news
        .insert(
            key,
            Watch {
                status,
                latest: status,
                concluded: count.unwrap_or(0),
                target: None,
                end: None,
                due: None,
                delay: env.limits.client.hinted,
                reading: false,
            },
        )
        .expect("internal change dependencies");
}

/// A recognised commit is read alone; an unknown dependency keeps the hint's fallback wake.
pub(super) fn hint(d: &mut Domain, env: &Env<Limits>, task: u64, head: Commit) -> bool {
    if !d.ci_news.contains_key(&(task, head)) {
        let Some(row) = d.changes.get(&task) else { return false };
        if !hints::known_commit(d, row, head) {
            return false;
        }
        d.ci_news
            .insert(
                (task, head),
                Watch {
                    status: Ci::None,
                    latest: Ci::None,
                    concluded: 0,
                    target: None,
                    end: None,
                    due: None,
                    delay: env.limits.client.hinted,
                    reading: false,
                },
            )
            .expect("internal change dependencies");
    }
    let watch = d.ci_news.get_mut(&(task, head)).expect("known commit");
    watch.target = Some(watch.target.unwrap_or(watch.concluded).saturating_add(1));
    if watch.end.is_none() {
        watch.end = Some(env.now.saturating_add(env.limits.client.settle));
    }
    let due = env.now.saturating_add(env.limits.client.hinted);
    watch.due = Some(earlier(watch.due, due));
    true
}

pub(super) fn deadline(d: &Domain) -> Option<Time> {
    let mut due = None;
    for (_, watch) in &d.ci_news {
        for next in [watch.end, if watch.reading { None } else { watch.due }].into_iter().flatten() {
            due = Some(earlier(due, next));
        }
    }
    due
}

fn wake(d: &mut Domain, task: u64) {
    let mut owners = List::with_capacity(d.steps.len());
    for (owner, step) in &d.steps {
        if step.task == task {
            owners.push(*owner).expect("sized from steps");
        }
    }
    for owner in &owners {
        d.steps.get_mut(owner).expect("pending step").woken = true;
    }
    d.procedure_wakes.insert(task, true).expect("one pending wake per change");
}

pub(super) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut ready = List::with_capacity(d.ci_news.len());
    for (key, watch) in &d.ci_news {
        if due(watch.end, env.now) || (!watch.reading && due(watch.due, env.now)) {
            ready.push(*key).expect("sized from watches");
        }
    }
    for (task, head) in &ready {
        let Some(row) = d.changes.get(task) else {
            d.ci_news.remove(&(*task, *head));
            continue;
        };
        if row.terminal() || !hints::known_commit(d, row, *head) {
            d.ci_news.remove(&(*task, *head));
            continue;
        }
        let repository = row.repository;
        let watch = d.ci_news.get_mut(&(*task, *head)).expect("due watch");
        if due(watch.end, env.now) {
            watch.end = None;
            watch.target = None;
            watch.due = None;
            wake(d, *task);
            continue;
        }
        watch.due = None;
        watch.reading = true;
        ci_refresh(d, env, repository, *head, out);
    }
}

/// A pull watch asks for the required-context verdict through the commit's checks.
pub(super) fn refreshing(d: &mut Domain, repository: Provider, head: Commit) {
    let mut keys = List::with_capacity(d.ci_news.len());
    for ((task, commit), _) in &d.ci_news {
        if *commit == head
            && let Some(row) = d.changes.get(task)
            && row.repository == repository
        {
            keys.push((*task, *commit)).expect("sized from watches");
        }
    }
    for key in &keys {
        d.ci_news.get_mut(key).expect("watch key").reading = true;
    }
}

pub(super) fn read(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: Provider,
    head: Commit,
    rows: Option<&[Status]>,
    status: Ci,
) {
    let mut wakes = List::with_capacity(env.limits.changes);
    let mut keys = List::with_capacity(d.ci_news.len());
    for (key, _) in &d.ci_news {
        keys.push(*key).expect("sized from watches");
    }
    for (task, commit) in &keys {
        let watch = d.ci_news.get_mut(&(*task, *commit)).expect("watch key");
        if *commit != head || !watch.reading {
            continue;
        }
        let Some(row) = d.changes.get(task) else { continue };
        if row.repository != repository {
            continue;
        }
        watch.reading = false;
        if let Some(rows) = rows {
            let changed = watch.status != status;
            watch.latest = status;
            watch.concluded = concluded(rows);
            if changed {
                wakes.push(*task).expect("one watch per task for a commit");
            }
            // The comparison remains with what the change last read for its decision.
            if shown(watch.target, watch.concluded) {
                watch.target = None;
                watch.end = None;
                watch.due = None;
            }
        }
        if let Some(end) = watch.end {
            let next = env.now.saturating_add(watch.delay).min(end);
            watch.due = Some(earlier(watch.due, next));
            watch.delay = watch.delay.saturating_mul(2).min(env.limits.client.poll.max(env.limits.client.hinted));
        }
    }
    for task in &wakes {
        wake(d, *task);
    }
}
