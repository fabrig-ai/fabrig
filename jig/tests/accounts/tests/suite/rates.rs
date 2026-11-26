//! A window's memory and pace (jig/docs/design/domain/accounts-lists.md, 1).
use jig_accounts_world::{HOLDER, LIMITS, PROVIDER, World, held, issued, program, provider, providers};
use jig_domain_accounts::{
    Event, FactValue, FactWindow, ORIGIN_PROGRAM, ORIGIN_USAGE_READ, Request, Stored, WindowReading,
};
use jig_domain_link::{UsageOutcome, UsageWindow};
use skein_lib::{Duration, Wall};

pub(super) const ACCOUNT: u32 = 0;
pub(super) const SOURCE: u32 = 1;
pub(super) const LABEL: &[u8] = b"five_hour";
pub(super) const MINUTE: u64 = 60_000_000_000;
const OPENED: u64 = 10 * MINUTE;
const START: u64 = OPENED + 120 * MINUTE;

pub(super) fn fresh(facts: Option<Stored>) -> World {
    let mut offered = provider(PROVIDER, true, true);
    offered.offers.programs = Box::new([b"claude_code".as_slice().into()]);
    let mut records = vec![held(ACCOUNT, b""), issued(SOURCE, b"personal"), program(ACCOUNT, SOURCE, 0, b"personal")];
    records.extend(facts);
    let mut world = World::with_start(LIMITS, providers(vec![offered]), &records);
    world.step(Event::Want);
    world
}
pub(super) fn report(world: &mut World, at: u64, value: FactWindow) -> Vec<Request> {
    world.wall = Wall::from_nanos(at);
    world.step(Event::Usage { source: SOURCE, window: UsageWindow { label: LABEL.into(), window: value } })
}
// The request is due on the monotonic clock; its answer can have any wall time.
pub(super) fn read_report(world: &mut World, due_minute: u64, at: u64, value: FactWindow) {
    let out = world.fire_at(due_minute * 60);
    let mut reads = out.iter().filter_map(|request| match request {
        Request::ReadUsage { read, source } => {
            assert_eq!(source.as_bytes(), b"personal");
            Some(*read)
        }
        _ => None,
    });
    let read = reads.next().expect("the account's usage read is due");
    assert_eq!(reads.next(), None);
    world.wall = Wall::from_nanos(at);
    world.step(Event::UsageRead {
        read,
        outcome: UsageOutcome::Read(Box::new([UsageWindow { label: LABEL.into(), window: value }])),
    });
}

pub(super) fn value(share: Option<u16>) -> FactWindow {
    FactWindow {
        used: share,
        length: Some(Duration::from_secs(5 * 3600)),
        resets: Some(Wall::from_nanos(OPENED + 300 * MINUTE)),
        state: jig_domain_accounts::WINDOW_OPEN,
    }
}
fn reading(at: u64, origin: u16, value: FactWindow) -> WindowReading {
    WindowReading {
        at: Wall::from_nanos(at),
        share: value.used.expect("a reading has a share"),
        origin,
        resets: value.resets,
    }
}

fn readings(world: &World) -> &[WindowReading] {
    world.domain.window_readings(ACCOUNT, LABEL)
}
fn rate(world: &World) -> Option<u32> {
    world.domain.window_rate(ACCOUNT, LABEL)
}

#[test]
fn minute_reports_keep_five_minute_readings_for_three_hours_and_five_minutes() {
    let mut world = fresh(None);
    for minute in 0_u16..=190 {
        report(&mut world, START + u64::from(minute) * MINUTE, value(Some(2000 + minute * 10)));
        let newest_minute = u64::from(minute / 5 * 5);
        let expected: Vec<_> = (0..=newest_minute)
            .step_by(5)
            .filter(|at| newest_minute - at <= 185)
            .map(|at| {
                reading(
                    START + at * MINUTE,
                    ORIGIN_PROGRAM,
                    value(Some(2000 + u16::try_from(at).expect("a minute in the test span") * 10)),
                )
            })
            .collect();
        assert_eq!(readings(&world), expected, "minute {minute}");
        let expected_rate = if minute >= 60 {
            600
        } else {
            u32::try_from((2000 + newest_minute * 10) * 60 / (120 + newest_minute)).expect("the fixture pace fits")
        };
        assert_eq!(rate(&world), Some(expected_rate), "minute {minute}");
    }
    let held = readings(&world).to_vec();
    report(&mut world, START + 195 * MINUTE, value(None));
    assert_eq!(readings(&world), held);
    assert_eq!(rate(&world), Some(600));
    // The fact still holds the previous share when a read gives none.
    read_report(&mut world, 30, START + 195 * MINUTE, value(None));
    assert_eq!(readings(&world), held);
    assert_eq!(rate(&world), Some(600));
    assert_eq!(world.domain.account_facts(ACCOUNT)[0].value, FactValue::Window(value(Some(3900))));
    assert_eq!(world.domain.account_facts(ACCOUNT)[0].origin, ORIGIN_USAGE_READ);
}

