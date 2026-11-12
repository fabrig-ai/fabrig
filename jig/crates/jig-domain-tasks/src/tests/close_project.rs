//! Project-close preconditions and output ordering (jig/docs/design/domain/tasks/changes.md).
use super::revisions::{Harness, to};
use crate::{Event, Funder, Key, Mode, Owner, PlacedHold, Request, Scope, Stored};

#[test]
fn project_holds_are_erased_without_touching_other_scopes_or_projects() {
    let mut test = Harness::new();
    let other = Funder::Period { project: 2, period: 0 };
    test.send(Event::OpenPeriod { reply_to: to(), project: 2, period: 0, budget: 100 });
    let other_before = *test.domain.funding(other).expect("other project period");
    // Project holds are supplied here because the placement entrance admits subtrees only.
    for (number, project, scope) in [(1, 1, Scope::Project), (2, 2, Scope::Project), (3, 2, Scope::Subtree { task: 9 })]
    {
        let hold = PlacedHold {
            number,
            project,
            scope,
            owner: Owner::Party { person: 9 },
            mode: Mode::Stop,
            reason: Box::new([]),
            placed_at: 0,
            revision: 1,
            exempt: Box::new([]),
            lifted: false,
        };
        test.domain.placed.insert(number, hold.clone()).expect("fixture hold room");
        test.stored.insert(Key::PlacedHold(number), Stored::PlacedHold(Box::new(hold)));
    }
    let out = test.send(Event::CloseProject { project: 1 });
    let mut erased = false;
    for request in out {
        if let Request::Erase { key: Key::PlacedHold(1) } = request {
            erased = true;
        }
    }
    assert!(erased);
    assert!(test.domain.placed_hold(1).is_none());
    assert!(!test.stored.contains_key(&Key::PlacedHold(1)));
    for number in [2, 3] {
        assert!(test.domain.placed_hold(number).is_some());
        assert!(test.stored.contains_key(&Key::PlacedHold(number)));
    }
    assert_eq!(test.domain.funding_records().len(), 1);
    assert_eq!(test.domain.funding(other), Some(&other_before));
    assert_eq!(test.send(Event::CloseProject { project: 1 }).len(), 0);
    assert_eq!(test.send(Event::CloseProject { project: 99 }).len(), 0);
}

#[test]
fn allotments_post_before_the_period_is_closed() {
    let mut test = Harness::new();
    let period = Funder::Period { project: 1, period: 0 };
    let pool = Funder::Pool { project: 1, person: 9, period: 0 };
    test.send(Event::CarvePool { reply_to: to(), project: 1, person: 9, period: 0, budget: 100 });
    test.domain.funding.get_mut(&period).expect("period").numbers.spent = 3;
    let numbers = &mut test.domain.funding.get_mut(&pool).expect("pool").numbers;
    numbers.spent = 7;
    numbers.spent_below = 11;
    let out = test.send(Event::CloseProject { project: 1 });
    let mut posted = false;
    let mut closed_pool = false;
    let mut closed_period = false;
    for request in out {
        if let Request::Save { record } = request {
            if let Stored::ClosedLedger(row) = record {
                if row.funder == pool {
                    assert_eq!((row.numbers.spent, row.numbers.spent_below), (7, 11));
                    closed_pool = true;
                } else if row.funder == period {
                    assert!(posted);
                    assert_eq!(row.numbers.spent, 3);
                    assert_eq!(row.numbers.spent_below, row.numbers.reserved), (18, 0);
                    closed_period = true;
                }
            } else if let Stored::Ledger(row) = record {
                assert_eq!(row.funder, period);
                assert!(closed_pool);
                assert_eq!(row.numbers.spent_below, row.numbers.reserved), (18, 0);
                posted = true;
            }
        }
    }
    assert!(closed_period);
    assert!(test.domain.funding_records().is_empty());
}

#[test]
fn period_opening_requires_the_cores_project_gate() {
    let mut test = Harness::new();
    test.send(Event::CloseProject { project: 1 });
    assert!(test.domain.funding_records().is_empty());
    assert_eq!(test.send(Event::PeriodNext { project: 1, period: 1, next: None }).len(), 0);
    // OpenPeriod is an authorized allocation from the core, which owns deleted project identities.
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 100 });
    assert!(test.domain.funding(Funder::Period { project: 1, period: 1 }).is_some());
}

#[test]
#[should_panic(expected = "project closure follows its last live task")]
fn closing_a_project_with_a_live_task_asserts_the_parents_precondition() {
    let mut test = Harness::new();
    test.make([super::revisions::task(1, &[])].into());
    test.send(Event::CloseProject { project: 1 });
}
