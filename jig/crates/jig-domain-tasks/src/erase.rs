//! Whole-task erasure and project release (draft playground-reset.md, 5;
//! jig/docs/design/domain/tasks.md, 3, retention class; protocol/store.md, 11.1 and 15.1).
use crate::{Domain, Funder, Key, Owner, Party, Request, Scope, TaskRecord};
use skein_lib::{List, Queue};

pub(crate) fn erase(domain: &mut Domain, record: &TaskRecord, out: &mut Queue<Request>) {
    if domain.names.contains_key(&record.number) {
        return;
    }
    out.push(Request::Erase { key: Key::Ended(record.number) });
    for index in crate::closing::ended_indices(record).into_boxed() {
        out.push(Request::Erase { key: index.key() });
    }
    match record.requester {
        Party::Task(requester) => {
            out.push(Request::Erase { key: Key::Delegate { requester, task: record.number } });
        }
        Party::Person(_) | Party::Deployment { .. } => {}
    }
    if domain.stubs.remove(&record.number).is_some() {
        out.push(Request::Erase { key: Key::Stub(record.number) });
    }
    let mut holds = List::with_capacity(domain.placed.len());
    for (number, hold) in &domain.placed {
        if hold.owner == (Owner::Task { task: record.number }) || hold.scope == (Scope::Subtree { task: record.number })
        {
            holds.push(*number).expect("subset of retained holds");
        }
    }
    erase_holds(domain, holds, out);
}

pub(crate) fn forget_project(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    for (number, _) in &domain.names {
        if crate::domain::record(domain, *number).expect("live name").project == project {
            return;
        }
    }
    let mut ledgers = List::with_capacity(domain.funding.len());
    for (funder, _) in &domain.funding {
        let belongs = match *funder {
            Funder::Pool { project: owner, .. }
            | Funder::Recurring { project: owner, .. }
            | Funder::Period { project: owner, .. } => owner == project,
            Funder::Task(_) => false,
        };
        if belongs {
            ledgers.push(*funder).expect("subset of live ledgers");
        }
    }
    for funder in ledgers.into_boxed() {
        crate::retire::close(domain, funder, out);
    }
    let mut proposals = List::with_capacity(domain.person_proposals.len());
    for (number, proposal) in &domain.person_proposals {
        if proposal.project == project {
            proposals.push(*number).expect("subset of pending proposals");
        }
    }
    for number in proposals.into_boxed() {
        domain.person_proposals.remove(&number);
        out.push(Request::Erase { key: Key::PersonProposal(number) });
    }
    let mut holds = List::with_capacity(domain.placed.len());
    for (number, hold) in &domain.placed {
        if hold.project == project {
            holds.push(*number).expect("subset of retained holds");
        }
    }
    erase_holds(domain, holds, out);
}

fn erase_holds(domain: &mut Domain, holds: List<u64>, out: &mut Queue<Request>) {
    for number in holds.into_boxed() {
        domain.placed.remove(&number);
        out.push(Request::Erase { key: Key::PlacedHold(number) });
    }
}
