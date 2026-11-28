//! Front matter of an overlay's charter files (temper/docs/design/client/ctl/overlays.md, 2).
use crate::{CHARTER_BYTES, File, Refusal, Set};
use alloc::boxed::Box;
use skein_toml::Limits;

pub(super) const SETTINGS_WORDS: &str = "`settings` is a project's charter file's key";
pub(super) const TOML_LIMITS: Limits =
    Limits { input_bytes: CHARTER_BYTES, depth: CHARTER_BYTES, entries: CHARTER_BYTES };

impl Set<'_> {
    /// Read the format of every file, including files the selected charter does not take.
    /// # Errors
    /// Refuses malformed front matter and formats other than 1.
    pub fn overlay_bodies(&self) -> Result<Self, Refusal> {
        Ok(Self {
            common: body(File::Common, self.common)?,
            goal: body(File::Goal, self.goal)?,
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
            Self::UnknownFile { .. }
            | Self::FrontMatter { .. }
            | Self::BadFormat { .. }
            | Self::NewerFormat { .. }
            | Self::Settings { .. } => return self,
        };
        let text = match file {
            File::Common => source.common,
            File::Goal => source.goal,
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

pub(super) fn opening(text: &str) -> Option<&str> {
    if text == "+++" {
        return Some("");
    }
    if let Some(rest) = text.strip_prefix("+++\n") {
        return Some(rest);
    }
    text.strip_prefix("+++\r\n")
}

// `rest` starts at line two, immediately after the opening delimiter.
pub(super) fn split_front(rest: &str) -> Option<(&str, &str, usize)> {
    let mut offset = 0_usize;
    for (index, line) in rest.split_inclusive('\n').enumerate() {
        let end = offset.checked_add(line.len()).expect("line within text");
        if line == "+++\n" || line == "+++\r\n" || line == "+++" {
            let front = rest.get(..offset).expect("front matter ends at a line boundary");
            let body = rest.get(end..).expect("body starts at a line boundary");
            return Some((front, body, index.checked_add(3).expect("body line after the delimiters")));
        }
        offset = end;
    }
    None
}

fn body(file: File, text: Option<&str>) -> Result<Option<&str>, Refusal> {
    let Some(text) = text else { return Ok(None) };
    match read_file(text) {
        Ok(text) => Ok(Some(text)),
        Err(FormatFault::FrontMatter { .. }) => Err(Refusal::FrontMatter { file }),
        Err(FormatFault::BadFormat) => Err(Refusal::BadFormat { file }),
        Err(FormatFault::NewerFormat { number }) => Err(Refusal::NewerFormat { file, number }),
        Err(FormatFault::Settings { line, .. }) => Err(Refusal::Settings { file, line }),
    }
}

/// A refusal of one file's front matter, independent of the file's name.
#[derive(Debug, PartialEq, Eq)]
pub enum FormatFault {
    FrontMatter {
        line: u32,
    },
    BadFormat,
    NewerFormat {
        number: Box<str>,
    },
    /// `front_matter_line` is the format-only judgement's location until it takes project files.
    Settings {
        line: u32,
        front_matter_line: u32,
    },
}

/// Read an overlay file's format and return its text without front matter.
/// # Errors
/// Refuses malformed front matter and formats other than 1.
pub fn read_file(text: &str) -> Result<&str, FormatFault> {
    let Some(rest) = opening(text) else { return Ok(text) };
    match legacy_format(rest) {
        Err(FormatFault::FrontMatter { line }) => {
            if let Some(settings) = settings_line(rest) {
                return Err(FormatFault::Settings { line: settings, front_matter_line: line });
            }
            Err(FormatFault::FrontMatter { line })
        }
        other => other,
    }
}

fn settings_line(rest: &str) -> Option<u32> {
    let (front, _, _) = split_front(rest)?;
    let doc = skein_toml::parse(front.as_bytes(), &TOML_LIMITS).ok()?;
    let settings = doc.get(doc.root(), b"settings")?;
    Some(source_line(settings.line))
}

pub(super) fn source_line(toml_line: u64) -> u32 {
    u32::try_from(toml_line.checked_add(1).expect("opening delimiter precedes TOML"))
        .expect("TOML is bounded to 65,536 bytes")
}

fn legacy_format(rest: &str) -> Result<&str, FormatFault> {
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
