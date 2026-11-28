//! Member commit messages and squash recognition (temper/docs/design/protocol/tooling.md, 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_lib::{Decimal, List, Writer, bytes};

#[cfg(test)]
mod tests;

const MESSAGE_BYTES: usize = 4096;
/// The most bytes `trailers` keeps: none past them fits a message.
pub const TRAILER_BYTES: usize = MESSAGE_BYTES;
const ELLIPSIS: &[u8] = "…".as_bytes();
const TITLE_BYTES: usize = 256;
const DESCRIPTION_BYTES: usize = 1024;

/// The frozen member and its producer's delivered title and words.
#[derive(Debug)]
pub struct Member<'a> {
    pub title: &'a [u8],
    pub words: &'a [u8],
    pub change: u64,
    pub pull: u64,
    pub head: Commit,
    pub goal: Option<u64>,
    /// The member's head commit's TRAILERS, as `trailers` gave them from
    /// its message; empty where the round read none.
    pub head_trailers: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Squash,
    Merge { into_main: bool },
}

/// Compose a squash with the member's own trailers, or a merge with a
/// goal trailer only on a train into main.
#[must_use]
pub fn commit_message(member: &Member<'_>, kind: Kind) -> Box<[u8]> {
    let words_trailers;
    let (own, with_goal) = match kind {
        Kind::Squash => {
            words_trailers = trailers(member.words);
            let own = if words_trailers.is_empty() { member.head_trailers } else { words_trailers.as_ref() };
            (own, true)
        }
        Kind::Merge { into_main } => (b"".as_slice(), into_main),
    };
    let required = required_trailers(member, with_goal);
    let title = fit(utf8_prefix(member.title).as_bytes(), TITLE_BYTES);
    compose(utf8_prefix(&title), &required, own, member.words, DescriptionSource::Words)
}

/// Compose a squash with the head's trailers in place of the supplied ones.
/// The producer's words must keep no trailers. A message that does not have
/// the squash's shape is returned unchanged.
#[must_use]
pub fn with_head_trailers(squash: &[u8], head_message: &[u8]) -> Box<[u8]> {
    let Some((title, description, required)) = squash_parts(squash) else { return squash.into() };
    compose(title, required.as_bytes(), &trailers(head_message), description.as_bytes(), DescriptionSource::Wrapped)
}

fn squash_parts(squash: &[u8]) -> Option<(&str, &str, &str)> {
    if squash.len() > MESSAGE_BYTES {
        return None;
    }
    let text = utf8_prefix(squash);
    if text.len() != squash.len() {
        return None;
    }
    let (body, last) = text.rsplit_once("\n\n")?;
    // The composed title may be empty, but cannot contain a newline.
    let (title, description) = body.split_once("\n\n").unwrap_or((body, ""));
    if title.len() > TITLE_BYTES || description.len() > DESCRIPTION_BYTES {
        return None;
    }
    for character in title.chars() {
        if character.is_control() {
            return None;
        }
    }
    if description.contains("\n\n") || description.starts_with('\n') || description.ends_with('\n') {
        return None;
    }
    for character in description.chars() {
        if character != '\n' && character.is_control() {
            return None;
        }
    }
    Some((title, description, required_suffix(last)?))
}

fn required_suffix(last: &str) -> Option<&str> {
    let (prefix, line) = last.rsplit_once('\n')?;
    let (prefix, head) = if let Some(goal) = line.strip_prefix("Goal: ") {
        bytes::decimal(goal.as_bytes())?;
        prefix.rsplit_once('\n')?
    } else {
        (prefix, line)
    };
    let head = head.strip_prefix("Head: ")?;
    if head.len() != 40 && head.len() != 64 {
        return None;
    }
    for byte in head.as_bytes() {
        if !byte.is_ascii_hexdigit() {
            return None;
        }
    }
    let (prefix, pull) = prefix.rsplit_once('\n')?;
    bytes::decimal(pull.strip_prefix("Pull-request: ")?.as_bytes())?;
    let (own, change) = prefix.rsplit_once('\n').unwrap_or(("", prefix));
    bytes::decimal(change.strip_prefix("Change: ")?.as_bytes())?;
    if own.is_empty() {
        return Some(last);
    }
    for line in own.split('\n') {
        if line.trim_end() != line || reserved_trailer(trailer_token(line)?) {
            return None;
        }
    }
    last.get(own.len().checked_add(1)?..)
}

