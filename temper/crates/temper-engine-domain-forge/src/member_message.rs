//! Member commit messages and squash recognition (temper/docs/design/protocol/tooling.md, 4).
use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_lib::{Decimal, List};

const MESSAGE_BYTES: usize = 4096;
/// The most bytes `trailers` keeps: none past them fits a message.
pub const TRAILER_BYTES: usize = MESSAGE_BYTES;
const ELLIPSIS: &[u8] = "…".as_bytes();

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
    let title = crate::text::fit(utf8_prefix(member.title).as_bytes(), 256, ELLIPSIS);
    let mut out = List::with_capacity(u32::try_from(MESSAGE_BYTES).expect("small checkout message bound"));
    for character in utf8_prefix(&title).chars() {
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
        let description = description(member.words, room.saturating_sub(own_bytes).saturating_sub(2).min(1024));
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
    append(&mut out, &required);
    out.into_boxed()
}

fn required_trailers(member: &Member<'_>, with_goal: bool) -> Box<[u8]> {
    let mut out = List::with_capacity(256);
    append(&mut out, b"Change: ");
    append(&mut out, Decimal::of(member.change).as_bytes());
    append(&mut out, b"\nPull-request: ");
    append(&mut out, Decimal::of(member.pull).as_bytes());
    append(&mut out, b"\nHead: ");
    append(&mut out, &crate::commit::commit_text(&member.head));
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
        if token.eq_ignore_ascii_case("Change")
            || token.eq_ignore_ascii_case("Pull-request")
            || token.eq_ignore_ascii_case("Head")
            || token.eq_ignore_ascii_case("Goal")
        {
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
    append(&mut expected, &crate::commit::commit_text(&head));
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
    let fitted = crate::text::fit(paragraph.as_slice(), bound, ELLIPSIS);
    if length > bound && fitted.len() <= ELLIPSIS.len() {
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
