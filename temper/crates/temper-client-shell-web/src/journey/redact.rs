//! Pure recording redaction (temper/docs/design/client/web/journey-redaction.md).
use super::{
    Framing, Line, Originals, Streams, Undecoded, Unreadable, fields_mut, lines, originals, placeholder, redact_field,
    typed,
};
use skein_ui::Act;

pub struct Redacted {
    pub recording: Box<[u8]>,
    pub originals: Originals,
    pub places: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Unredacted {
    Unreadable(Unreadable),
    Undecoded(Undecoded),
}

/// Take originals out of every field and joined stream, retaining frame sizes.
pub fn redact(recording: &[u8], given: &[Box<[u8]>]) -> Result<Redacted, Unredacted> {
    let (lines, originals, places) = redact_lines(lines(recording).map_err(Unredacted::Unreadable)?, given)?;
    Ok(Redacted { recording: written(lines), originals, places })
}
/// Redact already parsed lines, retaining originals and replacement counts.
pub fn redact_lines(mut lines: Vec<Line>, given: &[Box<[u8]>]) -> Result<(Vec<Line>, Originals, u32), Unredacted> {
    let originals = originals(&lines, given);
    let places = replace(&mut lines, &originals, true).map_err(Unredacted::Undecoded)?;
    Ok((lines, originals, places))
}

/// Replace the browser's link and stored credentials, leaving memory and typed values alone
/// (client/web/journeys.md, 3.1).
#[must_use]
pub fn saved(recording: &[u8]) -> Box<[u8]> {
    written(saved_lines(lines(recording).expect("a recorder's file is always readable")))
}
/// As `saved`, leaving parsed lines ready for further checks or replay.
#[must_use]
pub fn saved_lines(mut lines: Vec<Line>) -> Vec<Line> {
    let originals = super::originals::sign_in(&lines);
    if replace(&mut lines, &originals, false).is_err() {
        for line in &mut lines {
            if let Line::Heard(jig_client_shell_web::adapter::Heard::Piece { bytes }) = line {
                bytes.fill(b'x');
            }
        }
    }
    lines
}

fn replace(lines: &mut [Line], originals: &Originals, include_typed: bool) -> Result<u32, Undecoded> {
    let mut places = 0_u32;
    for line in lines.iter_mut() {
        if include_typed
            && typed(line)
            && let Line::Act { act: Act::Input { value }, .. } = line
        {
            let hidden = placeholder(value);
            if *value != hidden {
                *value = hidden;
                count(&mut places, 1);
            }
        }
        if let Line::Open { path, .. } = line {
            count(&mut places, redact_replaces(path, originals));
        }
        for (field, framing) in fields_mut(line) {
            count(&mut places, redact_field(field, framing, originals));
        }
    }
    let mut streams = Streams::read(lines)?;
    for event in streams.events_mut() {
        count(&mut places, redact_field(&mut event.record, Framing::Framed, originals));
    }
    for plain in streams.plain_mut() {
        count(&mut places, redact_field(&mut plain.bytes, Framing::Framed, originals));
    }
    streams.write(lines);
    Ok(places)
}

// The stream name is binary in Kept and lowercase hex in the opening's query.
fn redact_replaces(path: &mut Box<[u8]>, originals: &Originals) -> u32 {
    let Some(at) = path.windows(b"?replaces=".len()).position(|part| part == b"?replaces=" || part == b"&replaces=")
    else {
        return 0;
    };
    let start = at + b"?replaces=".len();
    let end = path[start..].iter().position(|byte| *byte == b'&').map_or(path.len(), |length| start + length);
    let Ok(hex) = std::str::from_utf8(&path[start..end]) else { return 0 };
    if hex.len() != 32 {
        return 0;
    }
    let bytes: Option<Vec<u8>> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| std::str::from_utf8(pair).ok().and_then(|pair| u8::from_str_radix(pair, 16).ok()))
        .collect();
    let Some(bytes) = bytes else { return 0 };
    let mut bytes = bytes.into_boxed_slice();
    let places = redact_field(&mut bytes, Framing::Framed, originals);
    if places != 0 {
        let mut replaced = path[..start].to_vec();
        for byte in bytes {
            replaced.extend_from_slice(format!("{byte:02x}").as_bytes());
        }
        replaced.extend_from_slice(&path[end..]);
        *path = replaced.into_boxed_slice();
    }
    places
}

fn written(lines: Vec<Line>) -> Box<[u8]> {
    let mut recording = Vec::new();
    for line in lines {
        line.write(&mut recording);
    }
    recording.into()
}

fn count(places: &mut u32, added: u32) {
    *places = places.checked_add(added).expect("replacement count fits u32");
}

#[cfg(test)]
mod tests;
