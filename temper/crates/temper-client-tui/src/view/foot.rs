//! The composer, quiet line and lists (temper/docs/design/client/README.md, 6.14, The foot).
use super::{Binds, text};
use crate::{
    Event, Local, Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::Link;
use skein_lib::{Wall, bytes::copy_of};
use skein_ui::{Binding, Builder, Role, State, Style, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{chat, chats, needs::Count},
};
use temper_client_view::{
    shared::{self, TerminalWord},
    words::Word as Shared,
};

pub(crate) mod lists;
mod notes;
#[cfg_attr(
    not(test),
    expect(unused_imports, reason = "read by the tasks tui-launchpad and tui-chat of tui-launchpad")
)]
pub(crate) use notes::{Notes, draw_notes, read_notes};

#[derive(Debug)]
pub(crate) struct Foot {
    pub state: Box<[u8]>,
    pub style: Style,
    pub composer: Option<Composer>,
    pub running: Option<Box<[u8]>>,
    pub hint: Box<[u8]>,
    pub needs: Count,
    pub closed: Option<Box<[u8]>>,
    pub columns: u16,
    pub list: Option<Options>,
}
#[derive(Debug)]
pub(crate) struct Composer {
    pub name: Box<[u8]>,
    pub placeholder: Box<[u8]>,
    pub value: Box<[u8]>,
    pub written: u64,
    pub focus: u64,
    pub refusal: Option<Box<[u8]>>,
}
#[derive(Debug)]
pub(crate) struct Options {
    pub head: Option<Box<[u8]>>,
    pub name: Box<[u8]>,
    pub items: Box<[OptionItem]>,
    pub escape: Event,
}
#[derive(Debug)]
pub(crate) struct OptionItem {
    pub name: Box<[u8]>,
    pub words: Box<[u8]>,
    pub selected: bool,
    pub event: Event,
}
fn state(tui: &Tui, root: &Domain) -> (Shared, Style) {
    match root.client().link().state {
        Link::Live => {
            if root.pages().me(root.client()).is_some() {
                (Shared::Live, look::LIVE)
            } else {
                (Shared::Connecting, look::WAITING)
            }
        }
        Link::Starting => (Shared::Connecting, look::WAITING),
        Link::Offline => (Shared::Offline, look::DOWN),
        Link::OutOfDate => (Shared::OutOfDate, look::DOWN),
        Link::SignedOut => {
            if tui.local() == Local::Signing {
                (Shared::Connecting, look::WAITING)
            } else {
                (Shared::SignedOut, look::DOWN)
            }
        }
    }
}

pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, _wall: Wall) -> Foot {
    let (state, style) = state(tui, root);
    let chat = root.pages().chat();
    let task = match chat {
        Some(chat) => chat.task(root.client()),
        None => None,
    };
    let project = match task {
        Some(task) => Some(task.project),
        None => root.pages().project(),
    };
    let value = match chat {
        Some(chat) => copy_of(&chat.words),
        None => match root.pages().chats() {
            Some(chats) => copy_of(&chats.words),
            None => Box::default(),
        },
    };
    let paused = match (task, root.pages().me(root.client())) {
        (Some(task), Some(me)) => chat::paused(&task.chip, me),
        (Some(_), None) | (None, Some(_) | None) => false,
    };
    let ended = match task {
        Some(task) => task.result.is_some() || chats::ended(&task.chip),
        None => false,
    };
    let placeholder = if chat.is_some() {
        words.say(if ended { Word::Ended } else { Word::Reply }, &[])
    } else if project.is_some() {
        words.say(Word::AskTemper, &[])
    } else {
        shared::say(words.view(), TerminalWord::Commands)
    };
    let signing_in = root.client().link().state == Link::SignedOut && tui.local() != Local::Signing;
    let list = if signing_in { None } else { lists::read(root, tui, words, &value) };
    Foot {
        state: words.say(Word::State, &[Hole::Name(words.view().say(state, &[]))]),
        style,
        composer: if signing_in {
            None
        } else {
            Some(Composer {
                name: words.say(if chat.is_some() { Word::Message } else { Word::AskTemper }, &[]),
                placeholder,
                value,
                written: tui.box_written(),
                focus: tui.focus(),
                refusal: shared::refusal(root, words.view()),
            })
        },
        running: if chat.is_some() { shared::running(root, words.view()) } else { None },
        hint: shared::say(words.view(), if paused { TerminalWord::Paused } else { TerminalWord::Commands }),
        needs: match project {
            Some(project) => root.pages().needs().count(root.client(), root.forge().approvals(), project),
            None => Count::default(),
        },
        closed: if project.is_some() && root.pages().choice().context.is_some() {
            Some(shared::closed_line(root, task, words.view()))
        } else {
            None
        },
        columns: tui.columns(),
        list,
    }
}

