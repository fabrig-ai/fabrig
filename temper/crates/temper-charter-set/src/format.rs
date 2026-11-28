//! Front matter of an overlay's charter files (temper/docs/design/client/ctl/overlays.md, 2).
use crate::{File, Refusal, Set};
use alloc::boxed::Box;

impl Set<'_> {
    /// Read the format of every file, including files the selected charter does not take.
    /// # Errors
    /// Refuses malformed front matter and formats other than 1.
    pub fn overlay_bodies(&self) -> Result<Self, Refusal> {
        Ok(Self {
            common: body(File::Common, self.common)?,
            goal: body(File::Goal, self.goal)?,
            design: body(File::Design, self.design)?,
            designer: body(File::Designer, self.designer)?,
            coordinator: body(File::Coordinator, self.coordinator)?,
            implementer: body(File::Implementer, self.implementer)?,
        })
    }
}

impl Refusal {
    /// Locate an assembly refusal in the source set, before front matter was stripped.
    #[must_use]
    pub fn in_source(mut self, source: &Set<'_>) -> Self {
        let (file, line) = match &mut self {
            Self::BeforeSection { file, line }
            | Self::EmptyReason { file, line }
            | Self::RepeatedReason { file, line, .. }
            | Self::UnknownReason { file, line, .. } => (*file, line),
            Self::UnknownFile { .. } | Self::FrontMatter { .. } | Self::BadFormat { .. } | Self::NewerFormat { .. } => {
                return self;
            }
        };
        let text = match file {
            File::Common => source.common,
            File::Goal => source.goal,
            File::Design => source.design,
            File::Designer => source.designer,
            File::Coordinator => source.coordinator,
            File::Implementer => source.implementer,
        };
        if let Some(text) = text
            && opening(text).is_some()
        {
            *line = line.checked_add(3).expect("source line includes the three front matter lines");
        }
        self
    }
}

fn opening(text: &str) -> Option<&str> {
    if text == "+++" {
        return Some("");
    }
    if let Some(rest) = text.strip_prefix("+++\n") {
        return Some(rest);
    }
    text.strip_prefix("+++\r\n")
}

fn body(file: File, text: Option<&str>) -> Result<Option<&str>, Refusal> {
    let Some(text) = text else { return Ok(None) };
    match read_file(text) {
        Ok(text) => Ok(Some(text)),
        Err(FormatFault::FrontMatter { .. }) => Err(Refusal::FrontMatter { file }),
        Err(FormatFault::BadFormat) => Err(Refusal::BadFormat { file }),
        Err(FormatFault::NewerFormat { number }) => Err(Refusal::NewerFormat { file, number }),
    }
}

/// A refusal of one file's front matter, independent of the file's name.
#[derive(Debug, PartialEq, Eq)]
pub enum FormatFault {
    FrontMatter { line: u32 },
    BadFormat,
    NewerFormat { number: Box<str> },
}

/// Read a charter file's format and return its text without front matter.
/// # Errors
/// Refuses malformed front matter and formats other than 1.
pub fn read_file(text: &str) -> Result<&str, FormatFault> {
    let Some(rest) = opening(text) else { return Ok(text) };
    let malformed = FormatFault::FrontMatter { line: 2 };
    let (format, rest) = rest.split_once('\n').unwrap_or((rest, ""));
    let (close, text) = rest.split_once('\n').unwrap_or((rest, ""));
    let format = format.strip_suffix('\r').unwrap_or(format);
    let Some(key) = format.strip_prefix("format") else { return Err(malformed) };
    let Some(value) = key.trim_start_matches(' ').strip_prefix('=') else { return Err(malformed) };
    if close.strip_suffix('\r').unwrap_or(close) != "+++" {
        return Err(FormatFault::FrontMatter { line: 3 });
    }
    let number = value.trim_start_matches(' ').trim_start_matches('0');
    if number.is_empty() {
        return Err(FormatFault::BadFormat);
    }
    for byte in number.bytes() {
        if !byte.is_ascii_digit() {
            return Err(FormatFault::BadFormat);
        }
    }
    if number != "1" {
        return Err(FormatFault::NewerFormat { number: number.into() });
    }
    Ok(text)
}

#[cfg(test)]
mod tests;
