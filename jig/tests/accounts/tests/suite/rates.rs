//! A window's memory and pace (jig/docs/design/domain/accounts.md, 3.7).
use jig_accounts_world::{HOLDER, LIMITS, PROVIDER, World, held, issued, program, provider, providers};
use jig_domain_accounts::{Event, FactWindow, Request, Stored, WindowReading};
use jig_domain_link::UsageWindow;
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
pub(super) fn value(share: Option<u16>) -> FactWindow {
    FactWindow {
        used: share,
        length: Some(Duration::from_secs(5 * 3600)),
        resets: Some(Wall::from_nanos(OPENED + 300 * MINUTE)),
        state: jig_domain_accounts::WINDOW_OPEN,
    }
}
fn readings(world: &World) -> &[WindowReading] {
    world.domain.window_readings(ACCOUNT, LABEL)
}
fn rate(world: &World) -> Option<u32> {
    world.domain.window_rate(ACCOUNT, LABEL)
}

#[test]
fn minute_reports_keep_five_minute_readings_and_use_the_oldest_hour() {
    let mut world = fresh(None);
    for minute in 0_u16..=65 {
        report(&mut world, START + u64::from(minute) * MINUTE, value(Some(2000 + minute * 10)));
        let newest_minute = u64::from(minute / 5 * 5);
        let expected: Vec<_> = (0..=newest_minute)
            .step_by(5)
            .filter(|at| newest_minute - at <= 60)
            .map(|at| WindowReading {
                at: Wall::from_nanos(START + at * MINUTE),
                share: 2000 + u16::try_from(at).unwrap() * 10,
            })
            .collect();
        assert_eq!(readings(&world), expected, "minute {minute}");
        let expected_rate = if minute >= 30 {
            600
        } else {
            u32::try_from((2000 + newest_minute * 10) * 60 / (120 + newest_minute)).unwrap()
        };
        assert_eq!(rate(&world), Some(expected_rate), "minute {minute}");
    }
    // The report merges a missing share before recording it, rather than discarding the reading.
    report(&mut world, START + 70 * MINUTE, value(None));
    assert_eq!(readings(&world).last().unwrap().share, 2650);
    assert_eq!(rate(&world), Some(550));
}

#[test]
fn young_windows_need_an_opening_and_rates_saturate() {
    for (length, resets) in [(None, value(None).resets), (value(None).length, None), (None, None)] {
        let mut world = fresh(None);
        let mut window = FactWindow { length, resets, ..value(Some(2000)) };
        report(&mut world, START, window);
        assert_eq!(rate(&world), None);
        report(&mut world, START + 25 * MINUTE, window);
        assert_eq!(rate(&world), None);
        window.used = Some(2300);
        report(&mut world, START + 30 * MINUTE, window);
        assert_eq!(rate(&world), Some(600));
    }
    let mut world = fresh(None);
    report(&mut world, START, value(None));
    assert_eq!(readings(&world), []);
    assert_eq!(rate(&world), None);
    // One nanosecond after opening makes the average exceed u32, not wrap.
    report(&mut world, OPENED, value(Some(2000)));
    assert_eq!(rate(&world), None);
    report(&mut world, OPENED + 5 * MINUTE, value(Some(0)));
    report(&mut world, OPENED + 1, value(Some(10_000)));
    // A backwards wall cannot add a reading or change the rate.
    assert_eq!(readings(&world).len(), 1);
    assert_eq!(rate(&world), Some(0));
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
    for minute in (0_u16..=35).step_by(5) {
        report(&mut world, START + u64::from(minute) * MINUTE, value(Some(2000 + minute * 10)));
    }
    assert_eq!(rate(&world), Some(600));
    let held = world.domain.account_facts(ACCOUNT).to_vec();
    assert_eq!(report(&mut world, START + 37 * MINUTE, value(Some(2350))), Vec::<Request>::new());
    assert_eq!(rate(&world), Some(600));
    for minute in (40..=95).step_by(5) {
        assert_eq!(report(&mut world, START + minute * MINUTE, value(Some(2350))), Vec::<Request>::new());
    }
    assert_eq!(world.domain.account_facts(ACCOUNT), held);
    assert_eq!(rate(&world), Some(0));
    report(&mut world, START + 96 * MINUTE, value(Some(100)));
    assert_eq!(readings(&world), [WindowReading { at: Wall::from_nanos(START + 96 * MINUTE), share: 100 }]);
    assert_eq!(rate(&world), Some(27));
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
