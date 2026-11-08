//! The chat's ordered conversation (temper/docs/design/client/README.md, 3.5 and 6.14).
use super::{Binds, foot, text};
use crate::{
    Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{Wall, Writer};
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::Domain;
use temper_client_view::{
    shared::{self, DelegationLine, RunLine, TerminalWord},
    words::Word as Shared,
};

mod read;

#[derive(Debug)]
pub(crate) struct Chat {
    pub project: Box<[u8]>,
    pub title: Box<[u8]>,
    pub task: u64,
    pub address: Box<[u8]>,
    pub columns: u16,
    pub to_end: u64,
    pub entries: Box<[Entry]>,
    pub notes: foot::Notes,
}
#[derive(Debug)]
pub(crate) enum Entry {
    Said(Box<[u8]>),
    Delegate { task: u64, words: Box<[u8]> },
    Agent { words: Box<[u8]>, growing: bool },
    Run(RunLine),
    Dots,
    Delegation(DelegationLine),
    Proposal(Box<[u8]>),
}
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, _wall: Wall) -> Chat {
    let mut part = Chat {
        project: Box::default(),
        title: Box::default(),
        task: 0,
        address: tui.address().into(),
        columns: tui.columns(),
        to_end: 0,
        entries: Box::default(),
        notes: foot::read_notes(root, tui, words),
    };
    if let Some(chat) = root.pages().chat() {
        part.task = chat.task;
        part.to_end = chat.to_end();
        if let Some(task) = chat.task(root.client()) {
            part.title = shared::heading(task.chip.title.bytes());
            if let Some(me) = root.pages().me(root.client()) {
                for project in &me.projects {
                    if project.project == task.project {
                        part.project = project.name.bytes().into();
                        break;
                    }
                }
            }
        }
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
#[expect(clippy::disallowed_methods, reason = "width walks the view words already admitted as UTF-8")]
fn width(bytes: &[u8]) -> u32 {
    let mut columns = 0_u32;
    for character in core::str::from_utf8(bytes).expect("view words are UTF-8").chars() {
        columns = columns.saturating_add(u32::from(skein_ui_screen::width(character)));
    }
    columns
}
fn top(b: &mut Builder, part: &Chat, words: &Words) {
    let star = join(&[&shared::say(words.view(), TerminalWord::Star), b" "]);
    let between = words.say(Word::Between, &[]);
    let number = join(&[b" ", &words.view().say(Shared::Chat, &[Hole::Number(part.task)])]);
    let total = width(&star)
        .saturating_add(width(&part.project))
        .saturating_add(width(&between))
        .saturating_add(width(&part.title))
        .saturating_add(width(&number))
        .saturating_add(width(&part.address))
        .saturating_add(4);
    b.open(Role::Region);
    b.region(look::CHAT_TOP);
    b.open(Role::Paragraph);
    text(b, Role::Text, &star, Some(look::STAR));
    text(b, Role::Text, &part.project, Some(look::NAME));
    text(b, Role::Text, &between, Some(look::FAINT));
    text(b, Role::Text, &part.title, Some(look::NAME));
    text(b, Role::Text, &number, Some(look::HINT));
    b.close();
    if part.columns == 0 || total <= u32::from(part.columns) {
        text(b, Role::Paragraph, &part.address, Some(look::ADDRESS));
    }
    b.close();
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
    top(b, part, words);
    b.open(Role::Log);
    b.end_request(part.to_end);
    b.open(Role::List);
    b.style(look::SPACED);
    for item in &part.entries {
        entry(b, item, words);
    }
    b.close();
    b.close();
    foot::draw_notes(b, &part.notes, words);
    b.close();
}
