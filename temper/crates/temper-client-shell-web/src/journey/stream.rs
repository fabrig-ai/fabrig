//! Joined recording streams (client/journeys.md, 3.2 and 3.6).
use super::Line;
use jig_client_shell_web::adapter::Heard;
use skein_lib::base64::{self, Alphabet};
use std::ops::Range;

/// One `data:` line's decoded record; `line` is its first piece's line, from 1.
pub struct Event {
    pub line: u32,
    pub record: Box<[u8]>,
}
/// Stream text outside data payloads and bare framing (client/journeys.md, 3.6).
pub struct Plain {
    pub line: u32,
    pub bytes: Box<[u8]>,
}
/// A data line whose payload is not canonical standard base64.
#[derive(Debug, PartialEq, Eq)]
pub struct Undecoded {
    pub line: u32,
}

/// A recording's joined streams and their original piece boundaries.
pub struct Streams {
    joined: Vec<Joined>,
    events: Vec<Event>,
    plain: Vec<Plain>,
}
#[derive(Default)]
struct Joined {
    bytes: Vec<u8>,
    pieces: Vec<Piece>,
    runs: Vec<Run>,
}
struct Piece {
    line: usize,
    range: Range<usize>,
}
struct Run {
    range: Range<usize>,
    source: Source,
}
enum Source {
    Event { index: usize, length: usize },
    Plain { index: usize },
}
impl Streams {
    /// Join pieces between each `open` or `heard opened` and decode their data lines.
    pub fn read(lines: &[Line]) -> Result<Self, Undecoded> {
        let mut joined = Vec::new();
        let mut current = Joined::default();
        for (line, record) in lines.iter().enumerate() {
            if matches!(record, Line::Open { .. } | Line::Heard(Heard::StreamOpened)) {
                joined.push(core::mem::take(&mut current));
            }
            if let Line::Heard(Heard::Piece { bytes }) = record {
                let start = current.bytes.len();
                current.bytes.extend_from_slice(bytes);
                current.pieces.push(Piece { line, range: start..current.bytes.len() });
            }
        }
        joined.push(current);
        let mut streams = Self { joined, events: Vec::new(), plain: Vec::new() };
        for stream in &mut streams.joined {
            stream.decode(&mut streams.events, &mut streams.plain)?;
        }
        Ok(streams)
    }
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    pub fn events_mut(&mut self) -> &mut [Event] {
        &mut self.events
    }
    #[must_use]
    pub fn plain(&self) -> &[Plain] {
        &self.plain
    }
    pub fn plain_mut(&mut self) -> &mut [Plain] {
        &mut self.plain
    }
    /// Encode records back and cut bytes at the original piece boundaries.
    ///
    /// # Panics
    /// A record or plain run whose length changed; the message names lengths alone.
    pub fn write(&self, lines: &mut [Line]) {
        for stream in &self.joined {
            let mut bytes = stream.bytes.clone();
            for run in &stream.runs {
                let replacement = match run.source {
                    Source::Event { index, length } => {
                        let record = &self.events[index].record;
                        assert!(record.len() == length, "{} {}", record.len(), length);
                        encode(record)
                    }
                    Source::Plain { index } => self.plain[index].bytes.clone(),
                };
                assert!(replacement.len() == run.range.len(), "{} {}", replacement.len(), run.range.len());
                bytes[run.range.clone()].copy_from_slice(&replacement);
            }
            for piece in &stream.pieces {
                let Line::Heard(Heard::Piece { bytes: target }) = &mut lines[piece.line] else {
                    panic!("expected original piece");
                };
                *target = bytes[piece.range.clone()].into();
            }
        }
    }
}
impl Joined {
    fn line_at(&self, at: usize) -> u32 {
        let index = self.pieces.partition_point(|piece| piece.range.end <= at);
        let line = self.pieces[index].line.checked_add(1).expect("recording line count fits");
        u32::try_from(line).expect("recording line count fits u32")
    }
    fn decode(&mut self, events: &mut Vec<Event>, plain: &mut Vec<Plain>) -> Result<(), Undecoded> {
        let mut start = 0;
        // Offsets in the joined stream are also the offsets used when writing it back.
        for bytes in self.bytes.split_inclusive(|byte| *byte == b'\n') {
            let end = start + bytes.len();
            let full = bytes.strip_suffix(b"\n");
            let text = full.unwrap_or(bytes);
            if let Some(payload) = text.strip_prefix(b"data: ") {
                let payload_start = start + b"data: ".len();
                let length = if full.is_some() { payload.len() } else { payload.len() / 4 * 4 };
                let line = self.line_at(start);
                let record =
                    base64::decode(Alphabet::Standard, &payload[..length], u32::MAX).map_err(|_| Undecoded { line })?;
                let source = Source::Event { index: events.len(), length: record.len() };
                events.push(Event { line, record });
                self.runs.push(Run { range: payload_start..payload_start + length, source });
                if length < payload.len() {
                    let tail = payload_start + length;
                    let source = Source::Plain { index: plain.len() };
                    plain.push(Plain { line: self.line_at(tail), bytes: payload[length..].into() });
                    self.runs.push(Run { range: tail..end, source });
                }
            } else if !text.is_empty() {
                let source = Source::Plain { index: plain.len() };
                plain.push(Plain { line: self.line_at(start), bytes: bytes.into() });
                self.runs.push(Run { range: start..end, source });
            }
            start = end;
        }
        Ok(())
    }
}
fn encode(record: &[u8]) -> Box<[u8]> {
    let length = u32::try_from(record.len()).expect("decoded record fits u32");
    let encoded = base64::encoded_len(Alphabet::Standard, length).expect("encoding fits u32");
    let mut writer = skein_lib::Writer::new(usize::try_from(encoded).expect("encoding fits usize"));
    base64::encode(Alphabet::Standard, record, &mut writer).expect("encoding has its measured length");
    writer.finish()
}

#[cfg(test)]
mod tests;
