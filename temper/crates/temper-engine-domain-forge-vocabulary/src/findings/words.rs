//! Finding notifications and their bounded text (temper/docs/design/domain/areas.md, 5).

use super::{self as forge, Filer};
use alloc::boxed::Box;
use skein_lib::{Decimal, List};

pub fn ownership_words(project: u32, numbers: &[u32], limit: u32) -> Box<[u8]> {
    let mut out = List::uncapped();
    append(&mut out, b"Findings on project ");
    append(&mut out, Decimal::of(u64::from(project)).as_bytes());
    append(&mut out, b" are yours now:");
    let header = out.as_slice().len();
    for (index, &number) in numbers.iter().enumerate() {
        if index == 0 {
            append(&mut out, b" ");
        } else {
            append(&mut out, b", ");
        }
        append(&mut out, Decimal::of(u64::from(number)).as_bytes());
    }
    let limit = skein_lib::usize_of(limit);
    if out.as_slice().len() <= limit {
        return out.into_boxed();
    }
    let mut end = header;
    let mut kept = header;
    let mut count = 0_usize;
    for (index, &number) in numbers.iter().enumerate() {
        end = end.saturating_add(if index == 0 { 1 } else { 2 });
        end = end.saturating_add(Decimal::of(u64::from(number)).as_bytes().len());
        let remaining = numbers.len().saturating_sub(index).saturating_sub(1);
        let suffix = Decimal::of(skein_lib::len_u64(remaining));
        if end.saturating_add(11).saturating_add(suffix.as_bytes().len()) <= limit {
            kept = end;
            count = index.saturating_add(1);
        }
    }
    let more = Decimal::of(skein_lib::len_u64(numbers.len().saturating_sub(count)));
    let mut words = List::uncapped();
    if count == 0 {
        // A tiny message bound may not fit even the header and the omitted count.
        let prefix = cut_words(
            out.as_slice().get(..header).expect("header prefix"),
            u32::try_from(limit.saturating_sub(10).saturating_sub(more.as_bytes().len())).expect("message bound"),
        );
        append(&mut words, &prefix);
    } else {
        append(&mut words, out.as_slice().get(..kept).expect("fitting number prefix"));
    }
    if count != 0 {
        append(&mut words, b", ");
    } else {
        append(&mut words, b" ");
    }
    append(&mut words, b"and ");
    append(&mut words, more.as_bytes());
    append(&mut words, b" more");
    words.into_boxed()
}

fn append(out: &mut List<u8>, words: &[u8]) {
    for &byte in words {
        out.push(byte).expect("bounded finding words");
    }
}
pub fn news_words(row: &forge::Finding, hand: bool, limit: u32) -> Box<[u8]> {
    let mut out = List::uncapped();
    for words in [
        b"Finding ".as_slice(),
        Decimal::of(u64::from(row.project)).as_bytes(),
        b"/",
        Decimal::of(u64::from(row.number)).as_bytes(),
        if row.state == forge::FindingState::Routed { b" for you: ".as_slice() } else { b" is yours: " },
        &row.title,
        b"\n",
    ] {
        append(&mut out, words);
    }
    let kind: &[u8] = match row.kind {
        forge::FindingKind::Qa => b"qa",
        forge::FindingKind::Flake => b"flake",
        forge::FindingKind::Red => b"red",
        forge::FindingKind::Review => b"review",
        forge::FindingKind::Feedback => b"feedback",
    };
    let severity: &[u8] = match row.severity {
        forge::Severity::Low => b"low",
        forge::Severity::Medium => b"medium",
        forge::Severity::High => b"high",
    };
    append(&mut out, kind);
    append(&mut out, b", ");
    append(&mut out, severity);
    append(&mut out, b", ");
    match &row.area {
        Some(area) => {
            append(&mut out, b"area ");
            append(&mut out, area);
        }
        None => append(&mut out, b"no area"),
    }
    append(&mut out, b"\n");
    if hand && let (Some(by), Some(why)) = (row.hander, &row.why) {
        append(&mut out, b"Handed on by ");
        match by {
            Filer::Task(number) => {
                append(&mut out, b"task ");
                append(&mut out, Decimal::of(number).as_bytes());
            }
            Filer::Party(number) => {
                append(&mut out, b"party ");
                append(&mut out, Decimal::of(number).as_bytes());
            }
        }
        append(&mut out, b": ");
        append(&mut out, why);
        append(&mut out, b"\n");
    }
    append(&mut out, &row.body);
    cut_words(out.as_slice(), limit)
}
pub fn cut_words(words: &[u8], limit: u32) -> Box<[u8]> {
    let limit = skein_lib::usize_of(limit);
    if words.len() <= limit {
        return words.into();
    }
    let marker = b"[cut]";
    let mut end = limit.saturating_sub(marker.len());
    while end > 0
        && match words.get(end) {
            Some(byte) => byte & 0xc0 == 0x80,
            None => false,
        }
    {
        end = end.saturating_sub(1);
    }
    let mut out = List::uncapped();
    append(&mut out, words.get(..end).expect("prefix within message"));
    append(&mut out, marker.get(..marker.len().min(limit)).expect("cut marker prefix"));
    out.into_boxed()
}
