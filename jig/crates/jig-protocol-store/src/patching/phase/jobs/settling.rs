//! Store-owned raw walks and retirement by proof (protocol/store/jobs.md, 5).
use super::{Running, invalid, relative, room, whole};
use crate::{JobCount, JobRefusal, Store};
use alloc::boxed::Box;
use skein_lib::{List, Map};

type Commit = (u64, Box<[skein_kv::Op]>, bool);
pub(super) fn run(store: &mut Store, running: &mut Running) -> Result<Commit, JobRefusal> {
    let (owner, job) = running.current();
    if running.first {
        for name in ["put", "retired"] {
            running.counts.push(JobCount { name: Box::from(name), count: 0 }).expect("two store counts");
        }
        running.first = false;
    }
    let set = *crate::patching::plan::set(&store.patch_sets, owner).expect("admitted owner");
    let mut ordered = Map::uncapped();
    for family in set.families {
        for ty in family.types {
            for range in ty.keys {
                let from = whole(store, owner, range.from).ok_or(invalid(None, "settling range does not fit"))?;
                let to = whole(store, owner, range.to).ok_or(invalid(None, "settling range does not fit"))?;
                ordered.insert(from, to).expect("admitted ranges");
            }
        }
    }
    let mut ranges = List::uncapped();
    for (start, end) in &ordered {
        ranges.push(skein_kv::Range { start: start.clone(), end: Some(end.clone()) }).expect("uncapped ranges");
    }
    let mut bound = 1_u64;
    for space in store.stores.store.census().keyspaces() {
        bound = bound.saturating_add(space.rows);
    }
    for _ in 0..=bound {
        if running.retiring {
            return retire(store, running);
        }
        let walked = match crate::pages::reader::walk_stored(
            store,
            ranges.as_slice(),
            running.after.as_ref(),
            store.limits.store.ops,
        ) {
            Ok(page) => page,
            Err(failure) => {
                store.open_failed(crate::restore::OpenFailure::Data(failure));
                return Err(invalid(None, "settling page does not read"));
            }
        };
        if walked.rows.is_empty() {
            if walked.next.is_some() {
                return Err(invalid(None, "settling page consumes nothing"));
            }
            let logs_pending = match crate::jobs::kept::done(store, crate::Owner::Jig, &crate::jobs::task_logs::JOB) {
                Ok(done) => !done,
                Err(_) => return Err(invalid(None, "task-logs done row does not read")),
            };
            let count = store
                .registry
                .retire_set(&set, logs_pending, store.limits.store.key)
                .ok_or(invalid(None, "settling retirement does not fit"))?;
            running.counts.get_mut(1).expect("retired count").count = count;
            running.retiring = true;
            continue;
        }
        let room = room(store, owner, &job).ok_or(invalid(None, "settling commit has no room"))?;
        let mut ops = List::with_capacity(room.writes);
        let mut bytes = 0_u64;
        let mut consumed = false;
        for row in walked.rows {
            let key = relative(owner, &row.row.key).ok_or(invalid(None, "settling key has no owner"))?;
            let refusal = Some(Box::from(key));
            let value = match store.registry.put_again(&row.row.key, &row.row.value, &mut store.layout_work) {
                Ok(value) => value,
                Err(why) => return Err(JobRefusal { key: refusal, why }),
            };
            if let Some(value) = value {
                let op = skein_kv::Op::Put { key: row.row.key.clone(), value };
                let size = super::size(&op, key).ok_or(invalid(refusal.clone(), "settling put does not fit"))?;
                let total = bytes.saturating_add(size);
                if ops.len() == room.writes || total > room.bytes {
                    if !consumed {
                        return Err(invalid(refusal, "settling put is past the store commit edge"));
                    }
                    break;
                }
                bytes = total;
                ops.push(op).expect("settling room checked");
                let count = &mut running.counts.get_mut(0).expect("put count").count;
                *count = count.saturating_add(1);
            }
            running.rows = running.rows.saturating_add(1);
            running.after = Some(row.cursor);
            consumed = true;
        }
        if !ops.is_empty() {
            let number = store.stores.store.last().checked_add(1).ok_or(invalid(None, "commit number exhausted"))?;
            return Ok((number, ops.into_boxed(), false));
        }
    }
    Err(invalid(None, "settling walk did not end"))
}
fn retire(store: &mut Store, running: &Running) -> Result<Commit, JobRefusal> {
    let number = store.stores.store.last().checked_add(1).ok_or(invalid(None, "commit number exhausted"))?;
    let ops = store.registry.batch(&store.limits).ok_or(invalid(None, "settling retirement does not fit"))?;
    if store.registry.has_writes() {
        return Ok((number, ops, false));
    }
    let (owner, job) = running.current();
    let done = crate::jobs::kept::operation(store, owner, &job, number)
        .ok_or(invalid(None, "settling done row does not fit"))?;
    let limit = store.limits.store.ops.min(store.limits.commits.writes);
    let mut bytes = u64::from(skein_kv::FRAME_BYTES);
    for op in &ops {
        bytes = bytes.saturating_add(super::size(op, op.key()).ok_or(invalid(None, "settling erase does not fit"))?);
    }
    bytes =
        bytes.saturating_add(super::size(&done, done.key()).ok_or(invalid(None, "settling done row does not fit"))?);
    if skein_lib::length::len_u64(ops.len()) >= u64::from(limit) || bytes > u64::from(store.limits.store.commit) {
        if ops.is_empty() {
            return Err(invalid(None, "settling done row is past the store commit edge"));
        }
        return Ok((number, ops, false));
    }
    let mut final_ops = List::with_capacity(limit);
    for op in ops {
        final_ops.push(op).expect("registry batch fits");
    }
    final_ops.push(done).expect("room for the done row");
    Ok((number, final_ops.into_boxed(), true))
}
