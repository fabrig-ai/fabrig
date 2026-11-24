//! Faults the core cannot deliberately construct at a quiet point.
use jig_domain_core::{AtBound, Bound, Unbounded, Wait as Record, WaitKind, WaitObject};
use skein_lib::{List, Time, Wall};
use skein_world::inspect::{Moment, Wait, Wake};

const TASK: u64 = 12;
const DEPENDENCY: u64 = 4;
const OTHER: u64 = 5;
const FLOOR: Wall = Wall::from_nanos(100);
const SINCE: Wall = Wall::from_nanos(200);
const WALL: Wall = Wall::from_nanos(300);
const MOMENT: Moment = Moment { seed: 29, at: Time::from_nanos(11), iteration: 7 };
const BOUND: Bound = Bound::At { at: Wall::from_nanos(250), then: AtBound::Hold };

fn record(kind: WaitKind, object: WaitObject, since: Wall, bound: Bound) -> Record {
    Record { task: TASK, kind, object, since, bound, hears: None }
}

fn dependency(since: Wall, bound: Bound) -> Record {
    record(WaitKind::Dependency, WaitObject::Task(DEPENDENCY), since, bound)
}

fn dependencies(numbers: &[u64]) -> Wait {
    Wait {
        waiter: format!("task {TASK}"),
        kind: "on its dependencies".into(),
        wakes: numbers.iter().map(|number| Wake::Waiter(format!("task {number}"))).collect(),
        bound: None,
    }
}

fn judge(waits: &[Wait], records: &[Record]) -> Result<(), String> {
    jig_inspect::check_records(TASK, waits, records, false, MOMENT, WALL, |kind| {
        (kind == WaitKind::Dependency).then_some(FLOOR)
    })
}

#[test]
fn each_pending_dependency_needs_its_own_record() {
    assert_eq!(
        judge(&[dependencies(&[DEPENDENCY, OTHER])], &[dependency(SINCE, BOUND)]),
        Err("seed 29: at 11 ns, iteration 7: jig task 12: on its dependencies: missing dependency record for task 5"
            .into())
    );
}

#[test]
fn a_record_cannot_start_after_the_snapshot_wall() {
    let since = Wall::from_nanos(301);
    let finding = judge(&[], &[dependency(since, Bound::None(Unbounded::NoDeadline))]).expect_err("future since");
    assert_eq!(
        finding,
        "seed 29: at 11 ns, iteration 7: jig task 12: dependency Task(4): since 301 ns is after the moment's wall time 300 ns"
    );
}

#[test]
fn a_record_cannot_start_before_its_kinds_floor() {
    let finding = judge(&[], &[dependency(Wall::from_nanos(99), BOUND)]).expect_err("since before its floor");
    assert_eq!(
        finding,
        "seed 29: at 11 ns, iteration 7: jig task 12: dependency Task(4): since 99 ns is before its floor 100 ns"
    );
}

#[test]
fn a_record_cannot_be_bound_before_it_started() {
    let bound = Bound::At { at: Wall::from_nanos(199), then: AtBound::Hold };
    let finding = judge(&[], &[dependency(SINCE, bound)]).expect_err("bound before since");
    assert_eq!(
        finding,
        "seed 29: at 11 ns, iteration 7: jig task 12: dependency Task(4): bound 199 ns is before its since 200 ns"
    );
}

#[test]
fn a_whole_record_has_a_bound_or_reason() {
    // BOUND is already in the past: a record's bound is not an armed deadline.
    let records = [
        dependency(FLOOR, BOUND),
        dependency(SINCE, Bound::None(Unbounded::NoDeadline)),
        record(
            WaitKind::News,
            WaitObject::News { subscriptions: List::with_capacity(0), timer: None },
            WALL,
            Bound::None(Unbounded::Standing),
        ),
        record(WaitKind::Turn, WaitObject::Task(OTHER), WALL, Bound::None(Unbounded::Behind(OTHER))),
        record(
            WaitKind::Step,
            WaitObject::Procedure { step: 1, things: List::with_capacity(0) },
            WALL,
            Bound::None(Unbounded::OperatorHold),
        ),
    ];
    for record in records {
        assert_eq!(judge(&[], std::slice::from_ref(&record)), Ok(()), "{record:?}");
    }
}
