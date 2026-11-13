use super::fixtures;
use crate::words::TimeForm::{Ago, Clock, Day, Span};
use skein_lib::Wall;

const SECOND: u64 = 1_000_000_000;
const MINUTE: u64 = 60;
const HOUR: u64 = 3600;
const DAY: u64 = 86400;
// Thursday 22 October 2026, noon UTC.
const NOON: u64 = 1_792_670_400;
const MIDNIGHT: u64 = NOON - 12 * HOUR;
const YEAR: u64 = 1_798_761_600; // Friday 1 January 2027, midnight UTC.
pub(crate) fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.checked_mul(SECOND).unwrap())
}
#[test]
fn the_four_time_forms_and_their_next_changes_at_each_bound() {
    let words = fixtures::words();
    for (form, at, now, said, next) in [
        (Ago, NOON, NOON, "just now", NOON + MINUTE),
        (Ago, NOON, NOON + MINUTE - 1, "just now", NOON + MINUTE),
        (Ago, NOON, NOON + MINUTE, "1 min ago", NOON + 2 * MINUTE),
        (Ago, NOON, NOON + 59 * MINUTE, "59 min ago", NOON + HOUR),
        (Ago, NOON, NOON + HOUR, "1 h ago", NOON + HOUR + MINUTE),
        (Ago, NOON, NOON + HOUR + 28 * MINUTE, "1 h 28 min ago", NOON + HOUR + 29 * MINUTE),
        (Ago, MIDNIGHT, NOON - 1, "11 h 59 min ago", NOON),
        (Ago, MIDNIGHT, NOON, "at 00:00", MIDNIGHT + DAY),
        (Ago, MIDNIGHT, MIDNIGHT + DAY - 1, "at 00:00", MIDNIGHT + DAY),
        (Ago, MIDNIGHT, MIDNIGHT + DAY, "yesterday", MIDNIGHT + 2 * DAY),
        (Ago, MIDNIGHT, MIDNIGHT + 2 * DAY - 1, "yesterday", MIDNIGHT + 2 * DAY),
        (Ago, MIDNIGHT, MIDNIGHT + 2 * DAY, "Thu", MIDNIGHT + 3 * DAY),
        (Ago, MIDNIGHT - 7 * DAY, NOON, "Thu", MIDNIGHT + DAY),
        (Ago, MIDNIGHT - 8 * DAY, NOON, "14 October", MIDNIGHT + DAY),
        (Ago, YEAR - 9 * DAY, YEAR - 1, "23 December", YEAR),
        (Ago, YEAR - 9 * DAY, YEAR, "23 December 2026", YEAR + DAY),
        (Ago, NOON + HOUR, NOON, "just now", NOON + HOUR + MINUTE),
        (Span, NOON, NOON, "under 1 min", NOON + MINUTE),
        (Span, NOON, NOON + MINUTE - 1, "under 1 min", NOON + MINUTE),
        (Span, NOON, NOON + MINUTE, "1 min", NOON + 2 * MINUTE),
        (Span, NOON, NOON + 12 * MINUTE, "12 min", NOON + 13 * MINUTE),
        (Span, NOON, NOON + HOUR - 1, "59 min", NOON + HOUR),
        (Span, NOON, NOON + HOUR, "1 h", NOON + HOUR + MINUTE),
        (Span, NOON, NOON + HOUR + 28 * MINUTE, "1 h 28 min", NOON + HOUR + 29 * MINUTE),
        (Span, NOON, NOON + 2 * HOUR, "2 h", NOON + 2 * HOUR + MINUTE),
        (Span, NOON, NOON + DAY - 1, "23 h 59 min", NOON + DAY),
        (Span, NOON, NOON + DAY, "1 d", NOON + 2 * DAY),
        (Span, NOON, NOON + 3 * DAY - 1, "2 d", NOON + 3 * DAY),
        (Span, NOON, NOON + 3 * DAY, "3 d", NOON + 4 * DAY),
        (Span, NOON + HOUR, NOON, "under 1 min", NOON + HOUR + MINUTE),
        (Clock, MIDNIGHT + 7 * HOUR + 20 * MINUTE, NOON, "07:20", MIDNIGHT + DAY),
        (Clock, MIDNIGHT, MIDNIGHT + DAY - 1, "00:00", MIDNIGHT + DAY),
        (Clock, MIDNIGHT, MIDNIGHT + DAY, "Thu 00:00", MIDNIGHT + 2 * DAY),
        (Clock, MIDNIGHT - 2 * DAY + 7 * HOUR + 20 * MINUTE, NOON, "Tue 07:20", MIDNIGHT + DAY),
        (Clock, MIDNIGHT - 6 * DAY, NOON, "Fri 00:00", MIDNIGHT + DAY),
        (Clock, MIDNIGHT - 7 * DAY, NOON, "15 October 00:00", MIDNIGHT + DAY),
        (Clock, YEAR - 8 * DAY, YEAR - 1, "24 December 00:00", YEAR),
        (Clock, YEAR - 8 * DAY, YEAR, "24 December 2026 00:00", YEAR + DAY),
        (Clock, NOON + HOUR, NOON, "13:00", MIDNIGHT + DAY),
        (Day, NOON, NOON, "Today · Thursday 22 October", MIDNIGHT + DAY),
        (Day, MIDNIGHT, MIDNIGHT + DAY - 1, "Today · Thursday 22 October", MIDNIGHT + DAY),
        (Day, MIDNIGHT, MIDNIGHT + DAY, "Yesterday · Thursday 22 October", MIDNIGHT + 2 * DAY),
        (Day, MIDNIGHT, MIDNIGHT + 2 * DAY - 1, "Yesterday · Thursday 22 October", MIDNIGHT + 2 * DAY),
        (Day, MIDNIGHT, MIDNIGHT + 2 * DAY, "Thursday 22 October", MIDNIGHT + 3 * DAY),
        (Day, MIDNIGHT - 2 * DAY, NOON, "Tuesday 20 October", MIDNIGHT + DAY),
        (Day, YEAR - DAY, YEAR - 1, "Today · Thursday 31 December", YEAR),
        (Day, YEAR - DAY, YEAR, "Yesterday · Thursday 31 December 2026", YEAR + DAY),
        (Day, NOON + HOUR, NOON, "Today · Thursday 22 October", MIDNIGHT + DAY),
        // The minute is elapsed from the supplied instant, including its seconds.
        (Ago, NOON + 20, NOON + 79, "just now", NOON + 80),
        (Ago, NOON + 20, NOON + 80, "1 min ago", NOON + 140),
    ] {
        let (text, tick) = words.time(form, wall(at), wall(now));
        assert_eq!(text.as_ref(), said.as_bytes(), "{form:?} at {at}, now {now}");
        assert_eq!(tick, Some(wall(next)), "{form:?} at {at}, now {now}");
    }
}

