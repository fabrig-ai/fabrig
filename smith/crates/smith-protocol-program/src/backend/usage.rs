//! Read allowance observations without floating point (protocol/programs-claude-code.md, 1.6).
//! No observation is retained here; every well-shaped event is sent to its owner.

use crate::mcp::json;
use skein_json::{Document, Kind};
use skein_lib::Duration;
use smith_domain::{UsageState, UsageWindow};

pub(super) fn window(document: &Document, info: Option<u32>) -> Option<UsageWindow> {
    let label = json::text_in(document, json::field_in(document, info, b"rateLimitType"))?;
    if label.is_empty() || label.len() > 32 {
        return None;
    }
    for byte in label {
        if !(b' '..=b'~').contains(byte) {
            return None;
        }
    }
    let state = match json::text_in(document, json::field_in(document, info, b"status")) {
        Some(b"allowed") => UsageState::Open,
        Some(b"allowed_warning") => UsageState::Warned,
        Some(b"rejected") => UsageState::Limited,
        Some(_) | None => return None,
    };
    let length = if label == b"five_hour" {
        Some(Duration::from_secs(18_000))
    } else if label.starts_with(b"seven_day") {
        Some(Duration::from_secs(604_800))
    } else {
        None
    };
    Some(UsageWindow {
        label: label.into(),
        length,
        used: share(document, json::field_in(document, info, b"utilization")),
        resets: super::turns::number(document, json::field_in(document, info, b"resetsAt")),
        state,
    })
}

fn share(document: &Document, index: Option<u32>) -> Option<u16> {
    let token = document.token(index?)?;
    if token.kind != Kind::Number {
        return None;
    }
    let text = document.text(token)?;
    let (&whole, rest) = text.split_first()?;
    if whole != b'0' && whole != b'1' {
        return None;
    }
    if rest.is_empty() {
        return Some(if whole == b'1' { 10_000 } else { 0 });
    }
    let fraction = rest.strip_prefix(b".")?;
    if fraction.is_empty() {
        return None;
    }
    let mut used = 0_u16;
    let mut place = 1_000_u16;
    for digit in fraction {
        if !digit.is_ascii_digit() || (whole == b'1' && *digit != b'0') {
            return None;
        }
        used = used.checked_add(u16::from(digit.checked_sub(b'0')?).checked_mul(place)?)?;
        place = place.checked_div(10)?;
    }
    Some(if whole == b'1' { 10_000 } else { used })
}
