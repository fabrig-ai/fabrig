//! Tell temper's single filing (temper/docs/design/client/pages/needs-you.md, 1).
mod context;
pub(super) use context::{remember, outcome, showing};
use super::{Pages, Request, team};
use crate::{Address, Limits, address, forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Key, Told};
use skein_lib::{Env, Queue};
use temper_client_protocol_wire::v1 as w;

/// The page and destination of a word (client/pages/needs-you.md, 1, Tell temper).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct About {
    pub address: Address,
    pub title: Option<Box<[u8]>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Goes {
    Area { name: Box<[u8]>, holder: team::Holder },
    Triager { holder: team::Holder, areas: bool },
    Owners { areas: bool },
}

#[must_use]
pub fn route(document: &w::Findings, about: Option<&About>) -> Goes {
    let task = match about { Some(about) => about_task(about), None => None };
    if let Some(task) = task {
        for area in document.areas() {
            let held = match area.holder() {
                Some(w::Holder::Task(holder)) => holder.number() == task,
                Some(w::Holder::Party(_) | w::Holder::Role(_)) | None => false,
            };
            if held {
                return Goes::Area { name: area.name().into(), holder: team::Holder::Task(task) };
            }
        }
    }
    let areas = !document.areas().is_empty();
    let holder = team::area_holder(document.holder());
    let stands = match document.triager() {
        Some(w::AreaOwner::Task(owner)) => holder == team::Holder::Task(owner.number()),
        Some(w::AreaOwner::Party(owner)) => holder == team::Holder::Party(owner.number()),
        Some(w::AreaOwner::Role(owner)) => holder == team::Holder::Role(owner.number()),
        Some(w::AreaOwner::Charter(_)) => match holder {
            team::Holder::Task(_) => true,
            team::Holder::Party(_) | team::Holder::Role(_) => false,
        },
        None => false,
    };
    if stands { Goes::Triager { holder, areas } } else { Goes::Owners { areas } }
}
fn about_task(about: &About) -> Option<u64> {
    match &about.address {
        Address::Task { task } | Address::Talk { task, .. } | Address::Chat { task }
        | Address::Team { role: Some(task), .. } => Some(*task),
        Address::Inbox { .. } | Address::Chats { .. } | Address::NewChat { .. }
        | Address::Accounts | Address::Forges | Address::ConnectForge { .. }
        | Address::ConnectModel { .. } | Address::Welcome { .. } | Address::Home { .. }
        | Address::AllHome | Address::Goals { .. } | Address::Changes { .. }
        | Address::Settings { .. } | Address::Repositories { .. } | Address::Models { .. }
        | Address::Builders { .. } | Address::Role { .. } | Address::Rule { .. }
        | Address::Team { role: None, .. } | Address::Ask { .. } | Address::Tell { .. } | Address::Adopt { .. }
        | Address::Work { .. } | Address::Members | Address::Remembered | Address::Told
        | Address::Limits => None,
    }
}
#[must_use]
pub fn goes(root: &crate::Domain, project: u32, about: Option<&About>) -> Option<Goes> {
    destination(&root.pages, project, about)
}
fn destination(pages: &Pages, project: u32, about: Option<&About>) -> Option<Goes> {
    Some(route(pages.needs.findings.document(project)?, about))
}
fn body(words: &[u8], about: Option<&About>) -> Box<[u8]> {
    let Some(about) = about else { return words.into() };
    let path = address::print(&about.address);
    let title = match &about.title {
        Some(title) => skein_lib::bytes::concat(&[b" (", title, b")"]),
        None => Box::default(),
    };
    let body = skein_lib::bytes::concat(&[words, b"\n\nPage: ", &path, &title]);
    if body.len() <= 4096 { body } else { words.into() }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tell {
    pub serial: u64,
    pub project: u32,
    pub words: Box<[u8]>,
    pub goes: Option<Goes>,
    pub state: State,
}
#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Sending,
    Filed { number: u32 },
    Refused(w::FindingRefusal),
    Lost,
    Long { over: u32 },
}
#[derive(Debug)]
pub(super) struct Store {
    last: Option<Tell>,
    pending: Option<Key>,
}
impl Store {
    pub(super) const fn new() -> Self {
        Self { last: None, pending: None }
    }
    pub(super) const fn last(&self) -> Option<&Tell> {
        self.last.as_ref()
    }
    pub(super) fn told(&mut self, told: &Told, client: &Client, limits: forge::Limits) -> bool {
        let Some(pending) = self.pending else { return false };
        let outcome = match told {
            Told::Answered { key, outcome } if *key == pending => outcome,
            Told::Pending { key } if *key == pending => {
                if let Some(request) = client.pending(*key)
                    && request.progress == jig_client::Progress::Unknown
                {
                    &jig_client::Outcome::Unknown
                } else {
                    return true;
                }
            }
            Told::Answered { .. }
            | Told::Pending { .. }
            | Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Page { .. }
            | Told::Text { .. } => return false,
        };
        let outcome = forge::findings::outcome(outcome, limits);
        let state = if let Some(w::Outcome::Filed(filed)) = &outcome {
            State::Filed { number: filed.number() }
        } else if let Some(w::Outcome::FindingRefused(refused)) = &outcome {
            State::Refused(refused.reason().clone())
        } else {
            State::Lost
        };
        self.pending = None;
        self.last.as_mut().expect("pending Tell has its words").state = state;
        true
    }
}

#[expect(clippy::disallowed_methods, reason = "Tell trims Unicode whitespace at the person's input boundary")]
pub(super) fn act(pages: &mut Pages, env: &Env<Limits>, project: u32, words: &[u8], about: Option<About>, out: &mut Queue<Request>) {
    if pages.tell.pending.is_some() {
        return;
    }
    let Ok(words) = core::str::from_utf8(words) else { return };
    let words = words.trim();
    if words.is_empty() {
        return;
    }
    let serial = match &pages.tell.last {
        Some(last) => last.serial.checked_add(1).expect("Tell serials fit"),
        None => 1,
    };
    let goes = destination(pages, project, about.as_ref());
    let over = words.len().saturating_sub(4096);
    if over > 0 {
        pages.tell.last = Some(Tell {
            serial,
            project,
            words: words.as_bytes().into(),
            goes,
            state: State::Long { over: u32::try_from(over).expect("input words fit u32") },
        });
        return;
    }
    let Some(bounds) = forge::wire_limits(&env.limits.forge) else { return };
    let first = words.lines().next().expect("nonempty words have a line");
    let title = skein_lib::bytes::utf8_prefix(first.as_bytes(), 200).into();
    let area = match &goes {
        Some(Goes::Area { name, .. }) => Some(name.clone()),
        Some(Goes::Triager { .. } | Goes::Owners { .. }) | None => None,
    };
    let Ok(filing) = w::FileFinding::new(
        &bounds,
        w::FileFindingParts {
            project,
            kind: w::FindingKind::Feedback,
            severity: w::Severity::Medium,
            area,
            title,
            body: body(words.as_bytes(), about.as_ref()),
        },
    ) else {
        return;
    };
    let Some(request) = forge::findings::request(project, w::Request::FileFinding(filing), env.limits.forge) else {
        return;
    };
    pages.serial = pages.serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: pages.serial };
    pages.tell.pending = Some(key);
    pages.tell.last = Some(Tell { serial, project, words: words.as_bytes().into(), goes, state: State::Sending });
    out.push(Request::Client(jig_client::Event::Send { key, name: None, request }));
}
