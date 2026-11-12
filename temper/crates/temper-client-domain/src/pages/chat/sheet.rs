//! A chat opens the shared goal sheet with its document (client/pages/README.md, client/pages/goal.md, 3).
use crate::{
    Domain, Limits,
    pages::{self, Act, Event, Request, home},
};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Writer};
use temper_client_protocol_wire::{GOAL_AGREED, GOAL_SCOPE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WayKind {
    LandWhole,
    TaskByTask,
    Agreed,
}

/// The snapshot's instructions, before its draft and chat provenance.
#[must_use]
pub fn words(name: &[u8], slug: &[u8], body: &[u8], kind: WayKind) -> Box<[u8]> {
    let mut size = name.len();
    for part in [
        b"\n\nImplement this draft. First commit it, as it stands, at docs/design/draft/".as_slice(),
        slug,
        b".md.\n\n",
        body,
        b"\n\n",
        GOAL_SCOPE,
    ] {
        size = size.checked_add(part.len()).expect("draft words fit");
    }
    if kind == WayKind::Agreed {
        size = size.checked_add(1).expect("separator fits").checked_add(GOAL_AGREED.len()).expect("agreed words fit");
    }
    let mut words = Writer::new(size);
    for part in [
        name,
        b"\n\nImplement this draft. First commit it, as it stands, at docs/design/draft/",
        slug,
        b".md.\n\n",
        body,
        b"\n\n",
        GOAL_SCOPE,
    ] {
        words.put(part).expect("measured draft words");
    }
    if kind == WayKind::Agreed {
        words.put(b"\n").expect("measured separator");
        words.put(GOAL_AGREED).expect("measured agreement");
    }
    words.finish()
}

#[must_use]
pub fn path(slug: &[u8]) -> Box<[u8]> {
    let size = b"docs/design/draft/"
        .len()
        .checked_add(slug.len())
        .expect("draft prefix fits")
        .checked_add(3)
        .expect("draft path fits");
    let mut path = Writer::new(size);
    for part in [b"docs/design/draft/".as_slice(), slug, b".md"] {
        path.put(part).expect("measured draft path");
    }
    path.finish()
}

pub(crate) fn snapshot(name: &[u8], path: &[u8], body: &[u8], chat: Option<u64>) -> Box<[u8]> {
    let slug = super::super::goal::file_of(path);
    let slug = slug.strip_suffix(b".md").unwrap_or(slug);
    let words = words(name, slug, body, WayKind::TaskByTask);
    let mut size = words
        .len()
        .checked_add(b"\nDraft: ".len())
        .expect("draft separator fits")
        .checked_add(path.len())
        .expect("draft provenance fits");
    let chat = match chat {
        Some(number) => Some(skein_lib::Decimal::of(number)),
        None => None,
    };
    if let Some(chat) = &chat {
        size = size
            .checked_add(b"\nChat: T".len())
            .expect("chat separator fits")
            .checked_add(chat.as_bytes().len())
            .expect("chat provenance fits");
    }
    let mut writer = Writer::new(size);
    for part in [words.as_ref(), b"\nDraft: ", path] {
        writer.put(part).expect("measured provenance");
    }
    if let Some(chat) = chat {
        writer.put(b"\nChat: T").expect("measured chat");
        writer.put(chat.as_bytes()).expect("measured chat number");
    }
    writer.finish()
}

/// Supply the shared sheet's project facts at its page boundary.
pub(crate) fn step(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    if let Event::Act { act: Act::Needs(act) } = event {
        pages::needs::act(&mut root.pages, env, &root.client, root.forge.approvals(), act, out);
        root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown count fits");
        return;
    }
    if let Event::Act { act: Act::AddDraft } = &event
        && let Some(chat) = root.pages.chat()
        && let Some(task) = chat.task(root.client())
        && !pages::chat_offers_draft(root, task)
    {
        return;
    }
    let mut event = event;
    let mut landed_run = false;
    let landed_opening = if let Event::Act { act: Act::OpenLanded { start, .. } } = &mut event {
        landed_run = *start;
        *start = false;
        root.pages.start().is_none()
    } else {
        false
    };
    let opening = if let Event::Act { act: Act::OpenStart { chat: Some(number), .. } } = &event {
        if let Some(chat) = root.pages.chat() {
            let Some(task) = chat.task(root.client()) else { return };
            let Some(draft) = &chat.draft else { return };
            if *number != chat.task || draft.body.is_empty() || chat.goal_from(root).is_some() {
                return;
            }
            let home::Repository::In { .. } = home::repository(root, task.project) else { return };
        }
        root.pages.start().is_none()
    } else {
        false
    };
    pages::step(&mut root.pages, env, &root.client, event, out);
    let me = root.pages.me(&root.client);
    if (opening || landed_opening)
        && let Some(start) = &mut root.pages.start
        && let Some(me) = me
    {
        let configured = match root.forge.ways(start.project) {
            Some(ways) => Some(ways.way),
            None => None,
        };
        start.way = pages::start_goal::preferred_way(me, start.project, configured);
        start.default = start.way;
        if landed_opening && landed_run {
            start.send(env, Some(me), &mut root.pages.serial, out);
        }
    }
    pages::chats::refresh(root, out);
    if let Some(chat) = &mut root.pages.chat {
        chat.goal_watch(&root.client, &root.forge, out);
    }
}
