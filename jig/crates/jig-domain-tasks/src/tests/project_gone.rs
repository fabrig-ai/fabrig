//! Project deletion retains only period and pool spending history (draft 5).
#![expect(clippy::disallowed_methods, reason = "unit fixtures use ordinary Rust (programming-model.md, 10.2)")]
use super::revisions::{Harness, to};
use crate::*;
use skein_lib::Wall;

#[test]
fn a_gone_projects_period_and_pools_are_closed_and_its_templates_dropped() {
    let mut h = Harness::new();
    let period = Funder::Period { project: 1, period: 0 };
    let pool = Funder::Pool { project: 1, person: 9, period: 0 };
    h.send(Event::CarvePool { reply_to: to(), project: 1, person: 9, period: 0, budget: 100 });
    h.send(Event::OpenPeriod { reply_to: to(), project: 2, period: 0, budget: 100 });
    h.domain.funding.get_mut(&pool).unwrap().numbers.spent_below = 17;
    h.domain.funding.get_mut(&period).unwrap().numbers.spent_below = 3;
    h.send(Event::ProjectGone { project: 1 });
    for funder in [pool, period] {
        assert!(h.domain.funding(funder).is_none());
        assert!(!h.stored.contains_key(&Key::Ledger(funder)));
        let Some(Stored::ClosedLedger(row)) = h.stored.get(&Key::ClosedLedger(funder)) else {
            panic!("closed spending ledger")
        };
        assert!(row.closed);
        assert_eq!(row.numbers.reserved, 0);
        assert_eq!(row.numbers.spent_below, if funder == pool { 17 } else { 20 });
    }
    assert!(h.domain.funding(Funder::Period { project: 2, period: 0 }).is_some());
    assert!(h.send(Event::ProjectGone { project: 1 }).is_empty());
    // Templates live on task records; tasks have already been erased at this entrance.
    assert!(h.domain.names.is_empty());
}

#[test]
fn a_gone_projects_recurring_ledgers_post_into_its_period_and_go() {
    let mut h = Harness::new();
    let period = Funder::Period { project: 1, period: 0 };
    let recurring = Funder::Recurring { project: 1, task: 8, period: 0 };
    let pool = Funder::Pool { project: 1, person: 9, period: 0 };
    h.send(Event::CarvePool { reply_to: to(), project: 1, person: 9, period: 0, budget: 100 });
    let row = FundingRecord {
        funder: recurring,
        parent: Some(period),
        period: None,
        numbers: Numbers { budget: 100, spent: 7, spent_below: 11, reserved: 0 },
        made: 2,
        closed: false,
    };
    h.domain.funding.insert(recurring, row).unwrap();
    h.stored.insert(Key::Ledger(recurring), Stored::Ledger(row));
    h.domain.funding.get_mut(&period).unwrap().numbers.reserved = 200;
    let out = h.send(Event::ProjectGone { project: 1 });
    let recurring_erased =
        out.iter()
            .position(|request| {
                if let Request::Erase { key: Key::Ledger(funder) } = request { *funder == recurring } else { false }
            })
            .unwrap();
    let pool_closed = out
        .iter()
        .position(|request| {
            if let Request::Save { record: Stored::ClosedLedger(row) } = request { row.funder == pool } else { false }
        })
        .unwrap();
    assert!(recurring_erased < pool_closed);
    assert!(h.domain.funding(recurring).is_none());
    assert!(!h.stored.contains_key(&Key::Ledger(recurring)));
    assert!(!h.stored.contains_key(&Key::ClosedLedger(recurring)));
    let Some(Stored::ClosedLedger(row)) = h.stored.get(&Key::ClosedLedger(period)) else { panic!("closed period") };
    assert_eq!(row.numbers.spent_below, 18);
    assert_eq!(row.numbers.reserved, 0);
}

#[test]
fn a_gone_projects_access_holds_and_pending_proposals_go_unanswered() {
    let mut h = Harness::new();
    for (number, project) in [(1, 1), (2, 1), (3, 2)] {
        h.send(Event::ResourceAccess {
            project,
            name: Name { connector: 1, path: Box::new([]) },
            access: ResourceAccess::Context,
        });
        let hold = PlacedHold {
            number,
            project,
            owner: Owner::Party { person: 9 },
            mode: Mode::Drain,
            scope: Scope::Project,
            reason: Box::new([]),
            placed_at: 0,
            revision: 1,
            exempt: Box::new([]),
            lifted: number == 2,
        };
        h.domain.placed.insert(number, hold.clone()).unwrap();
        h.stored.insert(Key::PlacedHold(number), Stored::PlacedHold(Box::new(hold)));
        let proposal = PersonProposal {
            number,
            project,
            proposer: 9,
            action: PersonAction::Settings { against: 1, items: Box::new([]) },
            reason: Box::new([]),
            state: PersonProposalState::Pending { since: Wall::EPOCH },
        };
        h.domain.person_proposals.insert(number, proposal.clone()).unwrap();
        h.stored.insert(Key::PersonProposal(number), Stored::PersonProposal(Box::new(proposal)));
    }
    let out = h.send(Event::ProjectGone { project: 1 });
    assert_eq!(h.domain.resource_access.len(), 1);
    assert_eq!(h.domain.placed.len(), 1);
    assert_eq!(h.domain.person_proposals.len(), 1);
    for number in [1, 2] {
        assert!(!h.stored.contains_key(&Key::PlacedHold(number)));
        assert!(!h.stored.contains_key(&Key::PersonProposal(number)));
        assert!(!h.stored.contains_key(&Key::DecidedProposal(number)));
    }
    for request in out {
        if let Request::PersonProposalDecided { .. } = request {
            panic!("gone proposals receive no decision");
        }
        if let Request::Save { record: Stored::Update(_) } = request {
            panic!("gone proposals emit no update");
        }
    }
}
