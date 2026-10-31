//! Time, deadlines and randomness (lib.md, 8).

use crate::{Deadlines, Duration, Rng, Time, Wall, earliest};

#[test]
fn arithmetic_is_checked_or_saturating() {
    let end = Time::from_nanos(u64::MAX);
    assert_eq!(end.checked_add(Duration::from_nanos(1)), None);
    assert_eq!(end.saturating_add(Duration::from_secs(1)), end);
    assert_eq!(Time::ZERO.saturating_since(end), Duration::ZERO);
    assert_eq!(Duration::from_secs(u64::MAX).as_nanos(), u64::MAX);
    assert_eq!(Duration::from_millis(3).as_nanos(), 3_000_000);
}

#[test]
fn wall_time_counts_from_the_epoch() {
    assert_eq!(Wall::EPOCH.as_nanos(), 0);
    let wall = Wall::from_nanos(1_700_000_000_999_999_999);
    assert_eq!(wall.as_secs(), 1_700_000_000);
    assert_eq!(wall.as_nanos(), 1_700_000_000_999_999_999);
    assert!(Wall::EPOCH < wall);
}

fn at(nanos: u64) -> Time {
    Time::from_nanos(nanos)
}

#[test]
fn the_earliest_of_two_deadlines_is_absent_only_when_both_are() {
    assert_eq!(earliest(None, None), None);
    assert_eq!(earliest(Some(at(5)), None), Some(at(5)));
    assert_eq!(earliest(None, Some(at(5))), Some(at(5)));
    assert_eq!(earliest(Some(at(5)), Some(at(9))), Some(at(5)));
    assert_eq!(earliest(Some(at(9)), Some(at(5))), Some(at(5)));
    assert_eq!(earliest(Some(at(7)), Some(at(7))), Some(at(7)), "equal times");
    assert_eq!(earliest(Some(at(u64::MAX)), Some(Time::ZERO)), Some(Time::ZERO));
}

#[test]
fn deadlines_fold_into_the_earliest_in_a_constant() {
    const NEXT: Option<Time> = earliest(earliest(None, Some(Time::from_nanos(30))), Some(Time::from_nanos(20)));
    assert_eq!(NEXT, Some(at(20)));
}

#[test]
fn timers_fire_in_order_once_due() {
    let mut timers = Deadlines::with_capacity(3);
    timers.arm('b', at(20)).expect("room");
    timers.arm('a', at(10)).expect("room");
    timers.arm('c', at(10)).expect("room");
    assert_eq!(timers.next(), Some(at(10)));
    assert_eq!(timers.expire(at(9)), None);
    assert_eq!(timers.expire(at(15)), Some('a'));
    assert_eq!(timers.expire(at(15)), Some('c'));
    assert_eq!(timers.expire(at(15)), None);
    assert_eq!(timers.expire(at(20)), Some('b'));
    assert!(timers.is_empty());
}

#[test]
fn rearming_moves_a_timer_and_cancelling_is_idempotent() {
    let mut timers = Deadlines::with_capacity(1);
    timers.arm('a', at(10)).expect("room");
    timers.arm('a', at(30)).expect("re-arming takes no new room");
    assert_eq!(timers.arm('b', at(5)), Err('b'));
    assert_eq!(timers.expire(at(10)), None);
    timers.cancel('a');
    timers.cancel('a');
    assert_eq!(timers.next(), None);
}

#[test]
fn the_same_seed_gives_the_same_stream() {
    let mut a = Rng::new(42);
    let mut b = Rng::new(42);
    for _ in 0_u32..100 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn draws_stay_in_range() {
    let mut rng = Rng::new(7);
    for _ in 0_u32..1000 {
        assert!(rng.below(10) < 10);
        let n = rng.between(5, 9);
        assert!((5_u64..=9).contains(&n));
    }
    assert_eq!(rng.below(0), 0);
    assert_eq!(rng.between(9, 5), 9_u64);
    let _: u64 = rng.between(0, u64::MAX);
    assert!(!rng.chance(0));
    assert!(rng.chance(1000));
}

#[test]
fn whole_milliseconds_round_down_without_overflow() {
    const WHOLE: u64 = Duration::from_nanos(1_999_999).as_millis();
    assert_eq!(WHOLE, 1);
    for (nanos, millis) in [(0, 0), (999_999, 0), (1_000_000, 1), (u64::MAX, 18_446_744_073_709)] {
        assert_eq!(Duration::from_nanos(nanos).as_millis(), millis);
    }
    assert_eq!(Duration::from_millis(123).as_millis(), 123);
}
