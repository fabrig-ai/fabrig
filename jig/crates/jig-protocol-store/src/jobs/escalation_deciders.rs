//! Type live tasks' rejection deciders once per store (protocol/store/jobs.md, 1 and 3).
use super::{JobAnswer, JobPage, JobRefusal, JobRow, JobWrite};
use crate::Store;
use alloc::boxed::Box;
use jig_protocol_store_tasks::tasks_v13 as wire;
use skein_lib::{List, Reader};
use wire::Record;

fn refusal(key: &[u8], why: &str) -> JobRefusal {
    JobRefusal { key: Some(Box::from(key)), why: Box::from(why) }
}

fn decode(row: &JobRow, limits: &wire::Limits) -> Result<wire::Row, JobRefusal> {
    match wire::Row::decode(limits, &mut Reader::new(&row.value)) {
        Ok(row) => Ok(row),
        Err(_) => Err(refusal(&row.key, "unreadable escalation decider row")),
    }
}

pub(super) fn page(page: &JobPage, store: &mut Store) -> Result<JobAnswer, JobRefusal> {
    let limits = crate::codec::tasks::limits(&store.limits.core.tasks).expect("admitted tasks codec limits");
    let mut writes = List::uncapped();
    let mut bytes = 0_u64;
    let mut through = None;
    let mut tasks = 0_u64;
    for row in &page.rows {
        if let Some(write) = judge(row, store, &limits)? {
            let cost = bytes.checked_add(write.bytes());
            let fits = match (cost, writes.len().checked_add(1)) {
                (Some(cost), Some(count)) => cost <= page.room.bytes && count <= page.room.writes,
                (None, _) | (_, None) => false,
            };
            if !fits {
                if through.is_none() {
                    return Err(refusal(&row.key, "row exceeds the escalation decider job edge"));
                }
                return Ok(JobAnswer { writes: writes.into_boxed(), through, counts: Box::new([tasks]) });
            }
            bytes = cost.expect("checked write cost");
            writes.push(write).expect("uncapped job writes");
            tasks = tasks.saturating_add(1);
        }
        through = Some(row.key.clone());
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through: None, counts: Box::new([tasks]) })
}

fn judge(row: &JobRow, store: &mut Store, limits: &wire::Limits) -> Result<Option<JobWrite>, JobRefusal> {
    let mut value = decode(row, limits)?;
    let wire::Value::LiveHot(hot) = &mut value.value else { return Ok(None) };
    let wire::Escalation::Rejected(rejected) = &mut hot.run.escalation else { return Ok(None) };
    match rejected.by_kind {
        wire::DeciderKind::Task => return Ok(None),
        wire::DeciderKind::Person => {}
    }
    if !ancestor(store, &hot.run.requester, rejected.by, limits, &row.key)? {
        return Ok(None);
    }
    rejected.by_kind = wire::DeciderKind::Task;
    let bytes = value.encode(limits).expect("only decider kind changed");
    Ok(Some(JobWrite::Put { key: row.key.clone(), value: bytes }))
}

fn ancestor(
    store: &mut Store,
    requester: &wire::Party,
    by: u64,
    limits: &wire::Limits,
    held: &[u8],
) -> Result<bool, JobRefusal> {
    let mut requester = requester.clone();
    for _ in 0..store.limits.core.tasks.depth {
        let number = match requester {
            wire::Party::Task(number) => number.value,
            wire::Party::Person(_) | wire::Party::Deployment(_) => return Ok(false),
        };
        if number == by {
            return Ok(true);
        }
        let key = crate::keys::encode(
            &jig_domain_core_vocabulary::Key::Tasks(jig_domain_tasks::Key::Live(number)),
            store.limits.store.key,
        )
        .expect("admitted hot key length");
        let Ok(read) = store.job_read(skein_kv::Range::prefix(&key), None, 1) else {
            return Err(refusal(&key, "cannot read escalation decider ancestor"));
        };
        let Some(row) = read.rows.into_iter().next() else { return Ok(false) };
        if row.key != key {
            return Ok(false);
        }
        let row = decode(&JobRow { key: row.key, value: row.value }, limits)?;
        let wire::Value::LiveHot(hot) = row.value else { return Ok(false) };
        requester = hot.run.requester;
    }
    match requester {
        wire::Party::Person(_) | wire::Party::Deployment(_) => Ok(false),
        wire::Party::Task(_) => Err(refusal(held, "escalation decider ancestry exceeds task depth")),
    }
}
