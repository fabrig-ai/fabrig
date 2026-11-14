//! Jobs follow patches and closed windows, before the seal (protocol/store/jobs.md, 2–3).
use super::{Patching, refuse, sealing, state, take};
use crate::{
    Below, Cursor, Event, Job, JobAnswer, JobPage, JobRefusal, JobRoom, JobRow, JobWrite, Limits, OpenRefusal, Owner,
    Store, commits, keys, open, pages,
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, length::len_u64};

#[derive(Debug)]
enum Waiting {
    Ready,
    Page { rows: Box<[pages::WalkRow]>, end: bool },
    Commit { number: u64, end: bool },
    Registry { number: u64 },
}
#[derive(Debug)]
pub(crate) struct Running {
    sealing: sealing::Sealing,
    pending: List<(Owner, Job)>,
    index: u32,
    after: Option<Cursor>,
    rows: u64,
    commits: u64,
    waiting: Waiting,
    registered: bool,
    first: bool,
}
impl Running {
    pub(super) fn ready(&self) -> bool {
        match self.waiting {
            Waiting::Ready => true,
            Waiting::Page { .. } | Waiting::Commit { .. } | Waiting::Registry { .. } => false,
        }
    }
    pub(crate) fn at_page(&self) -> bool {
        match self.waiting {
            Waiting::Page { .. } => true,
            Waiting::Ready | Waiting::Commit { .. } | Waiting::Registry { .. } => false,
        }
    }
    fn current(&self) -> (Owner, Job) {
        *self.pending.get(self.index).expect("one pending job at a time")
    }
    pub(super) fn committed(&mut self, number: u64, above: &mut Queue<Event>, token: skein_lib::Token) -> bool {
        match self.waiting {
            Waiting::Registry { number: expected } if number == expected => {
                self.waiting = Waiting::Ready;
                true
            }
            Waiting::Commit { number: expected, end } if number == expected => {
                self.commits = self.commits.checked_add(1).expect("commits bounded by commit number");
                if end {
                    let (of, job) = self.current();
                    above.push(Event::Ran {
                        owner: token,
                        of,
                        job: Box::from(job.name),
                        rows: self.rows,
                        commits: self.commits,
                    });
                    self.index = self.index.checked_add(1).expect("bounded pending jobs");
                    self.after = None;
                    self.first = true;
                    self.rows = 0;
                    self.commits = 0;
                }
                self.waiting = Waiting::Ready;
                true
            }
            Waiting::Ready | Waiting::Page { .. } | Waiting::Commit { .. } | Waiting::Registry { .. } => false,
        }
    }
    fn refusal(&self, refusal: JobRefusal) -> OpenRefusal {
        let (owner, job) = self.current();
        OpenRefusal::Job { owner, job: Box::from(job.name), key: refusal.key, why: refusal.why }
    }
}
pub(super) fn begin(
    store: &mut Store,
    env: &Env<Limits>,
    sealing: sealing::Sealing,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    let mut pending = List::uncapped();
    for of in [Owner::Skein, Owner::Smith, Owner::Jig, Owner::Application] {
        if let Some(set) = crate::patching::plan::set(&store.patch_sets, of) {
            for job in set.jobs {
                match crate::jobs::kept::done(store, of, job) {
                    Ok(true) => {}
                    Ok(false) => {
                        pending.push((of, *job)).expect("uncapped job list");
                    }
                    Err(why) => return open::refuse(store, store.directory.owner, why, above),
                }
            }
        }
    }
    state(
        store,
        Patching::Jobs(Box::new(Running {
            sealing,
            pending,
            index: 0,
            after: None,
            rows: 0,
            commits: 0,
            waiting: Waiting::Ready,
            registered: false,
            first: true,
        })),
    );
    resume(store, env, above, below);
}
fn ranges(store: &Store, owner: Owner, job: &Job) -> Option<Box<[skein_kv::Range]>> {
    let mut ranges = List::uncapped();
    for range in job.keys {
        ranges
            .push(skein_kv::Range {
                start: whole(store, owner, range.from)?,
                end: Some(whole(store, owner, range.to)?),
            })
            .ok()?;
    }
    Some(ranges.into_boxed())
}
fn whole(store: &Store, owner: Owner, key: &[u8]) -> Option<Box<[u8]>> {
    if owner == Owner::Application {
        keys::application(key, store.limits.store.key)
    } else if key.len() <= skein_lib::length::usize_of(store.limits.store.key) {
        Some(Box::from(key))
    } else {
        None
    }
}
fn relative(owner: Owner, key: &[u8]) -> Option<&[u8]> {
    if owner == Owner::Application { keys::of_application(key) } else { Some(key) }
}
fn load(store: &mut Store, running: &mut Running, above: &mut Queue<Event>) -> Option<()> {
    let (of, job) = running.current();
    let ranges = ranges(store, of, &job)?;
    let written = store.registry.written(&job);
    let id = match &written {
        Some((id, _)) => Some(*id),
        None => None,
    };
    let walked = load_values(store, running, &ranges, job.converts.is_some(), id)?;
    if walked.rows.is_empty() && walked.next.is_some() {
        return None;
    }
    let end = walked.rows.is_empty();
    let mut rows = List::with_capacity(u32::try_from(walked.rows.len()).ok()?);
    for row in &walked.rows {
        rows.push(JobRow { key: Box::from(relative(of, &row.row.key)?), value: row.row.value.clone() }).ok()?;
    }
    running.rows = running.rows.checked_add(u64::from(rows.len()))?;
    running.waiting = Waiting::Page { rows: walked.rows, end };
    above.push(Event::Job {
        owner: store.directory.owner,
        of,
        job: Box::from(job.name),
        page: JobPage {
            room: room(store, of, &job)?,
            rows: rows.into_boxed(),
            end,
            layout: if running.first {
                match written {
                    Some((_, layout)) => Some(layout),
                    None => None,
                }
            } else {
                None
            },
        },
    });
    running.first = false;
    Some(())
}
fn load_values(
    store: &mut Store,
    running: &mut Running,
    ranges: &[skein_kv::Range],
    written: bool,
    id: Option<u16>,
) -> Option<pages::Walked> {
    let mut bound = 1_u64;
    for space in store.stores.store.census().keyspaces() {
        bound = bound.checked_add(space.rows)?;
    }
    for _ in 0..bound {
        let page = if written {
            pages::reader::walk_stored(store, ranges, running.after.as_ref(), store.limits.store.ops)
        } else {
            pages::reader::walk(store, ranges, running.after.as_ref(), store.limits.store.ops)
        };
        let walked = match page {
            Ok(walked) => walked,
            Err(failure) => {
                store.open_failed(crate::restore::OpenFailure::Data(failure));
                return None;
            }
        };
        if !written {
            return Some(walked);
        }
        let mut rows = List::uncapped();
        let mut last = None;
        for row in walked.rows {
            last = Some(row.cursor.clone());
            if id.is_some() && crate::registry::head(&row.row.value) == id {
                rows.push(row).ok()?;
            }
        }
        if !rows.is_empty() || walked.next.is_none() {
            return Some(pages::Walked { rows: rows.into_boxed(), next: walked.next });
        }
        running.after = last;
    }
    None
}
pub(super) fn resume(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    let Patching::Jobs(mut running) = *take(store) else { unreachable!("resume routed jobs") };
    if !running.registered {
        if !running.ready() {
            state(store, Patching::Jobs(running));
            return;
        }
        let Some(ops) = store.registry.batch(&store.limits) else { return refuse(store, above) };
        if !ops.is_empty() {
            let Some(number) = store.stores.store.last().checked_add(1) else { return refuse(store, above) };
            running.waiting = Waiting::Registry { number };
            state(store, Patching::Jobs(running));
            commits::submit(store, env, number, ops, above, below);
            return;
        }
        store.registry.activate();
        running.registered = true;
    }
    if running.index == running.pending.len() {
        let Some(number) = store.stores.store.last().checked_add(1) else { return refuse(store, above) };
        running.sealing.number = number;
        return sealing::begin(store, env, running.sealing, above, below);
    }
    if running.ready() && load(store, &mut running, above).is_none() {
        return refuse(store, above);
    }
    state(store, Patching::Jobs(running));
}
fn invalid(key: Option<Box<[u8]>>, why: &str) -> JobRefusal {
    JobRefusal { key, why: Box::from(why) }
}
fn consumed(owner: Owner, rows: &[pages::WalkRow], through: Option<Box<[u8]>>) -> Result<Option<Cursor>, JobRefusal> {
    match through {
        None => Ok(match rows.last() {
            Some(row) => Some(row.cursor.clone()),
            None => None,
        }),
        Some(through) => {
            for row in rows {
                if relative(owner, &row.row.key) == Some(through.as_ref()) {
                    return Ok(Some(row.cursor.clone()));
                }
            }
            Err(invalid(Some(through), "through is not a key of the job page"))
        }
    }
}
fn size(op: &skein_kv::Op) -> Option<u64> {
    let key = u32::try_from(op.key().len()).ok()?;
    Some(u64::from(match op {
        skein_kv::Op::Put { value, .. } => skein_kv::put_bytes(key, u32::try_from(value.len()).ok()?)?,
        skein_kv::Op::Erase { .. } => skein_kv::erase_bytes(key)?,
    }))
}
fn room(store: &Store, of: Owner, job: &Job) -> Option<JobRoom> {
    let writes = store.limits.commits.writes.min(store.limits.store.ops).checked_sub(1)?;
    // A u64's largest encoding bounds the done row at any commit number.
    let done = crate::jobs::kept::operation(store, of, job, u64::MAX)?;
    let reserved = u64::from(skein_kv::FRAME_BYTES).checked_add(size(&done)?)?;
    let mut bytes = u64::from(store.limits.store.commit).checked_sub(reserved)?;
    if of == Owner::Application {
        // JobWrite::bytes counts relative keys. Each whole key adds one tag.
        bytes = bytes.checked_sub(u64::from(writes))?;
    }
    Some(JobRoom { writes, bytes })
}
fn operations(
    store: &Store,
    of: Owner,
    job: &Job,
    writes: Box<[JobWrite]>,
    number: u64,
    end: bool,
) -> Result<Box<[skein_kv::Op]>, JobRefusal> {
    let bound = invalid(None, "job answer is past the store commit edge");
    let count = len_u64(writes.len()).checked_add(1).ok_or(bound)?;
    if count > u64::from(store.limits.commits.writes.min(store.limits.store.ops)) {
        return Err(invalid(None, "job answer is past the store commit edge"));
    }
    let done = crate::jobs::kept::operation(store, of, job, number).ok_or(invalid(None, "job row does not fit"))?;
    let mut bytes = u64::from(skein_kv::FRAME_BYTES)
        .checked_add(size(&done).ok_or(invalid(None, "job row does not fit"))?)
        .ok_or(invalid(None, "job row does not fit"))?;
    let mut ops = List::with_capacity(store.limits.store.ops);
    for write in writes {
        let op = match write {
            JobWrite::Put { key, value } => {
                if value.len() > skein_lib::length::usize_of(store.limits.store.value) {
                    return Err(invalid(Some(key), "job value is past the store value bound"));
                }
                let whole = whole(store, of, &key).ok_or(invalid(Some(key), "job key is past the store key bound"))?;
                skein_kv::Op::Put { key: whole, value }
            }
            JobWrite::Erase { key } => {
                let whole = whole(store, of, &key).ok_or(invalid(Some(key), "job key is past the store key bound"))?;
                skein_kv::Op::Erase { key: whole }
            }
        };
        bytes = bytes
            .checked_add(size(&op).ok_or(invalid(None, "job write does not fit"))?)
            .ok_or(invalid(None, "job write does not fit"))?;
        ops.push(op).expect("answer count checked");
    }
    if bytes > u64::from(store.limits.store.commit) {
        return Err(invalid(None, "job answer is past the store commit byte bound"));
    }
    if end {
        ops.push(done).expect("room reserved for the done row");
    }
    Ok(ops.into_boxed())
}
pub(crate) fn answer(
    store: &mut Store,
    env: &Env<Limits>,
    owner: skein_lib::Token,
    answer: Result<JobAnswer, JobRefusal>,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    if owner != store.directory.owner || crate::directory::patching(store).is_none() {
        return refuse(store, above);
    }
    let Patching::Jobs(mut running) = *take(store) else { return refuse(store, above) };
    let (of, job) = running.current();
    let Waiting::Page { rows, end } = core::mem::replace(&mut running.waiting, Waiting::Ready) else {
        return refuse(store, above);
    };
    let answer = match answer {
        Ok(answer) => answer,
        Err(why) => return open::refuse(store, owner, running.refusal(why), above),
    };
    let through = match consumed(of, &rows, answer.through) {
        Ok(through) => through,
        Err(why) => return open::refuse(store, owner, running.refusal(why), above),
    };
    let Some(number) = store.stores.store.last().checked_add(1) else { return refuse(store, above) };
    let ops = match operations(store, of, &job, answer.writes, number, end) {
        Ok(ops) => ops,
        Err(why) => return open::refuse(store, owner, running.refusal(why), above),
    };
    running.after = through;
    running.waiting = Waiting::Commit { number, end };
    state(store, Patching::Jobs(running));
    commits::submit(store, env, number, ops, above, below);
    super::resume(store, env, above, below);
}