enum DescriptionSource {
    Words,
    Wrapped,
}

fn compose(title: &str, required: &[u8], own: &[u8], text: &[u8], source: DescriptionSource) -> Box<[u8]> {
    let mut out = List::with_capacity(u32::try_from(MESSAGE_BYTES).expect("small checkout message bound"));
    for character in title.chars() {
        put_character(&mut out, if character.is_control() { ' ' } else { character });
    }
    // The title and temper's trailers always fit. The member's lines take
    // room before the description; a dropped line leaves no description.
    let room = usize::try_from(out.room()).expect("small checkout message bound");
    let room = room
        .checked_sub(required.len().checked_add(2).expect("temper trailers and separator"))
        .expect("title and temper trailers fit");
    let kept = whole_lines(own, room.saturating_sub(1));
    if kept.len() == own.len() {
        let own_bytes = if kept.is_empty() { 0 } else { kept.len().checked_add(1).expect("trailer and LF fit") };
        let bound = room.saturating_sub(own_bytes).saturating_sub(2).min(DESCRIPTION_BYTES);
        let description = match source {
            DescriptionSource::Words => description(text, bound),
            DescriptionSource::Wrapped => refit_description(text, bound),
        };
        if !description.is_empty() {
            append(&mut out, b"\n\n");
            append(&mut out, &description);
        }
    }
    append(&mut out, b"\n\n");
    if !kept.is_empty() {
        append(&mut out, kept);
        append(&mut out, b"\n");
    }
    append(&mut out, required);
    out.into_boxed()
}

fn required_trailers(member: &Member<'_>, with_goal: bool) -> Box<[u8]> {
    let mut out = List::with_capacity(256);
    append(&mut out, b"Change: ");
    append(&mut out, Decimal::of(member.change).as_bytes());
    append(&mut out, b"\nPull-request: ");
    append(&mut out, Decimal::of(member.pull).as_bytes());
    append(&mut out, b"\nHead: ");
    append(&mut out, &commit_text(&member.head));
    if with_goal && let Some(goal) = member.goal {
        append(&mut out, b"\nGoal: ");
        append(&mut out, Decimal::of(goal).as_bytes());
    }
    out.into_boxed()
}

fn whole_lines(block: &[u8], room: usize) -> &[u8] {
    if block.len() <= room {
        return block;
    }
    // A line ending just before the room runs out fits without its LF.
    let prefix = block.get(..=room).expect("room is shorter than the block");
    for (index, byte) in prefix.iter().enumerate().rev() {
        if *byte == b'\n' {
            return prefix.get(..index).expect("prefix before LF");
        }
    }
    b""
}

/// A text's TRAILERS: kept lines, trimmed, in order, joined by LF with no
/// final LF. Empty where its last paragraph holds none or keeps none.
/// Lines past `TRAILER_BYTES`, with their LF, are dropped from the end.
#[must_use]
pub fn trailers(text: &[u8]) -> Box<[u8]> {
    let Some(paragraph) = trailer_paragraph(text) else {
        return Box::new([]);
    };
    let valid = utf8_prefix(paragraph);
    if valid.len() != paragraph.len() {
        return Box::new([]);
    }
    let mut out = List::with_capacity(u32::try_from(TRAILER_BYTES).expect("trailer bound"));
    let mut full = false;
    for line in valid.split('\n') {
        let line = line.trim_end();
        let Some(token) = trailer_token(line) else {
            return Box::new([]);
        };
        if reserved_trailer(token) {
            continue;
        }
        // Even after the bound, a non-trailer line makes the paragraph hold none.
        let needed = line.len().checked_add(usize::from(!out.is_empty())).expect("trailer and LF length");
        if full || needed > usize::try_from(out.room()).expect("trailer room") {
            full = true;
            continue;
        }
        if !out.is_empty() {
            append(&mut out, b"\n");
        }
        append(&mut out, line.as_bytes());
    }
    out.into_boxed()
}

