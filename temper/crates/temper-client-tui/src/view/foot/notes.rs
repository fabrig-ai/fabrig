//! Notes following a body's entries (client/README.md, 6.14, The terminal's notes).
use super::{Words, look, text};
use crate::{Tui, words::Word};
use alloc::boxed::Box;
use skein_lib::{List, Writer};
use skein_ui::{Builder, Role};
use temper_client_domain::{Domain, pages::terminal::Note};
use temper_client_view::shared;

#[derive(Debug)]
pub(crate) struct Notes {
    pub lines: Box<[Box<[u8]>]>,
}
pub(super) fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.checked_add(part.len()).expect("view words fit memory");
    }
    let mut out = Writer::new(bytes);
    for part in parts {
        out.put(part).expect("parts counted");
    }
    out.finish()
}
fn help(root: &Domain, words: &Words, lines: &mut List<Box<[u8]>>) {
    let commands = crate::offered(root, words);
    let mut longest = 0_usize;
    for command in &commands {
        longest = longest.max(command.name().len());
    }
    for command in commands {
        let mut name = Writer::new(longest.checked_add(2).expect("command padding fits"));
        name.put(command.name()).expect("command fits longest");
        for _ in command.name().len()..longest.checked_add(2).expect("command padding fits") {
            name.put(b" ").expect("padding counted");
        }
        lines.push(join(&[&name.finish(), &super::lists::description(command, words)])).expect("help lines counted");
    }
    for key in [Word::KeyEnter, Word::KeyTab, Word::KeyPage, Word::KeyEsc, Word::KeyClose] {
        lines.push(words.say(key, &[])).expect("help keys counted");
    }
}
pub(crate) fn read_notes(root: &Domain, tui: &Tui, words: &Words) -> Notes {
    let mut lines = List::with_capacity(
        8_u32
            .checked_mul(18)
            .expect("eight notes, thirteen commands and five keys")
            .checked_add(1)
            .expect("TUI note fits"),
    );
    for note in root.pages().notes() {
        match note {
            Note::Help => help(root, words, &mut lines),
            Note::Unknown { .. } | Note::Font { .. } | Note::Closed => {
                lines.push(shared::note(note)).expect("root notes counted");
            }
        }
    }
    if let Some(unknown) = tui.unknown() {
        lines.push(shared::note(&Note::Unknown { words: unknown.into() })).expect("TUI note counted");
    }
    Notes { lines: lines.into_boxed() }
}
pub(crate) fn draw_notes(b: &mut Builder, notes: &Notes, _words: &Words) {
    for line in &notes.lines {
        text(b, Role::Paragraph, line, Some(look::NOTE));
    }
}
