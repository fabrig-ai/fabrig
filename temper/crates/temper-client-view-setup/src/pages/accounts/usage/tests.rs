#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests compare formatted usage words"
)]
use super::*;
use crate::tests::fixtures;
use skein_lib::Duration;

fn fact(label: &[u8], length: Option<u64>, remaining: Option<u64>, used: Option<u16>, state: u16) -> c::AccountFact {
    c::AccountFact {
        kind: c::FACT_USAGE,
        label: Box::from(label),
        value: c::AccountFactValue::Window(c::AccountWindow {
            length: length.map(Duration::from_nanos),
            used,
            resets: remaining.map(Wall::from_nanos),
            state,
        }),
        seen: Wall::EPOCH,
        source: None,
        origin: 5,
    }
}
fn hours(count: u64) -> u64 {
    count.checked_mul(HOUR).expect("test hours")
}
fn only(fact: c::AccountFact) -> Window {
    let lines = windows(&List::from_boxed(Box::new([fact])), Wall::EPOCH, &fixtures::words());
    assert_eq!(lines.len(), 1);
    lines.into_boxed().into_iter().next().expect("one line")
}
fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).expect("usage words")
}

#[test]
fn held_windows_give_their_share_pace_and_colour_without_history() {
    for (label, length, remaining, used, state, name, pace, fills, look) in [
        (
            b"seven_day".as_slice(),
            Some(hours(168)),
            hours(110),
            2900,
            1,
            "Week",
            "resets in 4 d 14 h · 0.5%/h · about 84% at reset",
            "",
            Look::Plain,
        ),
        (
            b"five_hour",
            Some(hours(5)),
            hours(2),
            7500,
            1,
            "5 h",
            "resets in 2 h · 25%/h · ",
            "full in 1 h",
            Look::Plain,
        ),
        (
            b"seven_day",
            Some(hours(168)),
            hours(48),
            8300,
            1,
            "Week",
            "resets in 2 d · 0.7%/h · ",
            "full in 1 d",
            Look::Warned,
        ),
        (
            b"five_hour",
            Some(hours(5)),
            40 * MINUTE,
            10000,
            c::WINDOW_LIMITED,
            "5 h",
            "resets in 40 min",
            "",
            Look::Limited,
        ),
        (b"seven_day", None, hours(3), 4000, 1, "Week", "resets in 3 h", "", Look::Plain),
        (
            b"seven_day_opus",
            Some(hours(168)),
            hours(110),
            2900,
            1,
            "seven_day_opus",
            "resets in 4 d 14 h · 0.5%/h · about 84% at reset",
            "",
            Look::Plain,
        ),
    ] {
        let line = only(fact(label, length, Some(remaining), Some(used), state));
        assert_eq!(text(&line.name), name);
        assert_eq!(text(&line.share), format!("{}%", used / 100));
        assert_eq!(line.fraction, Fraction { done: u64::from(used), total: 10000 });
        assert_eq!(text(&line.pace), pace);
        assert_eq!(text(&line.fills), fills);
        assert_eq!(line.look, look);
    }
    let week = only(fact(b"seven_day", Some(hours(168)), Some(hours(110)), Some(2900), 1));
    assert_eq!(text(&week.line), "Week: 29% used · resets in 4 d 14 h · 0.5%/h · about 84% at reset");
    assert_eq!(only(fact(b"seven_day", None, None, Some(2999), 1)).share.as_ref(), b"29%");
    assert_eq!(only(fact(b"seven_day", None, None, Some(8000), 1)).look, Look::Warned);
    assert_eq!(only(fact(b"seven_day", None, None, Some(7999), 1)).look, Look::Plain);
}

#[test]
fn windows_only_draw_held_current_usage_shortest_first() {
    let mut wrong_kind = fact(b"seven_day", None, None, Some(10), 1);
    wrong_kind.kind = c::FACT_EMAIL;
    let mut wrong_value = wrong_kind.clone();
    wrong_value.kind = c::FACT_USAGE;
    wrong_value.value = c::AccountFactValue::Share(1000);
    let absent = List::from_boxed(Box::new([
        fact(b"five_hour", Some(hours(5)), Some(hours(1)), Some(1000), 1),
        fact(b"seven_day", Some(hours(168)), Some(0), Some(2900), 1),
        fact(b"seven_day", None, None, None, 1),
        wrong_kind,
        wrong_value,
    ]));
    assert!(windows(&absent, Wall::from_nanos(1), &fixtures::words()).is_empty());
    assert!(windows(&List::with_capacity(0), Wall::EPOCH, &fixtures::words()).is_empty());
    let facts = List::from_boxed(Box::new([
        fact(b"seven_day", Some(hours(168)), Some(hours(110)), Some(2900), 1),
        fact(b"z", None, None, Some(100), 1),
        fact(b"five_hour", Some(hours(5)), Some(hours(2)), Some(7500), 1),
        fact(b"a", None, None, Some(100), 1),
    ]));
    let lines = windows(&facts, Wall::EPOCH, &fixtures::words());
    let names: Vec<_> = lines.iter().map(|line| text(&line.name)).collect();
    assert_eq!(names, ["5 h", "Week", "a", "z"]);
}

#[test]
fn the_pace_rounds_for_sight_but_projects_from_the_unrounded_rate() {
    let w = fixtures::words();
    assert_eq!(text(&rate(&w, 45, hours(1))), "0.5%/h");
    assert_eq!(text(&rate(&w, 44, hours(1))), "0.4%/h");
    assert_eq!(text(&rate(&w, 2050, hours(1))), "21%/h");
    assert_eq!(text(&rate(&w, 2049, hours(1))), "20%/h");
    // At exactly the reset, never "full in".
    let equal = only(fact(b"seven_day", Some(hours(200)), Some(hours(81)), Some(5950), 1));
    assert!(equal.fills.is_empty());
    assert_eq!(text(&equal.pace), "resets in 3 d 9 h · 0.5%/h · about 100% at reset");
    let before = only(fact(b"seven_day", Some(hours(200)), Some(hours(82)), Some(5950), 1));
    assert_eq!(text(&before.fills), "full in 3 d 8 h");
    let precise = only(fact(b"seven_day", Some(hours(168)), Some(hours(120)), Some(2900), 1));
    assert_eq!(text(&precise.fills), "full in 4 d 21 h");
    let calm = only(fact(b"seven_day", Some(hours(168)), Some(hours(48)), Some(4200), 1));
    assert_eq!(text(&calm.pace), "resets in 2 d · 0.4%/h · about 59% at reset");
    let zero = only(fact(b"seven_day", Some(hours(168)), Some(hours(110)), Some(0), 1));
    assert_eq!(text(&zero.pace), "resets in 4 d 14 h · 0.0%/h · about 0% at reset");
    for length in [0, hours(1)] {
        assert_eq!(text(&only(fact(b"seven_day", Some(length), Some(hours(2)), Some(100), 1)).pace), "resets in 2 h");
    }
}

#[test]
fn usage_spans_keep_days_and_hours_and_floor_minutes() {
    for (nanos, expected) in [
        (MINUTE - 1, "under 1 min"),
        (40 * MINUTE + MINUTE - 1, "40 min"),
        (100 * MINUTE, "1 h 40 min"),
        (hours(12), "12 h"),
        (hours(48), "2 d"),
        (hours(110), "4 d 14 h"),
    ] {
        assert_eq!(text(&span(&fixtures::words(), nanos)), expected);
    }
}
