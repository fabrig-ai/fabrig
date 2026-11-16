//! The shared execution clock (domain/forge/change/stalls.md, 2.2, 2.5).
use super::{Clock, Execution, HEAD, NEW, Reading, at, reading};
use skein_lib::{Duration, Wall};

#[test]
fn clock_counts_closed_and_open_intervals_only_to_the_state_seen() {
    let mut clock = None;
    for (seconds, execution, ran, waited, unknown) in [
        (10, Execution::Running, 0, 0, 0),
        (20, Execution::Waiting, 10, 0, 0),
        (40, Execution::Unknown, 10, 20, 0),
        (70, Execution::Running, 10, 20, 30),
    ] {
        let next = Clock::observe(clock, at(0), HEAD, reading(execution), at(seconds));
        assert_eq!(next.at, at(seconds));
        assert_eq!(next.seen, execution);
        assert_eq!(
            (next.ran, next.waited, next.unknown),
            (Duration::from_secs(ran), Duration::from_secs(waited), Duration::from_secs(unknown),)
        );
        let (open_ran, open_waited, open_unknown) = match execution {
            Execution::Running => (ran + 5, waited, unknown),
            Execution::Waiting => (ran, waited + 5, unknown),
            Execution::Unknown => (ran, waited, unknown + 5),
        };
        assert_eq!(
            (next.ran(at(seconds + 5)), next.waited(at(seconds + 5)), next.unknown(at(seconds + 5))),
            (Duration::from_secs(open_ran), Duration::from_secs(open_waited), Duration::from_secs(open_unknown),)
        );
        // Querying an open interval does not commit it twice.
        clock = Some(next);
    }
}

#[test]
fn clock_resets_for_new_wait_head_run_or_attempt_but_not_missing_identifiers() {
    let first = Clock::observe(None, at(0), HEAD, reading(Execution::Running), at(10));
    let counted = Clock::observe(Some(first), at(0), HEAD, reading(Execution::Waiting), at(20));
    for (prior, since, head, run, attempt) in [
        (None, at(0), HEAD, 7, 1),
        (Some(counted), at(1), HEAD, 7, 1),
        (Some(counted), at(0), NEW, 7, 1),
        (Some(counted), at(0), HEAD, 8, 1),
        (Some(counted), at(0), HEAD, 7, 2),
    ] {
        let reading = Reading { execution: Execution::Unknown, run, attempt };
        let fresh = Clock::observe(prior, since, head, reading, at(30));
        assert_eq!((fresh.ran, fresh.waited, fresh.unknown), (Duration::ZERO, Duration::ZERO, Duration::ZERO));
        assert_eq!(
            (fresh.since, fresh.head, fresh.run, fresh.attempt, fresh.at, fresh.seen),
            (since, head, run, attempt, at(30), Execution::Unknown)
        );
    }
    let absent = Reading { execution: Execution::Unknown, run: 0, attempt: 0 };
    let kept = Clock::observe(Some(counted), at(0), HEAD, absent, at(30));
    assert_eq!((kept.ran, kept.waited), (Duration::from_secs(10), Duration::from_secs(10)));
    assert_eq!((kept.run, kept.attempt), (7, 1));
    let kept = Clock::observe(Some(kept), at(0), HEAD, reading(Execution::Running), at(40));
    assert_eq!(
        (kept.ran, kept.waited, kept.unknown),
        (Duration::from_secs(10), Duration::from_secs(10), Duration::from_secs(10))
    );
}

#[test]
fn clock_saturates_durations_and_backwards_time() {
    let maximum = Duration::from_nanos(u64::MAX);
    for execution in [Execution::Unknown, Execution::Waiting, Execution::Running] {
        let clock = Clock {
            ran: maximum,
            waited: maximum,
            unknown: maximum,
            ..Clock::observe(None, at(0), HEAD, reading(execution), at(10))
        };
        assert_eq!((clock.ran(at(0)), clock.waited(at(0)), clock.unknown(at(0))), (maximum, maximum, maximum));
        let next = Clock::observe(Some(clock), at(0), HEAD, reading(execution), Wall::from_nanos(u64::MAX));
        assert_eq!((next.ran, next.waited, next.unknown), (maximum, maximum, maximum));
    }
    let clock = Clock::observe(None, at(0), HEAD, reading(Execution::Running), at(10));
    assert_eq!(clock.ran(at(0)), Duration::ZERO);
}
