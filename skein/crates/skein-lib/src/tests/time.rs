//! Time, deadlines and randomness (runtime/lib.md, 8).

use crate::{Civil, Deadlines, Duration, Rng, Time, Wall, earliest};

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
fn reading_timers_keeps_only_armed_entries_in_firing_order() {
    let mut timers = Deadlines::with_capacity(3);
    timers.arm('b', at(20)).unwrap();
    timers.arm('a', at(10)).unwrap();
    timers.arm('c', at(5)).unwrap();
    timers.cancel('c');
    assert!(timers.iter().eq([('a', at(10)), ('b', at(20))]));
    assert_eq!(timers.expire(at(10)), Some('a'), "reading changes nothing");
    assert_eq!(timers.expire(at(20)), Some('b'));
    timers.arm('b', at(30)).unwrap();
    timers.arm('a', at(30)).unwrap();
    assert!(timers.iter().eq([('a', at(30)), ('b', at(30))]), "ties use key order");
}

#[test]
fn a_saved_rng_state_continues_the_stream() {
    let mut rng = Rng::new(42);
    let _draw = rng.next_u64();
    let mut restored = Rng::new(rng.state());
    assert_eq!(rng.next_u64(), restored.next_u64());
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

#[test]
#[expect(clippy::unreadable_literal, reason = "Day-count pins keep the calendar constants unique to the conversion")]
fn civil_dates_follow_gregorian_centuries_and_walls_bounds() {
    for (wall, date) in [
        (Wall::EPOCH, Civil { year: 1970, month: 1, day: 1 }),
        (Wall::from_nanos(951_782_400_000_000_000), Civil { year: 2000, month: 2, day: 29 }),
        (Wall::from_nanos(4_107_542_400_000_000_000), Civil { year: 2100, month: 3, day: 1 }),
        (Wall::from_nanos(u64::MAX), Civil { year: 2554, month: 7, day: 21 }),
    ] {
        assert_eq!(wall.civil(), date, "{wall:?}");
        assert_eq!(date.days(), Some(i64::try_from(wall.as_secs() / 86_400).unwrap()), "{date:?}");
        assert_eq!(date.wall().unwrap().as_secs(), wall.as_secs() / 86_400 * 86_400, "{date:?}");
    }
    for (date, days) in [
        (Civil { year: 1969, month: 12, day: 31 }, Some(-1)),
        (Civil { year: 0, month: 3, day: 1 }, Some(-719468)),
        (Civil { year: 0, month: 2, day: 29 }, Some(-719469)),
        (Civil { year: 1970, month: 13, day: 1 }, None),
        (Civil { year: 2023, month: 2, day: 29 }, None),
    ] {
        assert_eq!(date.days(), days, "{date:?}");
        assert_eq!(date.valid(), days.is_some(), "{date:?}");
    }
    for date in [
        Civil { year: 1969, month: 12, day: 31 },
        Civil { year: 2554, month: 7, day: 22 },
        Civil { year: u32::MAX, month: 12, day: 31 },
    ] {
        assert_eq!(date.wall(), None, "{date:?}");
    }
    for (month, name) in [(0, None), (1, Some(b"Jan".as_slice())), (12, Some(b"Dec".as_slice())), (13, None)] {
        assert_eq!(Civil::month_name(month), name, "{month}");
    }
}
