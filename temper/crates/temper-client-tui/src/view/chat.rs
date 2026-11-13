//! The chat's ordered conversation (temper/docs/design/client/pages/README.md and temper/docs/design/client/tui/pages/README.md).
use super::{
    Binds, foot, pane,
    printed::{self, Printed},
    text,
};
use crate::{
    Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{Wall, Writer};
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::Domain;
use temper_client_view::{
    shared::{self, DelegationLine, RunLine},
    words::Word as Shared,
};

pub(crate) mod read;

#[derive(Debug)]
pub(crate) struct Chat {
    pub head: Option<super::talk::Head>,
    pub to_end: u64,
    pub entries: Box<[Entry]>,
    pub notes: foot::Notes,
    pub pane: Option<pane::Pane>,
}
#[derive(Debug)]
pub(crate) enum Entry {
    Said(Box<[u8]>),
    Delegate { task: u64, words: Box<[u8]> },
    Agent { words: Box<[u8]>, growing: bool },
    Viewed(Box<[Piece]>),
    Run(RunLine),
    Dots,
    Delegation(DelegationLine),
    Proposal(Box<[u8]>),
}
#[derive(Debug)]
pub(crate) enum Piece {
    Text(Box<[u8]>),
    View(Printed),
}
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, _wall: Wall) -> Chat {
    let mut part = Chat {
        head: super::talk::read(root, tui, words),
        to_end: 0,
        entries: Box::default(),
        notes: foot::read_notes(root, tui, words),
        pane: None,
    };
    if let Some(chat) = root.pages().chat() {
        part.to_end = chat.to_end();
        part.pane = pane::read(chat, tui.columns());
        part.entries = read::entries(root, words);
    }
    part
}
fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("view words fit memory");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("parts measured");
    }
    out.finish()
}
/// The rule's short title (temper/docs/design/client/tui/pages/chat.md, 1, The rule above the box).
#[expect(clippy::disallowed_methods, reason = "the title's validated UTF-8 is cut at scalar boundaries")]
pub(crate) fn cut(title: &[u8], words: &Words) -> Box<[u8]> {
    let title = core::str::from_utf8(title).expect("view words are UTF-8");
    if title.chars().count() <= 20 {
        return title.as_bytes().into();
    }
    let mut end = 0;
    let mut word_end = None;
    let mut previous_end = 0;
    for (count, (at, character)) in title.char_indices().enumerate() {
        if character.is_whitespace() {
            word_end = Some(previous_end);
        }
        if count == 19 {
            end = at;
            break;
        }
        if !character.is_whitespace() {
            previous_end = at.checked_add(character.len_utf8()).expect("title fits memory");
        }
    }
    let end = match word_end {
        Some(end) => end,
        None => end,
    };
    join(&[title.as_bytes().get(..end).expect("title boundary"), &words.say(Word::Dots, &[])])
}
fn paragraphs(b: &mut Builder, bytes: &[u8], prefix: &[u8]) {
    let mut start = 0;
    let mut prefix = prefix;
    for (at, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            b.open(Role::Paragraph);
            text(b, Role::Text, prefix, None);
            text(b, Role::Text, bytes.get(start..at).expect("paragraph range"), None);
            b.close();
            prefix = b"";
            start = at.checked_add(1).expect("next paragraph within message");
        }
    }
    b.open(Role::Paragraph);
    text(b, Role::Text, prefix, None);
    text(b, Role::Text, bytes.get(start..).expect("last paragraph"), None);
    b.close();
}
fn entry(b: &mut Builder, entry: &Entry, words: &Words) {
    b.open(Role::Item);
    match entry {
        Entry::Said(bytes) => {
            b.style(look::SAID);
            paragraphs(b, bytes, b"");
        }
        Entry::Delegate { task, words: bytes } => {
            b.style(look::AGENT);
            let prefix =
                join(&[&words.view().say(Shared::Chat, &[Hole::Number(*task)]), &words.say(Word::Separator, &[])]);
            paragraphs(b, bytes, &prefix);
        }
        Entry::Agent { words: bytes, growing } => {
            b.style(look::AGENT);
            b.open(Role::Group);
            b.style(look::BULLETS);
            if *growing {
                skein_ui::markdown::read_growing(bytes, 2, |b| text(b, Role::Text, b" ", Some(look::END)), b);
            } else {
                skein_ui::markdown::read(bytes, 2, b);
            }
            b.close();
        }
        Entry::Viewed(pieces) => {
            b.style(look::AGENT);
            b.open(Role::Group);
            b.style(look::BULLETS);
            for piece in pieces {
                match piece {
                    Piece::Text(bytes) => skein_ui::markdown::read(bytes, 2, b),
                    Piece::View(view) => printed::draw(b, view, words),
                }
            }
            b.close();
        }
        Entry::Run(line) => {
            b.style(look::RUN);
            b.open(Role::Paragraph);
            text(b, Role::Text, &line.name, None);
            if let Some(description) = &line.description {
                text(b, Role::Text, b"  ", None);
                text(b, Role::Text, description, if line.failed > 0 { Some(look::RUN_FAILED) } else { None });
            }
            b.close();
        }
        Entry::Dots => {
            b.style(look::DOTS);
            text(b, Role::Paragraph, &words.say(Word::Dots, &[]), None);
        }
        Entry::Delegation(line) => {
            b.style(look::AGENT);
            let separator = words.say(Word::Separator, &[]);
            text(b, Role::Paragraph, &join(&[&line.kind, &separator, &line.title, &separator, &line.state]), None);
        }
        Entry::Proposal(title) => {
            b.style(look::AGENT);
            b.open(Role::Paragraph);
            text(b, Role::Text, &words.view().say(Shared::Proposal, &[]), Some(look::SIGNAL));
            text(b, Role::Text, b" ", None);
            text(b, Role::Text, title, None);
            b.close();
        }
    }
    b.close();
}
pub(crate) fn draw(b: &mut Builder, part: &Chat, words: &Words, _binds: &mut Binds) {
    b.open(Role::Main);
    if let Some(head) = &part.head {
        super::talk::draw(b, head, words);
    }
    if let Some(pane) = &part.pane
        && pane.mode == temper_client_domain::pages::chat::document::Mode::Draft
    {
        pane::draw(b, pane, words);
        b.close();
        return;
    }
    b.open(Role::Log);
    b.end_request(part.to_end);
    if part.head.is_some() {
        text(b, Role::Paragraph, b"", None);
    }
    b.open(Role::List);
    b.style(look::SPACED);
    for item in &part.entries {
        entry(b, item, words);
    }
    b.close();
    b.close();
    foot::draw_notes(b, &part.notes, words);
    if let Some(pane) = &part.pane
        && pane.mode == temper_client_domain::pages::chat::document::Mode::Split
    {
        pane::draw(b, pane, words);
    }
    b.close();
}
