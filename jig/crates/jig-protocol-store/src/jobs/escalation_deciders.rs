//! Type live tasks' rejection deciders once per store (protocol/store/jobs.md, 1 and 3).
use super::{JobAnswer, JobPage, JobRefusal, JobRow, JobWrite};
use crate::Store;
use alloc::boxed::Box;
use jig_protocol_store_tasks::tasks_v13 as wire;
use skein_lib::{List, Reader, Writer};

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
    let mut parts = decode(row, limits)?.into_parts();
    let wire::Value::LiveHot(hot) = parts.value else { return Ok(None) };
    let mut hot = hot.into_parts();
    let mut run = hot.run.into_parts();
    let wire::Escalation::Rejected(rejected) = run.escalation else { return Ok(None) };
    match rejected.by_kind() {
        wire::DeciderKind::Task => return Ok(None),
        wire::DeciderKind::Person => {}
    }
    if !ancestor(store, &run.requester, rejected.by(), limits, &row.key)? {
        return Ok(None);
    }
    let mut rejected = rejected.into_parts();
    rejected.by_kind = wire::DeciderKind::Task;
    run.escalation =
        wire::Escalation::Rejected(wire::RejectedEscalation::new(limits, rejected).expect("only decider kind changed"));
    hot.run = wire::TaskRun::new(limits, run).expect("only decider kind changed");
    parts.value = wire::Value::LiveHot(wire::LiveHot::new(limits, hot).expect("only decider kind changed"));
    let updated = wire::Row::new(limits, parts).expect("only decider kind changed");
    let mut writer = Writer::new(usize::try_from(updated.measure()).expect("bounded tasks row"));
    updated.encode(&mut writer).expect("measured tasks row");
    Ok(Some(JobWrite::Put { key: row.key.clone(), value: writer.finish() }))
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
            wire::Party::Task(number) => number.value(),
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
        let Ok(read) = store.patch_page(skein_kv::Range::prefix(&key), None, 1) else {
            return Err(refusal(&key, "cannot read escalation decider ancestor"));
        };
        let Some(row) = read.rows.into_iter().next() else { return Ok(false) };
        if row.key != key {
            return Ok(false);
        }
        let row = decode(&JobRow { key: row.key, value: row.value }, limits)?;
        let wire::Value::LiveHot(hot) = row.value() else { return Ok(false) };
        requester = hot.run().requester().clone();
    }
    match requester {
        wire::Party::Person(_) | wire::Party::Deployment(_) => Ok(false),
        wire::Party::Task(_) => Err(refusal(held, "escalation decider ancestry exceeds task depth")),
    }
}
