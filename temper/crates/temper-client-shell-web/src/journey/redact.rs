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
    let mut places = 0_u32;
    for line in &mut lines {
        if typed(line)
            && let Line::Act { act: Act::Input { value }, .. } = line
        {
            let hidden = placeholder(value);
            if *value != hidden {
                *value = hidden;
                count(&mut places, 1);
            }
        }
        for (field, framing) in fields_mut(line) {
            count(&mut places, redact_field(field, framing, &originals));
        }
    }
    let mut streams = Streams::read(&lines).map_err(Unredacted::Undecoded)?;
    for event in streams.events_mut() {
        count(&mut places, redact_field(&mut event.record, Framing::Framed, &originals));
    }
    for plain in streams.plain_mut() {
        count(&mut places, redact_field(&mut plain.bytes, Framing::Framed, &originals));
    }
    streams.write(&mut lines);
    let mut recording = Vec::new();
    for line in lines {
        line.write(&mut recording);
    }
    Ok(Redacted { recording: recording.into(), originals, places })
}

fn count(places: &mut u32, added: u32) {
    *places = places.checked_add(added).expect("replacement count fits u32");
}

#[cfg(test)]
mod tests;
