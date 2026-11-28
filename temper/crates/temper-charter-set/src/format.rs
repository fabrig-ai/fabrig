//! Front matter of an overlay's charter files (temper/docs/design/client/ctl/overlays.md, 2).
use crate::{File, Refusal, Set};

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
            Self::UnknownFile { .. } | Self::FrontMatter { .. } | Self::BadFormat { .. } | Self::NewerFormat { .. } => {
                return self;
            }
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
    let Some(rest) = opening(text) else { return Ok(Some(text)) };
    let malformed = Refusal::FrontMatter { file };
    let Some((format, rest)) = rest.split_once('\n') else { return Err(malformed) };
    let (close, text) = rest.split_once('\n').unwrap_or((rest, ""));
    if close.strip_suffix('\r').unwrap_or(close) != "+++" {
        return Err(malformed);
    }
    let format = format.strip_suffix('\r').unwrap_or(format);
    let Some(key) = format.strip_prefix("format") else { return Err(malformed) };
    let Some(value) = key.trim_start_matches(' ').strip_prefix('=') else { return Err(malformed) };
    let number = value.trim_start_matches(' ').trim_start_matches('0');
    if number.is_empty() {
        return Err(Refusal::BadFormat { file });
    }
    for byte in number.bytes() {
        if !byte.is_ascii_digit() {
            return Err(Refusal::BadFormat { file });
        }
    }
    if number != "1" {
        return Err(Refusal::NewerFormat { file, number: number.into() });
    }
    Ok(Some(text))
}

#[cfg(test)]
mod tests;
