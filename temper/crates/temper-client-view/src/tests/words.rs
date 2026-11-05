use super::fixtures;
use crate::words::{
    TimeForm::{Ago, Clock, Day, Span, Weekday},
    Words,
};
use skein_lib::{Queue, Wall};

const HOUR: u64 = 3600;
const DAY: u64 = 24 * HOUR;
// Thursday 22 October 2026, midnight UTC.
const MIDNIGHT: u64 = 1_792_627_200;
const YEAR: u64 = 1_798_761_600;

fn wall(seconds: u64) -> Wall {
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
fn the_day_turns_at_the_zones_midnight() {
    for (zone, midnight) in [(120_i32, MIDNIGHT - 2 * HOUR), (-300_i32, MIDNIGHT + 5 * HOUR)] {
        let words = words(zone);
        let at = wall(midnight - 1);
        assert_eq!(
            words.time(Day, at, at),
            (Box::from("Today · Wednesday 21 October".as_bytes()), Some(wall(midnight)))
        );
        assert_eq!(
            words.time(Day, at, wall(midnight)),
            (Box::from("Yesterday · Wednesday 21 October".as_bytes()), Some(wall(midnight + DAY)))
        );
        assert_eq!(words.time(Clock, at, wall(midnight)).0.as_ref(), b"Wed 23:59");
        assert_eq!(words.time(Weekday, wall(midnight), wall(midnight)).0.as_ref(), b"Thursday");
        assert_eq!(words.calendar_day(at).checked_add(1), Some(words.calendar_day(wall(midnight))));
        assert_chat_days(&words, midnight);
    }
    // The event's year and now's year both come from the local calendar.
    assert_eq!(words(120).time(Day, wall(YEAR - HOUR), wall(YEAR)).0.as_ref(), "Today · Friday 1 January".as_bytes());
    assert_eq!(
        words(-300).time(Day, wall(YEAR), wall(YEAR + 5 * HOUR)).0.as_ref(),
        "Yesterday · Thursday 31 December 2026".as_bytes()
    );
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

#[test]
fn a_build_reads_the_shells_zone_from_the_domain() {
    let (mut root, _, _) = fixtures::root_at(
        temper_client_domain::Address::Home { project: fixtures::PROJECT },
        &fixtures::domain_limits(),
    );
    assert_eq!(root.zone_minutes(), 0_i32);
    root.set_zone_minutes(120);
    assert_eq!(root.zone_minutes(), 120_i32);
    let mut view = fixtures::view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    crate::build(&mut view, &root, wall(MIDNIGHT + 12 * HOUR), &mut patches);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"14:00").is_some());
}

fn assert_chat_days(words: &Words, midnight: u64) {
    use jig_client::{
        contract as c,
        thread::{Limits, Reading, Thread},
    };
    use skein_lib::List;
    use skein_ui::{Builder, Role};
    let mut thread = Thread::new(1, &Limits { lines: 4, bytes: 4096, text: 512 });
    thread.reading = Some(Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(2);
    for (position, at) in [(2, midnight), (1, midnight.checked_sub(1).unwrap())] {
        rows.push(super::documents::line(
            position,
            wall(at),
            c::Activity::Words {
                message: position,
                by: c::Requester::Person(1),
                kind: c::MessageKind::Words,
                words: fixtures::first_run::text(b"A message"),
            },
        ))
        .unwrap();
    }
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    let mut builder = Builder::new(&fixtures::tree_limits());
    let mut times = crate::words::Times::new(wall(midnight));
    crate::pages::chat::conversation_with_times(
        &mut builder,
        &thread,
        words,
        (4, 512),
        crate::pages::chat::Speakers { person: Some(1), requester: true },
        (crate::pages::chat::Drawn::Lines, Some(&mut times)),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
    );
    let tree = builder.finish();
    assert!(fixtures::find(&tree, Role::Paragraph, "Yesterday · Wednesday 21 October".as_bytes()).is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, "Today · Thursday 22 October".as_bytes()).is_some());
    assert!(fixtures::find(&tree, Role::Time, b"23:59").is_some());
    assert!(fixtures::find(&tree, Role::Time, b"00:00").is_some());
}
