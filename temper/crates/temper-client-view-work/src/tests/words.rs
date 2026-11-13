use super::fixtures;
use crate::words::{
    TimeForm::{Clock, Day, Weekday},
    Words,
};
use skein_lib::Wall;

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

fn assert_chat_days(words: &Words, midnight: u64) {
    use jig_client::{
        contract as c,
        thread::{Limits, Reading, Thread},
    };
    use skein_lib::List;
    use skein_ui::{Builder, Role};
    let mut thread = Thread::new(1, &Limits { lines: 4, bytes: 4096, text: 512, live: 512 });
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
    );
    let tree = builder.finish();
    assert!(fixtures::find(&tree, Role::Paragraph, "Yesterday · Wednesday 21 October".as_bytes()).is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, "Today · Thursday 22 October".as_bytes()).is_some());
    assert!(fixtures::find(&tree, Role::Time, b"23:59").is_some());
    assert!(fixtures::find(&tree, Role::Time, b"00:00").is_some());
}
