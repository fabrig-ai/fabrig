//! The local command's one-line windows (protocol/programs-claude-code.md, 1.11).
//! Retains at most eight windows, placing UTC resets nearest the host's ask.

use alloc::boxed::Box;
use skein_lib::{Civil, Duration, List, Wall};
use smith_host_domain::{UsageState, UsageWindow};

#[expect(
    clippy::disallowed_methods,
    reason = "the usage protocol boundary validates the command's text before reading lines"
)]
pub(super) fn read(bytes: &[u8], asked: Wall) -> Box<[UsageWindow]> {
    let mut windows = List::with_capacity(8);
    let Ok(text) = core::str::from_utf8(bytes) else {
        return windows.into_boxed();
    };
    let mut started = false;
    for line in text.lines() {
        if started && line.trim().is_empty() {
            break;
        }
        if !line.starts_with("Current session") && !line.starts_with("Current week (") {
            continue;
        }
        started = true;
        if let Some(label) = heading(line)
            && let Some(used) = percent(line)
        {
            let length =
                if label.as_ref() == b"five_hour" { Duration::from_secs(18_000) } else { Duration::from_secs(604_800) };
            windows
                .push(UsageWindow {
                    label,
                    length: Some(length),
                    used: Some(used),
                    resets: reset(line, asked),
                    state: if used == 10_000 { UsageState::Limited } else { UsageState::Open },
                })
                .expect("at most eight usage windows");
            if windows.len() == 8 {
                break;
            }
        }
    }
    windows.into_boxed()
}

fn heading(line: &str) -> Option<Box<[u8]>> {
    if line.starts_with("Current session") {
        return Some(Box::from(b"five_hour".as_slice()));
    }
    if line.starts_with("Current week (all models)") {
        return Some(Box::from(b"seven_day".as_slice()));
    }
    let model = line.strip_prefix("Current week (")?.split([' ', ')']).next()?;
    if model.is_empty() || model.len().checked_add(b"seven_day_".len())? > 32 {
        return None;
    }
    let mut label = List::with_capacity(32);
    for byte in b"seven_day_" {
        label.push(*byte).expect("label prefix fits");
    }
    for byte in model.bytes() {
        if !(0x20..=0x7e).contains(&byte) {
            return None;
        }
        label.push(byte.to_ascii_lowercase()).expect("bounded model label");
    }
    Some(label.into_boxed())
}

fn percent(line: &str) -> Option<u16> {
    let end = line.find("% used")?;
    let number = line.get(..end)?.split_ascii_whitespace().next_back()?;
    let mut parts = number.split('.');
    let whole = digits(parts.next()?)?;
    let fraction = match parts.next() {
        Some(fraction) => {
            if fraction.is_empty() {
                return None;
            }
            let mut value = 0_u16;
            for (index, byte) in fraction.bytes().enumerate() {
                if !byte.is_ascii_digit() {
                    return None;
                }
                if index < 2 {
                    value = value.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
                }
            }
            if fraction.len() == 1 {
                value = value.checked_mul(10)?;
            }
            value
        }
        None => 0,
    };
    if parts.next().is_some() {
        return None;
    }
    let used = whole.checked_mul(100)?.checked_add(fraction)?;
    if used <= 10_000 { Some(used) } else { None }
}

fn reset(line: &str, asked: Wall) -> Option<Wall> {
    let (_, text) = line.split_once("resets ")?;
    let (text, _) = text.split_once(" (UTC)")?;
    let (date, clock) = text.split_once(", ")?;
    let (month, day) = date.split_once(' ')?;
    let month = month_number(month)?;
    let day = u8::try_from(digits(day)?).ok()?;
    let seconds = clock_seconds(clock)?;
    let nanos = seconds.checked_mul(1_000_000_000)?;
    let year = asked.civil().year;
    let mut nearest = None;
    let mut distance = u64::MAX;
    for year in [Some(year), year.checked_sub(1), year.checked_add(1)].into_iter().flatten() {
        if let Some(midnight) = (Civil { year, month, day }).wall()
            && let Some(at) = midnight.as_nanos().checked_add(nanos)
        {
            let apart = at.abs_diff(asked.as_nanos());
            if apart < distance {
                nearest = Some(Wall::from_nanos(at));
                distance = apart;
            }
        }
    }
    nearest
}

fn month_number(name: &str) -> Option<u8> {
    for month in 1..=12 {
        if Civil::month_name(month)? == name.as_bytes() {
            return Some(month);
        }
    }
    None
}

fn clock_seconds(clock: &str) -> Option<u64> {
    let (clock, afternoon) =
        if let Some(clock) = clock.strip_suffix("am") { (clock, false) } else { (clock.strip_suffix("pm")?, true) };
    let (hour, minute) = match clock.split_once(':') {
        Some((hour, minute)) => {
            if minute.len() != 2 {
                return None;
            }
            (digits(hour)?, digits(minute)?)
        }
        None => (digits(clock)?, 0),
    };
    if !(1..=12).contains(&hour) || minute > 59 {
        return None;
    }
    let hour = hour.rem_euclid(12).checked_add(if afternoon { 12 } else { 0 })?;
    u64::from(hour).checked_mul(3600)?.checked_add(u64::from(minute).checked_mul(60)?)
}

fn digits(text: &str) -> Option<u16> {
    if text.is_empty() {
        return None;
    }
    let mut number = 0_u16;
    for byte in text.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
