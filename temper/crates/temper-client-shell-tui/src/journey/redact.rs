//! TUI originals and fields (temper/docs/design/client/tui/journeys.md, 5).
use super::{Framing, Line, Originals, Unreadable};

pub struct Redacted {
    pub recording: Box<[u8]>,
    pub originals: Originals,
    pub places: u32,
}

pub fn fields_mut(_line: &mut Line) -> Vec<(&mut Box<[u8]>, Framing)> {
    Vec::new()
}

pub fn originals(_lines: &[Line], _given: &[Box<[u8]>]) -> Originals {
    Originals::new(Vec::new(), &[])
}

pub fn redact(recording: &[u8], given: &[Box<[u8]>]) -> Result<Redacted, Unreadable> {
    let lines = super::lines(recording)?;
    Ok(Redacted { recording: recording.into(), originals: originals(&lines, given), places: 0 })
}

#[cfg(test)]
pub(super) mod tests;
