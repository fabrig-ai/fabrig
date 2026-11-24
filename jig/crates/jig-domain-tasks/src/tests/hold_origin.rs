//! Public task entrances preserve immutable connector reference facts.

use super::revisions::{Harness, LIMITS, task, to};
use crate::{Active, Cause, Domain, End, Event, Hold, HoldSource, Key, Name, Party, Phase, Release, Request, Stored};
use alloc::boxed::Box;
use skein_lib::Wall;

fn resource(words: &[u8]) -> HoldSource {
    HoldSource::Resource(Name { connector: 7, path: Box::new([words.into()]) })
}
fn origin(phase: &Phase) -> Option<&HoldSource> {
    match phase {
        Phase::Held { source, .. } => source.as_deref(),
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => None,
    }
}

fn claim(h: &mut Harness, number: u64) {
    h.send(Event::Prepare { reply_to: to(), task: number });
    h.send(Event::Claim {
        fresh: None,
        session_end: None,
        offered: crate::Reasons::ALL,
        reasons: crate::Reasons::FIRST,
        account: None,
        reply_to: to(),
        task: number,
        attempt: 1,
        resumed: false,
        budget: 10,
        writes: Box::new([]),
    });
}

#[test]
fn hold_origin_survives_restart_terminal_and_release_in_exact_hold_rows() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    claim(&mut h, 1);
    h.env.wall = Wall::from_nanos(10);
    h.send(Event::HoldFrom { task: 1, why: Hold::Drift, source: Box::new(resource(b"original")) });
    let first = h.domain.task(1).unwrap().clone();
    h.send(Event::Hold { task: 1, why: Hold::Drift });
    h.send(Event::HoldFrom { task: 1, why: Hold::Drift, source: Box::new(resource(b"original")) });
    assert_eq!(h.domain.task(1), Some(&first), "duplicates cannot erase the source or move clocks");
    h.env.wall = Wall::from_nanos(12);
    h.send(Event::HoldFrom { task: 1, why: Hold::Drift, source: Box::new(resource(b"changed")) });
    assert_eq!(h.domain.task(1).unwrap().revisions.hold, first.revisions.hold + 1);
    assert_eq!(h.domain.task(1).unwrap().phase_since, h.env.wall);
    h.route(1, crate::EscalationHolder::Person(9), 1);
    h.restart();
    h.send(Event::Activation {
        reply_to: to(),
        task: 1,
        attempt: 1,
        end: End::Parked,
        saved: None,
        cause: Cause::Unpriced,
    });
    let row = h.domain.task(1).unwrap();
    assert_eq!(origin(&row.phase), Some(&resource(b"changed")));
    let position = row.milestone;
    h.send(Event::Release {
        reply_to: to(),
        by: Party::Person(9),
        operation: Release { task: 1, again: None, message: None },
    });
    assert_eq!(h.domain.task(1).unwrap().phase, Phase::Active(Active::Due));
    let Some(Stored::Hold(retained)) = h.stored.get(&Key::Hold { task: 1, position }) else {
        panic!("held row retained")
    };
    assert_eq!(retained.held.source.as_deref(), Some(&resource(b"changed")));
    let Some(Stored::Hold(retained)) = h.stored.get(&Key::Hold { task: 1, position: first.milestone }) else {
        panic!("original hold retained")
    };
    assert_eq!(
        Phase::Held { was: retained.held.was.clone(), why: retained.held.why, source: retained.held.source.clone() },
        first.phase
    );
}

#[test]
fn connector_accounts_are_nonempty_bounded_and_name_the_uncertain_entry() {
    let limits = LIMITS;
    for (size, valid) in [(0, false), (1, true), (512, true), (513, false)] {
        let source = HoldSource::Account { connector: 7, entry: Some(3), words: Box::from(&[b'x'; 513][..size]) };
        assert_eq!(crate::hold_origin::valid(&limits, Hold::Uncertain { entry: 3 }, Some(&source)), valid);
        assert!(!crate::hold_origin::valid(&limits, Hold::Uncertain { entry: 4 }, Some(&source)));
    }
}

