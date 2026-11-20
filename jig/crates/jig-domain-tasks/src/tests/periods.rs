//! Period inputs and their durable output records, through the child entry points.
use super::revisions::{Harness, LIMITS, task, to};
use crate::{Domain, Event, Executor, Funder, Key, PeriodTimes, PersonAddress, Request, Stored, TaskResult, step};
use skein_lib::{Queue, Wall};

#[test]
fn supplied_period_boundaries_preserve_the_opening_wall_across_duplicates_and_restart() {
    let mut test = Harness::new();
    test.env.wall = Wall::from_nanos(1200);
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 100 });
    let source = Funder::Period { project: 1, period: 1 };
    let first = PeriodTimes { began: test.env.wall, next: None };
    assert_eq!(test.domain.funding(source).unwrap().period, Some(first));
    let next = Some(Wall::from_nanos(1100));
    test.env.wall = Wall::from_nanos(900);
    let out = test.send(Event::PeriodNext { project: 1, period: 1, next });
    let mut saved = false;
    for request in out {
        if let Request::Save { record: Stored::Ledger(row) } = request {
            assert_eq!(row.funder, source);
            assert_eq!(row.period, Some(PeriodTimes { began: first.began, next }));
            saved = true;
        }
    }
    assert!(saved);
    assert_eq!(test.send(Event::PeriodNext { project: 1, period: 1, next }).len(), 0);
    test.restart();
    assert_eq!(test.domain.funding(source).unwrap().period, Some(PeriodTimes { began: first.began, next }));
    test.send(Event::PeriodNext { project: 1, period: 1, next: None });
    assert_eq!(test.domain.funding(source).unwrap().period, Some(first));
    test.env.wall = Wall::from_nanos(800);
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 2, budget: 100 });
    assert_eq!(test.send(Event::PeriodNext { project: 1, period: 1, next }).len(), 0);
    assert_eq!(test.send(Event::PeriodNext { project: 9, period: 2, next }).len(), 0);
    test.restart();
    assert_eq!(
        test.domain.funding(Funder::Period { project: 1, period: 2 }).unwrap().period,
        Some(PeriodTimes { began: Wall::from_nanos(800), next: None })
    );
}

#[test]
fn restoring_period_facts_does_not_repeat_funding_shape_admission() {
    let test = Harness::new();
    let original = *test.domain.funding(Funder::Period { project: 1, period: 0 }).unwrap();
    let mut without_opening = original;
    without_opening.period = None;
    let mut pool_with_opening = original;
    pool_with_opening.funder = Funder::Pool { project: 1, period: 0, person: 9 };
    pool_with_opening.parent = Some(original.funder);
    for row in [without_opening, pool_with_opening] {
        let mut domain = Domain::new(&LIMITS, crate::tests::KEPT, 91, 1, crate::tests::charters(), Box::new([0]));
        let mut out = Queue::uncapped();
        step(&mut domain, &test.env, Event::Restore { record: Stored::Ledger(row) }, &mut out);
        assert!(out.is_empty());
        assert_eq!(domain.funding(row.funder), Some(&row));
    }
}

#[test]
fn a_closed_period_leaves_memory_and_saves_under_closed_ledger() {
    let mut test = Harness::new();
    let old = Funder::Period { project: 1, period: 0 };
    let out = test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 100 });
    assert!(test.domain.funding(old).is_none());
    assert!(!test.stored.contains_key(&Key::Ledger(old)));
    let mut erased = false;
    for request in out {
        if let Request::Erase { key: Key::Ledger(source) } = request {
            erased |= source == old;
        }
    }
    assert!(erased);
    let Some(Stored::ClosedLedger(row)) = test.stored.get(&Key::ClosedLedger(old)) else {
        panic!("the closed period is history")
    };
    assert!(row.closed);
    assert_eq!(row.funder, old);
}

fn zero_budget_person(test: &mut Harness, funder: Funder) {
    let mut person = task(7, &[]);
    person.executor = Executor::Person(PersonAddress::Person(10));
    person.numbers.budget = 0;
    person.authority.budget.spend = 0;
    person.funder = funder;
    test.make([person].into());
}

fn end_person(test: &mut Harness) -> Box<[Request]> {
    let mut out = test.send(Event::AnswerPerson {
        reply_to: to(),
        task: 7,
        person: 10,
        result: TaskResult::Report { words: Box::new([1]) },
    });
    out.extend(test.send(Event::EffectsSettled { task: 7 }));
    out.extend(test.send(Event::Settled { task: 7 }));
    assert!(test.domain.task(7).is_none());
    assert!(test.stored.contains_key(&Key::Ended(7)));
    out.into_boxed_slice()
}

