//! Notes following a body's entries (temper/docs/design/client/tui/pages/README.md and temper/docs/design/client/tui/box.md).
use super::{Words, look, text};
use crate::{
    Tui, limits,
    view::front::{self, LogNote, StoppingNote},
    words::Word,
};
use alloc::boxed::Box;
use skein_lib::{List, Writer};
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::{Domain, pages::terminal::Note};
use temper_client_view::shared;

#[derive(Debug)]
pub(crate) struct Notes {
    pub lines: Box<[Box<[u8]>]>,
    pub gave_up: Option<Box<[u8]>>,
    pub log: Option<LogNote>,
    pub stopping: Option<StoppingNote>,
}
impl Notes {
    pub(crate) const fn follows_end(&self) -> bool {
        self.log.is_some() || self.stopping.is_some()
    }
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
    let commands = super::lists::command_items(root, tui, words);
    let mut longest = 0_usize;
    for command in &commands {
        longest = longest.max(command.name.len());
    }
    for command in commands {
        help_line(&command.name, &command.words, longest, lines);
    }
    for key in [
        Word::KeyEnter,
        Word::KeyTab,
        Word::KeyPage,
        Word::KeyEsc,
        if tui.front() { Word::KeyCloseFront } else { Word::KeyClose },
    ] {
        lines.push(words.say(key, &[])).expect("help keys counted");
        if key == Word::KeyEsc && crate::offered_own(root, tui).contains(&crate::Own::Needs) {
            lines.push(words.say(Word::KeyWalk, &[])).expect("walk key counted");
        }
    }
}
pub(crate) fn read_notes(root: &Domain, tui: &Tui, words: &Words) -> Notes {
    let mut lines = List::with_capacity(
        8_u32
            .checked_mul(25)
            .expect("eight notes, nineteen commands and six keys")
            .checked_add(2)
            .expect("walk and opener notes fit")
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
    if tui.walk().is_some() && super::lists::walk(root, tui, words).is_none() {
        lines.push(words.say(Word::WalkDone, &[])).expect("walk note counted");
    }
    if let Some(address) = tui.not_opened() {
        lines.push(words.say(Word::OpenAt, &[Hole::Name(address.into())])).expect("opener note counted");
    }
    Notes {
        lines: lines.into_boxed(),
        gave_up: front::gave_up_note(tui, words),
        log: front::log_note(tui, words),
        stopping: front::stopping_note(tui, words),
    }
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
        draw_log(b, log, None, None);
    }
    if let Some(stopping) = &notes.stopping {
        text(b, Role::Paragraph, &stopping.log.head, Some(look::HEAD));
        draw_log(b, &stopping.log, Some(&stopping.hint), stopping.end.as_deref());
    }
}
fn draw_log(b: &mut Builder, log: &LogNote, hint: Option<&[u8]>, end: Option<&[u8]>) {
    b.open(Role::Log);
    b.end_request(log.end);
    b.open(Role::Description);
    if let Some(hint) = hint {
        text(b, Role::Paragraph, hint, Some(look::HINT));
    }
    for line in &log.lines {
        text(b, Role::Paragraph, line, Some(look::NOTE));
    }
    if let Some(end) = end {
        text(b, Role::Paragraph, end, Some(look::NOTE));
    }
    b.close();
    b.close();
}
