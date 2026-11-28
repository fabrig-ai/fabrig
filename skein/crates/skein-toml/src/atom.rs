//! Booleans, integers, floats and date/time forms (skein/docs/design/data/toml.md, 2 and 4).

use skein_lib::{List, bytes};

use crate::cursor::Cursor;
use crate::document::{DateTimeKind, Stored, Text};
use crate::{Refusal, Why};

pub(crate) fn read(cursor: &mut Cursor, input: &[u8], text: &mut List<u8>) -> Result<Stored, Refusal> {
    let start = cursor.at;
    match cursor.peek(input) {
        Some(b't') => {
            word(cursor, input, b"true")?;
            Ok(Stored::Boolean(true))
        }
        Some(b'f') => {
            word(cursor, input, b"false")?;
            Ok(Stored::Boolean(false))
        }
        Some(b'0'..=b'9') if is_local_time(cursor, input) => {
            time(cursor, input)?;
            let text = keep(cursor, input, text, start);
            Ok(Stored::DateTime { kind: DateTimeKind::LocalTime, text })
        }
        Some(b'0'..=b'9') if is_date(cursor, input) => {
            let kind = date_time(cursor, input)?;
            let text = keep(cursor, input, text, start);
            Ok(Stored::DateTime { kind, text })
        }
        Some(b'0'..=b'9' | b'+' | b'-' | b'i' | b'n') => number(cursor, input, text, start),
        Some(_) | None => Err(cursor.grammar(input)),
    }
}

fn word(cursor: &mut Cursor, input: &[u8], word: &[u8]) -> Result<(), Refusal> {
    for &byte in word {
        cursor.need(input, byte)?;
    }
    Ok(())
}

fn keep(cursor: &Cursor, input: &[u8], text: &mut List<u8>, start: usize) -> Text {
    let first = text.len();
    for &byte in input.get(start..cursor.at).expect("atom occupies bytes already read") {
        text.push(byte).expect("kept atoms occupy at most their admitted input bytes");
    }
    Text::since(first, text)
}

fn number(cursor: &mut Cursor, input: &[u8], text: &mut List<u8>, start: usize) -> Result<Stored, Refusal> {
    let negative = cursor.eat(input, b'-');
    if !negative {
        cursor.eat(input, b'+');
    }
    let digits_from = cursor.at;
    match cursor.peek(input) {
        Some(b'i') => {
            word(cursor, input, b"inf")?;
            return Ok(Stored::Float(keep(cursor, input, text, start)));
        }
        Some(b'n') => {
            word(cursor, input, b"nan")?;
            return Ok(Stored::Float(keep(cursor, input, text, start)));
        }
        Some(b'0') => {
            cursor.need(input, b'0')?;
            if digits_from == start {
                let radix = match cursor.peek(input) {
                    Some(b'x') => 16,
                    Some(b'o') => 8,
                    Some(b'b') => 2,
                    Some(_) | None => 0,
                };
                if radix != 0 {
                    cursor.take(input)?;
                    let from = cursor.at;
                    digits(cursor, input, radix)?;
                    return integer(cursor, input, from, radix, false);
                }
            }
        }
        Some(b'1'..=b'9') => digits(cursor, input, 10)?,
        Some(_) | None => return Err(cursor.grammar(input)),
    }
    let fraction = cursor.eat(input, b'.');
    if fraction {
        digits(cursor, input, 10)?;
    }
    let exponent = cursor.eat(input, b'e') || cursor.eat(input, b'E');
    if exponent {
        if !cursor.eat(input, b'-') {
            cursor.eat(input, b'+');
        }
        digits(cursor, input, 10)?;
    }
    if fraction || exponent {
        return Ok(Stored::Float(keep(cursor, input, text, start)));
    }
    integer(cursor, input, digits_from, 10, negative)
}

fn digit(byte: Option<u8>, radix: u8) -> Option<u8> {
    let value = bytes::hex_value(byte?)?;
    if value < radix { Some(value) } else { None }
}

// At least one digit; underscores occur only between digits.
fn digits(cursor: &mut Cursor, input: &[u8], radix: u8) -> Result<(), Refusal> {
    if digit(cursor.peek(input), radix).is_none() {
        return Err(cursor.grammar(input));
    }
    cursor.take(input)?;
    for _ in cursor.at..input.len() {
        if cursor.eat(input, b'_') {
            if digit(cursor.peek(input), radix).is_none() {
                return Err(cursor.grammar(input));
            }
        } else if digit(cursor.peek(input), radix).is_none() {
            break;
        }
        cursor.take(input)?;
    }
    Ok(())
}

