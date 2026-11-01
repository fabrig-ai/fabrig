//! Supplied period boundaries (`authority.md`, 12; client plan QUESTIONS 32).
//! The period ledger owns its receipt wall and optional supplied next wall.
//! There is no calendar, reset policy or monotonic timer here.
use crate::{Domain, Funder, Request};
use skein_lib::{Queue, Wall};

/// A period's actual opening receipt wall and its source's optional next boundary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PeriodTimes {
    pub began: Wall,
    pub next: Option<Wall>,
}

pub(crate) fn observe(domain: &mut Domain, project: u32, period: u64, next: Option<Wall>, out: &mut Queue<Request>) {
    if !domain.ready() {
        return;
    }
    for (funder, _) in &domain.funding {
        match *funder {
            Funder::Period { project: owner, period: newer } if owner == project && newer > period => return,
            Funder::Task(_) | Funder::Pool { .. } | Funder::Recurring { .. } | Funder::Period { .. } => {}
        }
    }
    let funder = Funder::Period { project, period };
    let Some(row) = domain.funding.get_mut(&funder) else {
        return;
    };
    let Some(times) = &mut row.period else {
        return;
    };
    if times.next == next {
        return;
    }
    times.next = next;
    crate::funders::save_funding(domain, funder, out);
}

/// Close the project's accounting after its last task has been erased
/// (jig/docs/design/domain/core.md, 11).
pub(crate) fn project_gone(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut recurring = skein_lib::List::with_capacity(domain.funding.len());
    let mut pools = skein_lib::List::with_capacity(domain.funding.len());
    let mut periods = skein_lib::List::with_capacity(domain.funding.len());
    for (funder, _) in &domain.funding {
        match *funder {
            Funder::Recurring { project: owner, .. } => {
                if owner == project {
                    recurring.push(*funder).expect("bounded recurring ledgers");
                }
            }
            Funder::Pool { project: owner, .. } => {
                if owner == project {
                    pools.push(*funder).expect("bounded project pools");
                }
            }
            Funder::Period { project: owner, .. } => {
                if owner == project {
                    periods.push(*funder).expect("bounded project periods");
                }
            }
            Funder::Task(_) => {}
        }
    }
    for funder in recurring.into_boxed() {
        settle_allotment(domain, funder, out);
    }
    for funder in pools.into_boxed() {
        settle_allotment(domain, funder, out);
    }
    for funder in periods.into_boxed() {
        let mut row = domain.funding.remove(&funder).expect("collected project period");
        assert!(row.numbers.reserved == 0, "project allocations settled before period closure");
        row.closed = true;
        out.push(Request::Erase { key: crate::Key::Ledger(funder) });
        out.push(Request::Save { record: crate::Stored::ClosedLedger(row) });
    }
    crate::holds::project_gone(domain, project);
    crate::placed::project_gone(domain, project, out);
    crate::person_proposals::project_gone(domain, project, out);
}

fn settle_allotment(domain: &mut Domain, funder: Funder, out: &mut Queue<Request>) {
    let mut row = domain.funding.remove(&funder).expect("collected project ledger");
    assert!(row.numbers.reserved == 0, "project tasks erased before its ledgers");
    let parent = row.parent.expect("allotment's original period");
    let period = domain.funding.get_mut(&parent).expect("original period kept");
    period.numbers.reserved = period.numbers.reserved.checked_sub(row.numbers.budget).expect("allotment reserved");
    period.numbers.spent_below = period
        .numbers
        .spent_below
        .checked_add(crate::funders::total(row.numbers).expect("valid allotment"))
        .expect("project spend representable");
    crate::funders::save_funding(domain, parent, out);
    out.push(Request::Erase { key: crate::Key::Ledger(funder) });
    match funder {
        Funder::Pool { .. } => {
            row.closed = true;
            out.push(Request::Save { record: crate::Stored::ClosedLedger(row) });
        }
        Funder::Recurring { .. } => {}
        Funder::Task(_) | Funder::Period { .. } => unreachable!("collected child ledger"),
    }
}
