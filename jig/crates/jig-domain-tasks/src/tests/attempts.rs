//! Attempt and hold history through the tasks step (domain/tasks/lifecycle.md, 12).
#![expect(clippy::disallowed_types, reason = "the existing step harness returns ordinary Rust request lists")]
use super::revisions::{Harness, task, to};
use crate::{Active, AttemptRow, Cause, End, EndKind, Event, Key, Phase, Request, Stored};
use skein_lib::Wall;

fn claim(h: &mut Harness, attempt: u64, at: u64) -> Vec<Request> {
    h.env.wall = Wall::from_nanos(at.checked_sub(1).unwrap());
    h.send(Event::Prepare { reply_to: to(), task: 1 });
    h.env.wall = Wall::from_nanos(at);
    h.send(Event::Claim {
        session_end: None,
        offered: crate::Reasons::ALL,
        reasons: crate::Reasons::FIRST,
        account: None,
        reply_to: to(),
        task: 1,
        attempt,
        resumed: false,
        budget: 10,
        writes: Box::new([]),
    })
}
fn start(h: &mut Harness, attempt: u64, at: u64) -> Vec<Request> {
    h.env.wall = Wall::from_nanos(at);
    h.send(Event::Started { task: 1, attempt })
}
fn end(h: &mut Harness, attempt: u64, at: u64, end: End) -> Vec<Request> {
    h.env.wall = Wall::from_nanos(at);
    h.send(Event::Activation { reply_to: to(), task: 1, attempt, end, saved: None, cause: Cause::Unpriced })
}
fn row(h: &Harness, attempt: u64) -> &AttemptRow {
    let Stored::Attempt(row) = &h.stored[&Key::Attempt { task: 1, attempt }] else { panic!("attempt row") };
    row
}
fn puts(out: &[Request]) -> usize {
    let mut count: usize = 0;
    for request in out {
        if let Request::Save { record: Stored::Attempt(_) } = request {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}

#[test]
fn phases_keep_two_attempts_not_milestones_and_restart_keeps_the_open_attempt() {
    for restart in [false, true] {
        let mut h = Harness::new();
        h.make([task(1, &[])].into());
        assert_eq!(h.domain.task(1).unwrap().milestone, 2);
        assert_eq!(puts(&claim(&mut h, 1, 10)), 1);
        let from = h.domain.task(1).unwrap().milestone;
        assert_eq!(
            row(&h, 1),
            &AttemptRow {
                task: 1,
                attempt: 1,
                claimed: Some(Wall::from_nanos(10)),
                started: None,
                ended: None,
                end: None,
                phases: None
            }
        );
        assert_eq!(puts(&start(&mut h, 1, 11)), 1);
        assert_eq!(row(&h, 1).started, Some(Wall::from_nanos(11)));
        let open = h.domain.task(1).unwrap().open_attempt.clone();
        if restart {
            h.restart();
        }
        assert_eq!(h.domain.task(1).unwrap().open_attempt, open);
        assert_eq!(puts(&end(&mut h, 1, 12, End::Parked)), 1);
        let idle = h.domain.task(1).unwrap();
        assert_eq!(idle.phase, Phase::Active(Active::Idle));
        assert_eq!(idle.open_attempt, None);
        assert_eq!(
            row(&h, 1),
            &AttemptRow {
                task: 1,
                attempt: 1,
                claimed: Some(Wall::from_nanos(10)),
                started: Some(Wall::from_nanos(11)),
                ended: Some(Wall::from_nanos(12)),
                end: Some(EndKind::Parked),
                phases: Some(u32::try_from(idle.milestone - from).unwrap())
            }
        );
        h.env.wall = Wall::from_nanos(13);
        h.send(Event::NothingAwaitedNotice { task: 1, message: 1 });
        assert_eq!(h.domain.task(1).unwrap().phase, Phase::Active(Active::Due));
        assert_eq!(puts(&claim(&mut h, 2, 20)), 1);
        let from = h.domain.task(1).unwrap().milestone;
        assert_eq!(puts(&start(&mut h, 2, 21)), 1);
        assert_eq!(
            puts(&end(
                &mut h,
                2,
                22,
                End::Finished { result: crate::TaskResult::Report { words: Box::new([]) }, cancel_delegates: false }
            )),
            1
        );
        assert_eq!(
            row(&h, 2),
            &AttemptRow {
                task: 1,
                attempt: 2,
                claimed: Some(Wall::from_nanos(20)),
                started: Some(Wall::from_nanos(21)),
                ended: Some(Wall::from_nanos(22)),
                end: Some(EndKind::Finished),
                phases: Some(2)
            }
        );
        h.send(Event::EffectsSettled { task: 1 });
        h.send(Event::Settled { task: 1 });
        let Stored::Ended(ended) = &h.stored[&Key::Ended(1)] else { panic!("ended") };
        assert_eq!(ended.milestone, from + 6);
        assert_eq!(ended.open_attempt, None);
        let mut attempts: u32 = 0;
        let mut lines = skein_lib::List::uncapped();
        for stored in h.stored.values() {
            if let Stored::Attempt(_) = stored {
                attempts = attempts.checked_add(1).unwrap();
            }
            if let Stored::Activity(crate::ActivityLine { line: crate::Line::Milestone { position }, .. }) = stored {
                lines.push(*position).unwrap();
            }
        }
        assert_eq!(attempts, 2);
        assert_eq!(lines.as_slice(), [1, ended.milestone]);
    }
}

#[test]
fn a_held_running_attempt_stays_open_until_its_was_moves() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    claim(&mut h, 1, 10);
    start(&mut h, 1, 11);
    let open = h.domain.task(1).unwrap().open_attempt.clone();
    h.env.wall = Wall::from_nanos(12);
    assert_eq!(puts(&h.send(Event::Hold { task: 1, why: crate::Hold::Stopped })), 0);
    let held = h.domain.task(1).unwrap().clone();
    assert_eq!(held.open_attempt, open);
    let Stored::Hold(first) = &h.stored[&Key::Hold { task: 1, position: held.hold }] else { panic!("hold") };
    assert_eq!(first.held.was, crate::Was::Active(Active::Running { attempt: 1 }));
    assert_eq!(first.at, Wall::from_nanos(12));
    h.route(1, crate::EscalationHolder::Person(9), 1);
    h.restart();
    assert_eq!(h.domain.task(1).unwrap().open_attempt, open);
    end(&mut h, 1, 13, End::Parked);
    let current = h.domain.task(1).unwrap();
    assert!(current.hold > held.hold);
    assert_eq!(current.open_attempt, None);
    let Stored::Hold(second) = &h.stored[&Key::Hold { task: 1, position: current.hold }] else { panic!("moved hold") };
    assert_eq!(second.held.was, crate::Was::Active(Active::Idle));
    assert_eq!(row(&h, 1).end, Some(EndKind::Parked));
    assert_eq!(row(&h, 1).ended, Some(Wall::from_nanos(13)));
}

#[test]
fn a_restored_attempt_without_its_open_record_closes_with_unknown_start_and_phases() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    claim(&mut h, 1, 10);
    start(&mut h, 1, 11);
    let Stored::Live(live) = h.stored.get_mut(&Key::Live(1)).unwrap() else { panic!("live") };
    live.open_attempt = None;
    h.restart();
    assert_eq!(
        h.domain.task(1).unwrap().open_attempt,
        Some(crate::OpenAttempt { attempt: 1, claimed: None, started: None, from: 0, end: None })
    );
    end(&mut h, 1, 12, End::Parked);
    assert_eq!(
        row(&h, 1),
        &AttemptRow {
            task: 1,
            attempt: 1,
            claimed: None,
            started: None,
            ended: Some(Wall::from_nanos(12)),
            end: Some(EndKind::Parked),
            phases: None
        }
    );
}
