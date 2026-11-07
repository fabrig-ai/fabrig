//! The applicable words of skein/docs/design/ui-text.md, section 5, kept here
//! so the full-screen renderer does not depend on the text shell's io.

use super::canvas::Canvas;
use crate::{Frame, Paint};
use skein_ui::{EntryKind, Node, State};

pub(super) fn colon(name: &[u8], paint: Paint, out: &mut Canvas, frame: &mut Frame) {
    let marked = match name.last() {
        Some(last) => b"?!:".contains(last),
        None => false,
    };
    if !marked {
        out.safe(b":", paint, false, frame);
    }
}

pub(super) fn entry(n: &Node, paint: Paint, out: &mut Canvas, frame: &mut Frame) {
    let word = match n.entry {
        None | Some(EntryKind::Text | EntryKind::Secret) => b"".as_slice(),
        Some(EntryKind::Number) => b" (number)",
        Some(EntryKind::Email) => b" (email)",
        Some(EntryKind::Telephone) => b" (telephone)",
        Some(EntryKind::Url) => b" (url)",
        Some(EntryKind::Date) => b" (date, YYYY-MM-DD)",
        Some(EntryKind::Time) => b" (time, HH:MM)",
    };
    out.safe(word, paint, false, frame);
}

pub(super) fn states(n: &Node, paint: Paint, out: &mut Canvas, frame: &mut Frame) {
    for state in [
        State::Disabled,
        State::Expanded,
        State::Busy,
        State::Current,
        State::Invalid,
        State::Pressed,
        State::Selected,
        State::Required,
        State::Completed,
        State::SubmitsOnEnter,
    ] {
        if !n.states.as_slice().contains(&state) {
            continue;
        }
        let word = match state {
            State::Disabled => b" (disabled)".as_slice(),
            State::Expanded => b" (expanded)",
            State::Busy => b" (busy)",
            State::Current => b" (current)",
            State::Invalid => b" (invalid)",
            State::Pressed => b" (pressed)",
            State::Selected => b" (selected)",
            State::Required => b" (required)",
            State::Completed => b" (completed)",
            State::SubmitsOnEnter => {
                if n.role.is_text_field() {
                    b" (enter sends)".as_slice()
                } else {
                    b""
                }
            }
            State::FoldsAway | State::Continues | State::ToLogEnd | State::Options => b"",
        };
        out.safe(word, paint, false, frame);
    }
}
