//! The local command's line headings and decimal percent (protocol/programs.md, 5.11).
//! Retains only the pending heading and eight completed windows, never reset text.

use alloc::boxed::Box;
use skein_lib::{Duration, List};
use smith_host_domain::{UsageState, UsageWindow};

#[expect(
    clippy::disallowed_methods,
    reason = "the usage protocol boundary validates the command's text before reading lines"
)]
pub(super) fn read(bytes: &[u8]) -> Box<[UsageWindow]> {
    let mut windows = List::with_capacity(8);
    let Ok(text) = core::str::from_utf8(bytes) else {
        return windows.into_boxed();
    };
    let mut pending = None;
    for line in text.lines() {
        if line.starts_with("Current session") || line.starts_with("Current week (") {
            pending = heading(line);
        } else if let Some(used) = percent(line)
            && let Some(label) = pending.take()
        {
            let length =
                if label.as_ref() == b"five_hour" { Duration::from_secs(18_000) } else { Duration::from_secs(604_800) };
            windows
                .push(UsageWindow {
                    label,
                    length: Some(length),
                    used: Some(used),
                    resets: None,
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
