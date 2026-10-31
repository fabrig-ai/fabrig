//! Project rows close after its last live task (temper/docs/design/draft/playground-reset.md, 5).
//! Spend stays in closed funding history (protocol/store.md, 15.1).
use crate::{Domain, Funder, Key, Limits, Request};
use skein_lib::{Env, List, Queue};

pub(crate) fn drop_project(domain: &mut Domain, _env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    if !domain.ready() {
        return;
    }
    if domain.live_in(project) {
        out.push(Request::DropRefused { project });
        return;
    }
    let mut sources = List::with_capacity(domain.funding.capacity());
    for (funder, _) in &domain.funding {
        match *funder {
            Funder::Pool { project: owner, .. }
            | Funder::Recurring { project: owner, .. }
            | Funder::Period { project: owner, .. } => {
                if owner == project {
                    sources.push(*funder).expect("bounded funding sources");
                }
            }
            Funder::Task(_) => {}
        }
    }
    for funder in &sources {
        match *funder {
            Funder::Pool { .. } | Funder::Recurring { .. } => {
                release(domain, *funder);
                let row = *domain.funding.get(funder).expect("project source retained");
                crate::retire::close_pool(domain, *funder, row.parent.expect("allotment's period"), row.numbers, out);
            }
            Funder::Period { .. } | Funder::Task(_) => {}
        }
    }
    for funder in &sources {
        match *funder {
            Funder::Period { .. } => {
                release(domain, *funder);
                crate::retire::close(domain, *funder, out);
            }
            Funder::Pool { .. } | Funder::Recurring { .. } | Funder::Task(_) => {}
        }
    }
    let mut holds = List::with_capacity(domain.placed.capacity());
    for (number, hold) in &domain.placed {
        if hold.project == project {
            holds.push(*number).expect("bounded placed holds");
        }
    }
    for number in &holds {
        domain.placed.remove(number);
        out.push(Request::Erase { key: Key::PlacedHold(*number) });
    }
    let mut proposals = List::with_capacity(domain.person_proposals.capacity());
    for (number, proposal) in &domain.person_proposals {
        if proposal.project == project {
            proposals.push(*number).expect("bounded person proposals");
        }
    }
    for number in &proposals {
        domain.person_proposals.remove(number);
        out.push(Request::Erase { key: Key::PersonProposal(*number) });
    }
}

#[expect(
    clippy::disallowed_macros,
    reason = "jp-tasks requires stray reservations released and diagnosed in debug builds"
)]
fn release(domain: &mut Domain, funder: Funder) {
    let row = domain.funding.get_mut(&funder).expect("project source retained");
    let reserved = row.numbers.reserved;
    row.numbers.reserved = 0;
    debug_assert_eq!(reserved, 0, "no task reservation remains after the project's tasks ended");
}
