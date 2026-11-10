//! RFC 3339 and HTTP dates (skein/docs/design/forge/README.md, 4.4).
use skein_lib::{Civil, Wall};
fn number(text: &[u8], start: usize, end: usize) -> Option<u64> {
    skein_lib::bytes::decimal(text.get(start..end)?)
}
#[must_use]
pub fn read(text: &[u8]) -> Option<Wall> {
    for (at, byte) in [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')] {
        if text.get(at) != Some(&byte) {
            return None;
        }
    }
    let days = Civil {
        year: u32::try_from(number(text, 0, 4)?).ok()?,
        month: u8::try_from(number(text, 5, 7)?).ok()?,
        day: u8::try_from(number(text, 8, 10)?).ok()?,
    }
    .days()?;
    let hour = number(text, 11, 13)?;
    let minute = number(text, 14, 16)?;
    let second = number(text, 17, 19)?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let mut at = 19usize;
    if text.get(at) == Some(&b'.') {
        at = at.checked_add(1)?;
        let start = at;
        for byte in text.get(at..)? {
            if !byte.is_ascii_digit() {
                break;
            }
            at = at.checked_add(1)?;
        }
        if at == start {
            return None;
        }
    }
    let offset = match text.get(at..)? {
        b"Z" => 0i64,
        tail if tail.len() == 6
            && (tail.first() == Some(&b'+') || tail.first() == Some(&b'-'))
            && tail.get(3) == Some(&b':') =>
        {
            let h = number(tail, 1, 3)?;
            let m = number(tail, 4, 6)?;
            if h > 23 || m > 59 {
                return None;
            }
            let seconds = i64::try_from(h.checked_mul(3600)?.checked_add(m.checked_mul(60)?)?).ok()?;
            if tail.first() == Some(&b'-') { seconds.checked_neg()? } else { seconds }
        }
        _ => return None,
    };
    let daytime = hour.checked_mul(3600)?.checked_add(minute.checked_mul(60)?)?.checked_add(second)?;
    let seconds = days.checked_mul(86400)?.checked_add(i64::try_from(daytime).ok()?)?.checked_sub(offset)?;
    Some(Wall::from_nanos(u64::try_from(seconds).ok()?.checked_mul(1_000_000_000)?))
}
#[must_use]
pub fn write(wall: Wall) -> [u8; 20] {
    let seconds = wall.as_secs();
    let date = wall.civil();
    let mut output = *b"0000-00-00T00:00:00Z";
    for (start, width, mut value) in [
        (0usize, 4usize, u64::from(date.year)),
        (5, 2, u64::from(date.month)),
        (8, 2, u64::from(date.day)),
        (11, 2, seconds / 3600 % 24),
        (14, 2, seconds / 60 % 60),
        (17, 2, seconds % 60),
    ] {
        for pos in (start..start.saturating_add(width)).rev() {
            *output.get_mut(pos).expect("fixed date layout") =
                b'0'.saturating_add(u8::try_from(value % 10).expect("digit"));
            value /= 10;
        }
    }
    output
}
/// Reads IMF-fixdate, the only HTTP date syntax used by these documents.
#[must_use]
pub fn http_date(text: &[u8]) -> Option<Wall> {
    if text.len() != 29
        || text.get(3..5) != Some(b", ")
        || text.get(7) != Some(&b' ')
        || text.get(11) != Some(&b' ')
        || text.get(16) != Some(&b' ')
        || text.get(25..) != Some(b" GMT")
    {
        return None;
    }
    match text.get(..3)? {
        b"Mon" | b"Tue" | b"Wed" | b"Thu" | b"Fri" | b"Sat" | b"Sun" => {}
        _ => return None,
    }
    let name = text.get(8..11)?;
    let mut month = None;
    for value in 1..=12 {
        if Civil::month_name(value) == Some(name) {
            month = Some(value);
            break;
        }
    }
    let month = month?;
    let mut date = *b"0000-00-00T00:00:00Z";
    for (from, to, width) in [(12usize, 0usize, 4usize), (5, 8, 2), (17, 11, 8)] {
        for (input, output) in
            text.get(from..from.checked_add(width)?)?.iter().zip(date.get_mut(to..to.checked_add(width)?)?.iter_mut())
        {
            *output = *input;
        }
    }
    *date.get_mut(5)? = b'0'.checked_add(month.checked_div(10)?)?;
    *date.get_mut(6)? = b'0'.checked_add(month.checked_rem(10)?)?;
    read(&date)
}
