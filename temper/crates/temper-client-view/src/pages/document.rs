//! The document beside a chat (temper/docs/design/client/pages/README.md).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{Map, bytes::copy_of};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Key, Modifier, Role, Run, Shortcut, State};
use temper_client_domain::{
    Domain,
    pages::{
        Act,
        chat::{
            Chat,
            document::{self, Mode, NoteChange, NoteWrite},
        },
    },
};

pub(crate) fn mode(chat: Option<&Chat>) -> skein_ui::Style {
    let mode = match chat {
        Some(chat) => chat.mode,
        None => Mode::Chat,
    };
    style(mode)
}

pub(crate) fn style(mode: Mode) -> skein_ui::Style {
    match mode {
        Mode::Chat => look::MODE_CHAT,
        Mode::Split => look::MODE_SPLIT,
        Mode::Draft => look::MODE_DRAFT,
    }
}

pub(crate) fn offers_group(root: &Domain, chat: &Chat, task: &c::Task) -> bool {
    chat.draft.is_some()
        || (task.result.is_none()
            && !super::chats::ended(&task.chip)
            && temper_client_domain::pages::chat_offers_draft(root, task))
}

pub(crate) fn seg(b: &mut Builder, chat: &Chat, words: &Words, bindings: &mut Bindings) {
    modes(b, Word::DraftGroup, chat.mode, words, bindings, false);
}

pub(crate) fn modes(b: &mut Builder, name: Word, chosen: Mode, words: &Words, bindings: &mut Bindings, new: bool) {
    b.open(Role::Group);
    b.name(&words.say(name, &[]));
    b.style(look::SEG);
    let next = match chosen {
        Mode::Chat => Mode::Split,
        Mode::Split => Mode::Draft,
        Mode::Draft => Mode::Chat,
    };
    for (mode, word) in
        [(Mode::Chat, Word::DraftChat), (Mode::Split, Word::DraftSplit), (Mode::Draft, Word::DraftDraft)]
    {
        b.open(Role::Button);
        b.name(&words.say(word, &[]));
        if mode == Mode::Split {
            b.style(look::SEG_SPLIT);
        }
        if mode == chosen {
            b.state(State::Pressed);
        }
        let act = if new {
            Act::Go { address: temper_client_domain::Address::NewChat { draft: mode != Mode::Chat } }
        } else {
            Act::Mode { mode }
        };
        b.bind(bindings.insert(act));
        if mode == next {
            b.shortcut(Shortcut { key: b'\\', modifier: Modifier::Primary });
        }
        b.close();
    }
    b.close();
}

pub(crate) fn pane(
    b: &mut Builder,
    root: &Domain,
    chat: &Chat,
    task: &c::Task,
    words: &Words,
    times: &mut Times,
    bindings: &mut Bindings,
) {
    let Some(draft) = &chat.draft else {
        empty_chat(b, root, chat, task, words, bindings);
        return;
    };
    if chat.mode == Mode::Chat {
        return;
    }
    b.open(Role::Region);
    b.name(&draft.title);
    b.style(look::DOC);
    b.open(Role::Slot);
    b.style(look::DOC_INNER);
    b.open(Role::Paragraph);
    b.style(look::DOC_PATH);
    if super::chat::asking(root) {
        proposal_line(b, chat, words, times);
    } else {
        b.text(Run::Plain, &words.say(Word::DraftPath, &[Hole::Name(document::slug(&draft.title))]));
    }
    if let Some(changed) = draft.changed {
        let ago = times.say(words, TimeForm::Ago, changed);
        if words.shell == Shell::Terminal {
            b.text(Run::Plain, &words.say(Word::ToolSeparator, &[]));
        }
        b.text(Run::Plain, &words.say(Word::DraftEdited, &[Hole::Name(ago)]));
    }
    b.close();
    let write = chat.newest_write();
    let mut tinted = None;
    if let Some(write) = write {
        match write.change {
            NoteChange::First { .. } => {
                for (index, section) in document::sections(&draft.body).enumerate() {
                    if section.bytes.starts_with(b"## ") {
                        tinted = Some(index);
                        break;
                    }
                }
            }
            NoteChange::Later { first, .. } => tinted = Some(document::section_holding(&draft.body, first)),
        }
    }
    let flashed = match chat.shown {
        Some(line) => Some(document::section_holding(&draft.body, line)),
        None => None,
    };
    let mut names =
        Map::<Box<[u8]>, u64>::with_capacity(u32::try_from(draft.body.len()).expect("bounded body").saturating_add(1));
    for (index, section) in document::sections(&draft.body).enumerate() {
        let nth = names.get(section.heading).copied().unwrap_or(0);
        names
            .insert(copy_of(section.heading), nth.checked_add(1).expect("bounded sections"))
            .expect("one name per section");
        b.open(Role::Slot);
        b.style(look::DOC_SEC);
        b.key(Key { owner: crate::frame::owner(), kind: 11, a: heading_hash(section.heading), b: nth });
        let flashed = flashed == Some(index);
        if flashed && root.pages().switcher().renaming().is_none() {
            b.focus(chat.shown_count);
        }
        b.open(Role::Slot);
        if tinted == Some(index) {
            b.style(look::DOC_CHANGED);
        }
        if flashed {
            b.style(look::DOC_FLASH);
            b.key(Key { owner: crate::frame::owner(), kind: 12, a: chat.shown_count, b: 0 });
        }
        if index == 0 {
            first_section(b, section.bytes, chat, words);
        } else {
            crate::views::read(b, section.bytes, 1, Some(chat), words);
        }
        if tinted == Some(index)
            && let Some(write) = write
            && let Some(changed) = draft.changed
        {
            let ago = times.say(words, TimeForm::Ago, changed);
            let word = match write.change {
                NoteChange::First { .. } => Word::DraftAdded,
                NoteChange::Later { .. } => Word::DraftChanged,
            };
            b.open(Role::Paragraph);
            b.style(look::DOC_NOTE);
            b.text(Run::Plain, &words.say(word, &[Hole::Number(task.chip.task), Hole::Name(ago)]));
            b.close();
        }
        b.close();
        b.close();
    }
    b.close();
    b.close();
}