#[test]
fn young_windows_need_an_opening_and_rates_saturate() {
    for (length, resets) in [(None, value(None).resets), (value(None).length, None), (None, None)] {
        let mut world = fresh(None);
        let mut window = FactWindow { length, resets, ..value(Some(2000)) };
        report(&mut world, START, window);
        assert_eq!(rate(&world), None);
        report(&mut world, START + 55 * MINUTE, window);
        assert_eq!(rate(&world), None);
        window.used = Some(2600);
        report(&mut world, START + 60 * MINUTE, window);
        assert_eq!(rate(&world), Some(600));
    }
    let mut world = fresh(None);
    report(&mut world, START, value(None));
    assert_eq!(readings(&world), []);
    assert_eq!(rate(&world), None);
    // One nanosecond after opening makes the average exceed u32, not wrap.
    report(&mut world, OPENED, value(Some(2000)));
    assert_eq!(rate(&world), None);
    let latest = value(Some(1000));
    report(&mut world, OPENED + 5 * MINUTE, latest);
    // A backwards wall cannot clear the history even when the share falls.
    report(&mut world, OPENED + 1, value(Some(0)));
    assert_eq!(readings(&world), [reading(OPENED + 5 * MINUTE, ORIGIN_PROGRAM, latest)]);
    assert_eq!(rate(&world), Some(12_000));
    let mut saturated = fresh(None);
    report(&mut saturated, OPENED + 1, value(Some(10_000)));
    assert_eq!(rate(&saturated), Some(u32::MAX));
    let mut before_epoch = fresh(None);
    report(&mut before_epoch, 0, FactWindow { resets: Some(Wall::from_nanos(1)), ..value(Some(2000)) });
    assert_eq!(rate(&before_epoch), Some(400));
}

#[test]
fn a_reset_clears_readings_and_equal_reports_bring_the_rate_to_zero() {
    let mut world = fresh(None);
    for minute in (0_u16..=60).step_by(5) {
        report(&mut world, START + u64::from(minute) * MINUTE, value(Some(2000 + minute * 10)));
    }
    assert_eq!(rate(&world), Some(600));
    let held = world.domain.account_facts(ACCOUNT).to_vec();
    assert_eq!(report(&mut world, START + 62 * MINUTE, value(Some(2600))), []);
    assert_eq!(rate(&world), Some(600));
    for minute in (65..=240).step_by(5) {
        assert_eq!(report(&mut world, START + minute * MINUTE, value(Some(2600))), []);
    }
    assert_eq!(world.domain.account_facts(ACCOUNT), held);
    assert_eq!(rate(&world), Some(0));
    let reset = value(Some(100));
    report(&mut world, START + 241 * MINUTE, reset);
    assert_eq!(readings(&world), [reading(START + 241 * MINUTE, ORIGIN_PROGRAM, reset)]);
    assert_eq!(rate(&world), Some(16));
}

#[test]
fn a_burst_after_quiet_hours_rates_over_three_hours() {
    let mut world = fresh(None);
    for minute in (0_u16..=180).step_by(5) {
        let share = 2000 + minute.saturating_sub(150) * 10;
        report(&mut world, START + u64::from(minute) * MINUTE, value(Some(share)));
    }
    assert_eq!(rate(&world), Some(100));
}

#[test]
fn interleaved_reads_and_events_each_have_their_own_pace() {
    let mut world = fresh(None);
    let mut expected = Vec::new();
    for (index, (read_rate, event_rate)) in
        [(1000, 954), (840, 846), (200, 380), (200, 380), (200, 380)].into_iter().enumerate()
    {
        let index = u16::try_from(index).expect("a fixture index fits");
        let minute = u64::from(index) * 30;
        let at = START + minute * MINUTE;
        let whole = value(Some(2000 + index * 100));
        read_report(&mut world, minute + 30, at, whole);
        expected.push(reading(at, ORIGIN_USAGE_READ, whole));
        assert_eq!(readings(&world), expected);
        assert_eq!(rate(&world), Some(read_rate), "read at minute {minute}");

        let at = at + 10 * MINUTE;
        let decimal = value(Some(2067 + index * 190));
        report(&mut world, at, decimal);
        expected.push(reading(at, ORIGIN_PROGRAM, decimal));
        assert_eq!(readings(&world), expected);
        assert_eq!(rate(&world), Some(event_rate), "event after the read at minute {minute}");
    }
}

