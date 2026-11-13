use super::fixtures;
use crate::words::{
    TimeForm::{Ago, Clock, Day, Span, Weekday},
    Words,
};
use skein_lib::Wall;

const HOUR: u64 = 3600;
const DAY: u64 = 24 * HOUR;
// Thursday 22 October 2026, midnight UTC.
const MIDNIGHT: u64 = 1_792_627_200;

pub(crate) fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.checked_mul(1_000_000_000).unwrap())
}
fn words(zone: i32) -> Words {
    let mut words = fixtures::words();
    words.set_zone_minutes(zone);
    words
}

#[test]
fn a_clock_reads_in_the_zone() {
    for (zone, said) in [(120_i32, b"14:00"), (-300_i32, b"07:00")] {
        let words = words(zone);
        let noon = wall(MIDNIGHT + 12 * HOUR);
        assert_eq!(words.time(Clock, noon, noon).0.as_ref(), said);
        assert_eq!(words.turn_clock(noon).as_ref(), said);
    }
    // Local calendar arithmetic may cross either end of Wall without wrapping.
    assert_eq!(words(-300).turn_clock(Wall::EPOCH).as_ref(), b"19:00");
    assert_eq!(words(-300).time(Day, Wall::EPOCH, Wall::EPOCH).0.as_ref(), "Today · Wednesday 31 December".as_bytes());
    assert_eq!(words(120).time(Clock, Wall::from_nanos(u64::MAX), Wall::from_nanos(u64::MAX)).1, None);
}

#[test]
fn ago_at_hh_mm_reads_in_the_zone() {
    for (zone, said, midnight) in
        [(120_i32, "at 02:00", MIDNIGHT + 22 * HOUR), (-300_i32, "at 00:00", MIDNIGHT + 29 * HOUR)]
    {
        let words = words(zone);
        let at = if zone == 120_i32 { MIDNIGHT } else { MIDNIGHT + 5 * HOUR };
        assert_eq!(words.time(Ago, wall(at), wall(at + 12 * HOUR)), (Box::from(said.as_bytes()), Some(wall(midnight))));
        assert_eq!(words.time(Ago, wall(at), wall(midnight)).0.as_ref(), b"yesterday");
        assert_eq!(words.time(Ago, wall(at), wall(midnight + DAY)).0.as_ref(), b"Thu");
    }
}

#[test]
fn a_span_and_ago_differences_do_not_move() {
    let utc = words(0);
    for zone in [120_i32, -300_i32] {
        let words = words(zone);
        for elapsed in [0, 59, 60, HOUR, 12 * HOUR - 1, DAY, 3 * DAY] {
            let at = wall(MIDNIGHT + 20);
            let now = wall(MIDNIGHT + 20 + elapsed);
            assert_eq!(words.time(Span, at, now), utc.time(Span, at, now));
            if elapsed < 12 * HOUR {
                assert_eq!(words.time(Ago, at, now), utc.time(Ago, at, now));
            }
        }
        assert_eq!(
            words.time(Ago, wall(MIDNIGHT + HOUR), wall(MIDNIGHT)),
            utc.time(Ago, wall(MIDNIGHT + HOUR), wall(MIDNIGHT))
        );
    }
}

#[test]
fn zone_zero_says_what_it_said() {
    let words = words(0);
    let default = fixtures::words();
    let at = wall(MIDNIGHT + 7 * HOUR + 20 * 60);
    let now = wall(MIDNIGHT + 12 * HOUR);
    for form in [Ago, Span, Clock, Weekday, Day] {
        assert_eq!(words.time(form, at, now), default.time(form, at, now));
    }
    assert_eq!(words.time(Clock, at, now), (Box::from(b"07:20".as_slice()), Some(wall(MIDNIGHT + DAY))));
    assert_eq!(words.turn_clock(at).as_ref(), b"07:20");
}
