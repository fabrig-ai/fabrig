//! The TUI's last scan (temper/docs/design/client/tui/journeys.md, 5).
use super::redact::link_forms;
use super::{Line, Originals, Refusal, Replay, What, fields_mut, lines};
use jig_client::ApprovalCode;
use skein_lib::bytes::find_from;
use temper_client_redaction::scan::{original, shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Field,
    Frame { turn: u32 },
    Printed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub line: u32,
    pub record: &'static str,
    pub place: Place,
    pub what: What,
}

/// Approval codes as shown in a browse address's query, checked by their reader.
#[must_use]
pub fn codes(lines: &[Line]) -> Vec<Box<[u8]>> {
    let mut codes: Vec<Box<[u8]>> = Vec::new();
    for line in lines {
        if let Line::Browse(address) = line
            && let Some(query) = address.splitn(2, |byte| *byte == b'?').nth(1)
        {
            let query = query.split(|byte| *byte == b'#').next().expect("a split has a first part");
            for parameter in query.split(|byte| *byte == b'&') {
                if let Some(value) = parameter.strip_prefix(b"code=")
                    && ApprovalCode::parse(value).is_some()
                    && !codes.iter().any(|code| code.as_ref() == value)
                {
                    codes.push(value.into());
                }
            }
        }
    }
    codes
}

/// Originals in the whole; shapes in a copy with complete approval codes cut.
#[must_use]
pub fn scan_bytes(bytes: &[u8], codes: &[Box<[u8]>], originals: &Originals) -> Option<What> {
    if original(bytes, originals) || link_forms(originals).iter().any(|form| !form.occurrences(bytes).is_empty()) {
        return Some(What::Original);
    }
    let mut cuts = Vec::new();
    for code in codes {
        if code.is_empty() {
            continue;
        }
        let mut from = 0;
        while let Some(at) = find_from(bytes, code, from) {
            let end = at.checked_add(code.len()).expect("found code fits field");
            // A known code must not hide a different value with that prefix,
            // or a substring of another token.
            if (at == 0 || !token(bytes[at - 1])) && bytes.get(end).is_none_or(|byte| !token(*byte)) {
                cuts.push((at, end));
            }
            from = end;
        }
    }
    if cuts.is_empty() {
        return shape(bytes).map(What::Shape);
    }
    cuts.sort_unstable();
    let mut copy = Vec::with_capacity(bytes.len());
    let mut from = 0;
    for (at, end) in cuts {
        if at >= from {
            copy.extend_from_slice(&bytes[from..at]);
            from = end;
        }
    }
    copy.extend_from_slice(&bytes[from..]);
    shape(&copy).map(What::Shape)
}

fn token(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"._~%+/-".contains(&byte)
}

/// Scan fields first, then the replay's frame at every turn's end.
pub fn scan(recording: &[u8], originals: &Originals) -> Result<Option<Found>, Refusal> {
    let lines = lines(recording).map_err(Refusal::Unreadable)?;
    let codes = codes(&lines);
    for (index, line) in lines.iter().enumerate() {
        for (field, _) in fields_mut(&mut line.clone()) {
            if let Some(what) = scan_bytes(field, &codes, originals) {
                return Ok(Some(Found { line: number(index), record: line.kind(), place: Place::Field, what }));
            }
        }
    }
    // Validate the source envelope before temporary marks can give an empty
    // turn content. Only the scan's private replay copy gains marks.
    Replay::new(recording).map_err(Refusal::Unreadable)?;
    let mut copy = Vec::new();
    let mut source = Vec::new();
    let mut marks = Vec::new();
    let mut turn = 0_u32;
    for (index, line) in lines.iter().enumerate() {
        if let Line::Turn { .. } = line {
            turn = turn.checked_add(1).expect("turn count fits u32");
        }
        if let Line::Deadline(_) = line {
            Line::Mark(None).write(&mut copy);
            source.push(number(index));
            marks.push(Some((number(index), line.kind(), turn)));
        }
        if let Line::Mark(_) | Line::End(_) = line {
            marks.push(None);
        }
        line.write(&mut copy);
        source.push(number(index));
    }
    let mut replay = Replay::new(&copy).map_err(|mut error| {
        error.line = source_line(&source, error.line);
        Refusal::Unreadable(error)
    })?;
    for mark in marks {
        let frame = replay
            .next()
            .map_err(|mut stale| {
                stale.line = source_line(&source, stale.line);
                Refusal::Stale(stale)
            })?
            .expect("each private mark yields a frame")
            .frame;
        if let Some((line, record, turn)) = mark
            && let Some(what) = scan_bytes(&frame.text(), &codes, originals)
        {
            return Ok(Some(Found { line, record, place: Place::Frame { turn }, what }));
        }
    }
    Ok(None)
}

fn number(index: usize) -> u32 {
    u32::try_from(index.checked_add(1).expect("line count fits usize")).expect("line count fits u32")
}

fn source_line(source: &[u32], line: u32) -> u32 {
    let index = usize::try_from(line).expect("line fits usize").checked_sub(1).expect("lines start at one");
    source.get(index).copied().unwrap_or(line)
}

#[cfg(test)]
mod tests;
