//! Independent accounting observations over funding trees.

use skein_lib::List;

use crate::tests_fixtures::{authority, rules};
use crate::{Numbers, carve, charge, left, run_spend, settle};

fn add(a: u64, b: u64) -> u64 {
    a.checked_add(b).unwrap()
}

fn multiply(a: u64, b: u64) -> u64 {
    a.checked_mul(b).unwrap()
}

fn remainder(a: u64, b: u64) -> u64 {
    a.checked_rem(b).unwrap()
}

fn fresh(budget: u64) -> Numbers {
    Numbers { budget, spent: 0, spent_below: 0, reserved: 0 }
}

#[test]
fn a_run_keeps_half_for_delegates_after_spend_and_reservations_and_before_the_cap() {
    let authority = authority();
    let rules = rules(authority.clone());
    for (numbers, expected) in [
        (fresh(100), 50),
        (fresh(301), 80),
        (fresh(7), 3),
        (Numbers { spent: 10, spent_below: 20, reserved: 30, ..fresh(101) }, 20),
        (Numbers { spent: 101, ..fresh(100) }, 0),
    ] {
        assert_eq!(run_spend(&rules, &authority, numbers), expected, "{numbers:?}");
    }
}

#[test]
fn a_delegating_run_takes_the_whole_when_half_is_at_or_below_the_minimum() {
    let authority = authority();
    let mut rules = rules(authority.clone());
    rules.minimum_run_spend = 40;
    for (available, expected) in [(80, 80), (50, 50), (81, 80), (82, 41), (300, 80)] {
        assert_eq!(run_spend(&rules, &authority, fresh(available)), expected, "{available} left");
    }
}

#[test]
fn a_run_with_no_delegation_keeps_all_available_funds_up_to_the_cap() {
    let given = authority();
    let mut rules = rules(given.clone());
    let mut no_kinds = given.clone();
    no_kinds.delegation.kinds = Box::new([]);
    let mut no_tasks = given.clone();
    no_tasks.delegation.tasks = 0;
    let mut no_depth = given;
    no_depth.delegation.depth = 0;
    for minimum in [1, 40] {
        rules.minimum_run_spend = minimum;
        for authority in [&no_kinds, &no_tasks, &no_depth] {
            for (available, expected) in [(100, 80), (50, 50), (81, 80), (82, 80)] {
                assert_eq!(
                    run_spend(&rules, authority, fresh(available)),
                    expected,
                    "minimum {minimum}, {available} left"
                );
            }
        }
    }
}

#[test]
fn generated_run_shares_fit_available_funding_and_follow_the_minimum_and_cap() {
    for seed in 0..64_u64 {
        let mut rng = skein_lib::Rng::new(seed);
        let given = authority();
        let mut rules = rules(given.clone());
        rules.maximum_run_spend = add(1, rng.below(100));
        rules.minimum_run_spend = rng.below(rules.maximum_run_spend);
        for budget in [rng.below(1001), u64::MAX] {
            let spent = rng.below(budget.saturating_add(1));
            let spent_below = rng.below(budget.saturating_sub(spent).saturating_add(1));
            let reserved = rng.below(budget.saturating_sub(spent).saturating_sub(spent_below).saturating_add(1));
            let numbers = Numbers { budget, spent, spent_below, reserved };
            let available = left(numbers);
            for delegates in [true, false] {
                let mut authority = given.clone();
                if !delegates {
                    authority.delegation.tasks = 0;
                }
                let share = run_spend(&rules, &authority, numbers);
                assert!(share <= available, "seed {seed}: {numbers:?}");
                assert!(share <= rules.maximum_run_spend, "seed {seed}: {numbers:?}");
                let half = available / 2;
                let expected = if delegates && half > rules.minimum_run_spend { half } else { available };
                assert_eq!(
                    share,
                    expected.min(rules.maximum_run_spend),
                    "seed {seed}: {numbers:?}, delegates {delegates}"
                );
                let charged = charge(numbers, share).expect("the run's full share fits its task's budget");
                assert!(add(add(charged.spent, charged.spent_below), charged.reserved) <= budget, "seed {seed}");
            }
        }
    }
}

#[test]
fn accounting_refuses_unreserved_spend_and_invalid_numbers() {
    let start = fresh(10);
    assert_eq!(carve(start, &[6, 5]), None, "a batch cannot exceed what is available");
    assert_eq!(carve(start, &[u64::MAX, 1]), None, "the sum of reservations must be representable");
    let reserved = carve(start, &[6, 4]).unwrap();
    assert_eq!(left(reserved), 0);
    assert_eq!(charge(reserved, 3), None, "reserved funding cannot also be charged");
    let charged = charge(carve(start, &[6]).unwrap(), 4).unwrap();
    assert_eq!(charged.spent, 4);
    assert_eq!(charged.reserved, 6);
    assert_eq!(left(charged), 0);
    assert_eq!(settle(start, fresh(1)), None);
    assert_eq!(settle(reserved, reserved), None);
    let huge = Numbers { spent: u64::MAX, reserved: u64::MAX, ..fresh(u64::MAX) };
    assert_eq!(left(huge), 0);
    assert_eq!(charge(huge, 1), None);
    let impossible = Numbers { spent: u64::MAX, spent_below: 1, ..fresh(u64::MAX) };
    assert_eq!(carve(impossible, &[]), None);
    assert_eq!(charge(impossible, 0), None);
    assert_eq!(settle(fresh(0), impossible), None);
    let parent = Numbers { spent: u64::MAX, reserved: 1, ..fresh(u64::MAX) };
    assert_eq!(settle(parent, charge(fresh(1), 1).unwrap()), None);
    assert_eq!(start, fresh(10));
}

