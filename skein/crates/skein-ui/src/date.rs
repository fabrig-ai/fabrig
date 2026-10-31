//! The UTC proleptic Gregorian calendar shared by views and phrases
//! (skein/docs/design/ui.md, 9).

use skein_lib::Wall;

/// A proleptic Gregorian date. Public fields may describe an invalid date;
/// [`wall`] and [`parse`] validate calendar dates.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Civil {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

const MONTHS: [&[u8]; 12] =
    [b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"];

/// The UTC date containing `at`, discarding its time of day.
#[must_use]
pub fn civil(at: Wall) -> Option<Civil> {
    // March-based Gregorian eras: each 400-year era has 146_097 days.
    // The offset maps the Unix epoch to this civil-day numbering.
    let days = at.as_secs().checked_div(86_400)?.checked_add(719_468)?;
    let era = days.checked_div(146_097)?;
    let era_day = days.checked_rem(146_097)?;
    let era_year = era_day
        .checked_sub(era_day.checked_div(1_460)?)?
        .checked_add(era_day.checked_div(36_524)?)?
        .checked_sub(era_day.checked_div(146_096)?)?
        .checked_div(365)?;
    let year_day = era_day.checked_sub(
        era_year.checked_mul(365)?.checked_add(era_year.checked_div(4)?)?.checked_sub(era_year.checked_div(100)?)?,
    )?;
    let march_month = year_day.checked_mul(5)?.checked_add(2)?.checked_div(153)?;
    let day = year_day.checked_sub(march_month.checked_mul(153)?.checked_add(2)?.checked_div(5)?)?.checked_add(1)?;
    let month = if march_month < 10 { march_month.checked_add(3)? } else { march_month.checked_sub(9)? };
    let year = era.checked_mul(400)?.checked_add(era_year)?.checked_add(u64::from(month <= 2))?;
    Some(Civil { year: u32::try_from(year).ok()?, month: u8::try_from(month).ok()?, day: u8::try_from(day).ok()? })
}

/// Midnight UTC, or `None` for an invalid date or a midnight outside [`Wall`]'s range.
#[must_use]
pub fn wall(date: Civil) -> Option<Wall> {
    if !valid(date) || date.year < 1970 {
        return None;
    }
    let year = u64::from(date.year).checked_sub(u64::from(date.month <= 2))?;
    let era = year.checked_div(400)?;
    let era_year = year.checked_rem(400)?;
    let month = u64::from(date.month);
    let march_month = if month > 2 { month.checked_sub(3)? } else { month.checked_add(9)? };
    let year_day = march_month
        .checked_mul(153)?
        .checked_add(2)?
        .checked_div(5)?
        .checked_add(u64::from(date.day))?
        .checked_sub(1)?;
    let era_day = era_year
        .checked_mul(365)?
        .checked_add(era_year.checked_div(4)?)?
        .checked_sub(era_year.checked_div(100)?)?
        .checked_add(year_day)?;
    let days = era.checked_mul(146_097)?.checked_add(era_day)?.checked_sub(719_468)?;
    Some(Wall::from_nanos(days.checked_mul(86_400)?.checked_mul(1_000_000_000)?))
}

/// Read exactly `YYYY-MM-DD`, accepting valid dates in years 0000 through 9999.
/// The date need not fit in [`Wall`].
#[must_use]
pub fn parse(text: &[u8]) -> Option<Civil> {
    let [y0, y1, y2, y3, b'-', m0, m1, b'-', d0, d1] = text else {
        return None;
    };
    let date = Civil {
        year: number(&[*y0, *y1, *y2, *y3])?,
        month: u8::try_from(number(&[*m0, *m1])?).ok()?,
        day: u8::try_from(number(&[*d0, *d1])?).ok()?,
    };
    if valid(date) { Some(date) } else { None }
}

/// Write `YYYY-MM-DD`.
///
/// # Panics
/// The caller must supply a valid calendar date with a year at most 9999.
#[must_use]
pub fn format(date: Civil) -> [u8; 10] {
    assert!(valid(date) && date.year <= 9999, "date must be valid and have a four-digit year");
    [
        digit(date.year, 1000),
        digit(date.year, 100),
        digit(date.year, 10),
        digit(date.year, 1),
        b'-',
        digit(u32::from(date.month), 10),
        digit(u32::from(date.month), 1),
        b'-',
        digit(u32::from(date.day), 10),
        digit(u32::from(date.day), 1),
    ]
}

/// The English three-byte month abbreviation, or `None` outside 1 through 12.
#[must_use]
pub fn month_name(month: u8) -> Option<&'static [u8]> {
    MONTHS.get(usize::from(month.checked_sub(1)?)).copied()
}

fn valid(date: Civil) -> bool {
    let leap = date.year.is_multiple_of(4) && (!date.year.is_multiple_of(100) || date.year.is_multiple_of(400));
    let days = match date.month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        0 | 13..=u8::MAX => return false,
    };
    date.day >= 1 && date.day <= days
}

fn number(text: &[u8]) -> Option<u32> {
    let mut value = 0_u32;
    for byte in text {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

fn digit(value: u32, divisor: u32) -> u8 {
    let digit = value
        .checked_div(divisor)
        .expect("decimal divisor is nonzero")
        .checked_rem(10)
        .expect("decimal radix is nonzero");
    b'0'.checked_add(u8::try_from(digit).expect("decimal digit fits in u8")).expect("ASCII digit fits in u8")
}
