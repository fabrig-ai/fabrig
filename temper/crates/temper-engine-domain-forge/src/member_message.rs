//! Member commit messages and squash recognition (temper/docs/design/protocol/tooling.md, 4).
use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_lib::{Decimal, List};

/// The frozen member and its producer's delivered title and words.
#[derive(Debug)]
pub struct Member<'a> {
    pub title: &'a [u8],
    pub words: &'a [u8],
    pub change: u64,
    pub pull: u64,
    pub head: Commit,
    pub goal: Option<u64>,
}

/// Compose a squash, or a merge with a goal trailer only on a train into main.
#[must_use]
pub fn commit_message(member: &Member<'_>, with_goal: bool) -> Box<[u8]> {
    let description = description(member.words);
    let title = crate::text::fit(utf8_prefix(member.title).as_bytes(), 256, "…".as_bytes());
    // A pull title, the bounded description and the largest trailers fit the
    // small checkout's 4,096-byte message bound.
    let mut out = List::with_capacity(4096);
    for character in utf8_prefix(&title).chars() {
        put_character(&mut out, if character.is_control() { ' ' } else { character });
    }
    if !description.is_empty() {
        append(&mut out, b"\n\n");
        append(&mut out, &description);
    }
    append(&mut out, b"\n\nChange: ");
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

fn description(words: &[u8]) -> Box<[u8]> {
    // Keep at most one character and its separator beyond the byte bound,
    // enough to detect a cut without retaining the rest of the producer's words.
    let mut paragraph = List::with_capacity(1029);
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
            if length > 1024 {
                break 'lines;
            }
        }
        separator = length != 0;
    }
    let paragraph = crate::text::fit(&paragraph.into_boxed(), 1024, "…".as_bytes());
    wrap(utf8_prefix(&paragraph))
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
