//! Command lists (temper/docs/design/client/pages/README.md, A command's list; 6.14, The commands).
use crate::words::Words;
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
use temper_client_domain::{
    Address, Domain,
    pages::{Act, chats as chat_pages, terminal::Listing},
};
/// An option in the terminal's list (temper/docs/design/client/tui/pages/README.md, 2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Choice {
    pub name: Box<[u8]>,
    pub words: Box<[u8]>,
    pub selected: bool,
    pub act: Act,
}
/// A command's head (temper/docs/design/client/pages/README.md, A command's list).
#[must_use]
pub fn list_head(listing: Listing) -> &'static [u8] {
    match listing {
        Listing::Model => b"Model. A change applies from the next turn.",
        Listing::Effort => b"Effort. A change applies from the next turn.",
        Listing::Chats => b"Chats",
    }
}
/// The picker's models, without provider links (temper/docs/design/client/pages/README.md, A command's list).
#[must_use]
pub fn models(root: &Domain, _task: Option<&c::Task>, w: &Words) -> Box<[Choice]> {
    crate::choice::picker::model_choices(root, w)
}
/// The chosen model's efforts (temper/docs/design/client/pages/README.md, A command's list).
#[must_use]
pub fn efforts(root: &Domain, _task: Option<&c::Task>, _w: &Words) -> Box<[Choice]> {
    crate::choice::picker::effort_choices(root)
}
/// Live chats, newest first, without the open one (temper/docs/design/client/tui/pages/README.md, 2).
#[must_use]
pub fn chats(root: &Domain, w: &Words) -> Box<[Choice]> {
    let Some(chats) = root.pages().chats() else { return Box::default() };
    let Some(live) = chats.live(root.client()) else { return Box::default() };
    let current = match root.pages().chat() {
        Some(chat) => Some(chat.task),
        None => None,
    };
    let mut choices = List::with_capacity(live.rows.len());
    let mut previous = None;
    for _ in &live.rows {
        let mut next: Option<&c::TaskLine> = None;
        for row in &live.rows {
            let key = (row.last_activity, row.chip.task);
            if Some(row.chip.task) == current || chat_pages::ended(&row.chip) {
                continue;
            }
            if let Some(previous) = previous
                && key >= previous
            {
                continue;
            }
            if let Some(chosen) = next
                && key <= (chosen.last_activity, chosen.chip.task)
            {
                continue;
            }
            next = Some(row);
        }
        let Some(row) = next else { break };
        choices
            .push(Choice {
                name: w.say(crate::words::Word::Chat, &[skein_ui::phrases::Hole::Number(row.chip.task)]),
                words: row.chip.title.bytes().into(),
                selected: false,
                act: Act::Go { address: Address::Chat { task: row.chip.task } },
            })
            .expect("choices fit live rows");
        previous = Some((row.last_activity, row.chip.task));
    }
    choices.into_boxed()
}
/// A confirmation's question and acts (temper/docs/design/client/pages/README.md, A command's list).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Confirmation {
    pub head: &'static [u8],
    pub take: &'static [u8],
    pub keep: &'static [u8],
}
/// An unsent close or restart (temper/docs/design/client/tui/pages/README.md, 2).
#[must_use]
pub fn confirmation(root: &Domain) -> Option<Confirmation> {
    if !crate::common::terminal::commands::confirms(root) || root.pages().confirmation()?.sent.is_some() {
        return None;
    }
    Some(if root.pages().chat()?.restarting {
        Confirmation {
            head: b"Start over? A new Ask temper chat starts. This one stays in Chats.",
            take: b"Start over",
            keep: b"Keep this one",
        }
    } else {
        Confirmation { head: b"Close this chat? It stays in Chats.", take: b"Close", keep: b"Keep it open" }
    })
}