#[test]
fn hold_origin_admission_checks_semantics_and_restore_checks_bytes() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    let before = h.domain.task(1).unwrap().clone();
    for (why, source) in [
        (Hold::Budget, Box::new(resource(b"wrong reason"))),
        (Hold::Drift, Box::new(resource(&[1; 65]))),
        (Hold::Uncertain { entry: 3 }, Box::new(HoldSource::Effect { connector: 7, entry: 4 })),
        (Hold::EffectFailed, Box::new(HoldSource::Effect { connector: 7, entry: 0 })),
    ] {
        h.send(Event::HoldFrom { task: 1, why, source: source.clone() });
        assert_eq!(h.domain.task(1), Some(&before));
        let mut broken = before.clone();
        broken.phase = Phase::Held { was: crate::Was::Active(Active::Due), why, source: Some(source) };
        broken.revisions.hold = 1;
        broken.escalation = crate::Escalation::Waiting {
            revision: 1,
            holder: crate::EscalationHolder::Person(9),
            entry: 1,
            since: Wall::EPOCH,
        };
        let mut cold = Harness::new();
        cold.domain = Domain::new(&LIMITS, crate::tests::KEPT, 91, 1, crate::tests::charters(), Box::new([0]));

        let mut refused = false;
        for request in cold.send(Event::Restore { record: Stored::Live(Box::new(broken)) }) {
            let Request::RestoreRefused { .. } = request else { continue };
            refused = true;
        }
        assert_eq!(refused, why == Hold::Drift, "only oversized resource bytes refuse");
    }
    h.send(Event::HoldFrom {
        task: 1,
        why: Hold::Uncertain { entry: 3 },
        source: Box::new(HoldSource::Effect { connector: 7, entry: 3 }),
    });
    h.route(1, crate::EscalationHolder::Person(9), 1);
    h.restart();
    assert_eq!(origin(&h.domain.task(1).unwrap().phase), Some(&HoldSource::Effect { connector: 7, entry: 3 }));
}

const WAITER: u64 = 7;
const INPUT: u64 = 6;
const HOLDER: u64 = 9;
const ESCALATION: u64 = 1;
const WORDS_AT_BOUND: usize = 512; // engine/contracts.md, 1
const WORDS: &[u8] = b"Waited 24 h for news since 2026-11-24T06:07Z: subscriptions 3 and 5.";

#[test]
fn wait_holds_raise_escalation_and_restore_before_release() {
    use crate::{Escalation, EscalationHolder, Was};
    let at_bound = [b'x'; WORDS_AT_BOUND];
    for why in [Hold::Waited, Hold::Deadline] {
        for (was, released, dependencies, words) in [
            (Was::Waiting, Phase::Waiting, &[INPUT][..], &at_bound[..]),
            (Was::Active(Active::Due), Phase::Active(Active::Due), &[][..], WORDS),
            (Was::Active(Active::Idle), Phase::Active(Active::Idle), &[][..], WORDS),
        ] {
            let mut h = Harness::new();
            h.make([task(INPUT, &[]), task(WAITER, dependencies)].into());
            if was == Was::Active(Active::Idle) {
                claim(&mut h, WAITER);
                h.send(Event::Activation {
                    reply_to: to(),
                    task: WAITER,
                    attempt: 1,
                    end: End::Parked,
                    saved: None,
                    cause: Cause::Unpriced,
                });
            }
            let source = Box::new(HoldSource::Wait { words: words.into() });
            h.send(Event::HoldFrom { task: WAITER, why, source: source.clone() });
            let held = Phase::Held { was, why, source: Some(source) };
            let row = h.domain.task(WAITER).unwrap();
            assert_eq!(row.phase, held);
            assert_eq!(row.escalation, Escalation::Routing { revision: row.revisions.hold });
            h.route(WAITER, EscalationHolder::Person(HOLDER), ESCALATION);
            h.restart();
            assert_eq!(h.domain.task(WAITER).unwrap().phase, held);
            h.send(Event::Release {
                reply_to: to(),
                by: Party::Person(HOLDER),
                operation: Release { task: WAITER, again: None, message: None },
            });
            assert_eq!(h.domain.task(WAITER).unwrap().phase, released);
        }
    }
}

#[test]
fn invalid_wait_sources_leave_the_task_and_store_unchanged() {
    let mut h = Harness::new();
    h.make([task(WAITER, &[])].into());
    let before = h.domain.task(WAITER).unwrap().clone();
    let stored = h.stored.clone();
    let oversized = [b'x'; WORDS_AT_BOUND + 1];
    for (why, words) in
        [(Hold::Budget, WORDS), (Hold::Waited, &[][..]), (Hold::Waited, &oversized[..]), (Hold::Waited, &[0xff][..])]
    {
        let out =
            h.send(Event::HoldFrom { task: WAITER, why, source: Box::new(HoldSource::Wait { words: words.into() }) });
        assert!(out.is_empty(), "{why:?}: {out:?}");
        assert_eq!(h.domain.task(WAITER), Some(&before));
        assert_eq!(h.stored, stored);
    }
}
