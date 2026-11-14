//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{
    Limits, List, account_piece, c, capacity_fields, charge, hello_refusal_line_fields, host_piece, host_run_piece,
    person_capacity_refusal_fields, project_capacity_fields, store_fill_fields, task_capacity_refusal_fields,
    too_large, w,
};
use alloc::boxed::Box;

#[expect(
    clippy::disallowed_methods,
    reason = "the link converts the wire's validated text to the contract's owned str"
)]
fn census(value: &w::StoreCensus, room: &mut u64) -> Result<c::StoreCensus, w::Problem> {
    charge(room, u64::try_from(size_of::<c::StoreCensus>()).ok())?;
    charge(room, List::<c::CensusKeyspace>::price(value.keyspaces().len()))?;
    let mut keyspaces = List::with_capacity(value.keyspaces().len());
    for row in value.keyspaces() {
        charge(room, Some(skein_lib::length::len_u64(row.prefix().len())))?;
        charge(room, Some(skein_lib::length::len_u64(row.name().len())))?;
        charge(room, List::<c::CensusHead>::price(row.heads().len()))?;
        let mut heads = List::with_capacity(row.heads().len());
        for head in row.heads() {
            heads.push(c::CensusHead { head: *head.head(), rows: head.rows() }).expect("reserved supplied heads");
        }
        let name = match core::str::from_utf8(row.name()) {
            Ok(name) => name.into(),
            Err(_) => return Err(w::Problem { path: w::Path::CensusKeyspaceName, reason: skein_codec::Reason::Utf8 }),
        };
        keyspaces
            .push(c::CensusKeyspace {
                prefix: row.prefix().into(),
                name,
                rows: row.rows(),
                keys: row.keys(),
                values: row.values(),
                heads,
                puts: row.puts(),
                erases: row.erases(),
                written: row.written(),
            })
            .expect("reserved supplied keyspaces");
    }
    Ok(c::StoreCensus { commit: value.commit(), since: value.since(), keyspaces })
}

pub(crate) fn deployment_fields(value: &w::Deployment, limits: &Limits) -> Result<c::Deployment, w::Problem> {
    let mut room = super::document_room(limits);
    charge(&mut room, u64::try_from(size_of::<c::Deployment>()).ok())?;
    if value.projects().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.projects().len());
    for item in value.projects() {
        list0.push(project_capacity_fields(item)).expect("reserved supplied list");
    }
    if value.hello_refusals().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.hello_refusals().len());
    for item in value.hello_refusals() {
        list1.push(hello_refusal_line_fields(item)).expect("reserved supplied list");
    }
    if value.person_room_refusals().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list2 = List::with_capacity(value.person_room_refusals().len());
    for item in value.person_room_refusals() {
        list2.push(person_capacity_refusal_fields(item)).expect("reserved supplied list");
    }
    if value.task_room_refusals().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.task_room_refusals().len());
    for item in value.task_room_refusals() {
        list3.push(task_capacity_refusal_fields(item)).expect("reserved supplied list");
    }
    if value.connectors().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list4 = List::with_capacity(value.connectors().len());
    for item in value.connectors() {
        list4.push(*item).expect("reserved supplied list");
    }
    Ok(c::Deployment {
        providers: super::catalogue::offers(value.providers(), limits, &mut room)?,
        providers_cut: value.providers_cut(),
        key_bytes: value.key_bytes(),
        readiness: super::catalogue::readiness(value.readiness(), limits, &mut room)?,
        readiness_cut: value.readiness_cut(),
        store: store_fill_fields(value.store()),
        census: match value.census() {
            Some(value) => Some(Box::new(census(value, &mut room)?)),
            None => None,
        },
        hosts: host_piece(value.hosts(), limits, limits.client().rows, &mut room)?,
        attempts: host_run_piece(value.attempts(), limits, limits.client().rows)?,
        accounts: account_piece(value.accounts(), limits, &mut room)?,
        tasks: capacity_fields(value.tasks()),
        workers: capacity_fields(value.workers()),
        attempts_capacity: capacity_fields(value.attempts_capacity()),
        waiting_attempts: value.waiting_attempts(),
        proposals: capacity_fields(value.proposals()),
        projects: list0,
        grace_bound: value.grace_bound(),
        hello_refusals: list1,
        hosts_forgotten: value.hosts_forgotten(),
        hello_refusals_forgotten: value.hello_refusals_forgotten(),
        person_room_refusals: list2,
        person_room_refusals_forgotten: value.person_room_refusals_forgotten(),
        person_answer_retention: value.person_answer_retention(),
        task_room_refusals: list3,
        task_room_refusals_forgotten: value.task_room_refusals_forgotten(),
        cut: value.cut(),
        connectors: list4,
        connectors_cut: value.connectors_cut(),
    })
}
