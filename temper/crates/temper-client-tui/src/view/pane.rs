//! The draft beside a chat (temper/docs/design/client/tui/pages/design-session.md, 1).
use super::text;
use crate::{
    look, marked,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{contract as c, thread::Line};
use skein_lib::{
    List,
    bytes::{concat, copy_of},
};
use skein_ui::{Builder, Role};
use temper_client_domain::pages::chat::{
    Chat,
    document::{self, Mode, NoteChange},
};

/// The narrowest window in which both halves read.
pub const HALVES: u16 = 80;

#[derive(Debug)]
pub(crate) struct Pane {
    pub mode: Mode,
    pub next: Mode,
    pub name: Box<[u8]>,
    pub body: Box<[u8]>,
    pub new: Box<[usize]>,
    pub writing: bool,
}

pub(crate) fn shown(chat: &Chat, columns: u16) -> Option<Mode> {
    if chat.draft.is_none() && !chat.writing {
        return None;
    }
    Some(match chat.mode {
        Mode::Split if columns < HALVES => Mode::Draft,
        Mode::Chat => Mode::Chat,
        Mode::Split => Mode::Split,
        Mode::Draft => Mode::Draft,
    })
}

pub(crate) fn writes(chat: &Chat) -> Box<[NoteChange]> {
    let Some(draft) = &chat.draft else { return Box::default() };
    let mut writes = List::uncapped();
    for line in &chat.thread.lines() {
        match line {
            Line::Words { .. } => writes.clear(),
            Line::Activity { line } => {
                if let c::Activity::Words { by: c::Requester::Person(_), .. } = &line.source.line {
                    writes.clear();
                }
            }
            Line::Turn { turn, .. } => {
                for message in &turn.messages {
                    for block in &message.blocks {
                        if let c::Block::Call(call) = block
                            && !call.failed
                            && let Some(outcome) = &call.outcome
                            && let Some(write) = document::tool_write(call.tool.bytes(), outcome.bytes())
                            && write.note == draft.note
                        {
                            writes.push(write.change).expect("turn writes fit memory");
                        }
                    }
                }
            }
            Line::Undecoded { .. } | Line::Provisional { .. } => {}
        }
    }
    writes.into_boxed()
}

pub(crate) fn read(chat: &Chat, columns: u16) -> Option<Pane> {
    let mode = shown(chat, columns)?;
    let next = match mode {
        Mode::Chat if columns < HALVES => Mode::Draft,
        Mode::Chat => Mode::Split,
        Mode::Split => Mode::Draft,
        Mode::Draft => Mode::Chat,
    };
    let (name, body, writing) = match &chat.draft {
        Some(draft) => (concat(&[&document::slug(&draft.title), b".md"]), copy_of(&draft.body), false),
        None => (copy_of(b"untitled.md"), Box::default(), true),
    };
    let new = marked(&body, &writes(chat));
    Some(Pane { mode, next, name, body, new, writing })
}

// Only the pane disables flow reading: an empty info string keeps its fenced code.
fn code_flows(bytes: &[u8]) -> Box<[u8]> {
    let mut out = List::uncapped();
    let mut fenced = false;
    for line in bytes.split_inclusive(|byte| *byte == b'\n') {
        let mut line = line;
        if let Some(info) = line.strip_prefix(b"```") {
            if fenced {
                if info.trim_ascii().is_empty() {
                    fenced = false;
                }
            } else {
                fenced = true;
                if info.trim_ascii() == b"mermaid" {
                    line = if line.ends_with(b"\n") { b"```\n" } else { b"```" };
                }
            }
        }
        for byte in line {
            out.push(*byte).expect("pane Markdown fits memory");
        }
    }
    out.into_boxed()
}

pub(crate) fn draw(b: &mut Builder, pane: &Pane, words: &Words) {
    b.open(Role::Region);
    if pane.mode == Mode::Split {
        b.region(look::PANE);
    }
    b.open(Role::Paragraph);
    text(b, Role::Text, &pane.name, Some(if pane.writing { look::HINT } else { look::NAME }));
    if pane.writing {
        text(b, Role::Text, &words.say(Word::PaneWriting, &[]), Some(look::HINT));
    }
    b.close();
    text(b, Role::Paragraph, b"", None);
    if pane.writing {
        text(b, Role::Paragraph, &words.say(Word::PaneEmpty, &[]), Some(look::HINT));
    } else {
        let start = if pane.mode == Mode::Split { pane.new.first().copied().unwrap_or(0) } else { 0 };
        for (index, section) in document::sections(&pane.body).enumerate() {
            if index < start {
                continue;
            }
            if index > start {
                text(b, Role::Paragraph, b"", None);
            }
            let bytes = code_flows(section.bytes);
            if pane.new.contains(&index) && !section.heading.is_empty() {
                let end = skein_lib::bytes::find(&bytes, b"\n").unwrap_or(bytes.len());
                skein_ui::markdown::read_growing(
                    bytes.get(..end).expect("section heading"),
                    2,
                    |b| text(b, Role::Text, &words.say(Word::PaneNew, &[]), Some(look::SIGNAL)),
                    b,
                );
                skein_ui::markdown::read(bytes.get(end..).expect("section body"), 2, b);
            } else {
                skein_ui::markdown::read(&bytes, 2, b);
            }
        }
    }
    b.close();
}
