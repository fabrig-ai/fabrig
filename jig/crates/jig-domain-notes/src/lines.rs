//! A note's answer counts a single changed span (jig/docs/design/domain/engine.md, 10).

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
