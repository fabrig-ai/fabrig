#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests compare formatted usage words"
)]
use super::*;
use crate::tests::fixtures;
use skein_lib::Duration;

fn fact(
    label: &[u8],
    length: Option<u64>,
    remaining: Option<u64>,
    used: Option<u16>,
    state: u16,
    rate: Option<u32>,
) -> c::AccountFact {
    c::AccountFact {
        kind: c::FACT_USAGE,
        label: Box::from(label),
        value: c::AccountFactValue::Window(c::AccountWindow {
            length: length.map(Duration::from_nanos),
            used,
            resets: remaining.map(Wall::from_nanos),
            state,
            rate,
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
    for (label, remaining, used, state, rate, name, pace, fills, look) in [
        (
            b"seven_day".as_slice(),
            hours(110),
            2900,
            1,
            Some(50),
            "Week",
            "resets in 4 d 14 h · 0.5%/h · about 84% at reset",
            "",
            Look::Plain,
        ),
        (b"five_hour", hours(2), 7500, 1, Some(2500), "5 h", "resets in 2 h · 25%/h · ", "full in 1 h", Look::Plain),
        (b"seven_day", hours(48), 8300, 1, Some(70), "Week", "resets in 2 d · 0.7%/h · ", "full in 1 d", Look::Warned),
        (b"five_hour", 40 * MINUTE, 10000, c::WINDOW_LIMITED, Some(200), "5 h", "resets in 40 min", "", Look::Limited),
        (b"seven_day_", hours(48), 100, 1, None, "seven_day_", "resets in 2 d", "", Look::Plain),
    ] {
        let line = only(fact(label, None, Some(remaining), Some(used), state, rate));
        assert_eq!(text(&line.name), name);
        assert_eq!(text(&line.share), format!("{}%", used / 100));
        assert_eq!(line.fraction, Fraction { done: u64::from(used), total: 10000 });
        assert_eq!(text(&line.pace), pace);
        assert_eq!(text(&line.fills), fills);
        assert_eq!(line.look, look);
    }
    let lasting_model = fact(b"seven_day_opus", Some(hours(168)), Some(hours(110)), Some(2900), 1, Some(50));
    assert!(windows(&List::from_boxed(Box::new([lasting_model])), Wall::EPOCH, &fixtures::words()).is_empty());

    let week = only(fact(b"seven_day", Some(hours(168)), Some(hours(110)), Some(2900), 1, Some(50)));
    assert_eq!(text(&week.line), "Week: 29% used · resets in 4 d 14 h · 0.5%/h · about 84% at reset");
    assert_eq!(only(fact(b"seven_day", None, None, Some(2999), 1, None)).share.as_ref(), b"29%");
    assert_eq!(only(fact(b"seven_day", None, None, Some(8000), 1, None)).look, Look::Warned);
    assert_eq!(only(fact(b"seven_day", None, None, Some(7999), 1, None)).look, Look::Plain);
}

#[test]
fn windows_only_draw_held_current_usage_shortest_first() {
    let mut wrong_kind = fact(b"seven_day", None, None, Some(10), 1, None);
    wrong_kind.kind = c::FACT_EMAIL;
    let mut wrong_value = wrong_kind.clone();
    wrong_value.kind = c::FACT_USAGE;
    wrong_value.value = c::AccountFactValue::Share(1000);
    let absent = List::from_boxed(Box::new([
        fact(b"five_hour", Some(hours(5)), Some(hours(1)), Some(1000), 1, Some(250)),
        fact(b"seven_day", Some(hours(168)), Some(0), Some(2900), 1, Some(50)),
        fact(b"seven_day", None, None, None, 1, Some(200)),
        wrong_kind,
        wrong_value,
    ]));
    assert!(windows(&absent, Wall::from_nanos(1), &fixtures::words()).is_empty());
    assert!(windows(&List::with_capacity(0), Wall::EPOCH, &fixtures::words()).is_empty());
    let facts = List::from_boxed(Box::new([
        fact(b"seven_day", Some(hours(168)), Some(hours(110)), Some(2900), 1, Some(50)),
        fact(b"z", None, None, Some(100), 1, None),
        fact(b"five_hour", Some(hours(5)), Some(hours(2)), Some(7500), 1, Some(2500)),
        fact(b"a", None, None, Some(100), 1, None),
    ]));
    let lines = windows(&facts, Wall::EPOCH, &fixtures::words());
    let names: Vec<_> = lines.iter().map(|line| text(&line.name)).collect();
    assert_eq!(names, ["5 h", "Week", "a", "z"]);
}

#[test]
fn model_weeks_are_drawn_only_while_limited_or_filling_before_reset() {
    let length = Some(hours(168));
    let reset = Some(hours(48));
    let absent = List::from_boxed(Box::new([
        fact(b"seven_day_sonnet", length, reset, Some(0), 1, None),
        fact(b"seven_day_fable", length, reset, Some(0), 1, Some(0)),
        fact(b"seven_day_fable", length, reset, Some(6400), 1, None),
        fact(b"seven_day_fable", length, reset, Some(6400), 1, Some(75)),
    ]));
    assert!(windows(&absent, Wall::EPOCH, &fixtures::words()).is_empty());

    let fable = only(fact(b"seven_day_fable", length, reset, Some(6400), 1, Some(160)));
    assert_eq!(text(&fable.name), "Fable");
    assert_eq!(fable.look, Look::Plain);
    assert_eq!(text(&fable.fills), "full in 22 h 30 min");
    assert_eq!(text(&fable.line), "Fable: 64% used · resets in 2 d · 1.6%/h · full in 22 h 30 min");

    for (label, rate, name) in [(b"seven_day_sonnet".as_slice(), Some(50), "Sonnet"), (b"seven_day_opus", None, "Opus")]
    {
        let limited = only(fact(label, length, reset, Some(10000), c::WINDOW_LIMITED, rate));
        assert_eq!(text(&limited.name), name);
        assert_eq!(limited.look, Look::Limited);
        assert_eq!(text(&limited.pace), "resets in 2 d");
        assert!(limited.fills.is_empty());
    }
}

#[test]
fn model_weeks_follow_week_at_the_same_length() {
    let week = Some(hours(168));
    let facts = List::from_boxed(Box::new([
        fact(b"seven_day_sonnet", week, Some(hours(48)), Some(0), 1, None),
        fact(b"seven_day_fable", week, Some(hours(48)), Some(6400), 1, Some(160)),
        fact(b"seven_day", week, Some(hours(110)), Some(2900), 1, Some(50)),
        fact(b"five_hour", Some(hours(5)), Some(hours(2)), Some(7500), 1, Some(2500)),
    ]));
    let lines = windows(&facts, Wall::EPOCH, &fixtures::words());
    let names: Vec<_> = lines.iter().map(|line| text(&line.name)).collect();
    assert_eq!(names, ["5 h", "Week", "Fable"]);
}

#[test]
fn the_pace_rounds_for_sight_but_projects_from_the_unrounded_rate() {
    let w = fixtures::words();
    assert_eq!(text(&rate(&w, 45)), "0.5%/h");
    assert_eq!(text(&rate(&w, 44)), "0.4%/h");
    assert_eq!(text(&rate(&w, 2050)), "21%/h");
    assert_eq!(text(&rate(&w, 2049)), "20%/h");
    assert_eq!(text(&rate(&w, 200)), "2.0%/h");
    // The opening implies a different average; all projections use the served rate.
    let equal = only(fact(b"seven_day", Some(hours(168)), Some(hours(81)), Some(5950), 1, Some(50)));
    assert!(equal.fills.is_empty());
    assert_eq!(text(&equal.pace), "resets in 3 d 9 h · 0.5%/h · about 100% at reset");
    let before = only(fact(b"seven_day", None, Some(hours(82)), Some(5950), 1, Some(50)));
    assert_eq!(text(&before.fills), "full in 3 d 9 h");
    let precise = only(fact(b"seven_day", None, Some(hours(120)), Some(2900), 1, Some(61)));
    assert_eq!(text(&precise.pace), "resets in 5 d · 0.6%/h · ");
    assert_eq!(text(&precise.fills), "full in 4 d 20 h");
    let calm = only(fact(b"seven_day", None, Some(hours(48)), Some(4200), 1, Some(35)));
    assert_eq!(text(&calm.pace), "resets in 2 d · 0.4%/h · about 59% at reset");
    let zero = only(fact(b"seven_day", None, Some(hours(110)), Some(2900), 1, Some(0)));
    assert_eq!(text(&zero.pace), "resets in 4 d 14 h · 0.0%/h · about 29% at reset");
    let ceiling = only(fact(b"seven_day", None, Some(u64::MAX), Some(100), 1, Some(u32::MAX)));
    assert_eq!(text(&ceiling.fills), "full in under 1 min");
}

#[test]
fn absent_rates_never_invent_an_average_and_a_reset_is_optional() {
    let absent = only(fact(b"seven_day", Some(hours(168)), Some(hours(110)), Some(2900), 1, None));
    assert_eq!(text(&absent.pace), "resets in 4 d 14 h");
    assert!(absent.fills.is_empty());
    assert!(only(fact(b"seven_day", Some(hours(168)), None, Some(2900), 1, None)).pace.is_empty());
    let rolling = only(fact(b"seven_day", None, None, Some(2900), 1, Some(200)));
    assert_eq!(text(&rolling.pace), "2.0%/h");
    assert!(rolling.fills.is_empty());
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
