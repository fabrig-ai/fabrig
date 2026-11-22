//! The fault ledger's records, intervals, identities and independent counters
//! (`skein/docs/design/testing/sweeps.md`, 3 and 6).

use alloc::collections::BTreeSet;
use skein_lib::Time;

use crate::faults::{Counter, FaultLedger, Injection, Kind, Struck};

const LOST_RUNNER: Kind = Kind { family: "forge", name: "lost runner" };
const RATE_WINDOW: Kind = Kind { family: "provider", name: "rate window" };
const START: Time = Time::from_nanos(5);
const STOP: Time = Time::from_nanos(9);

fn started(struck: Struck) -> (FaultLedger, Injection) {
    let mut ledger = FaultLedger::new();
    let entry = ledger.inject(LOST_RUNNER, START, struck);
    (ledger, entry)
}

fn runner() -> Struck {
    Struck { runner: Some("r".into()), gate_run: Some("7".into()), ..Struck::default() }
}

fn all_identities() -> Struck {
    Struck {
        account: Some("a".into()),
        runner: Some("r".into()),
        gate_run: Some("7".into()),
        candidate: Some("c".into()),
        base: Some("b".into()),
        request: Some("q".into()),
    }
}

#[test]
fn entry_keeps_given_kind_start_and_struck_identities() {
    let (ledger, entry) = started(runner());
    let injected = ledger.get(entry);
    assert_eq!(injected.kind, LOST_RUNNER);
    assert_eq!(injected.from, START);
    assert_eq!(injected.until, None);
    assert_eq!(injected.struck, runner());
    assert_eq!(ledger.entries().len(), 1, "one injection makes one retained entry");
    assert!(
        core::ptr::eq(ledger.entries().first().expect("one entry"), injected),
        "entries exposes the injected record"
    );
}

#[test]
fn stop_sets_the_closed_interval_end() {
    let (mut ledger, entry) = started(runner());
    assert_eq!(ledger.stop(entry, STOP), Ok(()));
    assert_eq!(ledger.get(entry).until, Some(STOP));
}

#[test]
fn second_stop_finds_the_entry_without_changing_it() {
    let (mut ledger, entry) = started(all_identities());
    ledger.stop(entry, STOP).expect("first stop");
    let finding = ledger.stop(entry, Time::from_nanos(10)).expect_err("a second stop is a finding");
    for fact in [
        "forge",
        "lost runner",
        "Time(5)",
        "Some(Time(9))",
        "account: Some(\"a\")",
        "runner: Some(\"r\")",
        "gate_run: Some(\"7\")",
        "candidate: Some(\"c\")",
        "base: Some(\"b\")",
        "request: Some(\"q\")",
        "already stopped",
    ] {
        assert!(finding.contains(fact), "finding names {fact}: {finding}");
    }
    assert_eq!(finding.lines().count(), 1, "a finding is one line");
    assert_eq!(ledger.get(entry).until, Some(STOP), "a refused stop leaves the interval alone");
}

#[test]
fn stop_before_start_is_a_finding_not_a_stop() {
    let (mut ledger, entry) = started(runner());
    let finding = ledger.stop(entry, Time::from_nanos(4)).expect_err("a fault cannot stop before it starts");
    for fact in [
        "forge",
        "lost runner",
        "Time(5)",
        "None",
        "runner: Some(\"r\")",
        "gate_run: Some(\"7\")",
        "stop at 4 ns precedes its start",
    ] {
        assert!(finding.contains(fact), "finding names {fact}: {finding}");
    }
    assert_eq!(finding.lines().count(), 1, "a finding is one line");
    assert_eq!(ledger.get(entry).until, None, "a refused stop leaves the fault in force");
}

#[test]
fn stopping_at_start_leaves_that_instant_in_force() {
    let (mut ledger, entry) = started(runner());
    ledger.stop(entry, START).expect("an instant's fault stops at its start");
    let injected = ledger.get(entry);
    for (at, expected) in [(4, false), (5, true), (6, false)] {
        assert_eq!(injected.in_force(Time::from_nanos(at)), expected, "instant fault at {at}");
    }
}

#[test]
fn in_force_includes_both_ends_of_the_interval() {
    let (mut ledger, entry) = started(runner());
    ledger.stop(entry, STOP).expect("stop");
    let injected = ledger.get(entry);
    for (at, expected) in [(4, false), (5, true), (7, true), (9, true), (10, false)] {
        assert_eq!(injected.in_force(Time::from_nanos(at)), expected, "closed interval at {at}");
    }
}