fn assert_ledger_untouched(out: &[Request], funder: Funder) {
    for request in out {
        if let Request::Save { record } = request {
            assert_ne!(record.key(), Key::Ledger(funder));
            assert_ne!(record.key(), Key::ClosedLedger(funder));
        }
        if let Request::Erase { key } = request {
            assert_ne!(*key, Key::Ledger(funder));
            assert_ne!(*key, Key::ClosedLedger(funder));
        }
    }
}

#[test]
fn zero_budget_person_tasks_outlive_period_and_pool_ledgers() {
    for old in [Funder::Period { project: 1, period: 0 }, Funder::Pool { project: 1, person: 9, period: 0 }] {
        let mut test = Harness::new();
        test.send(Event::CarvePool { reply_to: to(), project: 1, person: 9, period: 0, budget: 40 });
        zero_budget_person(&mut test, old);
        test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 100 });
        assert!(test.domain.funding(old).is_none());
        let closed = test.stored.get(&Key::ClosedLedger(old)).expect("closed source history").clone();
        assert!(!test.stored.contains_key(&Key::Ledger(old)));
        test.restart();
        assert!(test.domain.ready());
        assert_eq!(test.domain.task(7).expect("person still live").funder, old);
        let current = Funder::Period { project: 1, period: 1 };
        let before = *test.domain.funding(current).expect("current period");
        let out = end_person(&mut test);
        assert_ledger_untouched(&out, old);
        assert_ledger_untouched(&out, current);
        assert_eq!(test.domain.funding(current), Some(&before));
        assert_eq!(test.stored.get(&Key::ClosedLedger(old)), Some(&closed));
    }
}

#[test]
fn a_closed_ledgers_task_posts_its_total_to_the_newest_live_project_period() {
    let mut test = Harness::new();
    let old = Funder::Period { project: 1, period: 0 };
    zero_budget_person(&mut test, old);
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 1000 });
    let mut keeper = task(8, &[]);
    keeper.funder = Funder::Period { project: 1, period: 1 };
    test.make([keeper].into());
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 2, budget: 1000 });
    test.send(Event::OpenPeriod { reply_to: to(), project: 2, period: 9, budget: 1000 });
    let Some(Stored::Live(person)) = test.stored.get_mut(&Key::Live(7)) else { panic!("live person record") };
    person.numbers.spent = 3;
    person.numbers.spent_below = 4;
    test.restart();
    let current = Funder::Period { project: 1, period: 2 };
    let mut expected = *test.domain.funding(current).expect("newest live period");
    expected.numbers.spent_below = expected.numbers.spent_below.checked_add(7).expect("test spend fits");
    let older = Funder::Period { project: 1, period: 1 };
    let other = Funder::Period { project: 2, period: 9 };
    let before_older = *test.domain.funding(older).expect("older live period");
    let before_other = *test.domain.funding(other).expect("other project's period");
    let out = end_person(&mut test);
    assert_ledger_untouched(&out, old);
    assert_ledger_untouched(&out, older);
    assert_ledger_untouched(&out, other);
    assert_eq!(test.domain.funding(older), Some(&before_older));
    assert_eq!(test.domain.funding(other), Some(&before_other));
    assert_eq!(test.domain.funding(current), Some(&expected));
    assert_eq!(test.stored.get(&Key::Ledger(current)), Some(&Stored::Ledger(expected)));
    let mut saves = 0_u32;
    for request in out {
        if let Request::Save { record: Stored::Ledger(row) } = request {
            assert_eq!(row, expected);
            saves = saves.checked_add(1).expect("test saves fit");
        }
    }
    assert_eq!(saves, 1);
}

#[test]
fn the_newest_period_stays_live_after_its_pools_close() {
    let mut test = Harness::new();
    let pool = Funder::Pool { project: 1, person: 9, period: 0 };
    test.send(Event::CarvePool { reply_to: to(), project: 1, person: 9, period: 0, budget: 40 });
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 100 });
    assert!(test.domain.funding(pool).is_none());
    assert!(test.stored.contains_key(&Key::ClosedLedger(pool)));
    let newest = test.domain.funding(Funder::Period { project: 1, period: 1 }).unwrap();
    assert!(!newest.closed);
    assert_eq!(newest.numbers.reserved, 0);
}

