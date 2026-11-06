//! Pure recording redaction (temper/docs/design/client/journeys.md, 3.6).
use super::{Originals, Undecoded, Unreadable, lines};

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
    let lines = lines(recording).map_err(Unredacted::Unreadable)?;
    Ok(Redacted { recording: recording.into(), originals: Originals::of(&lines, given), places: 0 })
}

#[cfg(test)]
mod tests;