fn integer(cursor: &Cursor, input: &[u8], from: usize, radix: u8, negative: bool) -> Result<Stored, Refusal> {
    let mut value = 0_i64;
    let range = cursor.refuse(Why::Range);
    for &byte in input.get(from..cursor.at).expect("the integer's digits were just read") {
        if byte != b'_' {
            let digit = bytes::hex_value(byte).expect("integer syntax checked the digit");
            let digit = i64::from(digit);
            let scaled = value.checked_mul(i64::from(radix)).ok_or(range)?;
            // Accumulate a negative integer in its negative range, including i64::MIN.
            value = if negative { scaled.checked_sub(digit) } else { scaled.checked_add(digit) }.ok_or(range)?;
        }
    }
    Ok(Stored::Integer(value))
}

fn is_local_time(cursor: &Cursor, input: &[u8]) -> bool {
    digit(cursor.ahead(input, 1), 10).is_some() && cursor.ahead(input, 2) == Some(b':')
}

fn is_date(cursor: &Cursor, input: &[u8]) -> bool {
    for offset in 1..4 {
        if digit(cursor.ahead(input, offset), 10).is_none() {
            return false;
        }
    }
    cursor.ahead(input, 4) == Some(b'-')
}

fn date_time(cursor: &mut Cursor, input: &[u8]) -> Result<DateTimeKind, Refusal> {
    let year = fixed_decimal(cursor, input, 4)?;
    cursor.need(input, b'-')?;
    let month = component(cursor, input, 12)?;
    if month == 0 {
        return Err(cursor.refuse(Why::Range));
    }
    cursor.need(input, b'-')?;
    let day = component(cursor, input, days_in_month(year, month))?;
    if day == 0 {
        return Err(cursor.refuse(Why::Range));
    }
    if !date_time_separator(cursor, input)? {
        return Ok(DateTimeKind::LocalDate);
    }
    time(cursor, input)?;
    match cursor.peek(input) {
        Some(b'Z' | b'z') => {
            cursor.take(input)?;
            Ok(DateTimeKind::OffsetDateTime)
        }
        Some(b'+' | b'-') => {
            cursor.take(input)?;
            component(cursor, input, 23)?;
            cursor.need(input, b':')?;
            component(cursor, input, 59)?;
            Ok(DateTimeKind::OffsetDateTime)
        }
        Some(_) | None => Ok(DateTimeKind::LocalDateTime),
    }
}

fn date_time_separator(cursor: &mut Cursor, input: &[u8]) -> Result<bool, Refusal> {
    match cursor.peek(input) {
        Some(b'T' | b't') => {}
        Some(b' ') if digit(cursor.ahead(input, 1), 10).is_some() => {}
        Some(_) | None => return Ok(false),
    }
    cursor.take(input)?;
    Ok(true)
}

fn time(cursor: &mut Cursor, input: &[u8]) -> Result<(), Refusal> {
    component(cursor, input, 23)?;
    cursor.need(input, b':')?;
    component(cursor, input, 59)?;
    cursor.need(input, b':')?;
    component(cursor, input, 60)?;
    if cursor.eat(input, b'.') {
        if digit(cursor.peek(input), 10).is_none() {
            return Err(cursor.grammar(input));
        }
        for _ in cursor.at..input.len() {
            if digit(cursor.peek(input), 10).is_none() {
                break;
            }
            cursor.take(input)?;
        }
    }
    Ok(())
}

fn component(cursor: &mut Cursor, input: &[u8], max: u16) -> Result<u16, Refusal> {
    let component = fixed_decimal(cursor, input, 2)?;
    if component > max { Err(cursor.refuse(Why::Range)) } else { Ok(component) }
}

fn fixed_decimal(cursor: &mut Cursor, input: &[u8], count: u32) -> Result<u16, Refusal> {
    let mut number = 0_u16;
    for _ in 0..count {
        let Some(digit) = digit(cursor.peek(input), 10) else { return Err(cursor.grammar(input)) };
        cursor.take(input)?;
        number = number.saturating_mul(10).saturating_add(u16::from(digit));
    }
    Ok(number)
}

const fn days_in_month(year: u16, month: u16) -> u16 {
    match month {
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