#[test]
fn a_build_keeps_the_earliest_change_of_the_forms_it_drew() {
    let words = fixtures::words();
    let mut times = crate::words::Times::new(wall(12 * MINUTE));
    assert_eq!(times.next, None);
    let _clock = times.say(&words, Clock, Wall::EPOCH);
    let _span = times.say(&words, Span, wall(5));
    let _ago = times.say(&words, Ago, Wall::EPOCH);
    assert_eq!(times.next, Some(wall(12 * MINUTE + 5)));
}

#[test]
fn a_turns_bare_clock_stays_fixed_across_midnight_without_its_own_wake() {
    let words = fixtures::words();
    for now in [MIDNIGHT + DAY - 1, MIDNIGHT + DAY] {
        let times = crate::words::Times::new(wall(now));
        assert_eq!(words.turn_clock(wall(NOON)).as_ref(), b"12:00");
        assert_eq!(times.next, None);
    }
}

#[test]
fn dated_names_today_then_the_week_or_date_and_ticks_at_midnight() {
    use crate::words::TimeForm::Dated;
    let w = fixtures::words();
    // Monday, four days after the fixed Thursday: Friday is three days before it.
    let today = MIDNIGHT + 4 * DAY;
    let morning = today + 8 * HOUR + 12 * MINUTE;
    for (at, now, expected, tick) in [
        (morning, today + 12 * HOUR, "today 08:12", today + DAY),
        (today - 3 * DAY + 18 * HOUR + 40 * MINUTE, today + 12 * HOUR, "Fri 18:40", today + DAY),
        (MIDNIGHT - 14 * DAY + 7 * HOUR + 20 * MINUTE, NOON, "8 October 07:20", MIDNIGHT + DAY),
        (morning, today + DAY - 1, "today 08:12", today + DAY),
        (morning, today + DAY, "Mon 08:12", today + 2 * DAY),
    ] {
        let (said, next) = w.time(Dated, wall(at), wall(now));
        assert_eq!(said.as_ref(), expected.as_bytes());
        assert_eq!(next, Some(wall(tick)));
    }
}
