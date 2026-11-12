//! Every recording field (client/web/journeys.md, 3.6).
use super::{Framing, Line};
use jig_client_shell_web::adapter::Heard;
use skein_ui::Act;

/// Each byte field in order, except stream pieces, which `Streams` reads.
#[must_use]
pub fn fields_mut(line: &mut Line) -> Vec<(&mut Box<[u8]>, Framing)> {
    use Framing::{Framed, Text};
    match line {
        Line::Ways { content, origin } => vec![(content, Text), (origin, Text)],
        Line::TabOpen(bytes)
        | Line::TabPoint(bytes)
        | Line::Forge(bytes)
        | Line::Path(bytes)
        | Line::Secret(bytes)
        | Line::Place(bytes)
        | Line::Copy { words: bytes, .. }
        | Line::Navigate(bytes)
        | Line::Replace(bytes)
        | Line::Show { path: bytes, .. } => vec![(bytes, Text)],
        Line::Stored { key, value } | Line::Save { key, value } => vec![(key, Framed), (value, Framed)],
        Line::Written(bytes) | Line::Unwritten(bytes) | Line::Erase(bytes) => vec![(bytes, Framed)],
        Line::Heard(heard) => match heard {
            Heard::Answered { body, .. } => vec![(body, Framed)],
            Heard::Failed { .. }
            | Heard::StreamOpened
            | Heard::StreamRefused { .. }
            | Heard::Piece { .. }
            | Heard::StreamEnded
            | Heard::Stored { .. }
            | Heard::NotStored { .. } => Vec::new(),
        },
        Line::Act { act, event, name } => {
            let mut fields = match act {
                Act::Input { value } | Act::Drawing { data: value } => vec![(value, Text)],
                Act::Press | Act::Submit | Act::Toggle | Act::Near { .. } => Vec::new(),
            };
            fields.extend([(event, Text), (name, Text)]);
            fields
        }
        Line::Exchange { path, body, headers, .. } => {
            let mut fields = vec![(path, Text), (body, Framed)];
            fields.extend(headers.iter_mut().flat_map(|(name, value)| [(name, Text), (value, Text)]));
            fields
        }
        Line::Open { path, headers, .. } => {
            let mut fields = vec![(path, Text)];
            fields.extend(headers.iter_mut().flat_map(|(name, value)| [(name, Text), (value, Text)]));
            fields
        }
        Line::Mark(html) | Line::End(html) => html.iter_mut().map(|bytes| (bytes, Text)).collect(),
        Line::Journey { .. }
        | Line::Seed(_)
        | Line::Clocks { .. }
        | Line::Primary(_)
        | Line::Zone(_)
        | Line::Turn { .. }
        | Line::Start
        | Line::Copied { .. }
        | Line::Due
        | Line::TabClose
        | Line::TabRefused
        | Line::Carried { .. }
        | Line::Lost(_)
        | Line::Close
        | Line::Writes(_)
        | Line::Deadline(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests;
