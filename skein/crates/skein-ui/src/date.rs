//! The UTC proleptic Gregorian calendar shared by views and phrases
//! (skein/docs/design/ui/README.md, 9).

use skein_lib::Wall;

pub use skein_lib::Civil;

/// The UTC date containing `at`, discarding its time of day.
#[must_use]
pub fn civil(at: Wall) -> Option<Civil> {
    Some(at.civil())
}

/// Midnight UTC, or `None` for an invalid date or a midnight outside [`Wall`]'s range.
#[must_use]
pub fn wall(date: Civil) -> Option<Wall> {
    date.wall()
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
    if date.valid() { Some(date) } else { None }
}

/// Write `YYYY-MM-DD`.
///
/// # Panics
/// The caller must supply a valid calendar date with a year at most 9999.
#[must_use]
pub fn format(date: Civil) -> [u8; 10] {
    assert!(date.valid() && date.year <= 9999, "date must be valid and have a four-digit year");
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
    Civil::month_name(month)
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
