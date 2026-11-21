//! `CatchUp` starts both halves; the outbox lane alone orders writes.
use super::*;
use skein_forge::api::{Answer, Error, Op, Read, Write};
use skein_forge_client::{Condition, Entry, RecoveryClock};

fn entry(number: u64) -> Entry {
    let mut entry = held_issue();
    entry.number = number;
    entry.effect.write =
        Write::CreateBranch { branch: format!("entry-{number}").into_bytes().into(), commit: fixtures::HEAD };
    entry
}

fn restore(entries: &[(u64, bool)]) -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for (number, held) in entries {
        outputs(&mut d, Event::Restore { record: Stored::Entry { entry: entry(*number), held: *held } });
    }
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Names {
                task: 7,
                resources: Box::new([Name {
                    forge: REPO.forge,
                    repository: REPO.repository,
                    what: What::Branch(Box::new([b"watched".as_slice().into()])),
                }]),
            },
        },
    );
    d
}

fn calls(d: &mut Domain) -> (Token, Token) {
    let (first, op) = fixtures::next_call(d);
    let (second, other) = fixtures::next_call(d);
    let fresh = Op::Read(Read::Branch { branch: b"watched".as_slice().into() });
    let find = Op::Read(Read::Branch { branch: b"entry-11".as_slice().into() });
    if op == fresh {
        assert_eq!(other, find);
        (first, second)
    } else {
        assert_eq!(op, find);
        assert_eq!(other, fresh);
        (second, first)
    }
}

#[test]
fn catch_up_hands_restored_entries_in_number_order_before_kept_entries() {
    let mut d = restore(&[(12, false), (13, true), (11, false)]);
    let mut landing = entry(14);
    landing.effect.write = Write::Merge { number: 1, head: fixtures::HEAD, how: skein_forge::api::MergeHow::Style };
    landing.effect.condition = Condition::Merge { base: b"main".as_slice().into() };
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry: landing, held: false } });
    d.landing.held = true;
    outputs(&mut d, Event::Restored { clock: RecoveryClock::Monotonic });
    let mut kept = entry(15);
    kept.start = None;
    kept.attempt = None;
    outputs(&mut d, Event::Enqueue { entry: kept.clone() });
    outputs(&mut d, Event::Committed { entry: 15 });
    let mut uncommitted = kept.clone();
    uncommitted.number = 16;
    outputs(&mut d, Event::Enqueue { entry: uncommitted });
    assert!(ready(&mut d).is_empty(), "no handoff before CatchUp");
    let start = outputs(&mut d, Event::CatchUp);
    assert!(!start.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }), "{start:?}");
    assert!(!start.contains(&Request::RestartDone { stage: RestartStage::Settled }), "{start:?}");
    let (fresh, first) = calls(&mut d);
    let looked = fixtures::answer(&mut d, first, Ok(Answer::Commit(fixtures::HEAD)));
    assert!(!looked.contains(&Request::RestartDone { stage: RestartStage::Settled }));
    let (second, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"entry-12".as_slice().into() }));
    let looked = fixtures::answer(&mut d, second, Ok(Answer::Commit(fixtures::HEAD)));
    assert!(
        looked.contains(&Request::RestartDone { stage: RestartStage::Settled }),
        "all eligible restored entries handed before SettleOutbox"
    );
    assert!(!d.fresh_reads_in(), "the fresh read is still out");
    fixtures::answer_next(&mut d, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
    let (write, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(kept.effect.write));
    fixtures::answer(&mut d, write, Ok(Answer::Branch(skein_forge::api::BranchCreation::Created)));
    assert!(ready(&mut d).is_empty(), "held, held landing and uncommitted entries stay out");
    let done = fixtures::answer(&mut d, fresh, Ok(Answer::Commit(fixtures::HEAD)));
    assert!(done.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
}

#[test]
fn catching_up_reports_can_finish_in_either_order() {
    for fresh_first in [false, true] {
        let mut d = restore(&[(11, false)]);
        outputs(&mut d, Event::Restored { clock: RecoveryClock::Monotonic });
        let start = outputs(&mut d, Event::CatchUp);
        assert!(!start.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }), "{start:?}");
        assert!(!start.contains(&Request::RestartDone { stage: RestartStage::Settled }), "{start:?}");
        assert!(d.room.ready);
        let (fresh, find) = calls(&mut d);
        let (first, last, stage, other) = if fresh_first {
            (fresh, find, RestartStage::ReadAfresh, RestartStage::Settled)
        } else {
            (find, fresh, RestartStage::Settled, RestartStage::ReadAfresh)
        };
        if fresh_first {
            let failed = fixtures::answer(&mut d, find, Err(Error::Unavailable));
            assert!(!failed.contains(&Request::RestartDone { stage: RestartStage::Settled }));
        }
        let done = fixtures::answer(&mut d, first, Ok(Answer::Commit(fixtures::HEAD)));
        assert!(done.contains(&Request::RestartDone { stage }));
        assert!(!done.contains(&Request::RestartDone { stage: other }));
        let mut clock = env();
        let last = if fresh_first {
            clock.now = d.next_deadline().expect("retrying find");
            clock.wall = Wall::from_nanos(clock.now.as_nanos());
            let mut out = Queue::uncapped();
            fire(&mut d, &clock, &mut out);
            fixtures::next_at(&mut d, &clock).0
        } else {
            last
        };
        let done = fixtures::answer_at(&mut d, &clock, last, Ok(Answer::Commit(fixtures::HEAD)));
        let mut reported = false;
        for request in &done {
            reported |= *request == Request::RestartDone { stage: other };
            assert_ne!(*request, Request::RestartDone { stage });
        }
        assert!(reported);
    }
}
