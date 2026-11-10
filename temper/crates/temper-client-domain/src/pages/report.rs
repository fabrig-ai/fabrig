//! A goal's report, read as bytes (temper/docs/design/client/pages/README.md).
use alloc::boxed::Box;
use jig_client::contract::{self as c, NoteScope, Read, Text};
use skein_lib::{List, Wall, bytes};

/// A report note's description opens so (temper/docs/design/domain/agent/agents.md, 3).
pub const OPENING: &[u8] = b"report: ";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The description past its opening, its first line.
    pub headline: Box<[u8]>,
    /// In order; a paragraph's lines are joined by one space.
    pub paragraphs: List<Box<[u8]>>,
    pub left: Option<Box<[u8]>>,
    pub risk: Option<Box<[u8]>>,
    /// `Decided:`'s items, split at " · ", each kept whole with its time.
    pub decided: List<Box<[u8]>>,
    /// The note's `changed`.
    pub at: Option<Wall>,
}

/// Total: any note reads, a body cut as far as it goes.
#[must_use]
pub fn parse(note: &c::Note) -> Report {
    let description = note.description.bytes();
    let headline = description.strip_prefix(OPENING).unwrap_or(description);
    let end = bytes::find(headline, b"\n").unwrap_or(headline.len());
    let headline = headline.get(..end).expect("the line ends within the description");
    let body = note.body.bytes();
    let bound = u32::try_from(body.len()).expect("document text fits its u32 bound");
    let mut report = Report {
        headline: bytes::copy_of(headline),
        paragraphs: List::with_capacity(bound),
        left: None,
        risk: None,
        decided: List::with_capacity(0),
        at: note.changed,
    };
    let mut paragraph = List::with_capacity(bound);
    let mut start = 0;
    for (end, byte) in body.iter().enumerate() {
        if *byte == b'\n' {
            let line = body.get(start..end).expect("line boundaries stay within the body");
            read_line(&mut report, &mut paragraph, line);
            start = end.checked_add(1).expect("the newline has a byte within the body");
        }
    }
    let line = body.get(start..).expect("the last line starts within the body");
    read_line(&mut report, &mut paragraph, line);
    finish_paragraph(&mut report, &mut paragraph);
    report
}

fn read_line(report: &mut Report, paragraph: &mut List<u8>, line: &[u8]) {
    let line = line.trim_ascii_start();
    if let Some(value) = line.strip_prefix(b"Left: ") {
        finish_paragraph(report, paragraph);
        report.left = Some(bytes::copy_of(value.trim_ascii()));
    } else if let Some(value) = line.strip_prefix(b"Risk: ") {
        finish_paragraph(report, paragraph);
        report.risk = Some(bytes::copy_of(value.trim_ascii()));
    } else if let Some(value) = line.strip_prefix(b"Decided: ") {
        finish_paragraph(report, paragraph);
        report.decided = decisions(value.trim_ascii());
    } else {
        let line = line.trim_ascii_end();
        if line.is_empty() {
            finish_paragraph(report, paragraph);
        } else {
            if !paragraph.is_empty() {
                paragraph.push(b' ').expect("joined lines fit within the body's byte count");
            }
            for byte in line {
                paragraph.push(*byte).expect("paragraph bytes fit within the body's byte count");
            }
        }
    }
}

fn finish_paragraph(report: &mut Report, paragraph: &mut List<u8>) {
    if !paragraph.is_empty() {
        let empty = List::with_capacity(paragraph.capacity());
        let words = core::mem::replace(paragraph, empty).into_boxed();
        report.paragraphs.push(words).expect("each paragraph uses at least one body byte");
    }
}

fn decisions(value: &[u8]) -> List<Box<[u8]>> {
    const SEPARATOR: &[u8] = b" \xc2\xb7 ";
    let bound = u32::try_from(value.len()).expect("a field fits the document's u32 bound");
    let mut items = List::with_capacity(bound);
    if value.is_empty() {
        return items;
    }
    let mut start = 0;
    for (at, byte) in value.iter().enumerate() {
        if at >= start
            && *byte == b' '
            && value.get(at..).expect("a byte starts within the field").starts_with(SEPARATOR)
        {
            let item = value.get(start..at).expect("item boundaries stay within the field");
            items.push(bytes::copy_of(item.trim_ascii())).expect("items fit within the field's byte count");
            start = at.checked_add(SEPARATOR.len()).expect("the separator fits within the field");
        }
    }
    let item = value.get(start..).expect("the last item starts within the field");
    items.push(bytes::copy_of(item.trim_ascii())).expect("items fit within the field's byte count");
    items
}

/// Notes scoped to the goal and filtered by the report opening.
#[must_use]
pub fn read(project: u32, task: u64) -> Read {
    let description = Text::new(bytes::copy_of(OPENING), 256).expect("the report opening fits the description");
    Read::Notes { scope: NoteScope::Task { project, task }, description: Some(description), after: None }
}

/// The newest report by change time, then note number; absence is normal.
#[must_use]
pub fn newest(rows: &List<c::Note>) -> Option<&c::Note> {
    let mut newest: Option<&c::Note> = None;
    for row in rows {
        if row.description.bytes().starts_with(OPENING) {
            let replace = match newest {
                Some(previous) => (row.changed, row.number) > (previous.changed, previous.number),
                None => true,
            };
            if replace {
                newest = Some(row);
            }
        }
    }
    newest
}
