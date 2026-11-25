//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/reads.md, 1, and client/domain/contract/watches.md, 1).
use super::{Limits, List, account_piece, c, charge, host_piece, host_run_piece, too_large, w};
use crate::generated::client;
use alloc::boxed::Box;

fn census(value: &w::StoreCensus, room: &mut u64) -> Result<c::StoreCensus, w::Problem> {
    charge(room, u64::try_from(size_of::<c::StoreCensus>()).ok())?;
    charge(room, List::<c::CensusKeyspace>::price(value.keyspaces.len()))?;
    let mut keyspaces = List::with_capacity(value.keyspaces.len());
    for row in &value.keyspaces {
        charge(room, Some(skein_lib::length::len_u64(row.prefix.len())))?;
        charge(room, Some(skein_lib::length::len_u64(row.name.len())))?;
        charge(room, List::<c::CensusHead>::price(row.heads.len()))?;
        keyspaces.push(client::census_keyspace_to_domain(row.clone())?).expect("reserved supplied keyspaces");
    }
    Ok(c::StoreCensus { commit: value.commit, since: value.since, keyspaces })
}

pub(crate) fn deployment_fields(value: &w::Deployment, limits: &Limits) -> Result<c::Deployment, w::Problem> {
    let mut room = super::document_room(limits);
    charge(&mut room, u64::try_from(size_of::<c::Deployment>()).ok())?;
    if value.projects.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.projects.len());
    for item in &value.projects {
        list0.push(client::project_capacity_to_domain(item.clone())).expect("reserved supplied list");
    }
    if value.hello_refusals.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.hello_refusals.len());
    for item in &value.hello_refusals {
        list1.push(client::hello_refusal_line_to_domain(item.clone())).expect("reserved supplied list");
    }
    if value.person_room_refusals.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list2 = List::with_capacity(value.person_room_refusals.len());
    for item in &value.person_room_refusals {
        list2.push(client::person_capacity_refusal_to_domain(item.clone())).expect("reserved supplied list");
    }
    if value.task_room_refusals.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.task_room_refusals.len());
    for item in &value.task_room_refusals {
        list3.push(client::task_capacity_refusal_to_domain(item.clone())).expect("reserved supplied list");
    }
    if value.connectors.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list4 = List::with_capacity(value.connectors.len());
    for item in &value.connectors {
        list4.push(*item).expect("reserved supplied list");
    }
    charge(
        &mut room,
        u64::from(value.worker_settings.len())
            .checked_mul(u64::try_from(size_of::<c::WorkerState>()).expect("worker state size")),
    )?;
    let mut worker_settings = List::with_capacity(value.worker_settings.len());
    for row in &value.worker_settings {
        charge(&mut room, u64::try_from(row.name.len()).ok())?;
        worker_settings.push(client::worker_state_to_domain(row.clone())).expect("worker row count");
    }
    Ok(c::Deployment {
        worker_settings,
        providers: super::catalogue::offers(&value.providers, limits, &mut room)?,
        providers_cut: value.providers_cut,
        key_bytes: value.key_bytes,
        readiness: super::catalogue::readiness(&value.readiness, limits, &mut room)?,
        readiness_cut: value.readiness_cut,
        store: client::store_fill_to_domain(value.store.clone()),
        census: match &value.census {
            Some(value) => Some(Box::new(census(value, &mut room)?)),
            None => None,
        },
        hosts: host_piece(&value.hosts, limits, limits.client().rows, &mut room)?,
        attempts: host_run_piece(&value.attempts, limits, limits.client().rows)?,
        accounts: account_piece(&value.accounts, limits, &mut room)?,
        tasks: client::capacity_to_domain(value.tasks.clone()),
        workers: client::capacity_to_domain(value.workers.clone()),
        attempts_capacity: client::capacity_to_domain(value.attempts_capacity.clone()),
        waiting_attempts: value.waiting_attempts,
        proposals: client::capacity_to_domain(value.proposals.clone()),
        projects: list0,
        grace_bound: value.grace_bound,
        hello_refusals: list1,
        hosts_forgotten: value.hosts_forgotten,
        hello_refusals_forgotten: value.hello_refusals_forgotten,
        person_room_refusals: list2,
        person_room_refusals_forgotten: value.person_room_refusals_forgotten,
        person_answer_retention: value.person_answer_retention,
        task_room_refusals: list3,
        task_room_refusals_forgotten: value.task_room_refusals_forgotten,
        cut: value.cut,
        connectors: list4,
        connectors_cut: value.connectors_cut,
    })
}
