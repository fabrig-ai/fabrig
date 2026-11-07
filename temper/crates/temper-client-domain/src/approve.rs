//! The browser's approval page (client/README.md, 3.5, The approval page; client/browser.md, 2.3 and 2.4).
use crate::{Domain, Limits, Place, Request};
use alloc::boxed::Box;
use jig_client::ApprovalCode;
use skein_lib::{Duration, Env, Queue, Time, Wall};
/// How long an approved page waits before it reads again.
pub const AGAIN: Duration = Duration::from_secs(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// No code: the field; `unread` after a Check that didn't read.
    Typing { unread: bool },
    Reading,
    /// The engine's waiting, asked at that wall time.
    Asking { asked: Wall },
    Deciding { approve: bool, asked: Wall },
    /// Approved, read again each `AGAIN` until another answer.
    Approved,
    SignedIn,
    Refused,
    Expired,
    Used,
    Unknown,
    SignedOut,
    /// Busy, not ready or lost: Try again reads again.
    Busy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act { Words { words: Box<[u8]> }, Check, Approve, Refuse, Type, Again }
#[derive(Debug)]
pub struct Page {
    code: Option<ApprovalCode>,
    words: Box<[u8]>,
    over: bool,
    state: State,
    again: Option<Time>,
}
impl Page {
    #[must_use]
    pub const fn code(&self) -> Option<ApprovalCode> { self.code }
    #[must_use]
    pub fn words(&self) -> &[u8] { &self.words }
    #[must_use]
    pub const fn state(&self) -> State { self.state }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Flight { Idle, Pending }
pub(crate) fn changed(root: &mut Domain) {
    root.approval_revision = root.approval_revision.checked_add(1).expect("approval revision fits");
}
pub(crate) fn went(root: &mut Domain, place: &Place) {
    let page = match place {
        Place::Approve { code } => Some(Page {
            code: *code,
            words: Box::from([]),
            over: false,
            state: if code.is_some() { State::Reading } else { State::Typing { unread: false } },
            again: None,
        }),
        Place::Address(_) | Place::Authorize { .. } | Place::AuthorizeMalformed | Place::Missing | Place::Public(_) => None,
    };
    if page.is_some() || root.approval.is_some() {
        root.approval = page;
        root.approving = Flight::Idle;
        changed(root);
    }
}
pub(crate) fn act(root: &mut Domain, act: Act) {
    let Some(page) = &mut root.approval else { return };
    match act {
        Act::Words { words } => {
            if let State::Typing { .. } = page.state {
                page.over = words.len() > jig_client_protocol_wire::CODE_TYPED_BYTES;
                let length = words.len().min(jig_client_protocol_wire::CODE_TYPED_BYTES);
                page.words = skein_lib::bytes::copy_of(words.get(..length).expect("bounded typed code"));
                page.state = State::Typing { unread: false };
            } else { return; }
        }
        Act::Check => {
            if let State::Typing { .. } = page.state {
                let code = if page.over { None } else { ApprovalCode::parse(&page.words) };
                page.code = code;
                page.state = if code.is_some() { State::Reading } else { State::Typing { unread: true } };
            } else { return; }
        }
        Act::Approve | Act::Refuse => {
            if let State::Asking { asked } = page.state {
                page.state = State::Deciding { approve: act == Act::Approve, asked };
            } else { return; }
        }
        Act::Type => {
            if page.state != State::Unknown { return; }
            page.code = None;
            page.words = Box::from([]);
            page.over = false;
            page.state = State::Typing { unread: false };
        }
        Act::Again => {
            if page.state != State::Busy { return; }
            page.state = if page.code.is_some() { State::Reading } else { State::Typing { unread: false } };
        }
    }
    changed(root);
}
pub(crate) fn progress(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(page) = &mut root.approval else { return };
    if root.approving == Flight::Pending { return; }
    match root.client.link().state {
        jig_client::Link::SignedOut => {
            match page.state {
                State::SignedIn | State::Refused | State::Expired | State::Used | State::Unknown | State::SignedOut => return,
                State::Typing { .. } | State::Reading | State::Asking { .. } | State::Deciding { .. } | State::Approved | State::Busy => {
                    page.state = State::SignedOut;
                    page.again = None;
                    changed(root);
                }
            }
            return;
        }
        jig_client::Link::Live => {}
        jig_client::Link::Starting | jig_client::Link::Offline | jig_client::Link::OutOfDate => return,
    }
    if root.client.admission().attempts >= env.limits.client.exchanges { return; }
    let Some(code) = page.code else { return };
    let event = match page.state {
        State::Reading => jig_client::Event::ReadApproval { code },
        State::Approved => {
            let Some(again) = page.again else { return };
            if env.now < again { return; }
            page.again = None;
            jig_client::Event::ReadApproval { code }
        }
        State::Deciding { approve, .. } => jig_client::Event::DecideApproval { code, approve },
        State::Typing { .. } | State::Asking { .. } | State::SignedIn | State::Refused | State::Expired | State::Used | State::Unknown | State::SignedOut | State::Busy => return,
    };
    root.approving = Flight::Pending;
    changed(root);
    crate::route::client(root, env, event, out);
}
pub(crate) fn answered(root: &mut Domain, env: &Env<Limits>, answer: jig_client::Approval) {
    let Some(page) = &mut root.approval else { return };
    let (code, state) = match answer {
        jig_client::Approval::Read { code, state } => {
            if page.state != State::Reading && page.state != State::Approved { return; }
            (code, read_state(state, page.state))
        }
        jig_client::Approval::Decided { code, decided } => {
            if let State::Deciding { .. } = page.state {
                (code, decided_state(decided))
            } else { return; }
        }
        jig_client::Approval::Asked { .. } | jig_client::Approval::NotAsked(_) => return,
    };
    if page.code != Some(code) { return; }
    root.approving = Flight::Idle;
    page.state = state;
    page.again = if state == State::Approved { Some(env.now.saturating_add(AGAIN)) } else { None };
    changed(root);
}
fn read_state(state: jig_client::ApprovalState, from: State) -> State {
    use jig_client::ApprovalState;
    match state {
        ApprovalState::Waiting { asked } => State::Asking { asked },
        ApprovalState::Approved => State::Approved,
        ApprovalState::Taken => State::SignedIn,
        ApprovalState::Refused => State::Refused,
        ApprovalState::Expired => State::Expired,
        ApprovalState::Unknown => State::Unknown,
        ApprovalState::NotABrowser | ApprovalState::SignedOut => State::SignedOut,
        ApprovalState::NotReady | ApprovalState::Busy | ApprovalState::Lost => {
            if from == State::Approved { State::Approved } else { State::Busy }
        }
    }
}
fn decided_state(decided: jig_client::Decided) -> State {
    use jig_client::Decided;
    match decided {
        Decided::Approved => State::Approved,
        Decided::Refused => State::Refused,
        Decided::Used => State::Used,
        Decided::Expired => State::Expired,
        Decided::Unknown => State::Unknown,
        Decided::NotABrowser | Decided::SignedOut => State::SignedOut,
        Decided::NotReady | Decided::Busy | Decided::Lost => State::Busy,
    }
}
pub(crate) fn next_deadline(root: &Domain) -> Option<Time> {
    match &root.approval { Some(page) => page.again, None => None }
}
