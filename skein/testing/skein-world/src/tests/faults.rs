//! The fault ledger's records, intervals, identities and independent counters
//! (`skein/docs/design/testing/sweeps.md`, 3 and 6).

use alloc::collections::BTreeSet;
use skein_lib::Time;

use crate::faults::{Counter, FaultLedger, Injection, Kind, Struck};
use crate::inspect::{Entry, Inspection, Refused, Release, Table};
use crate::sweep::Sweep;

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
fn fallen_keeps_stopped_and_active_faults_in_fall_order_only_through_at() {
    let mut ledger = FaultLedger::new();
    let stopped = ledger.inject(LOST_RUNNER, Time::from_nanos(7), runner());
    ledger.stop(stopped, STOP).expect("stopped before the query");
    let active = ledger.inject(RATE_WINDOW, START, runner());
    let at = Time::from_nanos(10);
    let boundary = ledger.inject(LOST_RUNNER, at, runner());
    ledger.inject(LOST_RUNNER, Time::from_nanos(11), runner());
    ledger.inject(LOST_RUNNER, START, Struck { gate_run: Some("8".into()), ..Struck::default() });
    ledger.inject(LOST_RUNNER, START, Struck { runner: Some("7".into()), ..Struck::default() });
    assert_eq!(
        ledger.fallen(at, &Struck { gate_run: Some("7".into()), ..Struck::default() }),
        vec![stopped, active, boundary]
    );
    assert_eq!(ledger.fallen(at, &Struck::default()), vec![]);
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

const TABLE_BOUND: Kind = Kind { family: "f", name: "t" };
// The two-slot table from sweeps.md, section 6's saturation cases.
const TABLE_CAPACITY: u64 = 2;
const FULL: usize = 2;
const BELOW: usize = 1;

fn inspected_table(name: &'static str, capacity: u64, held: usize) -> Table {
    Table {
        name,
        capacity,
        entries: (0..held)
            .map(|key| Entry { key: key.to_string(), rule: "settlement", release: Release::Pending })
            .collect(),
        refused: Vec::new(),
    }
}

fn table_snapshot(component: &'static str, name: &'static str, capacity: u64, held: usize) -> Inspection {
    let mut inspection = Inspection::new(component, &[]);
    inspection.table(inspected_table(name, capacity, held));
    inspection
}

fn refused_snapshot(capacity: u64, held: usize) -> Inspection {
    let mut inspection = Inspection::new("component", &[]);
    let mut table = inspected_table("t", capacity, held);
    table.refused.push(Refused { key: "newcomer".into(), since: Time::from_nanos(1), bound: None });
    inspection.table(table);
    inspection
}

fn full_snapshot() -> Inspection {
    table_snapshot("component", "t", TABLE_CAPACITY, FULL)
}

fn ledger_at_snapshot(inspection: &Inspection) -> FaultLedger {
    let mut ledger = FaultLedger::new();
    ledger.saturation(&[TABLE_BOUND], &[inspection], Time::from_nanos(1)).expect("initial saturation");
    ledger
}

#[test]
fn saturation_tracks_full_intervals_and_records_each_kind_once() {
    let full = full_snapshot();
    let below = table_snapshot("component", "t", TABLE_CAPACITY, BELOW);
    let mut ledger = ledger_at_snapshot(&full);
    let first = ledger.entries().first().expect("full table fell");
    assert_eq!(first.kind, TABLE_BOUND);
    assert_eq!(first.from, Time::from_nanos(1));
    assert_eq!(first.until, None);
    assert_eq!(first.struck, Struck::default());
    assert_eq!(ledger.entries().len(), 1);

    ledger.saturation(&[TABLE_BOUND], &[&full], Time::from_nanos(2)).expect("still full");
    assert_eq!(ledger.entries().len(), 1, "remaining full keeps one entry");
    let first = ledger.entries().first().expect("still full");
    assert_eq!(first.from, Time::from_nanos(1));
    assert_eq!(first.until, None);

    ledger.saturation(&[TABLE_BOUND], &[&below], Time::from_nanos(3)).expect("below capacity");
    assert_eq!(ledger.entries().len(), 1);
    assert_eq!(ledger.entries().first().expect("stopped entry").until, Some(Time::from_nanos(3)));

    ledger.saturation(&[TABLE_BOUND], &[&full], Time::from_nanos(4)).expect("full again");
    assert_eq!(ledger.entries().len(), 2);
    let second = ledger.entries().last().expect("second interval");
    assert_eq!(second.kind, TABLE_BOUND);
    assert_eq!(second.from, Time::from_nanos(4));
    assert_eq!(second.until, None);
    assert_eq!(second.struck, Struck::default());
    assert_eq!(ledger.fell(), BTreeSet::from([TABLE_BOUND]));
}

#[test]
fn saturation_records_refused_newcomers_below_capacity() {
    let inspection = refused_snapshot(TABLE_CAPACITY, BELOW);
    let ledger = ledger_at_snapshot(&inspection);
    assert_eq!(ledger.fell(), BTreeSet::from([TABLE_BOUND]));
    assert_eq!(ledger.entries().first().expect("refused newcomer fell").until, None);
}

#[test]
fn saturation_ignores_missing_names_and_unbounded_tables() {
    let other = table_snapshot("component", "other", TABLE_CAPACITY, FULL);
    let unbounded = refused_snapshot(u64::MAX, FULL);
    for inspection in [other, unbounded, Inspection::new("empty", &[])] {
        assert!(ledger_at_snapshot(&inspection).entries().is_empty());
    }
}

#[test]
fn saturation_uses_any_component_holding_the_named_table() {
    let full = full_snapshot();
    let below = table_snapshot("other component", "t", TABLE_CAPACITY, BELOW);
    for inspections in [[&below, &full], [&full, &below]] {
        let mut ledger = FaultLedger::new();
        ledger.saturation(&[TABLE_BOUND], &inspections, Time::from_nanos(1)).expect("one component full");
        assert_eq!(ledger.fell(), BTreeSet::from([TABLE_BOUND]));
        ledger.saturation(&[TABLE_BOUND], &inspections, Time::from_nanos(2)).expect("one still full");
        assert_eq!(ledger.entries().len(), 1);
        assert_eq!(ledger.entries().first().expect("one component full").until, None);
        ledger.saturation(&[TABLE_BOUND], &[&below, &below], Time::from_nanos(3)).expect("both below");
        assert_eq!(ledger.entries().first().expect("both below").until, Some(Time::from_nanos(3)));
    }
}

#[test]
fn saturation_coverage_requires_a_seed_reaching_the_bound() {
    let full = full_snapshot();
    let below = table_snapshot("component", "t", TABLE_CAPACITY, BELOW);
    let mut sweep = Sweep::new("bounds");
    sweep.configure(TABLE_BOUND);
    sweep.ran(&ledger_at_snapshot(&below));
    assert_eq!(sweep.check_fell(), Err("sweep bounds: f/t never fell in 1 seeds".into()));
    sweep.ran(&ledger_at_snapshot(&full));
    assert_eq!(sweep.check_fell(), Ok(()));
}

#[test]
fn saturation_returns_a_refused_stop_without_changing_the_interval() {
    let mut ledger = ledger_at_snapshot(&full_snapshot());
    let below = table_snapshot("component", "t", TABLE_CAPACITY, BELOW);
    let finding = ledger
        .saturation(&[TABLE_BOUND], &[&below], Time::from_nanos(0))
        .expect_err("cannot stop before the table filled");
    assert!(finding.contains("stop at 0 ns precedes its start"), "{finding}");
    assert_eq!(ledger.entries().first().expect("refused stop").until, None);
}

#[test]
fn saturation_includes_zero_capacity_and_overfull_tables() {
    const OVERFULL: usize = 3; // The two slots and one extra entry.
    for (capacity, held) in [(0, 0), (TABLE_CAPACITY, OVERFULL)] {
        let inspection = table_snapshot("component", "t", capacity, held);
        assert_eq!(ledger_at_snapshot(&inspection).fell(), BTreeSet::from([TABLE_BOUND]));
    }
}
