use crate::date::{Civil, civil, format, month_name, parse, wall};
use skein_lib::Wall;

#[test]
fn known_midnights_follow_gregorian_centuries() {
    for (date, seconds) in [
        (Civil { year: 1970, month: 1, day: 1 }, 0_u64),
        (Civil { year: 2000, month: 2, day: 29 }, 951_782_400),
        (Civil { year: 2100, month: 2, day: 28 }, 4_107_456_000),
        (Civil { year: 2100, month: 3, day: 1 }, 4_107_542_400),
        (Civil { year: 2400, month: 2, day: 29 }, 13_574_563_200),
    ] {
        let at = Wall::from_nanos(seconds * 1_000_000_000);
        assert_eq!(civil(at), Some(date));
        assert_eq!(wall(date), Some(at));
        assert_eq!(parse(&format(date)), Some(date));
    }
}

#[test]
fn every_wall_day_round_trips_at_midnight() {
    let day_nanos = 86_400_000_000_000;
    for elapsed in 0..=u64::MAX / day_nanos {
        let midnight = Wall::from_nanos(elapsed * day_nanos);
        let date = civil(midnight).unwrap();
        assert_eq!(wall(date), Some(midnight));
        assert_eq!(parse(&format(date)), Some(date));
        let end = Wall::from_nanos(midnight.as_nanos().saturating_add(day_nanos - 1));
        assert_eq!(civil(end), Some(date));
    }
}

#[test]
fn wall_refuses_invalid_dates_and_unrepresentable_midnights() {
    for date in [
        Civil { year: 2100, month: 2, day: 29 },
        Civil { year: 2001, month: 2, day: 29 },
        Civil { year: 2000, month: 4, day: 31 },
        Civil { year: 2000, month: 0, day: 1 },
        Civil { year: 2000, month: 13, day: 1 },
        Civil { year: 2000, month: 1, day: 0 },
        Civil { year: 2000, month: 1, day: 32 },
        Civil { year: 1969, month: 12, day: 31 },
        Civil { year: 2554, month: 7, day: 22 },
        Civil { year: u32::MAX, month: 12, day: 31 },
    ] {
        assert_eq!(wall(date), None, "{date:?}");
    }
    let last = Civil { year: 2554, month: 7, day: 21 };
    assert_eq!(civil(Wall::from_nanos(u64::MAX)), Some(last));
    assert!(wall(last).is_some());
}

#[test]
fn text_round_trips_beyond_the_wall_range() {
    for (text, date) in [
        (b"0000-02-29", Civil { year: 0, month: 2, day: 29 }),
        (b"0001-01-01", Civil { year: 1, month: 1, day: 1 }),
        (b"1900-02-28", Civil { year: 1900, month: 2, day: 28 }),
        (b"9999-12-31", Civil { year: 9999, month: 12, day: 31 }),
    ] {
        assert_eq!(parse(text), Some(date));
        assert_eq!(&format(date), text);
        assert_eq!(wall(date), None);
    }
}

#[test]
fn malformed_text_is_refused() {
    let malformed: &[&[u8]] = &[
        b"",
        b"2000-2-29",
        b"2000-02-9",
        b"200-02-29",
        b"02000-02-29",
        b"2000/02/29",
        b"2000-02-29\n",
        b" 2000-02-29",
        b"2000-02-29 ",
        b"+000-02-29",
        b"200x-02-29",
        b"2000-0x-29",
        b"2000-02-2x",
        b"2000-02-2\xff",
        b"2000-02-2\0",
        b"2100-02-29",
        b"1900-02-29",
        b"2001-02-29",
        b"2000-04-31",
        b"2000-00-01",
        b"2000-13-01",
        b"2000-01-00",
        b"2000-01-32",
        b"2000-99-99",
    ];
    for text in malformed {
        assert_eq!(parse(text), None, "{text:?}");
    }
}

#[test]
fn month_names_use_the_phrase_abbreviations() {
    for (offset, name) in
        [b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"]
            .iter()
            .enumerate()
    {
        assert_eq!(month_name(u8::try_from(offset + 1).unwrap()), Some(name.as_slice()));
    }
    for month in [0, 13, u8::MAX] {
        assert_eq!(month_name(month), None);
    }
}
