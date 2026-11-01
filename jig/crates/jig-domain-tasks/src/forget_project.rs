//! A deleted project's funding and pending rows (jig/docs/design/domain/tasks.md, sections 2–3).
use crate::{Domain, Funder, FundingRecord, Key, Request};
use skein_lib::{List, Map, Queue};

pub(crate) fn project(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    for (number, _) in &domain.names {
        assert!(
            crate::domain::record(domain, *number).expect("live name").project != project,
            "forget project only after its last live task ends"
        );
    }
    let mut rows = List::with_capacity(domain.funding.capacity());
    for (funder, row) in &domain.funding {
        if names(*funder, project) {
            rows.push(*row).expect("bounded funding rows");
        }
    }
    spend(rows.as_slice(), true, out);
    for row in &rows {
        domain.funding.remove(&row.funder).expect("selected funding row");
        out.push(Request::Erase { key: Key::Ledger(row.funder) });
    }
    let mut proposals = List::with_capacity(domain.person_proposals.capacity());
    for (number, row) in &domain.person_proposals {
        if row.project == project {
            proposals.push(*number).expect("bounded person proposals");
        }
    }
    for number in &proposals {
        domain.person_proposals.remove(number).expect("selected person proposal");
        out.push(Request::Erase { key: Key::PersonProposal(*number) });
    }
    let mut holds = List::with_capacity(domain.placed.capacity());
    for (number, row) in &domain.placed {
        if row.project == project {
            holds.push(*number).expect("bounded placed holds");
        }
    }
    for number in &holds {
        domain.placed.remove(number).expect("selected placed hold");
        out.push(Request::Erase { key: Key::PlacedHold(*number) });
    }
    crate::holds::forget_project(domain, project);
}

pub(crate) fn ledgers(project: u32, rows: &[FundingRecord], out: &mut Queue<Request>) {
    for row in rows {
        assert!(names(row.funder, project), "forgotten closed ledger belongs to the project");
        assert!(row.closed, "forget ledgers receives closed history");
    }
    spend(rows, false, out);
    for row in rows {
        out.push(Request::Erase { key: Key::ClosedLedger(row.funder) });
    }
}

fn names(funder: Funder, project: u32) -> bool {
    match funder {
        Funder::Period { project: owner, .. }
        | Funder::Pool { project: owner, .. }
        | Funder::Recurring { project: owner, .. } => owner == project,
        Funder::Task(_) => false,
    }
}

fn spend(rows: &[FundingRecord], live: bool, out: &mut Queue<Request>) {
    let mut deployment = 0_u64;
    let mut parties: Map<u64, u64> = Map::uncapped();
    for row in rows {
        let spent = crate::funders::total(row.numbers).expect("funding spend fits accounting unit");
        match row.funder {
            Funder::Period { .. } => {
                deployment = deployment.checked_add(spent).expect("project spend fits accounting unit");
            }
            Funder::Pool { person, .. } => {
                let previous = parties.get(&person).copied().unwrap_or(0);
                let total = previous.checked_add(spent).expect("party spend fits accounting unit");
                parties.insert(person, total).expect("internal spend aggregation");
                // Live allotments have not posted to their periods; closed ones have.
                if live {
                    deployment = deployment.checked_add(spent).expect("project spend fits accounting unit");
                }
            }
            Funder::Recurring { .. } => {
                if live {
                    deployment = deployment.checked_add(spent).expect("project spend fits accounting unit");
                }
            }
            Funder::Task(_) => unreachable!("external funding has no task ledger"),
        }
    }
    if deployment != 0 {
        out.push(Request::SpendForgotten { party: None, spent: deployment });
    }
    for (party, spent) in &parties {
        if *spent != 0 {
            out.push(Request::SpendForgotten { party: Some(*party), spent: *spent });
        }
    }
}
