//! A reply's trailing choices (temper/docs/design/client/tui/pages/design-session.md, 1).
use alloc::boxed::Box;
use skein_lib::{
    List,
    bytes::{copy_of, decimal, find},
};

#[derive(Debug, PartialEq, Eq)]
pub struct ReplyChoices {
    pub choices: Box<[ReplyChoice]>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ReplyChoice {
    pub words: Box<[u8]>,
    pub reason: Option<Box<[u8]>>,
}

/// Read consecutive numbered items at the end, with the drafter's escape last.
#[must_use]
pub fn reply_choices(text: &[u8]) -> Option<ReplyChoices> {
    let mut items = List::uncapped();
    let mut valid = true;
    let mut start = 0;
    for (end, byte) in text.iter().enumerate() {
        if *byte == b'\n' {
            line(text.get(start..end)?, &mut items, &mut valid);
            start = end.checked_add(1)?;
        }
    }
    line(text.get(start..)?, &mut items, &mut valid);
    if !valid || items.len() < 2 {
        return None;
    }
    let last = items.last()?;
    let words = last.words.strip_suffix(b".").unwrap_or(&last.words);
    if words != b"Tell me something else" && words != b"Tell it something else" {
        return None;
    }
    items.truncate(items.len().checked_sub(1)?);
    Some(ReplyChoices { choices: items.into_boxed() })
}

fn line(bytes: &[u8], items: &mut List<ReplyChoice>, valid: &mut bool) {
    if bytes.trim_ascii().is_empty() {
        return;
    }
    if let Some((number, words)) = numbered(bytes) {
        *valid &= number == u64::from(items.len()).checked_add(1).expect("item number fits");
        items.push(choice(words)).expect("reply items fit memory");
    } else if !bytes.starts_with(b"    ") && !bytes.starts_with(b"\t") {
        items.clear();
        *valid = true;
    }
}
fn numbered(bytes: &[u8]) -> Option<(u64, &[u8])> {
    let mut bytes = bytes;
    for _ in 0_u8..3 {
        if let Some(rest) = bytes.strip_prefix(b" ") {
            bytes = rest;
        }
    }
    let dot = find(bytes, b". ")?;
    let number = decimal(bytes.get(..dot)?)?;
    let words = bytes.get(dot.checked_add(2)?..)?.trim_ascii();
    if words.is_empty() { None } else { Some((number, words)) }
}
fn choice(bytes: &[u8]) -> ReplyChoice {
    let mut plain = List::uncapped();
    let mut skip = false;
    for (index, byte) in bytes.iter().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        let rest = bytes.get(index..).expect("index is in item");
        if rest.starts_with(b"**") || rest.starts_with(b"__") {
            skip = true;
        } else {
            plain.push(*byte).expect("item bytes fit memory");
        }
    }
    let plain = plain.into_boxed();
    let mut cut = None;
    for separator in [" — ".as_bytes(), " – ".as_bytes(), b" - ", b": "] {
        if let Some(at) = find(&plain, separator) {
            let earlier = match cut {
                Some((before, _)) => at < before,
                None => true,
            };
            if earlier {
                cut = Some((at, separator.len()));
            }
        }
    }
    match cut {
        Some((at, length)) => ReplyChoice {
            words: copy_of(plain.get(..at).expect("separator is in item").trim_ascii()),
            reason: Some(copy_of(
                plain.get(at.checked_add(length).expect("separator fits")..).expect("separator fits").trim_ascii(),
            )),
        },
        None => ReplyChoice { words: copy_of(plain.trim_ascii()), reason: None },
    }
}
