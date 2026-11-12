//! A note's answer counts a single changed span (jig/docs/design/domain/engine/briefs.md, 2).

use crate::Refusal;
use alloc::boxed::Box;
use skein_lib::Writer;

/// The lines a range write replaces (jig/docs/design/domain/engine/briefs.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LineRange {
    /// The 1-based first line replaced; with `remove` 0, any line past the last appends.
    pub first: u32,
    /// Lines replaced from `first`; 0 to insert.
    pub remove: u32,
    /// Continue the preceding line without a newline before the replacement.
    pub join: bool,
}

/// Replace the range's lines, checking the resulting length before allocating.
pub fn splice(old: &[u8], range: LineRange, text: &[u8], bound: u32) -> Result<Box<[u8]>, Refusal> {
    let before_lines = u64::from(range.first).saturating_sub(1);
    let after_lines = before_lines.saturating_add(u64::from(range.remove));
    let mut lines = 0_u32;
    let mut start = 0;
    let mut end = 0;
    for (at, byte) in old.iter().enumerate() {
        let next = at.checked_add(1).expect("byte lies within body");
        if *byte == b'\n' || next == old.len() {
            lines = lines.checked_add(1).expect("body line count fits u32");
            if u64::from(lines) == before_lines {
                start = next;
            }
            if u64::from(lines) == after_lines {
                end = next;
            }
        }
    }
    if range.first == 0 || (range.remove > 0 && after_lines > u64::from(lines)) {
        return Err(Refusal::PastEnd { lines });
    }
    if range.remove == 0 && before_lines > u64::from(lines) {
        start = old.len();
        end = old.len();
    }
    let (before, _) = old.split_at_checked(start).expect("range starts within body");
    let (_, after) = old.split_at_checked(end).expect("range ends within body");
    let before = if range.join { before.strip_suffix(b"\n").unwrap_or(before) } else { before };
    let join_before =
        !range.join && !before.is_empty() && !before.ends_with(b"\n") && (!text.is_empty() || !after.is_empty());
    let join_after = !text.is_empty() && !text.ends_with(b"\n") && !after.is_empty();
    let size = before
        .len()
        .saturating_add(text.len())
        .saturating_add(after.len())
        .saturating_add(usize::from(join_before))
        .saturating_add(usize::from(join_after));
    let bytes = u32::try_from(size).unwrap_or(u32::MAX);
    if size > usize::try_from(bound).expect("body bound fits usize") {
        return Err(Refusal::PastBound { bytes, bound });
    }
    let mut writer = Writer::new(size);
    writer.put(before).expect("measured prefix");
    if join_before {
        writer.put(b"\n").expect("measured separator");
    }
    writer.put(text).expect("measured replacement");
    if join_after {
        writer.put(b"\n").expect("measured separator");
    }
    writer.put(after).expect("measured suffix");
    Ok(writer.finish())
}

/// The new body's lines and the span left after common end lines are trimmed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineChange {
    /// The new body's lines.
    pub lines: u32,
    /// New lines remaining after the common prefix and suffix.
    pub added: u32,
    /// Old lines remaining after the common prefix and suffix.
    pub removed: u32,
    /// The first differing new line, clamped to its last, or 1 for no lines.
    pub first: u32,
}

/// Compare logical lines without allocating or finding a shortest edit.
#[must_use]
pub fn line_change(old: Option<&[u8]>, new: &[u8]) -> LineChange {
    let Some(mut old) = old else {
        let lines = count(new);
        return LineChange { lines, added: lines, removed: 0, first: 1 };
    };
    let mut new = new;
    let mut prefix = 0_u32;
    for _ in 0..old.len().min(new.len()) {
        if old.is_empty() || new.is_empty() {
            break;
        }
        let (old_line, old_rest) = front(old);
        let (new_line, new_rest) = front(new);
        if old_line != new_line {
            break;
        }
        prefix = prefix.checked_add(1).expect("body line count fits u32");
        old = old_rest;
        new = new_rest;
    }
    let mut suffix = 0_u32;
    for _ in 0..old.len().min(new.len()) {
        if old.is_empty() || new.is_empty() {
            break;
        }
        let (old_line, old_rest) = back(old);
        let (new_line, new_rest) = back(new);
        if old_line != new_line {
            break;
        }
        suffix = suffix.checked_add(1).expect("body line count fits u32");
        old = old_rest;
        new = new_rest;
    }
    let added = count(new);
    let removed = count(old);
    let lines = prefix
        .checked_add(added)
        .expect("body line count fits u32")
        .checked_add(suffix)
        .expect("body line count fits u32");
    let first = prefix.checked_add(1).expect("body line count fits u32").min(lines.max(1));
    LineChange { lines, added, removed, first }
}

fn front(body: &[u8]) -> (&[u8], &[u8]) {
    match skein_lib::bytes::find(body, b"\n") {
        Some(at) => {
            let (line, rest) = body.split_at_checked(at).expect("line delimiter found within body");
            (line, rest.strip_prefix(b"\n").expect("line delimiter found"))
        }
        None => (body, &[]),
    }
}

fn back(body: &[u8]) -> (&[u8], &[u8]) {
    let line = body.strip_suffix(b"\n").unwrap_or(body);
    for (at, byte) in line.iter().enumerate().rev() {
        if *byte == b'\n' {
            let end = at.checked_add(1).expect("delimiter lies within body");
            let (rest, line) = line.split_at_checked(end).expect("line delimiter ends within body");
            return (line, rest);
        }
    }
    (line, &[])
}

fn count(body: &[u8]) -> u32 {
    let mut lines = 0_u32;
    for byte in body {
        if *byte == b'\n' {
            lines = lines.checked_add(1).expect("body line count fits u32");
        }
    }
    if !body.is_empty() && !body.ends_with(b"\n") {
        lines = lines.checked_add(1).expect("body line count fits u32");
    }
    lines
}
