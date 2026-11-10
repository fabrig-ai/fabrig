//! Report words and cuts (temper/docs/design/domain/forge.md, 11.2).

use alloc::boxed::Box;
use skein_lib::bytes::{concat, find, utf8_prefix};
use skein_lib::{Decimal, List, Writer};

pub const SELECTION_PREFIX: &[u8] = b"selection: ";
pub const SELECTION_LINE_BYTES: usize = 256;
pub const LOG_START_BYTES: u32 = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub goal: u64,
    pub title: Box<[u8]>,
    pub landed: Box<[Landed]>,
    pub unlanded: Box<[Unlanded]>,
    pub landing: Landing,
    pub selections: Box<[Selection]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    pub task: u64,
    pub title: Box<[u8]>,
    pub pull: u64,
    pub branch: Box<[u8]>,
    pub left: Option<Box<[u8]>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unlanded {
    pub task: u64,
    pub title: Box<[u8]>,
    pub why: Why,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    Cancelled,
    Failed(Box<[u8]>),
    NoChange(Box<[u8]>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Landing {
    Landed { pull: u64, base: Box<[u8]>, commit: Box<[u8]>, repairs: Box<[u64]>, resolutions: Box<[u64]> },
    Held,
    NoBranch,
    Ended { pull: Option<u64>, task: u64, why: Why },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub pull: u64,
    pub line: Option<Box<[u8]>>,
}

/// The first line, without its LF or CRLF ending.
#[must_use]
pub fn first_line(words: &[u8]) -> &[u8] {
    let end = find(words, b"\n").unwrap_or(words.len());
    let line = words.get(..end).expect("the first newline is within the input");
    match line.strip_suffix(b"\r") {
        Some(line) => line,
        None => line,
    }
}

/// The first Left section, without the newline before the next heading or end.
#[must_use]
pub fn left_section(words: &[u8]) -> Option<&[u8]> {
    let mut start = None;
    let mut offset = 0_usize;
    for _ in 0..=words.len() {
        let rest = words.get(offset..)?;
        let end = find(rest, b"\n").unwrap_or(rest.len());
        let line = rest.get(..end)?;
        if let Some(begin) = start {
            if line.starts_with(b"#") {
                let section = words.get(begin..offset.saturating_sub(1))?;
                return Some(section);
            }
        } else {
            let mut content = line;
            for byte in line {
                if *byte != b'#' && *byte != b'*' && *byte != b' ' {
                    break;
                }
                content = content.get(1..)?;
            }
            if let Some(after) = content.strip_prefix(b"Left")
                && (after.is_empty() || after.starts_with(b":") || after == b"\r")
            {
                start = Some(offset);
            }
        }
        if end == rest.len() {
            break;
        }
        offset = offset.checked_add(line.len())?.checked_add(1)?;
    }
    let begin = start?;
    let section = words.get(begin..)?;
    Some(match section.strip_suffix(b"\n") {
        Some(section) => section,
        None => section,
    })
}

/// A selection from the log's first line, cut at a UTF-8 boundary.
#[must_use]
pub fn selection_line(log_start: &[u8]) -> Option<&[u8]> {
    let line = first_line(log_start);
    if line.starts_with(SELECTION_PREFIX) { Some(utf8_prefix(line, SELECTION_LINE_BYTES)) } else { None }
}

// The same render counts first, then fills an exactly sized writer.
struct Output {
    length: usize,
    writer: Option<Writer>,
}

impl Output {
    fn put(&mut self, bytes: &[u8]) {
        self.length = self.length.checked_add(bytes.len()).expect("report fragments fit a length");
        if let Some(writer) = &mut self.writer {
            writer.put(bytes).expect("the counting pass sized the report");
        }
    }

    fn number(&mut self, number: u64) {
        self.put(Decimal::of(number).as_bytes());
    }
}

#[derive(Debug)]
struct Left {
    start: usize,
    end: usize,
}

fn reason(output: &mut Output, why: &Why) {
    match why {
        Why::Cancelled => output.put(b"cancelled"),
        Why::Failed(text) => {
            output.put(b"failed: ");
            output.put(text);
        }
        Why::NoChange(text) => {
            output.put(b"no change: ");
            output.put(text);
        }
    }
}

fn tasks(output: &mut Output, heading: &[u8], tasks: &[u64]) {
    if tasks.is_empty() {
        return;
    }
    output.put(heading);
    let mut separator: &[u8] = b"";
    for task in tasks {
        output.put(separator);
        output.put(b"T");
        output.number(*task);
        separator = b", ";
    }
}

fn landing(output: &mut Output, landing: &Landing) {
    output.put(b"The landing: ");
    match landing {
        Landing::Landed { pull, base, commit, repairs, resolutions } => {
            output.put(b"pull request #");
            output.number(*pull);
            output.put(b", merged into ");
            output.put(base);
            output.put(b" as ");
            output.put(commit);
            tasks(output, b"; repairs ", repairs);
            tasks(output, b"; resolutions ", resolutions);
        }
        Landing::Held => output.put(b"none, its base already held the goal's branch"),
        Landing::NoBranch => output.put(b"none, no goal branch"),
        Landing::Ended { pull, task, why } => {
            output.put(b"none, ");
            match pull {
                Some(pull) => {
                    output.put(b"pull request #");
                    output.number(*pull);
                }
                None => {
                    output.put(b"T");
                    output.number(*task);
                }
            }
            output.put(b" ended ");
            reason(output, why);
        }
    }
    output.put(b".\n");
}

fn render(output: &mut Output, report: &Report) -> List<Left> {
    let mut lefts = List::uncapped();
    output.put(b"Goal T");
    output.number(report.goal);
    output.put(b": ");
    output.put(&report.title);
    output.put(b"\n");
    if !report.landed.is_empty() {
        output.put(b"Landed:\n");
    }
    for change in &report.landed {
        output.put(b"- T");
        output.number(change.task);
        output.put(b" ");
        output.put(&change.title);
        output.put(b": pull request #");
        output.number(change.pull);
        output.put(b" into ");
        output.put(&change.branch);
        output.put(b"\n");
        let start = output.length;
        match &change.left {
            Some(section) => {
                let mut rest = section.as_ref();
                for _ in 0..=section.len() {
                    let end = find(rest, b"\n").unwrap_or(rest.len());
                    output.put(b"  ");
                    output.put(rest.get(..end).expect("line ends inside the section"));
                    output.put(b"\n");
                    if end == rest.len() {
                        break;
                    }
                    rest = rest.get(end.saturating_add(1)..).expect("the newline has a following suffix");
                }
                let end = output.length;
                lefts.push(Left { start, end }).expect("the report's Left count fits a list");
            }
            None => output.put(b"  Left: none given\n"),
        }
    }
    if !report.unlanded.is_empty() {
        output.put(b"Ended without landing:\n");
    }
    for change in &report.unlanded {
        output.put(b"- T");
        output.number(change.task);
        output.put(b" ");
        output.put(&change.title);
        output.put(b": ");
        reason(output, &change.why);
        output.put(b"\n");
    }
    landing(output, &report.landing);
    if !report.selections.is_empty() {
        output.put(b"CI selection:\n");
    }
    for selection in &report.selections {
        output.put(b"- pull request #");
        output.number(selection.pull);
        output.put(b": ");
        match &selection.line {
            Some(line) => output.put(line),
            None => output.put(b"unknown"),
        }
        output.put(b"\n");
    }
    lefts
}

fn marker(count: usize, prefix: &[u8]) -> Box<[u8]> {
    let count = u64::try_from(count).expect("a held report length fits a u64");
    concat(&[prefix, Decimal::of(count).as_bytes(), b" bytes cut]\n"])
}

fn fit(bytes: Box<[u8]>, room: usize, cut: usize) -> Box<[u8]> {
    if bytes.len() <= room {
        return bytes;
    }
    let mut kept = utf8_prefix(&bytes, room);
    for _ in 0..=room {
        let dropped = bytes.len().saturating_sub(kept.len()).saturating_add(cut);
        let ending = marker(dropped, b"\n[");
        if kept.len().saturating_add(ending.len()) <= room {
            return concat(&[kept, &ending]);
        }
        if kept.is_empty() {
            break;
        }
        kept = utf8_prefix(kept, kept.len().saturating_sub(1));
    }
    Box::new([])
}

/// Compose in input order, cutting Left sections last first, then the suffix.
/// A bound too small for the cut marker yields an empty box.
#[must_use]
pub fn words(report: &Report, bound: u32) -> Box<[u8]> {
    let room = usize::try_from(bound).expect("a words bound fits an addressable length");
    let mut output = Output { length: 0, writer: None };
    let lefts = render(&mut output, report);
    let mut size = output.length;
    output.length = 0;
    output.writer = Some(Writer::new(size));
    let _lefts = render(&mut output, report);
    let bytes = output.writer.expect("the writing pass has a writer").finish();
    let mut from = bytes.len();
    let mut cut = 0_usize;
    for left in lefts.iter().rev() {
        if size <= room {
            break;
        }
        let count = left.end.saturating_sub(left.start);
        let replacement = marker(count, b"  Left: [");
        size = size.saturating_sub(count).saturating_add(replacement.len());
        cut = cut.saturating_add(count);
        from = left.start;
    }
    if cut == 0 {
        return fit(bytes, room, 0);
    }
    let mut writer = Writer::new(size);
    let mut offset = 0;
    for left in &lefts {
        if left.start < from {
            continue;
        }
        writer.put(bytes.get(offset..left.start).expect("Left spans are ordered")).expect("replacement size counted");
        let count = left.end.saturating_sub(left.start);
        writer.put(&marker(count, b"  Left: [")).expect("replacement size counted");
        offset = left.end;
    }
    writer.put(bytes.get(offset..).expect("Left spans end within the report")).expect("replacement size counted");
    fit(writer.finish(), room, cut)
}
