//! Shared shell words for a full deployment (client/README.md, 6.7).
use alloc::boxed::Box;
use skein_lib::{Decimal, Wall, Writer};
const FULL: [&[u8]; 4] = [
    b": all ",
    b" sign-ins are in use; the earliest ends at ",
    b"; this link stays usable until ",
    b"; after that, temper serve prints a fresh one when it is tried\n",
];

/// Universal time to the second. Wall's u64 nanoseconds bound the year to 2554.
#[must_use]
#[expect(clippy::arithmetic_side_effects, reason = "Wall bounds Gregorian era intermediates and clock fields")]
pub fn utc_second(at: Wall) -> [u8; 20] {
    let seconds = at.as_nanos() / 1_000_000_000;
    let days = seconds / 86_400 + 719_468;
    let era = days / 146_097;
    let doe = days % 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = era * 400 + yoe + u64::from(month <= 2);
    let hour = seconds % 86_400 / 3600;
    let minute = seconds % 3600 / 60;
    let second = seconds % 60;
    let mut text = *b"0000-00-00T00:00:00Z";
    for (at, value) in [(0, year / 100), (2, year % 100), (5, month), (8, day), (11, hour), (14, minute), (17, second)]
    {
        *text.get_mut(at).expect("UTC field offset") = b'0' + u8::try_from(value / 10).expect("decimal tens digit");
        *text.get_mut(at + 1).expect("UTC field offset") =
            b'0' + u8::try_from(value % 10).expect("decimal units digit");
    }
    text
}

/// The complete diagnostic, including its prefix and final newline.
#[must_use]
pub fn full_line(prefix: &str, sign_ins: u32, earliest: Wall, link_until: Wall) -> Box<[u8]> {
    let count = Decimal::of(u64::from(sign_ins));
    let earliest = utc_second(earliest);
    let link_until = utc_second(link_until);
    let [lead, end, link, fresh] = FULL;
    let parts = [prefix.as_bytes(), lead, count.as_bytes(), end, &earliest, link, &link_until, fresh];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("diagnostic length fits");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured full diagnostic");
    }
    writer.finish()
}

/// Largest full diagnostic, including a u32 count and the two UTC moments.
#[must_use]
pub fn full_line_bound(prefix: &str) -> Option<u64> {
    let mut bytes = u64::try_from(prefix.len()).ok()?.checked_add(10)?.checked_add(40)?;
    for literal in FULL {
        bytes = bytes.checked_add(u64::try_from(literal.len()).ok()?)?;
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_universal_midnight() {
        assert_eq!(&utc_second(Wall::EPOCH), b"1970-01-01T00:00:00Z");
    }
    #[test]
    fn a_leap_day_keeps_its_date_and_drops_fractional_seconds() {
        assert_eq!(&utc_second(Wall::from_nanos(951_827_696_999_999_999)), b"2000-02-29T12:34:56Z");
    }
    #[test]
    fn the_last_second_of_a_year_precedes_next_years_midnight() {
        assert_eq!(&utc_second(Wall::from_nanos(1_767_225_599_000_000_000)), b"2025-12-31T23:59:59Z");
        assert_eq!(&utc_second(Wall::from_nanos(1_767_225_600_000_000_000)), b"2026-01-01T00:00:00Z");
    }
}
