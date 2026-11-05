//! The document beside a chat (temper/docs/design/client/README.md, 3.5).
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
            document::{self, Mode, NoteChange},
        },
    },
};

pub(crate) fn mode(chat: Option<&Chat>) -> skein_ui::Style {
    let mode = match chat {
        Some(chat) if chat.draft.is_some() => chat.mode,
        Some(_) | None => Mode::Chat,
    };
    match mode {
        Mode::Chat => look::MODE_CHAT,
        Mode::Split => look::MODE_SPLIT,
        Mode::Draft => look::MODE_DRAFT,
    }
}

pub(crate) fn seg(b: &mut Builder, chat: &Chat, words: &Words, bindings: &mut Bindings) {
    if chat.draft.is_none() {
        return;
    }
    b.open(Role::Slot);
    b.name(&words.say(Word::DraftGroup, &[]));
    b.style(look::SEG);
    let next = match chat.mode {
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
        if mode == chat.mode {
            b.state(State::Pressed);
        }
        b.bind(bindings.insert(Act::Mode { mode }));
        if mode == next {
            b.shortcut(Shortcut { key: b'\\', modifier: Modifier::Primary });
        }
        b.close();
    }
    b.close();
}

pub(crate) fn pane(b: &mut Builder, _root: &Domain, chat: &Chat, task: &c::Task, words: &Words, times: &mut Times) {
    let Some(draft) = &chat.draft else { return };
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
    b.text(Run::Plain, &words.say(Word::DraftPath, &[Hole::Name(document::slug(&draft.title))]));
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
        if tinted == Some(index) {
            b.style(look::DOC_CHANGED);
        }
        if index == 0 {
            opening(b, section.bytes);
        } else {
            skein_ui::markdown::read(section.bytes, 1, b);
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
    }
    b.close();
    b.close();
}

fn heading_hash(heading: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in heading {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

// The introductory metadata is a separate muted paragraph beneath the
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
    b.open(Role::Paragraph);
    b.style(look::DOC_MUTED);
    b.text(Run::Plain, rest.get(..end).expect("introductory paragraph"));
    b.close();
    skein_ui::markdown::read(rest.get(end..).expect("remaining body"), 1, b);
}