#[test]
fn a_read_below_an_event_does_not_clear_the_events() {
    let mut world = fresh(None);
    report(&mut world, START, value(Some(2000)));
    report(&mut world, START + 60 * MINUTE, value(Some(2037)));
    let mut expected = readings(&world).to_vec();
    let at = START + 61 * MINUTE;
    let whole = value(Some(2000));
    read_report(&mut world, 30, at, whole);
    expected.push(reading(at, ORIGIN_USAGE_READ, whole));
    assert_eq!(readings(&world), expected);
    assert_eq!(rate(&world), Some(662));
    report(&mut world, START + 65 * MINUTE, value(Some(2130)));
    assert_eq!(rate(&world), Some(120));
}

#[test]
fn a_changed_reset_instant_clears_only_that_origin() {
    let mut world = fresh(None);
    let unchanged = value(Some(2000));
    let mut expected = Vec::new();
    for minute in [0, 10] {
        let at = START + minute * MINUTE;
        report(&mut world, at, unchanged);
        expected.push(reading(at, ORIGIN_PROGRAM, unchanged));
        let read_at = at + 5 * MINUTE;
        read_report(&mut world, 30 + minute * 3, read_at, unchanged);
        expected.push(reading(read_at, ORIGIN_USAGE_READ, unchanged));
    }
    let at = START + 20 * MINUTE;
    let omitted = FactWindow { resets: None, ..unchanged };
    report(&mut world, at, omitted);
    expected.push(reading(at, ORIGIN_PROGRAM, omitted));
    assert_eq!(readings(&world), expected);
    assert_eq!(world.domain.account_facts(ACCOUNT)[0].value, FactValue::Window(unchanged));

    // A different reset compared to an absent one also keeps the earlier readings.
    let different = FactWindow { resets: Some(Wall::from_nanos(OPENED + 301 * MINUTE)), ..unchanged };
    let at = START + 25 * MINUTE;
    report(&mut world, at, different);
    expected.push(reading(at, ORIGIN_PROGRAM, different));
    assert_eq!(readings(&world), expected);
    let at = START + 26 * MINUTE;
    report(&mut world, at, unchanged);
    expected.retain(|prior| prior.origin == ORIGIN_USAGE_READ);
    expected.push(reading(at, ORIGIN_PROGRAM, unchanged));
    assert_eq!(readings(&world), expected);
}

#[test]
fn under_an_hour_of_readings_the_rate_is_the_average_since_opening() {
    let mut world = fresh(None);
    for (minute, share, expected) in [(0, 2000, 1000), (55, 2550, 874), (60, 2600, 600)] {
        report(&mut world, START + minute * MINUTE, value(Some(share)));
        assert_eq!(rate(&world), Some(expected), "minute {minute}");
    }
}

#[test]
fn the_reading_nearest_three_hours_back_is_used_with_the_younger_on_a_tie() {
    for (older, younger, expected) in [(185, 175, 240), (182, 152, 329)] {
        let mut world = fresh(None);
        let newest = START + 185 * MINUTE;
        for (ago, share) in [(older, 2000), (younger, 2300), (0, 3000)] {
            report(&mut world, newest - ago * MINUTE, value(Some(share)));
        }
        assert_eq!(rate(&world), Some(expected), "readings {older} and {younger} minutes back");
    }
}

#[test]
fn readings_older_than_three_hours_and_five_minutes_go_for_each_origin() {
    let mut world = fresh(None);
    let mut expected = Vec::new();
    let same = value(Some(2000));
    for minute in (0..=180).step_by(5) {
        let at = START + minute * MINUTE;
        report(&mut world, at, same);
        read_report(&mut world, (minute / 5 + 1) * 30, at, same);
        expected.extend([reading(at, ORIGIN_PROGRAM, same), reading(at, ORIGIN_USAGE_READ, same)]);
    }
    let at = START + 185 * MINUTE;
    report(&mut world, at, same);
    expected.push(reading(at, ORIGIN_PROGRAM, same));
    assert_eq!(readings(&world), expected);

    let at = START + 186 * MINUTE;
    read_report(&mut world, 38 * 30, at, same);
    expected.drain(..2);
    expected.push(reading(at, ORIGIN_USAGE_READ, same));
    assert_eq!(readings(&world), expected);
}

#[test]
fn a_restart_and_an_account_gone_take_the_readings() {
    let mut world = fresh(None);
    report(&mut world, START, value(Some(2000)));
    assert_eq!(rate(&world), Some(1000));
    let stored = Stored::Facts { account: ACCOUNT, facts: world.domain.account_facts(ACCOUNT).into() };
    let restarted = fresh(Some(stored));
    assert_ne!(restarted.domain.account_facts(ACCOUNT), []);
    assert_eq!(readings(&restarted), []);
    assert_eq!(rate(&restarted), None);
    world.step(Event::Change {
        request: skein_lib::Token::new(97),
        holder: HOLDER,
        change: jig_domain_accounts::Change::Remove { account: ACCOUNT },
    });
    assert_eq!(world.domain.account_facts(ACCOUNT), []);
    assert_eq!(readings(&world), []);
    assert_eq!(rate(&world), None);
}
