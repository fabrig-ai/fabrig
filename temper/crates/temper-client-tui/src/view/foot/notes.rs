//! Notes following a body's entries (temper/docs/design/client/README.md, 6.14 and 6.17).
use super::{Words, look, text};
use crate::{
    Tui, limits,
    view::front::{self, LogNote},
    words::Word,
};
use alloc::boxed::Box;
use skein_lib::{List, Writer};
use skein_ui::{Builder, Role};
use temper_client_domain::{Domain, pages::terminal::Note};
use temper_client_view::shared;

#[derive(Debug)]
pub(crate) struct Notes {
    pub lines: Box<[Box<[u8]>]>,
    pub gave_up: Option<Box<[u8]>>,
    pub log: Option<LogNote>,
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
fn help_line(name: &[u8], description: &[u8], longest: usize, lines: &mut List<Box<[u8]>>) {
    let padded = longest.checked_add(2).expect("command padding fits");
    let mut out = Writer::new(padded);
    out.put(name).expect("command fits longest");
    for _ in name.len()..padded {
        out.put(b" ").expect("padding counted");
    }
    lines.push(join(&[&out.finish(), description])).expect("help lines counted");
}
fn help(root: &Domain, tui: &Tui, words: &Words, lines: &mut List<Box<[u8]>>) {
    let commands = crate::offered(root, words);
    let mut longest = 0_usize;
    for command in &commands {
        longest = longest.max(command.name().len());
    }
    if tui.front() {
        longest = longest.max(crate::LOG_COMMAND.len());
    }
    for command in commands {
        if tui.front() && command == temper_client_domain::pages::terminal::Command::Exit {
            help_line(crate::LOG_COMMAND, &words.say(Word::LogCommand, &[]), longest, lines);
        }
        help_line(command.name(), &super::lists::description(command, words, tui.front()), longest, lines);
    }
    for key in [
        Word::KeyEnter,
        Word::KeyTab,
        Word::KeyPage,
        Word::KeyEsc,
        if tui.front() { Word::KeyCloseFront } else { Word::KeyClose },
    ] {
        lines.push(words.say(key, &[])).expect("help keys counted");
    }
}
pub(crate) fn read_notes(root: &Domain, tui: &Tui, words: &Words) -> Notes {
    let mut lines = List::with_capacity(
        8_u32
            .checked_mul(18)
            .expect("eight notes, thirteen commands and five keys")
            .checked_add(1)
            .expect("TUI note fits")
            .checked_add(limits::BOX_LOG)
            .expect("box log fits")
            .checked_add(2)
            .expect("box log and its notes fit"),
    );
    for note in root.pages().notes() {
        match note {
            Note::Help => help(root, tui, words, &mut lines),
            Note::Unknown { .. } | Note::Font { .. } | Note::Closed => {
                lines.push(shared::note(note)).expect("root notes counted");
            }
        }
    }
    if let Some(unknown) = tui.unknown() {
        lines.push(shared::note(&Note::Unknown { words: unknown.into() })).expect("TUI note counted");
    }
    Notes { lines: lines.into_boxed(), gave_up: front::gave_up_note(tui, words), log: front::log_note(tui, words) }
}
pub(crate) fn draw_notes(b: &mut Builder, notes: &Notes, _words: &Words) {
    for line in &notes.lines {
        text(b, Role::Paragraph, line, Some(look::NOTE));
    }
    if let Some(note) = &notes.gave_up {
        text(b, Role::Paragraph, note, Some(look::REFUSED));
    }
    if let Some(log) = &notes.log {
        text(b, Role::Paragraph, &log.head, Some(look::HEAD));
        b.open(Role::Log);
        b.end_request(log.end);
        b.open(Role::Description);
        for line in &log.lines {
            text(b, Role::Paragraph, line, Some(look::NOTE));
        }
        b.close();
        b.close();
    }
}
