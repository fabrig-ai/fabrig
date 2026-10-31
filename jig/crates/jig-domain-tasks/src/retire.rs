//! Closed funding sources move to history (protocol/store.md, 15.1).
use crate::funders::{newer_period, save_funding, source_live, total};
use crate::{Domain, Funder, Key, Request, Stored};
use skein_lib::Queue;

// The closed rows remain durable history. Their counters do not move again.
// The pool posts its expense once and returns its original allotment to its period.
pub(crate) fn retire(domain: &mut Domain, out: &mut Queue<Request>) {
    let count = domain.funding.len();
    for _ in 0..count {
        let mut ready = None;
        for (funder, row) in &domain.funding {
            let period = match *funder {
                Funder::Pool { project, period, .. } | Funder::Recurring { project, period, .. } => {
                    Some((project, period))
                }
                Funder::Task(_) | Funder::Period { .. } => None,
            };
            if let Some((project, period)) = period
                && row.numbers.reserved == 0
                && newer_period(domain, project, period)
            {
                ready = Some((*funder, row.parent.expect("allotment's period"), row.numbers));
                break;
            }
        }
        let Some((pool, parent, numbers)) = ready else { break };
        close_allotment(domain, pool, parent, numbers, out);
    }
    let count = domain.funding.len();
    for _ in 0..count {
        let mut ready = None;
        for (funder, row) in &domain.funding {
            match *funder {
                Funder::Period { project, period }
                    if row.numbers.reserved == 0
                        && newer_period(domain, project, period)
                        && !source_live(domain, *funder) =>
                {
                    ready = Some(*funder);
                    break;
                }
                Funder::Period { .. } | Funder::Task(_) | Funder::Pool { .. } | Funder::Recurring { .. } => {}
            }
        }
        let Some(period) = ready else { break };
        close(domain, period, out);
    }
}

pub(crate) fn close_allotment(
    domain: &mut Domain,
    funder: Funder,
    parent: Funder,
    numbers: crate::Numbers,
    out: &mut Queue<Request>,
) {
    assert_eq!(numbers.reserved, 0, "allotment has no live reservations");
    let period = domain.funding.get_mut(&parent).expect("original period kept");
    period.numbers.reserved = period.numbers.reserved.checked_sub(numbers.budget).expect("allotment reserved");
    period.numbers.spent_below = period
        .numbers
        .spent_below
        .checked_add(total(numbers).expect("valid allotment"))
        .expect("allotment spend representable");
    close(domain, funder, out);
    save_funding(domain, parent, out);
}

pub(crate) fn close(domain: &mut Domain, funder: Funder, out: &mut Queue<Request>) {
    let mut row = domain.funding.remove(&funder).expect("funding source kept until closure");
    row.closed = true;
    out.push(Request::Erase { key: Key::Ledger(funder) });
    out.push(Request::Save { record: Stored::ClosedLedger(row) });
}
