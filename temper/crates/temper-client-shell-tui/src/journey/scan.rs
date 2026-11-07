//! The TUI's last scan (temper/docs/design/client/tui/journeys.md, 5).
use super::{Line, Originals, Refusal, What};

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

pub fn codes(_lines: &[Line]) -> Vec<Box<[u8]>> {
    Vec::new()
}

pub fn scan_bytes(_bytes: &[u8], _codes: &[Box<[u8]>], _originals: &Originals) -> Option<What> {
    None
}

pub fn scan(_recording: &[u8], _originals: &Originals) -> Result<Option<Found>, Refusal> {
    Ok(None)
}

#[cfg(test)]
mod tests;
