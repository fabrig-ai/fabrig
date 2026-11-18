//! Recording placeholders (temper/docs/design/client/journey-placeholders.md, 3 and 6).
use super::{Line, Normalised, Unreadable, codes, fields_mut, lines};
use temper_client_redaction::Varied;

/// Replace the build, ports and shown approval codes, preserving framed widths.
pub fn normalise(recording: &[u8]) -> Result<Normalised, Unreadable> {
    let mut lines = lines(recording)?;
    let mut varied = Varied::new();
    varied.decimal_ports();
    for line in &lines {
        if let Line::Version(build) = line {
            varied.build(build);
        }
    }
    for code in codes(&lines) {
        varied.code(&code);
    }
    for line in &mut lines {
        for (field, framing) in fields_mut(line) {
            varied.seen(field, framing);
        }
    }
    let mut recording = Vec::new();
    for line in &mut lines {
        for (field, _) in fields_mut(line) {
            varied.replace(field);
        }
        line.write(&mut recording);
    }
    Ok(varied.counts(recording.into()))
}

#[cfg(test)]
mod tests;
