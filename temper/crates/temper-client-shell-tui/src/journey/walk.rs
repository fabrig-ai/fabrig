//! Native walk checks (temper/docs/design/client/tui/journeys.md, 8.1).
use super::{Line, Refusal, Replay, Unreadable, lines};
use skein_ui_screen_terminal::Key;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WalkStep {
    See(Box<[u8]>),
    Key(Key),
    Mark,
    Approve,
    End,
}

/// Read a version-one walk, including its final end step.
pub fn walk(file: &[u8]) -> Result<Vec<WalkStep>, Unreadable> {
    let mut steps = Vec::new();
    let mut number = 0_u32;
    let mut ended = false;
    for part in file.split_inclusive(|byte| *byte == b'\n') {
        number = number.checked_add(1).ok_or(Unreadable { line: u32::MAX, why: "too many lines" })?;
        let bad = |why| Unreadable { line: number, why };
        let line = part.strip_suffix(b"\n").ok_or_else(|| bad("missing final newline"))?;
        if number == 1 {
            if line != b"walk 1" {
                return Err(bad("expected walk 1"));
            }
            continue;
        }
        if ended {
            return Err(bad("record after end"));
        }
        let step = read(line).map_err(bad)?;
        ended = step == WalkStep::End;
        steps.push(step);
    }
    if !ended {
        return Err(Unreadable { line: number.saturating_add(1), why: "missing end" });
    }
    Ok(steps)
}

fn read(line: &[u8]) -> Result<WalkStep, &'static str> {
    match line {
        b"mark" => return Ok(WalkStep::Mark),
        b"approve" => return Ok(WalkStep::Approve),
        b"end" => return Ok(WalkStep::End),
        _ => {}
    }
    // Decode the recording's fields with its codec: see has browse's bytes,
    // and a walk's key has no event field.
    let mut record = if let Some(rest) = line.strip_prefix(b"see ") {
        let mut record = b"browse ".to_vec();
        record.extend_from_slice(rest);
        record
    } else if line.starts_with(b"key ") {
        let mut record = line.to_vec();
        record.extend_from_slice(b" none");
        record
    } else {
        return Err("unknown walk record or fields");
    };
    record.push(b'\n');
    let mut decoded = lines(&record).map_err(|error| error.why)?;
    let line = decoded.pop().expect("one record given to the field codec");
    if let Line::Browse(text) = line {
        return Ok(WalkStep::See(text));
    }
    if let Line::Key { key, .. } = line {
        return Ok(WalkStep::Key(key));
    }
    unreachable!("only browse and key are given to the field codec")
}

/// The first step, from one, whose keys or approval place differ.
#[must_use]
pub fn follows(walk: &[WalkStep], recording: &[Line]) -> Option<u32> {
    let mut remaining = recording;
    let mut browsed = false;
    for (index, step) in walk.iter().enumerate() {
        let number = u32::try_from(index + 1).expect("walk step count fits u32");
        match step {
            WalkStep::See(_) | WalkStep::Mark => {}
            WalkStep::Key(wanted) => {
                before_key(&mut remaining);
                let Some((Line::Key { key, .. }, rest)) = remaining.split_first() else {
                    return Some(number);
                };
                if key != wanted {
                    return Some(number);
                }
                remaining = rest;
                browsed = false;
            }
            WalkStep::Approve => {
                browsed |= before_key(&mut remaining);
                if !browsed {
                    return Some(number);
                }
            }
            WalkStep::End => {
                before_key(&mut remaining);
                if !remaining.is_empty() {
                    return Some(number);
                }
            }
        }
    }
    None
}

/// Advance only within the interval before the next key.
fn before_key(remaining: &mut &[Line]) -> bool {
    let mut browsed = false;
    while let Some((line, rest)) = remaining.split_first() {
        if let Line::Key { .. } = line {
            break;
        }
        if let Line::Browse(_) = line {
            browsed = true;
        }
        *remaining = rest;
    }
    browsed
}

/// Replay the entire save before searching the final frame's text.
pub fn shows(save: &[u8], text: &[u8]) -> Result<bool, Refusal> {
    let mut replay = Replay::new(save).map_err(Refusal::Unreadable)?;
    while replay.next().map_err(Refusal::Stale)?.is_some() {}
    let frame = replay.host().frame().text();
    Ok(skein_lib::bytes::find(&frame, text).is_some())
}

/// Read the last browse address, including saves without one.
pub fn browsed(save: &[u8]) -> Result<Option<Box<[u8]>>, Unreadable> {
    Ok(lines(save)?.into_iter().rev().find_map(
        |line| {
            if let Line::Browse(address) = line { Some(address) } else { None }
        },
    ))
}

#[cfg(test)]
mod tests;