fn trailer_paragraph(text: &[u8]) -> Option<&[u8]> {
    let mut start = 0;
    let mut paragraph_start = 0;
    let mut paragraph_end = 0;
    let mut earlier = false;
    let mut separated = true;
    for (end, byte) in text.iter().chain(b"\n").enumerate() {
        if *byte != b'\n' {
            continue;
        }
        let line = text.get(start..end).expect("line between LF separators");
        let valid = utf8_prefix(line);
        if valid.len() == line.len() && valid.trim_end().is_empty() {
            separated = true;
        } else {
            if separated {
                earlier = paragraph_end != 0;
                paragraph_start = start;
                separated = false;
            }
            paragraph_end = end;
        }
        start = end.checked_add(1).expect("line offset after LF");
    }
    if earlier { text.get(paragraph_start..paragraph_end) } else { None }
}

fn trailer_token(line: &str) -> Option<&str> {
    let (token, value) = line.split_once(": ")?;
    if !token.as_bytes().first()?.is_ascii_alphabetic() || value.is_empty() {
        return None;
    }
    for byte in token.as_bytes() {
        if !byte.is_ascii_alphanumeric() && *byte != b'-' {
            return None;
        }
    }
    for character in value.chars() {
        if character.is_control() {
            return None;
        }
    }
    Some(token)
}

fn reserved_trailer(token: &str) -> bool {
    token.eq_ignore_ascii_case("Change")
        || token.eq_ignore_ascii_case("Pull-request")
        || token.eq_ignore_ascii_case("Head")
        || token.eq_ignore_ascii_case("Goal")
}

/// Only an exact Head line in the last paragraph names this member's head.
#[must_use]
pub fn names_head(message: &[u8], head: Commit) -> bool {
    let mut paragraph = 0;
    for (index, pair) in message.windows(2).enumerate() {
        if pair == b"\n\n" {
            paragraph = index.checked_add(2).expect("message offset");
        }
    }
    let mut expected = List::with_capacity(70);
    append(&mut expected, b"Head: ");
    append(&mut expected, &commit_text(&head));
    let expected = expected.into_boxed();
    let paragraph = message.get(paragraph..).expect("last paragraph");
    let mut start = 0;
    for (index, byte) in paragraph.iter().enumerate() {
        if *byte == b'\n' {
            if paragraph.get(start..index) == Some(expected.as_ref()) {
                return true;
            }
            start = index.checked_add(1).expect("message offset");
        }
    }
    paragraph.get(start..) == Some(expected.as_ref())
}

// The adapter's commit uses a zero tail after byte 20 for SHA-1.
fn commit_text(commit: &Commit) -> Box<[u8]> {
    let mut length = 20;
    for byte in commit.iter().skip(20) {
        if *byte != 0 {
            length = 32;
            break;
        }
    }
    let mut out = List::with_capacity(64);
    for byte in commit.get(..length).expect("a commit has 20 or 32 bytes") {
        append(&mut out, &[bytes::hex_digit(*byte >> 4), bytes::hex_digit(*byte)]);
    }
    out.into_boxed()
}

// Keep valid UTF-8 whole where it fits; otherwise reserve room for the ellipsis.
fn fit(text: &[u8], bound: usize) -> Box<[u8]> {
    if text.len() <= bound {
        return text.into();
    }
    let Some(room) = bound.checked_sub(ELLIPSIS.len()) else { return Box::new([]) };
    let prefix = bytes::utf8_prefix(text, room);
    let mut out = Writer::new(prefix.len().checked_add(ELLIPSIS.len()).expect("bounded text"));
    out.put(prefix).expect("text prefix fits");
    out.put(ELLIPSIS).expect("ellipsis fits");
    out.finish()
}