fn width(bytes: &[u8]) -> u32 {
    let mut columns = 0_u32;
    for character in core::str::from_utf8(bytes).expect("view words are UTF-8").chars() {
        columns = columns.saturating_add(u32::from(skein_ui_screen::width(character)));
    }
    columns
}
fn quiet(b: &mut Builder, part: &Foot, words: &Words) {
    let separator = words.say(Word::Separator, &[]);
    let needing = if part.needs.count > 0 {
        words.say(
            if part.needs.cut { Word::NeedingCut } else { Word::Needing },
            &[Hole::Number(u64::from(part.needs.count))],
        )
    } else {
        Box::default()
    };
    let mut left_width = width(&part.hint);
    if !needing.is_empty() {
        left_width = left_width.saturating_add(width(&separator)).saturating_add(width(&needing));
    }
    let closed = match &part.closed {
        Some(closed) => {
            part.columns == 0
                || left_width
                    .saturating_add(width(closed))
                    .saturating_add(width(&separator))
                    .saturating_add(width(&part.state))
                    .saturating_add(4)
                    <= u32::from(part.columns)
        }
        None => false,
    };
    b.open(Role::ContentInfo);
    b.region(look::BOTTOM);
    b.open(Role::Paragraph);
    text(b, Role::Text, &part.hint, Some(look::HINT));
    if !needing.is_empty() {
        text(b, Role::Text, &separator, Some(look::HINT));
        text(b, Role::Text, &needing, Some(look::SIGNAL));
    }
    b.close();
    b.open(Role::Status);
    if closed {
        text(b, Role::Text, part.closed.as_deref().expect("closed line is present"), Some(look::HINT));
        text(b, Role::Text, &separator, Some(look::HINT));
    }
    text(b, Role::Text, &part.state, Some(part.style));
    b.close();
    b.close();
}

pub(crate) fn draw(b: &mut Builder, part: &Foot, words: &Words, binds: &mut Binds) -> Option<Event> {
    let Some(composer) = &part.composer else {
        b.open(Role::ContentInfo);
        b.region(look::BOTTOM);
        text(b, Role::Paragraph, &words.say(Word::QuitKeys, &[]), Some(look::HINT));
        text(b, Role::Status, &part.state, Some(part.style));
        b.close();
        return None;
    };
    if let Some(running) = &part.running {
        b.open(Role::Region);
        b.region(look::WORKING);
        b.open(Role::List);
        text(b, Role::Item, running, Some(look::RUNNING));
        b.close();
        b.close();
    }
    b.open(Role::Form);
    b.open(Role::Region);
    b.region(look::BOX);
    b.open(Role::TextArea);
    b.style(look::FIELD);
    b.name(&composer.name);
    b.state(State::SubmitsOnEnter);
    b.value(&composer.value, composer.written);
    b.focus(composer.focus);
    b.bind(Binding { owner: look::OWNER, token: super::BOX });
    if part.list.is_none() {
        b.placeholder(&composer.placeholder);
    }
    b.close();
    if let Some(refusal) = &composer.refusal {
        text(b, Role::Paragraph, refusal, Some(look::REFUSED));
    }
    b.close();
    let escape = if let Some(list) = &part.list {
        lists::draw(b, list, binds);
        Some(list.escape.clone())
    } else {
        quiet(b, part, words);
        None
    };
    b.close();
    escape
}
