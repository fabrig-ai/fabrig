//! Closer reclamation (temper/docs/design/domain/forge/branches.md, 2).
use super::*;

fn closer(task: u64, goal: u64, done: bool) -> CloserRow {
    CloserRow {
        task,
        goal,
        closer: temper_engine_domain_forge_closer::Closer::new(),
        making: None,
        close_changes: Box::new([]),
        landing_changes: Box::new([]),
        done,
        relay: temper_engine_domain_forge_closer::relay::Relay::new(),
    }
}

#[test]
fn a_forgotten_goal_reclaims_its_completed_closer_rows() {
    let mut d = domain();
    let live = closer(1, 7, false);
    let other = closer(2, 8, true);
    outputs(&mut d, Event::Closer { row: live.clone() });
    outputs(&mut d, Event::Closer { row: other.clone() });
    for task in 100..100 + u64::from(LIMITS.tasks) * 2 {
        outputs(&mut d, Event::Closer { row: closer(task, 7, true) });
        let erased = outputs(&mut d, Event::ForgetProjection { goal: 7 });
        assert!(d.closer(task).is_none(), "completed closer {task} retained");
        assert!(erased.contains(&Request::Erase { key: Key::Closer(task) }));
        assert_eq!(d.closer(1), Some(&live));
        assert_eq!(d.closer(2), Some(&other));
    }
}

fn checking(d: &mut Domain, step: u64) {
    outputs(
        d,
        Event::CheckCloser {
            task: 3,
            step,
            repository: REPO,
            branch: b"g".as_slice().into(),
            base: b"main".as_slice().into(),
        },
    );
    fixtures::answer_next(d, Ok(skein_forge::api::Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(d, Ok(skein_forge::api::Answer::Commit(fixtures::BASE)));
}

fn checking_domain() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    outputs(&mut d, Event::Closer { row: closer(3, 7, false) });
    d
}

#[test]
fn closer_checks_the_reverse_ancestry_first_page() {
    use skein_forge::api::{Op, Read};
    use temper_engine_domain_forge_change::Status;
    for (result, status) in fixtures::ancestry_cases() {
        let mut d = checking_domain();
        checking(&mut d, 1);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
        fixtures::answer_once(&mut d, call, result);
        let contained = match status {
            Status::Passed => Some(true),
            Status::Failed => Some(false),
            Status::Unknown | Status::Pending => None,
        };
        assert_eq!(d.closer_checked(3, 1).expect("checked").ok(), contained);
    }
}

#[test]
fn unknown_closer_checks_double_their_wait_to_poll_max_and_known_resets() {
    use skein_forge::api::{Answer, CommitMessage, Error};
    for unknown in [Err(Error::Unavailable), Ok(Answer::Commits { commits: Box::new([]), more: true })] {
        for contained in [true, false] {
            let mut d = checking_domain();
            for (step, seconds) in [1, 2, 4, 8, 16, 30, 30].into_iter().enumerate() {
                d.offered_change(3);
                checking(&mut d, u64::try_from(step).unwrap());
                let (call, _) = fixtures::next_call(&mut d);
                fixtures::answer_once(&mut d, call, unknown.clone());
                assert_eq!(d.closer_checked(3, u64::try_from(step).unwrap()).expect("checked").ok(), None);
                assert_eq!(
                    d.next_wall_deadline(),
                    Some(Wall::from_nanos(env().wall.as_nanos() + seconds * 1_000_000_000))
                );
            }
            d.offered_change(3);
            checking(&mut d, 8);
            let commits: Box<[_]> = if contained {
                Box::new([])
            } else {
                Box::new([CommitMessage { id: fixtures::HEAD, parents: Box::new([]), message: Box::new([]) }])
            };
            fixtures::answer_next(&mut d, Ok(Answer::Commits { commits, more: false }));
            assert_eq!(d.closer_checked(3, 8), Some(Ok(contained)));
            assert_eq!(d.next_wall_deadline(), None);
            checking(&mut d, 9);
            let (call, _) = fixtures::next_call(&mut d);
            fixtures::answer_once(&mut d, call, unknown.clone());
            assert_eq!(d.next_wall_deadline(), Some(Wall::from_nanos(env().wall.as_nanos() + 1_000_000_000)));
        }
    }
}

#[test]
fn a_finished_closer_discards_its_containment_check() {
    let mut d = domain();
    outputs(&mut d, Event::Closer { row: closer(3, 7, false) });
    outputs(
        &mut d,
        Event::CheckCloser {
            task: 3,
            step: 1,
            repository: REPO,
            branch: b"g".as_slice().into(),
            base: b"main".as_slice().into(),
        },
    );
    assert!(!d.closing.reads.is_empty() || d.closer_checked(3, 1).is_some());
    outputs(&mut d, Event::Closer { row: closer(3, 7, true) });
    assert!(d.closing.reads.is_empty());
    assert_eq!(d.closer_checked(3, 1), None);
}
