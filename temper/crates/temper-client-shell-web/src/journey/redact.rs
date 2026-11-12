//! Pure recording redaction (temper/docs/design/client/web/journeys.md, 3.6).
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
    let mut lines = lines(recording).map_err(Unredacted::Unreadable)?;
    let originals = originals(&lines, given);
    let places = replace(&mut lines, &originals, true).map_err(Unredacted::Undecoded)?;
    Ok(Redacted { recording: written(lines), originals, places })
}

/// Replace the browser's link and stored credentials, leaving memory and typed values alone
/// (client/web/journeys.md, 3.1).
#[must_use]
pub fn saved(recording: &[u8]) -> Box<[u8]> {
    let mut lines = lines(recording).expect("a recorder's file is always readable");
    let originals = super::originals::sign_in(&lines);
    if replace(&mut lines, &originals, false).is_err() {
        for line in &mut lines {
            if let Line::Heard(jig_client_shell_web::adapter::Heard::Piece { bytes }) = line {
                bytes.fill(b'x');
            }
        }
    }
    written(lines)
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