#[test]
fn unstopped_fault_is_in_force_at_every_later_time() {
    let (ledger, entry) = started(runner());
    for (at, expected) in [(4, false), (5, true), (10, true), (u64::MAX, true)] {
        assert_eq!(ledger.get(entry).in_force(Time::from_nanos(at)), expected, "open interval at {at}");
    }
}

#[test]
fn striking_filters_time_and_identity_on_simultaneous_faults() {
    let (mut ledger, account) = started(Struck { account: Some("a".into()), ..Struck::default() });
    ledger.inject(LOST_RUNNER, START, runner());
    let stopped = ledger.inject(RATE_WINDOW, START, Struck { account: Some("a".into()), ..Struck::default() });
    ledger.stop(stopped, STOP).expect("stopped before the query");
    ledger.inject(RATE_WINDOW, START, Struck { account: Some("other".into()), ..Struck::default() });
    ledger.inject(RATE_WINDOW, Time::from_nanos(11), Struck { account: Some("a".into()), ..Struck::default() });
    assert_eq!(
        ledger.striking(Time::from_nanos(10), &Struck { account: Some("a".into()), ..Struck::default() }),
        vec![account]
    );
}

#[test]
fn striking_keeps_fall_order_not_start_time_or_kind_order() {
    let mut ledger = FaultLedger::new();
    let first =
        ledger.inject(RATE_WINDOW, Time::from_nanos(7), Struck { account: Some("a".into()), ..Struck::default() });
    let second = ledger.inject(LOST_RUNNER, START, runner());
    assert_eq!(
        ledger.striking(STOP, &Struck { account: Some("a".into()), runner: Some("r".into()), ..Struck::default() }),
        vec![first, second]
    );
}

#[test]
fn striking_matches_each_field_only_when_both_name_it() {
    let (ledger, entry) = started(all_identities());
    for query in [
        Struck { account: Some("a".into()), ..Struck::default() },
        Struck { runner: Some("r".into()), ..Struck::default() },
        Struck { gate_run: Some("7".into()), ..Struck::default() },
        Struck { candidate: Some("c".into()), ..Struck::default() },
        Struck { base: Some("b".into()), ..Struck::default() },
        Struck { request: Some("q".into()), ..Struck::default() },
    ] {
        assert_eq!(ledger.striking(START, &query), vec![entry], "query {query:?}");
    }
    for query in [Struck::default(), Struck { account: Some("r".into()), ..Struck::default() }] {
        assert_eq!(ledger.striking(START, &query), vec![], "absent or cross-field identity {query:?}");
    }
}

#[test]
fn counters_are_independent_and_default_to_zero() {
    let (mut ledger, entry) = started(runner());
    ledger.count(entry, Counter::TaskTries);
    ledger.count(entry, Counter::TaskTries);
    ledger.count(entry, Counter::GateRetries);
    let injected = ledger.get(entry);
    for (counter, expected) in [
        (Counter::ProviderRetries, 0),
        (Counter::SessionRetries, 0),
        (Counter::TaskTries, 2),
        (Counter::GateRetries, 1),
        (Counter::Repairs, 0),
        (Counter::Asks, 0),
    ] {
        assert_eq!(injected.count(counter), expected, "counter {counter:?}");
    }
}

#[test]
fn each_counter_can_count_without_touching_another_entry() {
    let (mut ledger, untouched) = started(runner());
    let counted = ledger.inject(RATE_WINDOW, START, Struck::default());
    for counter in [
        Counter::ProviderRetries,
        Counter::SessionRetries,
        Counter::TaskTries,
        Counter::GateRetries,
        Counter::Repairs,
        Counter::Asks,
    ] {
        ledger.count(counted, counter);
        assert_eq!(ledger.get(counted).count(counter), 1, "counter {counter:?} records a transition");
        assert_eq!(ledger.get(untouched).count(counter), 0, "counter {counter:?} belongs to its entry");
    }
}

#[test]
fn fell_keeps_each_kind_once_including_stopped_faults() {
    let (mut ledger, entry) = started(runner());
    ledger.stop(entry, STOP).expect("a stopped kind still fell");
    ledger.inject(RATE_WINDOW, START, Struck::default());
    ledger.inject(RATE_WINDOW, STOP, Struck::default());
    assert_eq!(ledger.fell(), BTreeSet::from([LOST_RUNNER, RATE_WINDOW]));
}
