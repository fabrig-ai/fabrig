//! Tell temper's single filing (temper/docs/design/client/pages/needs-you.md, 1).
use super::{Pages, Request};
use crate::{Limits, forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Key, Told};
use skein_lib::{Env, Queue};
use temper_client_protocol_wire::v1 as w;

#[derive(Debug, PartialEq, Eq)]
pub struct Tell {
    pub serial: u64,
    pub project: u32,
    pub words: Box<[u8]>,
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
pub(super) fn act(pages: &mut Pages, env: &Env<Limits>, project: u32, words: &[u8], out: &mut Queue<Request>) {
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
    let over = words.len().saturating_sub(4096);
    if over > 0 {
        pages.tell.last = Some(Tell {
            serial,
            project,
            words: words.as_bytes().into(),
            state: State::Long { over: u32::try_from(over).expect("input words fit u32") },
        });
        return;
    }
    let Some(bounds) = forge::wire_limits(&env.limits.forge) else { return };
    let first = words.lines().next().expect("nonempty words have a line");
    let title = skein_lib::bytes::utf8_prefix(first.as_bytes(), 200).into();
    let Ok(filing) = w::FileFinding::new(&bounds, w::FileFindingParts {
        project,
        kind: w::FindingKind::Feedback,
        severity: w::Severity::Medium,
        area: None,
        title,
        body: words.as_bytes().into(),
    }) else { return };
    let Some(request) = forge::findings::request(project, w::Request::FileFinding(filing), env.limits.forge) else { return };
    pages.serial = pages.serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: pages.serial };
    pages.tell.pending = Some(key);
    pages.tell.last = Some(Tell { serial, project, words: words.as_bytes().into(), state: State::Sending });
    out.push(Request::Client(jig_client::Event::Send { key, name: None, request }));
}