#[test]
fn a_period_raise_keeps_its_spending_reservations_times_and_made() {
    let mut test = Harness::new();
    let source = Funder::Period { project: 1, period: 0 };
    let row = test.domain.funding.get_mut(&source).unwrap();
    row.numbers = crate::Numbers { budget: 100, spent: 10, spent_below: 20, reserved: 30 };
    row.made = 4;
    row.period = Some(PeriodTimes { began: Wall::from_nanos(100), next: Some(Wall::from_nanos(200)) });
    let mut expected = *row;
    expected.numbers.budget = 200;
    let out = test.send(Event::ResizePeriod { reply_to: to(), project: 1, period: 0, budget: 200 });
    assert_eq!(test.domain.funding(source), Some(&expected));
    assert_eq!(
        out.as_slice(),
        &[Request::Save { record: Stored::Ledger(expected) }, Request::Done { reply_to: to() },]
    );
}

#[test]
fn a_period_resize_admits_exactly_what_is_used() {
    let mut test = Harness::new();
    let source = Funder::Period { project: 1, period: 0 };
    test.domain.funding.get_mut(&source).unwrap().numbers =
        crate::Numbers { budget: 100, spent: 10, spent_below: 20, reserved: 30 };
    let out = test.send(Event::ResizePeriod { reply_to: to(), project: 1, period: 0, budget: 60 });
    assert_eq!(test.domain.funding(source).unwrap().numbers.budget, 60);
    assert_eq!(
        out.as_slice(),
        &[
            Request::Save { record: Stored::Ledger(*test.domain.funding(source).unwrap()) },
            Request::Done { reply_to: to() },
        ]
    );
    let before = *test.domain.funding(source).unwrap();
    let out = test.send(Event::ResizePeriod { reply_to: to(), project: 1, period: 0, budget: 59 });
    assert_eq!(
        out.as_slice(),
        &[Request::Refused { reply_to: to(), problem: crate::Problem::new(None, crate::Refusal::Funding) }]
    );
    assert_eq!(test.domain.funding(source), Some(&before));
}

#[test]
fn an_older_period_cannot_be_resized() {
    let mut test = Harness::new();
    test.make([task(1, &[])].into());
    test.send(Event::OpenPeriod { reply_to: to(), project: 1, period: 1, budget: 100 });
    let source = Funder::Period { project: 1, period: 0 };
    let before = *test.domain.funding(source).unwrap();
    let out = test.send(Event::ResizePeriod { reply_to: to(), project: 1, period: 0, budget: 20_000 });
    assert_eq!(
        out.as_slice(),
        &[Request::Refused { reply_to: to(), problem: crate::Problem::new(None, crate::Refusal::Funding) }]
    );
    assert_eq!(test.domain.funding(source), Some(&before));
}

#[test]
fn an_unopened_period_cannot_be_resized() {
    let mut test = Harness::new();
    let before = *test.domain.funding(Funder::Period { project: 1, period: 0 }).unwrap();
    let out = test.send(Event::ResizePeriod { reply_to: to(), project: 1, period: 9, budget: 100 });
    assert_eq!(
        out.as_slice(),
        &[Request::Refused { reply_to: to(), problem: crate::Problem::new(None, crate::Refusal::Unknown) }]
    );
    assert!(test.domain.funding(Funder::Period { project: 1, period: 9 }).is_none());
    assert_eq!(test.domain.funding(before.funder), Some(&before));
}

#[test]
fn a_period_resize_before_ready_is_refused() {
    let mut test = Harness::new();
    test.domain = Domain::new(&LIMITS, crate::tests::KEPT, 91, 1, crate::tests::charters(), Box::new([0]));
    let out = test.send(Event::ResizePeriod { reply_to: to(), project: 1, period: 0, budget: 100 });
    assert_eq!(
        out.as_slice(),
        &[Request::Refused { reply_to: to(), problem: crate::Problem::new(None, crate::Refusal::NotReady) }]
    );
    assert!(test.domain.funding(Funder::Period { project: 1, period: 0 }).is_none());
}

#[test]
fn cold_restore_retains_period_facts_without_rejudging_family_shape() {
    let test = Harness::new();
    let mut row = *test.domain.funding(Funder::Period { project: 1, period: 0 }).unwrap();
    let original = row;
    for invalid in [false, true] {
        row = original;
        if invalid {
            row.funder = Funder::Pool { project: 1, period: 0, person: 9 };
            row.parent = Some(original.funder);
        } else {
            row.period = None;
        }
        let mut domain = Domain::new(&LIMITS, crate::tests::KEPT, 91, 1, crate::tests::charters(), Box::new([0]));
        let mut out = Queue::uncapped();
        step(&mut domain, &test.env, Event::Restore { record: Stored::Ledger(row) }, &mut out);
        let mut refused = false;
        while let Some(request) = out.pop() {
            if let Request::RestoreRefused { .. } = request {
                refused = true;
            }
        }
        assert!(!refused);
        assert_eq!(domain.funding(row.funder), Some(&row));
    }
}
