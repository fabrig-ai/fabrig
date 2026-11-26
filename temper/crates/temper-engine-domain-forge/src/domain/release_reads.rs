//! A closing task's cleanup read bound (temper/docs/design/domain/forge/waits.md, 8–10).
use super::{Domain, Env, Limits, List, Name, Queue, Request, goal_branch, release_failure};
use alloc::boxed::Box;
use skein_forge::api::Error;
use skein_lib::{Wall, bytes};

#[derive(Debug, Hash)]
pub(super) struct Clock {
    since: Wall,
    on: Option<Read>,
}

#[derive(Debug, Hash)]
pub(super) enum Read {
    Fresh,
    Writer,
    Making(Name),
}

pub(super) fn advanced(d: &mut Domain, task: u64, since: Wall) {
    if d.releases.contains_key(&task) {
        d.release_reads.insert(task, Clock { since, on: None }).expect("one clock per release");
    }
}

pub(super) fn waiting(d: &mut Domain, env: &Env<Limits>, task: u64, on: Read) {
    if !can_fail(d, task) {
        return;
    }
    if !d.release_reads.contains_key(&task) {
        advanced(d, task, env.wall);
    }
    d.release_reads.get_mut(&task).expect("active release clock").on = Some(on);
}

pub(super) fn ready(d: &mut Domain, env: &Env<Limits>, task: u64) {
    let Some(clock) = d.release_reads.get_mut(&task) else { return };
    let on = clock.on.take();
    let progressed = match on {
        Some(Read::Fresh) => true,
        Some(Read::Making(name)) => !goal_branch::reading(d, &name),
        Some(Read::Writer) | None => false,
    };
    if progressed {
        advanced(d, task, env.wall);
    }
}

fn can_fail(d: &Domain, task: u64) -> bool {
    if let Some(row) = d.releases.get(&task)
        && row.releasing
        && row.effects_settled
        && !row.failed
        && row.pending.is_none()
    {
        for (_, entry) in &d.entries {
            if entry.entry.task == task && !entry.held {
                return false;
            }
        }
        return true;
    }
    false
}

pub(super) fn deadline(d: &Domain) -> Option<Wall> {
    let mut earliest: Option<Wall> = None;
    for (task, clock) in &d.release_reads {
        if clock.on.is_some() && can_fail(d, *task) {
            let at = crate::waits::step_bound(clock.since, d.landing.limits.waits.step);
            earliest = Some(match earliest {
                Some(first) => first.min(at),
                None => at,
            });
        }
    }
    earliest
}

pub(super) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut expired = List::with_capacity(d.release_reads.len());
    for (task, clock) in &d.release_reads {
        if clock.on.is_some()
            && can_fail(d, *task)
            && crate::waits::step_bound(clock.since, env.limits.waits.step) <= env.wall
        {
            expired.push(*task).expect("one bound per release");
        }
    }
    for task in expired.into_boxed() {
        let on = d.release_reads.get_mut(&task).expect("collected release").on.take().expect("collected read");
        let (what, last) = describe(d, env, task, on);
        let last = crate::waits::read_answer(last);
        let reason = crate::waits::stalled_words(b"release", env.limits.waits.step, &what, Some(&last));
        failed(d, task, reason, out);
    }
}

pub(super) fn rereading(d: &mut Domain, env: &Env<Limits>, task: u64) {
    if pending_writer(d, task).is_some() {
        waiting(d, env, task, Read::Writer);
    }
}

fn pending_writer(d: &Domain, task: u64) -> Option<&super::PendingLost> {
    for (_, read) in &d.lost {
        if read.task == task {
            return Some(read);
        }
        if let Some(hold) = d.holds.get(&read.name)
            && hold.task == task
        {
            return Some(read);
        }
    }
    None
}

fn describe(d: &Domain, env: &Env<Limits>, task: u64, on: Read) -> (Box<[u8]>, Option<Error>) {
    match on {
        Read::Fresh => {
            let last = match pending_writer(d, task) {
                Some(read) => read.last_answer,
                None => None,
            };
            (Box::from(b"fresh observations from the forge".as_slice()), last)
        }
        Read::Writer => {
            if let Some(read) = pending_writer(d, task)
                && let Some(resource) = super::to_client(&read.name, &env.limits)
                && let skein_forge_client::What::Branch(branch) = resource.what
            {
                return (bytes::concat(&[b"a writer's reread of branch ", &branch]), read.last_answer);
            }
            (Box::from(b"a writer's reread".as_slice()), None)
        }
        Read::Making(name) => (Box::from(b"the goal branch's making".as_slice()), goal_branch::last_answer(d, &name)),
    }
}

pub(super) fn failed(d: &mut Domain, task: u64, words: Box<[u8]>, out: &mut Queue<Request>) {
    if !can_fail(d, task) {
        return;
    }
    let mut row = d.releases.get(&task).expect("closing release").clone();
    release_failure::held(d, &mut row, words, out);
}
