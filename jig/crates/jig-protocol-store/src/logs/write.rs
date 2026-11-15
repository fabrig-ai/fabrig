//! One history record per task per commit (protocol/store/logs.md, 3).
use crate::{CommitCheck, Store, codec, jobs, keys, logs, view};
use alloc::boxed::Box;
use skein_lib::{List, Map, Set};

type Rows = Map<Box<[u8]>, Option<Box<[u8]>>>;

pub(crate) fn enabled(store: &mut Store, ops: &[skein_kv::Op]) -> Result<bool, CommitCheck> {
    if store.history_routing {
        return Ok(true);
    }
    if let Some(crate::patching::phase::Patching::Jobs(running)) = crate::directory::patching(store)
        && running.writes_history()
    {
        return Ok(true);
    }
    let key = jobs::kept::key(store, crate::Owner::Jig, &jobs::task_logs::JOB).ok_or(CommitCheck::Encoding)?;
    for op in ops {
        if let skein_kv::Op::Put { key: written, .. } = op
            && *written == key
        {
            return Ok(true);
        }
    }
    Ok(view::raw_get(store, key).or(Err(CommitCheck::Reads))?.is_some())
}

pub(crate) fn route(store: &mut Store, ops: Box<[skein_kv::Op]>) -> Result<Box<[skein_kv::Op]>, CommitCheck> {
    let mut tasks: Map<u64, Rows> = Map::uncapped();
    let mut drops = Set::with_capacity(store.limits.store.ops);
    let mut kept = List::uncapped();
    for op in &ops {
        if let skein_kv::Op::Drop { log } = op {
            drops.insert(*log).expect("drops bounded by input operations");
        }
    }
    for op in ops {
        let (key, value) = match op {
            skein_kv::Op::Put { key, value } => (key, Some(value)),
            skein_kv::Op::Erase { key } => (key, None),
            op @ (skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. }) => {
                kept.push(op).expect("operations bounded at submission");
                continue;
            }
        };
        let Some(task) = keys::logged(&key) else {
            let op = match value {
                Some(value) => skein_kv::Op::Put { key, value },
                None => skein_kv::Op::Erase { key },
            };
            kept.push(op).expect("operations bounded at submission");
            continue;
        };
        let log = logs::history(task).ok_or(CommitCheck::Capacity)?;
        let dropping = drops.contains(&log);
        if dropping && value.is_some() {
            return Err(CommitCheck::Capacity);
        }
        if view::raw_get(store, key.clone()).or(Err(CommitCheck::Reads))?.is_some() {
            kept.push(skein_kv::Op::Erase { key: key.clone() }).expect("operations bounded at submission");
        }
        // Erases accompanying a whole drop remove map copies, not log rows.
        if dropping {
            continue;
        }
        if !tasks.contains_key(&task) {
            tasks.insert(task, Map::uncapped()).expect("tasks bounded by input operations");
        }
        tasks.get_mut(&task).expect("inserted task").insert(key, value).expect("unique input keys");
    }
    for (task, rows) in &tasks {
        history(store, *task, rows, &mut kept)?;
    }
    Ok(kept.into_boxed())
}

fn history(store: &mut Store, task: u64, rows: &Rows, ops: &mut List<skein_kv::Op>) -> Result<(), CommitCheck> {
    let log = logs::history(task).ok_or(CommitCheck::Capacity)?;
    let at = store.stores.store.log_end(log);
    let mut puts = false;
    let mut ended = None;
    let mut record = List::uncapped();
    for (key, value) in rows {
        puts |= value.is_some();
        if key.starts_with(&[1, 1, 6]) {
            ended = Some(match &value {
                Some(_) => Some(at),
                None => None,
            });
        }
        record.push((key.clone(), value.clone())).expect("record bounded by input operations");
    }
    // An erase must not create a log, even when it removes a pre-job map row.
    if at == 0 && !puts {
        return Ok(());
    }
    let body = logs::record::encode(record.as_slice()).ok_or(CommitCheck::Encoding)?;
    let frame = store.stores.store.frame(&body).or(Err(CommitCheck::Encoding))?;
    ops.push(skein_kv::Op::Append { log, frame }).expect("operations bounded at submission");
    if at == 0 || ended.is_some() {
        let place = codec::component::HistoryLog { record: ended.flatten() };
        let key =
            keys::encode_own(&keys::Own::History { task }, store.limits.store.key).ok_or(CommitCheck::Encoding)?;
        let value = place.encode().or(Err(CommitCheck::Encoding))?;
        ops.push(skein_kv::Op::Put { key, value }).expect("operations bounded at submission");
    }
    Ok(())
}

/// Count payload operations and their map extents, and the bytes skein-kv commits.
pub(crate) fn bounds(store: &Store, ops: &[skein_kv::Op]) -> Result<(), CommitCheck> {
    let mut count = u32::try_from(ops.len()).or(Err(CommitCheck::Capacity))?;
    let mut bytes = u64::from(skein_kv::FRAME_BYTES);
    for op in ops {
        let size = match op {
            skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. } => {
                count = count.checked_add(1).ok_or(CommitCheck::Capacity)?;
                match op {
                    skein_kv::Op::Append { .. } => skein_kv::put_bytes(10, 8),
                    skein_kv::Op::Drop { .. } => skein_kv::erase_bytes(10),
                    skein_kv::Op::Put { .. } | skein_kv::Op::Erase { .. } => unreachable!("payload operation"),
                }
            }
            skein_kv::Op::Put { key, value } => skein_kv::put_bytes(
                u32::try_from(key.len()).or(Err(CommitCheck::Bytes))?,
                u32::try_from(value.len()).or(Err(CommitCheck::Bytes))?,
            ),
            skein_kv::Op::Erase { key } => skein_kv::erase_bytes(u32::try_from(key.len()).or(Err(CommitCheck::Bytes))?),
        }
        .ok_or(CommitCheck::Bytes)?;
        bytes = bytes.checked_add(u64::from(size)).ok_or(CommitCheck::Bytes)?;
    }
    if count > store.limits.store.ops {
        return Err(CommitCheck::Capacity);
    }
    if bytes > u64::from(store.limits.store.commit) {
        return Err(CommitCheck::Bytes);
    }
    Ok(())
}