fn empty_chat(b: &mut Builder, root: &Domain, chat: &Chat, task: &c::Task, words: &Words, bindings: &mut Bindings) {
    if chat.mode != Mode::Chat {
        let path = if super::chat::asking(root) {
            words.say(Word::DraftNotYet, &[Hole::Number(chat.task)])
        } else {
            words.say(Word::DraftFile, &[Hole::Name(document::slug(b""))])
        };
        let can_write = !chat.writing
            && task.result.is_none()
            && !super::chats::ended(&task.chip)
            && temper_client_domain::pages::chat_offers_draft(root, task);
        empty(
            b,
            words,
            &words.say(Word::DraftDraft, &[]),
            &path,
            chat.writing,
            if can_write { Some(bindings) } else { None },
        );
    }
}

fn heading_hash(heading: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in heading {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn first_section(b: &mut Builder, bytes: &[u8], chat: &Chat, words: &Words) {
    for part in &crate::views::parts(bytes) {
        match part {
            temper_client_domain::views::Part::Text(text) => opening(b, text.as_bytes()),
            temper_client_domain::views::Part::View(address) => {
                crate::views::block(b, address, chat.view(address), words);
            }
        }
    }
}

// The introductory metadata reads as Markdown in a muted slot beneath the
// document’s Markdown heading.
fn opening(b: &mut Builder, bytes: &[u8]) {
    let Some(heading) = bytes.strip_prefix(b"# ") else {
        skein_ui::markdown::read(bytes, 1, b);
        return;
    };
    let Some(end) = skein_lib::bytes::find(heading, b"\n") else {
        skein_ui::markdown::read(bytes, 1, b);
        return;
    };
    let end = end.checked_add(3).expect("heading fits body");
    skein_ui::markdown::read(bytes.get(..end).expect("heading prefix"), 1, b);
    let rest = bytes.get(end..).expect("body after heading").trim_ascii_start();
    if rest.is_empty()
        || rest.starts_with(b"#")
        || rest.starts_with(b"```")
        || rest.starts_with(b"-")
        || rest.starts_with(b"|")
    {
        skein_ui::markdown::read(rest, 1, b);
        return;
    }
    let end = skein_lib::bytes::find(rest, b"\n\n").unwrap_or(rest.len());
    b.open(Role::Slot);
    b.style(look::DOC_MUTED);
    skein_ui::markdown::read(rest.get(..end).expect("introductory paragraph"), 1, b);
    b.close();
    skein_ui::markdown::read(rest.get(end..).expect("remaining body"), 1, b);
}

fn proposal_line(b: &mut Builder, chat: &Chat, words: &Words, times: &mut Times) {
    for card in &chat.cards {
        if let Some(c::ReadValue::Proposal(proposal)) = &card.page {
            // Every named holder means the Ask temper proposal waits for its owner.
            b.text(Run::Plain, &words.say(Word::DraftWaits, &[Hole::Number(proposal.key.number)]));
            return;
        }
    }
    if let Some((_, at)) = chat.decided {
        let at = times.say(words, TimeForm::Clock, at);
        b.text(Run::Plain, &words.say(Word::DraftAccepted, &[Hole::Name(at)]));
    } else if let Some(number) = chat.departed_proposal {
        b.text(Run::Plain, &words.say(Word::DraftDecided, &[Hole::Number(number)]));
    } else {
        b.text(Run::Plain, &words.say(Word::DraftNotProposed, &[Hole::Number(chat.task)]));
    }
}

/// Successful writes leave the ordinary tool disclosure.
pub(crate) fn write(call: &c::Call) -> Option<NoteWrite> {
    if call.failed || !document::writes(call.tool.bytes()) {
        return None;
    }
    document::tool_write(call.tool.bytes(), call.outcome.as_ref()?.bytes())
}

pub(crate) fn tool_line(b: &mut Builder, call: &c::Call, chat: Option<&Chat>, words: &Words, bindings: &mut Bindings) {
    let Some(write) = write(call) else { return };
    let draft = match chat {
        Some(chat) => chat.draft.as_ref(),
        None => None,
    };
    let name = document::slug(match draft {
        Some(draft) => &draft.title,
        None => b"",
    });
    let (verb, detail, line) = match write.change {
        NoteChange::First { lines } => {
            let mut line = 1;
            if let Some(draft) = draft {
                for section in document::sections(&draft.body) {
                    if section.bytes.starts_with(b"## ") {
                        line = section.line;
                        break;
                    }
                }
            }
            (Word::DraftWrote, words.say(Word::DraftLines, &[Hole::Number(u64::from(lines))]), line)
        }
        NoteChange::Later { added, removed, first } => (
            Word::DraftEditedTool,
            words.say(Word::DraftCounts, &[Hole::Number(u64::from(added)), Hole::Number(u64::from(removed))]),
            first,
        ),
    };
    b.open(Role::Slot);
    b.style(look::TURN_TOOL);
    b.open(Role::Paragraph);
    b.text(Run::Plain, &words.say(verb, &[]));
    b.close();
    b.open(Role::Paragraph);
    b.style(look::TOOL_FILE);
    b.text(Run::Plain, &words.say(Word::DraftFile, &[Hole::Name(name)]));
    b.close();
    b.open(Role::Paragraph);
    b.style(look::DETAILS);
    b.text(Run::Plain, &detail);
    b.close();
    b.open(Role::Button);
    b.name(&words.say(Word::DraftShow, &[]));
    b.bind(bindings.insert(Act::ShowSection { line }));
    b.close();
    b.close();
}

/// An empty pane keeps its path and offers the write-up only before it is asked.
pub(crate) fn empty(
    b: &mut Builder,
    words: &Words,
    name: &[u8],
    path: &[u8],
    writing: bool,
    bindings: Option<&mut Bindings>,
) {
    b.open(Role::Region);
    b.name(name);
    b.style(look::DOC);
    b.open(Role::Slot);
    b.style(look::DOC_INNER);
    b.open(Role::Paragraph);
    b.style(look::DOC_PATH);
    b.text(Run::Plain, path);
    if writing {
        if words.shell == Shell::Terminal {
            b.text(Run::Plain, &words.say(Word::ToolSeparator, &[]));
        }
        b.text(Run::Plain, &words.say(Word::DraftWriting, &[]));
    } else {
        b.text(Run::Plain, b"");
    }
    b.close();
    b.open(Role::Paragraph);
    b.style(look::DOC_EMPTY);
    b.text(Run::Plain, &words.say(Word::DraftEmpty, &[]));
    b.close();
    if let Some(bindings) = bindings {
        b.open(Role::Slot);
        b.style(look::DOC_ACTS);
        b.open(Role::Button);
        b.name(&words.say(Word::DraftWriteUp, &[]));
        b.style(look::DOC_WRITE_UP);
        b.bind(bindings.insert(Act::AddDraft));
        b.close();
        b.close();
    }
    b.close();
    b.close();
}
