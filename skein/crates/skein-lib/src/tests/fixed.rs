//! A fixed start and a deadline-driven clock (skein/docs/design/runtime/lib.md, 8).

use crate::{CryptoRng, Duration, FixedClock, FixedStart, Secret, Time, Wall};

fn start() -> FixedStart {
    FixedStart { wall: Wall::from_nanos(1_767_690_000_000_000_000), seed: 7 }
}

fn native(seconds: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_secs(seconds))
}

fn fixed(seconds: u64) -> Time {
    FixedStart::NOW.saturating_add(Duration::from_secs(seconds))
}

#[test]
fn fixed_start_reads_canonical_seconds_and_seed() {
    assert_eq!(FixedStart::NOW, Time::from_nanos(3_600_000_000_000));
    assert_eq!(FixedStart::read(b"1767690000,7"), Some(start()));
    assert_eq!(FixedStart::read(b"0,0"), Some(FixedStart { wall: Wall::EPOCH, seed: 0 }));
    assert_eq!(
        FixedStart::read(b"18446744073,18446744073709551615"),
        Some(FixedStart { wall: Wall::from_nanos(18_446_744_073_000_000_000), seed: u64::MAX })
    );
    for text in [
        b"".as_slice(),
        b"1",
        b",7",
        b"1,",
        b"1,7,2",
        b"+1,7",
        b"-1,7",
        b"1,+7",
        b"1,-7",
        b" 1,7",
        b"1, 7",
        b"1,7 ",
        b"1,7\n",
        b"01,7",
        b"1,07",
        b"1,18446744073709551616",
        b"18446744074,7",
        b"18446744073709551616,7",
        b"a,7",
        b"1,\xff",
    ] {
        assert_eq!(FixedStart::read(text), None, "refuse {text:?}");
    }
}

#[test]
fn fixed_start_draws_restart_seeded_streams() {
    let start = FixedStart { seed: 0x0807_0605_0403_0201, ..start() };
    let mut first = [0_u8; 32];
    let mut again = [0xff_u8; 32];
    let mut short = [0_u8; 8];
    start.draw(7, &mut first);
    start.draw(7, &mut again);
    start.draw(7, &mut short);
    assert_eq!(first, again, "each call restarts and fills the entire output");
    assert_eq!(short.as_slice(), &first[..8], "eight-byte draws use the same stream prefix");
    assert_ne!(short, [0; 8], "the short output is filled");

    let key = [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let key = Secret::new(Box::new(key));
    let mut expected = [0_u8; 32];
    CryptoRng::new(&key, 7).expect("32-byte key").fill(&mut expected);
    assert_eq!(first, expected, "little-endian seed followed by 24 zero bytes");

    start.draw(8, &mut again);
    assert_ne!(first, again, "stream numbers separate draws");
    FixedStart { seed: start.seed + 1, ..start }.draw(7, &mut again);
    assert_ne!(first, again, "seeds separate draws");
    FixedStart { wall: Wall::EPOCH, ..start }.draw(7, &mut again);
    assert_eq!(first, again, "wall time is not part of the key");
}

#[test]
fn fixed_clock_stays_put_until_a_requested_deadline_is_due() {
    let start = start();
    let mut clock = FixedClock::new(&start, native(5));
    assert_eq!(clock.read(native(5)), (fixed(0), start.wall));
    assert_eq!(clock.read(native(15)), (fixed(0), start.wall), "idle time does not advance the clock");
    assert_eq!(clock.until(Some(fixed(1)), native(15)), Some(native(16)));
    let before = Time::from_nanos(native(16).as_nanos() - 1);
    assert_eq!(clock.read(before), (fixed(0), start.wall));
    let advanced = (fixed(1), Wall::from_nanos(start.wall.as_nanos() + 1_000_000_000));
    assert_eq!(clock.read(native(16)), advanced, "the deadline advances both times exactly when due");
    assert_eq!(clock.read(native(50)), advanced, "a consumed deadline leaves the clock frozen");
}

#[test]
fn fixed_clock_reuses_an_ask_and_replaces_it_from_the_native_reading() {
    let start = start();
    let mut clock = FixedClock::new(&start, native(5));
    assert_eq!(clock.until(Some(fixed(2)), native(7)), Some(native(9)));
    assert_eq!(clock.until(Some(fixed(2)), native(8)), Some(native(9)), "repeated asks keep their instant");
    assert_eq!(clock.until(Some(fixed(3)), native(8)), Some(native(11)), "a new ask starts at its native reading");
    assert_eq!(clock.read(native(9)), (fixed(0), start.wall), "the replaced deadline cannot advance the clock");
    assert_eq!(clock.read(native(11)), (fixed(3), Wall::from_nanos(start.wall.as_nanos() + 3_000_000_000)));
    assert_eq!(clock.until(Some(fixed(4)), native(20)), Some(native(21)), "the span starts at the fixed reading");
}

#[test]
fn fixed_clock_forgets_a_cancelled_deadline() {
    let start = start();
    let mut clock = FixedClock::new(&start, native(5));
    assert_eq!(clock.until(Some(fixed(2)), native(7)), Some(native(9)));
    assert_eq!(clock.until(None, native(8)), None);
    assert_eq!(clock.read(native(10)), (fixed(0), start.wall), "a cancelled deadline never advances time");
    assert_eq!(clock.until(Some(fixed(2)), native(10)), Some(native(12)), "cancellation forgets the old instant");
}

#[test]
fn fixed_clock_due_or_past_deadlines_never_rewind() {
    let start = start();
    let mut clock = FixedClock::new(&start, native(5));
    assert_eq!(clock.until(Some(fixed(2)), native(5)), Some(native(7)));
    let advanced = (fixed(2), Wall::from_nanos(start.wall.as_nanos() + 2_000_000_000));
    assert_eq!(clock.read(native(7)), advanced);
    for deadline in [fixed(2), fixed(1), Time::ZERO] {
        assert_eq!(clock.until(Some(deadline), native(10)), Some(native(10)), "due or past asks do not wait");
        assert_eq!(clock.read(native(10)), advanced, "past asks never rewind either time");
    }
}

#[test]
fn fixed_clock_saturates_native_and_wall_time() {
    let start = FixedStart { wall: Wall::from_nanos(u64::MAX - 1), ..start() };
    let end = Time::from_nanos(u64::MAX);
    let mut clock = FixedClock::new(&start, native(5));
    assert_eq!(clock.until(Some(end), end), Some(end), "native deadlines saturate instead of wrapping");
    assert_eq!(clock.read(Time::from_nanos(u64::MAX - 1)), (fixed(0), start.wall));
    assert_eq!(clock.read(end), (end, Wall::from_nanos(u64::MAX)), "wall time saturates too");
}