#[test]
fn every_expense_is_counted_once_up_generated_funding_trees() {
    for seed in 0..64_u64 {
        let mut root = fresh(10_000);
        let mut expenses = List::with_capacity(17);
        let mut funded = List::with_capacity(4);
        for child in 0..4_u64 {
            let budget = add(100, remainder(add(multiply(seed, 13), multiply(child, 19)), 97));
            root = carve(root, &[budget]).unwrap();
            let mut budgets = List::with_capacity(3);
            for leaf in 0..3_u64 {
                let amount = add(10, remainder(add(add(seed, child), leaf), 13));
                budgets.push(amount).unwrap();
            }
            let mut node = carve(fresh(budget), budgets.as_slice()).unwrap();
            for (leaf, amount) in budgets.iter().enumerate() {
                let actual = remainder(
                    add(add(multiply(seed, 7), multiply(child, 3)), multiply(u64::try_from(leaf).unwrap(), 17)),
                    add(*amount, 1),
                );
                expenses.push(actual).unwrap();
                let charged = charge(fresh(*amount), actual).unwrap();
                node = settle(node, charged).unwrap();
            }
            let actual = remainder(add(multiply(seed, 11), child), add(left(node), 1));
            expenses.push(actual).unwrap();
            let charged = charge(node, actual).unwrap();
            node = charged;
            funded.push(node).unwrap();
        }
        for ended in &funded {
            root = settle(root, *ended).unwrap();
        }
        root = charge(root, 17).unwrap();
        expenses.push(17).unwrap();
        let mut expected = 0_u64;
        for expense in &expenses {
            expected = add(expected, *expense);
        }
        assert_eq!(add(root.spent, root.spent_below), expected, "seed {seed}");
        assert_eq!(root.reserved, 0, "seed {seed}");
        assert_eq!(left(root), 10_000_u64.saturating_sub(expected), "seed {seed}");
    }
}

#[test]
fn a_new_period_does_not_clear_the_reservations_of_its_predecessor() {
    let old = carve(fresh(100), &[80]).unwrap();
    let current = fresh(100);
    assert_eq!(left(old), 20);
    assert_eq!(left(current), 100);
    let ended = charge(fresh(80), 35).unwrap();
    let settled = settle(old, ended).unwrap();
    assert_eq!(settled.spent_below, 35);
    assert_eq!(left(settled), 65);
    assert_eq!(current, fresh(100));
    assert_eq!(charge(fresh(80), 110), None);
    assert_eq!(current.reserved, 0);
}

#[test]
fn generated_open_maxima_settle_without_losing_or_reusing_spend() {
    for seed in 0..64_u64 {
        let mut maxima = [0_u64; 5];
        for (index, maximum) in maxima.iter_mut().enumerate() {
            *maximum = 12 + remainder(seed * 7 + u64::try_from(index).unwrap() * 13, 23);
        }
        let mut funder = carve(fresh(200), &maxima).unwrap();
        let mut expected = 0_u64;
        let mut open = 0_u64;
        for maximum in maxima {
            open = add(open, maximum);
        }
        for index in [3, 0, 4, 1, 2] {
            let maximum = maxima[index];
            let actual = remainder(seed * 11 + u64::try_from(index).unwrap() * 17, maximum + 1);
            let completion = charge(fresh(maximum), actual).unwrap();
            funder = settle(funder, completion).unwrap();
            expected += actual;
            open -= maximum;
            assert_eq!(funder.spent_below, expected, "seed {seed}, completion {index}");
            assert_eq!(funder.reserved, open, "seed {seed}, completion {index}");
            assert_eq!(left(funder), 200 - expected - open, "seed {seed}, completion {index}");
            assert_eq!(charge(funder, left(funder) + 1), None, "an open maximum cannot fund another charge");
        }
    }
}

#[test]
fn generated_standing_periods_keep_live_delegates_with_their_original_funder() {
    for seed in 0..64_u64 {
        let old_actual = remainder(seed * 7, 21);
        let new_actual = remainder(seed * 13, 21);
        let mut old_period = carve(fresh(100), &[60]).unwrap();
        let mut old_standing = carve(fresh(60), &[20]).unwrap();
        let old_delegate = charge(fresh(20), old_actual).unwrap();

        // A renewed allotment is funded by the new period while the old delegate stays open.
        let mut new_period = carve(fresh(100), &[60]).unwrap();
        let new_standing = settle(carve(fresh(60), &[20]).unwrap(), charge(fresh(20), new_actual).unwrap()).unwrap();
        new_period = settle(new_period, new_standing).unwrap();
        assert_eq!(old_period.reserved, 60, "new settlement cannot release the old period");
        assert_eq!(old_standing.reserved, 20, "old delegate remains live");
        assert_eq!(new_period.spent_below, new_actual);
        assert_eq!(new_period.reserved, 0);

        old_standing = settle(old_standing, old_delegate).unwrap();
        old_period = settle(old_period, old_standing).unwrap();
        assert_eq!(old_period.spent_below, old_actual);
        assert_eq!(old_period.reserved, 0);
        assert_eq!(new_period.spent_below, new_actual, "late old settlement cannot debit the new period");
    }
}
