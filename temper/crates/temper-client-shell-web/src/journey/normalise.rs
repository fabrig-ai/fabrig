//! Recording placeholders (temper/docs/design/client/journey-placeholders.md, 3 and 6).
use super::{Framing, Normalised, Streams, Unredacted, fields_mut, lines};
use temper_client_redaction::Varied;

/// Replace loopback ports in every field and decoded stream, preserving framed widths.
pub fn normalise(recording: &[u8]) -> Result<Normalised, Unredacted> {
    let mut lines = lines(recording).map_err(Unredacted::Unreadable)?;
    let mut streams = Streams::read(&lines).map_err(Unredacted::Undecoded)?;
    let mut varied = Varied::new();
    varied.decimal_ports();
    let mut stream_fields = streams.fields().into_iter().peekable();
    for (index, line) in lines.iter_mut().enumerate() {
        for (field, framing) in fields_mut(line) {
            varied.seen(field, framing);
        }
        let number = u32::try_from(index + 1).expect("line count fits u32");
        while stream_fields.peek().is_some_and(|(start, _)| *start == number) {
            let (_, field) = stream_fields.next().expect("peeked stream field");
            varied.seen(field, Framing::Framed);
        }
    }
    for line in &mut lines {
        for (field, _) in fields_mut(line) {
            varied.replace(field);
        }
    }
    for event in streams.events_mut() {
        varied.replace(&mut event.record);
    }
    for plain in streams.plain_mut() {
        varied.replace(&mut plain.bytes);
    }
    streams.write(&mut lines);
    let mut recording = Vec::new();
    for line in lines {
        line.write(&mut recording);
    }
    Ok(varied.counts(recording.into()))
}

#[cfg(test)]
mod tests;