#[expect(clippy::disallowed_methods, reason = "tooling.md, 4 requires the producer's valid UTF-8 prefix")]
fn utf8_prefix(bytes: &[u8]) -> &str {
    match core::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => core::str::from_utf8(bytes.get(..error.valid_up_to()).expect("UTF-8 prefix"))
            .expect("validated UTF-8 prefix"),
    }
}

fn append(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("text bound");
    }
}

fn put_character(out: &mut List<u8>, character: char) {
    let mut bytes = [0; 4];
    append(out, character.encode_utf8(&mut bytes).as_bytes());
}

#[expect(clippy::manual_find, reason = "the strict subset scans without a predicate closure")]
fn first_content(line: &str) -> Option<char> {
    for character in line.chars() {
        if !character.is_control() && !character.is_whitespace() {
            return Some(character);
        }
    }
    None
}

fn description(words: &[u8], bound: usize) -> Box<[u8]> {
    // Keep at most one character and its separator beyond the byte bound,
    // enough to detect a cut without retaining the rest of the producer's words.
    let capacity = u32::try_from(bound).expect("description bound").checked_add(5).expect("cut detection");
    let mut paragraph = List::with_capacity(capacity);
    let mut length = 0_usize;
    let mut separator = false;
    'lines: for line in utf8_prefix(words).split('\n') {
        match first_content(line) {
            None => {
                if length != 0 {
                    break;
                }
                continue;
            }
            Some('#') => continue,
            Some(_) => {}
        }
        for character in line.chars() {
            if character.is_control() || character.is_whitespace() {
                separator = length != 0;
                continue;
            }
            if separator {
                append(&mut paragraph, b" ");
                length = length.checked_add(1).expect("paragraph bound");
            }
            put_character(&mut paragraph, character);
            length = length.checked_add(character.len_utf8()).expect("paragraph bound");
            separator = false;
            if length > bound {
                break 'lines;
            }
        }
        separator = length != 0;
    }
    fit_description(paragraph.as_slice(), bound)
}

fn refit_description(wrapped: &[u8], bound: usize) -> Box<[u8]> {
    if wrapped.len() <= bound {
        return wrapped.into();
    }
    // Wrapping substitutes LF for one space. Undo that before cutting,
    // then wrap again: a shortened word may fit on the preceding line.
    let mut paragraph = List::with_capacity(u32::try_from(wrapped.len()).expect("description bound"));
    for byte in wrapped {
        paragraph.push(if *byte == b'\n' { b' ' } else { *byte }).expect("unwrapped description fits");
    }
    fit_description(paragraph.as_slice(), bound)
}

fn fit_description(paragraph: &[u8], bound: usize) -> Box<[u8]> {
    let fitted = fit(paragraph, bound);
    if paragraph.len() > bound && fitted.len() <= ELLIPSIS.len() {
        return Box::new([]);
    }
    wrap(utf8_prefix(&fitted))
}

fn wrap(paragraph: &str) -> Box<[u8]> {
    let mut out = List::with_capacity(u32::try_from(paragraph.len()).expect("paragraph bound"));
    let mut column = 0_usize;
    for word in paragraph.split(' ') {
        if word.is_empty() {
            continue;
        }
        let width = word.chars().count();
        if column != 0 {
            if column.checked_add(1).expect("separator width").checked_add(width).expect("paragraph width") > 72 {
                append(&mut out, b"\n");
                column = 0;
            } else {
                append(&mut out, b" ");
                column = column.checked_add(1).expect("paragraph width");
            }
        }
        append(&mut out, word.as_bytes());
        column = column.checked_add(width).expect("paragraph width");
    }
    out.into_boxed()
}
