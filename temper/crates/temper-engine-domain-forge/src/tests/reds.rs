//! Goal-owned records and their release (domain/forge/reds.md, 8).
use super::*;
use skein_lib::List;

pub(crate) fn name(goal: u64) -> Name {
    Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([
            b"temper".as_slice().into(),
            b"7".as_slice().into(),
            format!("g{goal}").into_bytes().into(),
        ])),
    }
}
pub(crate) fn take(d: &mut Domain, limits: &Limits, goal: u64) {
    d.restore_begin(Wall::EPOCH);
    let hold = Hold { name: name(goal), task: goal, writer: None, design_writers: Box::new([]), drift: None };
    step(d, &env().with(*limits), Event::Restore { record: Stored::Hold(hold) }, &mut Queue::uncapped());
}
pub(crate) fn call(goal: u64, key: &[u8]) -> RecordRed {
    RecordRed {
        goal,
        key: key.into(),
        head: Box::new([4; 20]),
        why: b"carried by the base".as_slice().into(),
        by: Recorder::Task(goal),
        at: 99,
    }
}
fn red(goal: u64, key: &[u8], generation: u64) -> Red {
    let call = call(goal, key);
    Red { goal, key: call.key, head: call.head, why: call.why, by: call.by, at: call.at, generation }
}
fn fresh(limits: &Limits) -> Domain {
    let mut d = domain_with_limits(limits);
    d.restore_begin(Wall::EPOCH);
    step(
        &mut d,
        &env().with(*limits),
        Event::Restore { record: Stored::Repository(adopted()) },
        &mut Queue::uncapped(),
    );
    take(&mut d, limits, 7);
    d
}
fn rows(d: &Domain, goal: u64) -> Box<[Red]> {
    let mut rows = List::with_capacity(d.reds.len());
    for (_, row) in d.reds(goal) {
        rows.push(row.clone()).unwrap();
    }
    rows.into_boxed()
}
#[test]
fn a_goal_records_both_rows_and_replaces_whole_even_at_the_bound() {
    let limits = Limits { branch_reds: 1, ..LIMITS };
    let mut d = fresh(&limits);
    let mut out = Queue::uncapped();
    assert_eq!(
        d.record_red(&limits, call(7, b"test a::t x"), &mut out),
        Ok(RedRecorded { generation: 1, replaced: None })
    );
    assert_eq!(
        out.pop(),
        Some(Request::Save { record: Stored::Red(red(7, b"test a::t x", 1)), release: None, read_afresh: None })
    );
    assert_eq!(
        out.pop(),
        Some(Request::Save { record: Stored::RedNumber { last: 1 }, release: None, read_afresh: None })
    );
    assert!(out.is_empty());
    let replacement = RecordRed {
        head: Box::new([8; 32]),
        why: b"a new reason".as_slice().into(),
        by: Recorder::Party(3),
        at: 101,
        ..call(7, b"test a::t x")
    };
    assert_eq!(
        d.record_red(&limits, replacement.clone(), &mut out),
        Ok(RedRecorded { generation: 2, replaced: Some(red(7, b"test a::t x", 1)) })
    );
    let expected = Red {
        goal: 7,
        key: replacement.key,
        head: replacement.head,
        why: replacement.why,
        by: replacement.by,
        at: replacement.at,
        generation: 2,
    };
    assert_eq!(rows(&d, 7).as_ref(), core::slice::from_ref(&expected));
    assert_eq!(out.pop(), Some(Request::Save { record: Stored::Red(expected), release: None, read_afresh: None }));
    assert_eq!(
        out.pop(),
        Some(Request::Save { record: Stored::RedNumber { last: 2 }, release: None, read_afresh: None })
    );
    assert!(out.is_empty());
}
#[test]
fn a_full_goal_refuses_only_new_keys_without_saves_or_a_generation() {
    let limits = Limits { branch_reds: 1, ..LIMITS };
    let mut d = fresh(&limits);
    d.record_red(&limits, call(7, b"a"), &mut Queue::uncapped()).unwrap();
    let mut out = Queue::uncapped();
    assert_eq!(d.record_red(&limits, call(7, b"b"), &mut out), Err(RedRefusal::Full { bound: 1 }));
    assert!(out.is_empty());
    assert_eq!(rows(&d, 7).as_ref(), &[red(7, b"a", 1)]);
    assert_eq!(d.record_red(&limits, call(7, b"a"), &mut out).unwrap().generation, 2);
}
#[test]
fn only_a_task_holding_a_goal_branch_can_record_or_end() {
    let mut d = fresh(&LIMITS);
    let ordinary = Name { what: What::Branch(Box::new([b"main".as_slice().into()])), ..name(8) };
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: ordinary,
                task: 8,
                writer: None,
                design_writers: Box::new([]),
                drift: None,
            }),
        },
    );
    for task in [8, 9] {
        let mut out = Queue::uncapped();
        assert_eq!(d.record_red(&LIMITS, call(task, b"a"), &mut out), Err(RedRefusal::NoGoal));
        assert_eq!(d.end_red(task, b"a", &mut out), Err(RedRefusal::NoGoal));
        assert!(out.is_empty());
    }
}
#[test]
fn ending_erases_only_the_held_key_and_lists_are_goal_local_in_key_order() {
    let mut d = fresh(&LIMITS);
    take(&mut d, &LIMITS, 8);
    for (goal, key) in [(7, b"z"), (8, b"a"), (7, b"a"), (7, b"m")] {
        d.record_red(&LIMITS, call(goal, key), &mut Queue::uncapped()).unwrap();
    }
    assert_eq!(rows(&d, 7).as_ref(), &[red(7, b"a", 3), red(7, b"m", 4), red(7, b"z", 1)]);
    let mut out = Queue::uncapped();
    assert_eq!(d.end_red(7, b"a", &mut out), Ok(red(7, b"a", 3)));
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Red { goal: 7, key: b"a".as_slice().into() } }));
    assert_eq!(d.end_red(7, b"a", &mut out), Err(RedRefusal::NoSuch));
    assert_eq!(d.end_red(7, b"absent", &mut out), Err(RedRefusal::NoSuch));
    assert!(out.is_empty());
    assert_eq!(rows(&d, 8).as_ref(), &[red(8, b"a", 2)]);
    assert_eq!(rows(&d, u64::MAX).as_ref(), &[]);
}
#[test]
fn ending_a_goal_erases_its_reds_in_the_hold_erasure_decision() {
    for ending in [ReleaseEnding::Done, ReleaseEnding::Cancelled, ReleaseEnding::Failed] {
        let mut d = fresh(&LIMITS);
        take(&mut d, &LIMITS, 8);
        for (goal, key) in [(7, b"a"), (7, b"b"), (8, b"a")] {
            d.record_red(&LIMITS, call(goal, key), &mut Queue::uncapped()).unwrap();
        }
        let out = outputs(&mut d, Event::Release { task: 7, root: 8, ending, reason: None, entry: 22 });
        for key in [b"a", b"b"] {
            assert!(out.contains(&Request::Erase { key: Key::Red { goal: 7, key: key.as_slice().into() } }), "{out:?}");
        }
        assert!(out.contains(&Request::Erase { key: Key::Hold(name(7)) }), "{out:?}");
        assert_eq!(rows(&d, 7).as_ref(), &[]);
        assert_eq!(rows(&d, 8).as_ref(), &[red(8, b"a", 3)]);
        assert_eq!(d.record_red(&LIMITS, call(8, b"b"), &mut Queue::uncapped()).unwrap().generation, 4);
    }
}
#[test]
fn restoring_reds_and_the_counter_preserves_records_and_the_next_generation() {
    let mut d = fresh(&LIMITS);
    for row in [Stored::Red(red(7, b"a", 3)), Stored::RedNumber { last: 6 }] {
        assert!(outputs(&mut d, Event::Restore { record: row }).is_empty());
    }
    assert_eq!(d.restore_fit_failure(), None);
    assert_eq!(rows(&d, 7).as_ref(), &[red(7, b"a", 3)]);
    let result = d.record_red(&LIMITS, call(7, b"a"), &mut Queue::uncapped()).unwrap();
    assert_eq!(result, RedRecorded { generation: 7, replaced: Some(red(7, b"a", 3)) });
}
#[test]
fn restore_refuses_per_goal_total_and_counter_overflows() {
    for same_goal in [true, false] {
        let limits = Limits { branch_reds: 2, holds: if same_goal { 2 } else { 1 }, ..LIMITS };
        let mut d = domain_with_limits(&limits);
        d.restore_begin(Wall::EPOCH);
        let third = if same_goal { (7, b"c".as_slice()) } else { (8, b"a".as_slice()) };
        for (goal, key) in [(7, b"a".as_slice()), (7, b"b".as_slice()), third] {
            step(
                &mut d,
                &env().with(limits),
                Event::Restore { record: Stored::Red(red(goal, key, 1)) },
                &mut Queue::uncapped(),
            );
        }
        let failure = d.restore_fit_failure().unwrap();
        assert_eq!(failure.family, Some(RecordKind::Red));
        assert_eq!((failure.held, failure.additional, failure.limit), (2, 1, 2));
        assert_eq!(d.reds.len(), 2);
    }
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::RedNumber { last: 4 } });
    outputs(&mut d, Event::Restore { record: Stored::RedNumber { last: 5 } });
    assert_eq!(d.restore_fit_failure().unwrap().family, Some(RecordKind::RedNumber));
    assert_eq!(d.red_number, Some(4));
}
