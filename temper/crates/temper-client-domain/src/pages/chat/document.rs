//! A chat's document and its call's words (temper/docs/design/client/pages/README.md).
use alloc::boxed::Box;
use skein_lib::{List, Wall};

/// The person's fixed first message from Ask temper's opening (client/pages/ask-temper.md, 1).
pub const HOW_TEMPER_WORKS: &[u8] =
    b"Briefly, how does temper work, from my ask to main? And how do these three differ?";

/// The instruction appended to a new draft chat.
pub const WRITE_THIS_UP: &[u8] = b"Write this up as a draft beside us.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Chat,
    Split,
    Draft,
}

/// A chat's document as last read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub note: u64,
    pub revision: u32,
    pub title: Box<[u8]>,
    pub body: Box<[u8]>,
    pub changed: Option<Wall>,
}

/// What a `note` call's answer says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteChange {
    First { lines: u32 },
    Later { added: u32, removed: u32, first: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteWrite {
    pub note: u64,
    pub revision: u32,
    pub change: NoteChange,
}

/// Whether the tool writes the chat's document (client/pages/README.md).
#[must_use]
pub fn writes(tool: &[u8]) -> bool {
    tool == b"note" || tool == b"document"
}

/// Parse a note's whole answer, or a document answer's first line.
#[must_use]
pub fn tool_write(tool: &[u8], outcome: &[u8]) -> Option<NoteWrite> {
    match tool {
        b"note" => note_write(outcome),
        b"document" => {
            let end = skein_lib::bytes::find(outcome, b"\n").unwrap_or(outcome.len());
            note_write(outcome.get(..end).expect("first answer line"))
        }
        _ => None,
    }
}

#[must_use]
pub fn note_write(outcome: &[u8]) -> Option<NoteWrite> {
    let words = outcome.strip_prefix(b"Note ")?;
    let (note, words) = number(words)?;
    let words = words.strip_prefix(b" revision ")?;
    let (revision, words) = count(words)?;
    let words = words.strip_prefix(" written · ".as_bytes())?;
    let change = if let Some(words) = words.strip_prefix(b"+") {
        let (added, words) = count(words)?;
        let words = words.strip_prefix(" −".as_bytes())?;
        let (removed, words) = count(words)?;
        let words = words.strip_prefix(" · line ".as_bytes())?;
        let (first, tail) = count(words)?;
        if !tail.is_empty() {
            let tail = tail.strip_prefix(" · ".as_bytes())?;
            let (_lines, tail) = count(tail)?;
            if tail != b" lines" {
                return None;
            }
        }
        NoteChange::Later { added, removed, first }
    } else {
        let (lines, tail) = count(words)?;
        if tail != b" lines" {
            return None;
        }
        NoteChange::First { lines }
    };
    Some(NoteWrite { note, revision, change })
}

fn number(bytes: &[u8]) -> Option<(u64, &[u8])> {
    let mut number = 0_u64;
    let mut end: usize = 0;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            break;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
        end = end.checked_add(1)?;
    }
    if end == 0 { None } else { Some((number, bytes.get(end..)?)) }
}

fn count(bytes: &[u8]) -> Option<(u32, &[u8])> {
    let (number, tail) = number(bytes)?;
    Some((u32::try_from(number).ok()?, tail))
}

/// The mock's slug of a title, 28 characters, "untitled" when empty.
#[must_use]
pub fn slug(title: &[u8]) -> Box<[u8]> {
    let mut bytes = List::with_capacity(28);
    let mut dash = false;
    for byte in title {
        let byte = byte.to_ascii_lowercase();
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            bytes.push(byte).expect("slug is at most 28 bytes");
            dash = false;
        } else if !dash {
            bytes.push(b'-').expect("slug is at most 28 bytes");
            dash = true;
        }
        if bytes.len() == 28 {
            break;
        }
    }
    let bytes = bytes.as_slice();
    let mut start = 0;
    let mut end = bytes.len();
    if bytes.first() == Some(&b'-') {
        start = 1;
    }
    if bytes.last() == Some(&b'-') {
        end = end.saturating_sub(1);
    }
    let bytes = bytes.get(start..end).unwrap_or_default();
    if bytes.is_empty() { Box::from(*b"untitled") } else { Box::from(bytes) }
}

/// One section: its 1-based first line, heading's bytes, and bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section<'a> {
    pub line: u32,
    pub heading: &'a [u8],
    pub bytes: &'a [u8],
}

/// Split at each `#` and `##` line outside a fenced block.
pub fn sections(body: &[u8]) -> impl Iterator<Item = Section<'_>> {
    let capacity = skein_lib::bytes::count(body, b"\n", u32::MAX).saturating_add(1);
    let mut result = List::with_capacity(capacity);
    let mut start = 0;
    let mut from = 0;
    let mut first = 1;
    let mut number: u32 = 1;
    let mut heading: &[u8] = &[];
    let mut fenced = false;
    for (end, byte) in body.iter().chain(core::iter::once(&b'\n')).enumerate() {
        if *byte != b'\n' || from == body.len() {
            continue;
        }
        let line = body.get(from..end).expect("line range").trim_ascii_end();
        if line.starts_with(b"```") {
            fenced = !fenced;
        } else if !fenced {
            let title = line.strip_prefix(b"# ").or(line.strip_prefix(b"## "));
            if let Some(title) = title {
                if from > start {
                    result
                        .push(Section { line: first, heading, bytes: body.get(start..from).expect("section range") })
                        .expect("sections fit lines");
                }
                start = from;
                first = number;
                heading = title;
            }
        }
        number = number.saturating_add(1);
        from = end.saturating_add(1);
    }
    result
        .push(Section { line: first, heading, bytes: body.get(start..).expect("section tail") })
        .expect("sections fit lines");
    result.into_boxed().into_iter()
}

/// The index holding a line, the last if it is past the end.
#[must_use]
pub fn section_holding(body: &[u8], line: u32) -> usize {
    let mut holding = 0;
    for (index, section) in sections(body).enumerate() {
        if section.line > line {
            break;
        }
        holding = index;
    }
    holding
}
