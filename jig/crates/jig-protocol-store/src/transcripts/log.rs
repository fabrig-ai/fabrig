//! New turns and their reserved places (protocol/store/logs.md, 5).
use super::LogPlace;
use crate::{CommitCheck, Store, codec, keys, logs, open};
use jig_domain_core_vocabulary::TurnRecord;
use skein_lib::{List, Token};

pub(crate) fn file(store: &mut Store, task: u64) -> Result<bool, CommitCheck> {
    if store.transcripts.places.contains_key(&task) {
        return Ok(true);
    }
    let key = keys::encode_own(&keys::Own::Transcript { task }, store.limits.store.key).ok_or(CommitCheck::Encoding)?;
    Ok(open::get(store, Token::new(0), key).or(Err(CommitCheck::Reads))?.is_some())
}

pub(crate) fn place(
    store: &mut Store,
    number: u64,
    turn: &TurnRecord,
    length: u64,
    ops: &mut List<skein_kv::Op>,
) -> Result<(), CommitCheck> {
    let task = turn.task;
    let log = logs::turns(task).ok_or(CommitCheck::Capacity)?;
    let key = keys::encode_own(&keys::Own::TurnLog { task }, store.limits.store.key).ok_or(CommitCheck::Encoding)?;
    let (offset, mut row) = match store.transcripts.logs.get(&task) {
        Some(place) => (place.end, place.row),
        None => {
            let row = match open::get(store, Token::new(0), key.clone()).or(Err(CommitCheck::Reads))? {
                Some(bytes) => codec::component::TurnLog::decode(&bytes).or(Err(CommitCheck::Reads))?,
                None => codec::component::TurnLog { window: None },
            };
            (store.stores.store.log_end(log), row)
        }
    };
    let end = offset.checked_add(length).ok_or(CommitCheck::Capacity)?;
    if turn.opens_window {
        row.window = Some(offset);
    }
    let _: Option<LogPlace> =
        store.transcripts.logs.insert(task, LogPlace { number, end, row }).or(Err(CommitCheck::Capacity))?;
    ops.push(skein_kv::Op::Put { key, value: row.encode().or(Err(CommitCheck::Encoding))? })
        .or(Err(CommitCheck::Capacity))?;
    let attempt = (task, turn.attempt);
    if !store.transcripts.attempts.contains_key(&attempt) {
        let key = keys::encode_own(&keys::Own::LogAttempt { task, attempt: turn.attempt }, store.limits.store.key)
            .ok_or(CommitCheck::Encoding)?;
        if open::get(store, Token::new(0), key.clone()).or(Err(CommitCheck::Reads))?.is_none() {
            let value = codec::component::LogAttempt { first: offset }.encode().or(Err(CommitCheck::Encoding))?;
            ops.push(skein_kv::Op::Put { key, value }).or(Err(CommitCheck::Capacity))?;
            let _: Option<u64> = store.transcripts.attempts.insert(attempt, number).or(Err(CommitCheck::Capacity))?;
        }
    }
    Ok(())
}

pub(crate) fn committed(store: &mut Store, number: u64) {
    let mut tasks = List::with_capacity(store.transcripts.logs.len());
    for (task, place) in &store.transcripts.logs {
        if place.number <= number {
            tasks.push(*task).expect("bounded log reservations");
        }
    }
    for task in tasks.into_boxed() {
        let _: Option<LogPlace> = store.transcripts.logs.remove(&task);
    }
}
