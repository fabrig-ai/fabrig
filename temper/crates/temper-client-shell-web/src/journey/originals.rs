//! The originals of a recording (temper/docs/design/client/web/journeys.md, 3.3 and 3.6).
use super::{Kind, Line, Originals};
use jig_client_protocol_wire as w;
use skein_lib::Reader;
use skein_ui::Act;

/// How an event shows a secret's length (client/web/journeys.md, 3.3).
pub const SECRET_EVENT: &[u8] = b"Secret { len: ";

/// Read the originals of a recording's lines and the given texts (client/web/journeys.md, 3.6).
#[must_use]
pub fn originals(lines: &[Line], given: &[Box<[u8]>]) -> Originals {
    collect(lines, given, true)
}

pub(super) fn sign_in(lines: &[Line]) -> Originals {
    collect(lines, &[], false)
}

fn collect(lines: &[Line], given: &[Box<[u8]>], include_typed: bool) -> Originals {
    let mut candidates = Vec::new();
    for line in lines {
        match line {
            Line::Secret(value) => candidates.push((Kind::Link, value.clone())),
            Line::Stored { key: _, value } => candidates.push((Kind::Stored, stored(value).into())),
            Line::Save { key, value } => {
                if key.as_ref() == jig_client::STREAM_KEY {
                    candidates.push((Kind::Stored, stored(value).into()));
                }
            }
            Line::Act { act, event: _, name: _ } => {
                if include_typed && typed(line) {
                    match act {
                        Act::Input { value } => candidates.push((Kind::Typed, value.clone())),
                        Act::Press | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => {}
                    }
                }
            }
            Line::Journey { .. }
            | Line::Seed(_)
            | Line::Clocks { .. }
            | Line::Primary(_)
            | Line::Zone(_)
            | Line::Ways { .. }
            | Line::Forge(_)
            | Line::Path(_)
            | Line::Turn { .. }
            | Line::Start
            | Line::Due
            | Line::TabOpen(_)
            | Line::TabPoint(_)
            | Line::TabClose
            | Line::TabRefused
            | Line::Place(_)
            | Line::Heard(_)
            | Line::Carried { .. }
            | Line::Written(_)
            | Line::Unwritten(_)
            | Line::Lost(_)
            | Line::Exchange { .. }
            | Line::Open { .. }
            | Line::Close
            | Line::Erase(_)
            | Line::Writes(_)
            | Line::Copy { .. }
            | Line::Copied { .. }
            | Line::Navigate(_)
            | Line::Replace(_)
            | Line::Show { .. }
            | Line::Deadline(_)
            | Line::Mark(_)
            | Line::End(_) => {}
        }
    }
    Originals::new(candidates, given)
}
/// Whether the line is an act whose event shows a secret's length.
#[must_use]
pub fn typed(line: &Line) -> bool {
    match line {
        Line::Act { event, .. } => event.windows(SECRET_EVENT.len()).any(|part| part == SECRET_EVENT),
        Line::Journey { .. }
        | Line::Seed(_)
        | Line::Clocks { .. }
        | Line::Primary(_)
        | Line::Zone(_)
        | Line::Ways { .. }
        | Line::Forge(_)
        | Line::Path(_)
        | Line::Secret(_)
        | Line::Stored { .. }
        | Line::Turn { .. }
        | Line::Start
        | Line::Due
        | Line::TabOpen(_)
        | Line::TabPoint(_)
        | Line::TabClose
        | Line::TabRefused
        | Line::Place(_)
        | Line::Heard(_)
        | Line::Carried { .. }
        | Line::Written(_)
        | Line::Unwritten(_)
        | Line::Lost(_)
        | Line::Exchange { .. }
        | Line::Open { .. }
        | Line::Close
        | Line::Save { .. }
        | Line::Erase(_)
        | Line::Writes(_)
        | Line::Copy { .. }
        | Line::Copied { .. }
        | Line::Navigate(_)
        | Line::Replace(_)
        | Line::Show { .. }
        | Line::Deadline(_)
        | Line::Mark(_)
        | Line::End(_) => false,
    }
}

/// A whole `Kept` value's head length, decoded under the wire ceilings.
#[must_use]
pub fn kept_head(value: &[u8]) -> Option<usize> {
    if u32::try_from(value.len()).is_err() {
        return None;
    }
    let mut reader = Reader::new(value);
    let kept = w::Kept::decode(&w::CEILINGS, &mut reader).ok()?;
    reader.is_empty().then(|| value.len() - kept.record().len())
}
fn stored(value: &[u8]) -> &[u8] {
    kept_head(value).map_or(value, |head| &value[head..])
}

#[cfg(test)]
mod tests;
