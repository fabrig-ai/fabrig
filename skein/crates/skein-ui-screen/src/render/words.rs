//! The applicable words of skein/docs/design/ui-text.md, section 5, kept here
//! so the full-screen renderer does not depend on the text shell's io.

use super::canvas::Canvas;
use crate::{Frame, Paint};
use skein_lib::{Decimal, List};
use skein_ui::{EntryKind, Node, Role, State, Tree};

use super::Info;

pub(super) const DISABLED: &[u8] = b" (disabled)";

pub(super) fn mark(n: &Node, paint: Paint, out: &mut Canvas, frame: &mut Frame) {
    if n.role.is_selectable() {
        out.safe(if n.states.as_slice().contains(&State::Selected) { b"(*) " } else { b"( ) " }, paint, false, frame);
    }
    if n.role == Role::Check {
        let on = n.states.as_slice().contains(&State::Pressed) || n.states.as_slice().contains(&State::Selected);
        out.safe(if on { b"[x] " } else { b"[ ] " }, paint, false, frame);
    }
}

pub(super) fn sort(n: &Node, paint: Paint, out: &mut Canvas, frame: &mut Frame) {
    if let Some(sort) = n.sort {
        let word = match sort {
            skein_ui::Sort::Ascending => b" (ascending)".as_slice(),
            skein_ui::Sort::Descending => b" (descending)",
        };
        out.safe(word, paint, false, frame);
    }
}

pub(super) fn position(tree: &Tree, at: u32, info: &List<Info>, paint: Paint, out: &mut Canvas, frame: &mut Frame) {
    let n = tree.get(at).expect("positioned node exists");
    let Some(position) = n.position else { return };
    out.safe(b" (", paint, false, frame);
    out.safe(Decimal::of(u64::from(position)).as_bytes(), paint, false, frame);
    let mut parent = info.get(at).expect("node information exists").parent;
    for _ancestor in 0..tree.len() {
        let Some(at) = parent else { break };
        let n = tree.get(at).expect("parent exists");
        if n.role.is_collection() {
            if let Some(count) = n.count {
                out.safe(b" of ", paint, false, frame);
                out.safe(Decimal::of(u64::from(count)).as_bytes(), paint, false, frame);
            }
            break;
        }
        parent = info.get(at).expect("parent information exists").parent;
    }
    out.safe(b")", paint, false, frame);
}

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
            State::Disabled => DISABLED,
            State::Expanded => b" (expanded)",
            State::Busy => b" (busy)",
            State::Current => b" (current)",
            State::Invalid => b" (invalid)",
            State::Pressed | State::Selected => {
                if n.role == Role::Choice || n.role == Role::Option || n.role == Role::Check {
                    b""
                } else if state == State::Pressed {
                    b" (pressed)".as_slice()
                } else {
                    b" (selected)"
                }
            }
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
