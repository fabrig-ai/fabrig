//! The format and its moves, shared by the reader, renderer and pages
//! (temper/docs/design/protocol/configuration/file-formats.md, 2–5).
use crate::{FILE, Refusal, Shape, binary, source::diagnostic};
use toml::{Table, Value, de::DeTable};

pub(crate) const KEY: &str = "format";
pub(crate) const FIRST: i64 = 1;

#[derive(Clone, Copy)]
pub(crate) struct Formats {
    pub(crate) newest: i64,
    /// In order of format, from oldest to newest.
    pub(crate) moves: &'static [Move],
}
pub(crate) const PRODUCTION: Formats = Formats { newest: FIRST, moves: &[] };

#[derive(Debug)]
pub struct Move {
    pub format: i64,
    pub key: &'static str,
    pub kind: MoveKind,
}

#[derive(Debug)]
pub enum MoveKind {
    Renamed { new: &'static str },
    Converted { new: &'static str, convert: fn(Value) -> Result<Value, &'static str>, words: &'static str },
    MovedOut { home: &'static str, words: &'static str },
    Gone { release: &'static str },
}

/// A written value for a home's seeding. The reader itself never seeds.
#[derive(Debug)]
pub struct MovedOut {
    pub key: &'static str,
    pub line: usize,
    pub value: Value,
    pub movement: &'static Move,
}

impl Formats {
    pub(crate) fn read(self, text: &str, table: &DeTable<'_>, decoded: &Table, shape: Shape) -> Result<i64, Refusal> {
        let Some((key, value)) = table.iter().find(|(key, _)| key.get_ref().as_ref() == KEY) else { return Ok(FIRST) };
        let raw = &text[value.span()];
        let Some(format) = decoded
            .get(KEY)
            .and_then(Value::as_integer)
            .filter(|&n| n >= FIRST && !["0x", "0o", "0b"].iter().any(|prefix| raw.starts_with(prefix)))
        else {
            let line = crate::source::at(text, key.span().start);
            return Err(Refusal::one(diagnostic(KEY, Some(line), "a whole number from 1", false)));
        };
        if format > self.newest {
            let newest = self.newest;
            let reads =
                if newest == FIRST { format!("format {FIRST}") } else { format!("formats {FIRST} to {newest}") };
            return Err(Refusal::one(format!(
                "{FILE}: format {format}; this {} reads {reads}: upgrade it, or write the file at format {newest}",
                binary(shape)
            )));
        }
        Ok(format)
    }
}

impl Move {
    pub(crate) fn notice(&self, written: i64, newest: i64) -> String {
        match self.kind {
            MoveKind::Renamed { new } => format!("format {written}; format {newest} writes it as {new}"),
            MoveKind::Converted { words, .. } => format!("format {written}; format {newest} writes it as {words}"),
            MoveKind::MovedOut { words, .. } => format!("moved: {words}"),
            MoveKind::Gone { release } => format!("ignored; {release} took it away"),
        }
    }

    pub(crate) fn description(&self) -> String {
        match self.kind {
            MoveKind::Renamed { new } => format!("renamed {new}"),
            MoveKind::Converted { new, words, .. } => format!("converted to {new}: {words}"),
            MoveKind::MovedOut { words, .. } => format!("moved out: {words}"),
            MoveKind::Gone { release } => format!("gone in {release}"),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
