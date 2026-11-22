//! The browser's approval page (client/pages/README.md, The approval page; client/web/addresses.md, 1, and client/web/sign-in.md, 1).
use crate::{Domain, Limits, Place, Request};
use alloc::boxed::Box;
use core::net::IpAddr;
use jig_client::{ApprovalCode, ApprovalKind, Loopback};
use skein_lib::{Decimal, Duration, Env, Queue, Time, Wall, Writer, base64};
/// How long an approved page waits before it reads again.
pub const AGAIN: Duration = Duration::from_secs(1);
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// No code: the field; `unread` after a Check that didn't read.
    Typing {
        unread: bool,
    },
    Reading,
    /// The engine's waiting, asked at that wall time.
    Asking {
        asked: Wall,
        kind: ApprovalKind,
        source: Option<IpAddr>,
        reader: Option<IpAddr>,
    },
    Deciding {
        approve: bool,
        asked: Wall,
        kind: ApprovalKind,
        source: Option<IpAddr>,
        reader: Option<IpAddr>,
    },
    /// Approved, read again each `AGAIN` until another answer.
    Approved,
    /// Approved, handing the sign-in to the terminal; reads again as approved.
    Handing,
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
pub enum Act {
    Words { words: Box<[u8]> },
    Check,
    Approve,
    Refuse,
    Type,
    Again,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CodeFrom {
    Address,
    Field,
}
#[derive(Debug)]
pub struct Page {
    code: Option<ApprovalCode>,
    code_from: CodeFrom,
    /// Held only until progress emits the hand-off address; Secret redacts Debug.
    loopback: Option<Loopback>,
    words: Box<[u8]>,
    over: bool,
    state: State,
    again: Option<Time>,
}
impl Page {
    #[must_use]
    pub const fn code(&self) -> Option<ApprovalCode> {
        self.code
    }
    #[must_use]
    pub fn words(&self) -> &[u8] {
        &self.words
    }
    #[must_use]
    pub fn state(&self) -> State {
        self.state.clone()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Flight {
    Idle,
    Pending,
}
pub(crate) fn changed(root: &mut Domain) {
    root.approval_revision = root.approval_revision.checked_add(1).expect("approval revision fits");
}
pub(crate) fn went(root: &mut Domain, place: &Place) {
    let page = match place {
        Place::Approve { code } => Some(Page {
            code: *code,
            code_from: CodeFrom::Address,
            loopback: None,
            words: Box::from([]),
            over: false,
            state: if code.is_some() { State::Reading } else { State::Typing { unread: false } },
            again: None,
        }),
        Place::Address(_)
        | Place::Authorize { .. }
        | Place::AuthorizeMalformed
        | Place::Root
        | Place::Missing
        | Place::Public(_) => None,
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
            } else {
                return;
            }
        }
        Act::Check => {
            if let State::Typing { .. } = page.state {
                let code = if page.over { None } else { ApprovalCode::parse(&page.words) };
                page.code = code;
                page.code_from = CodeFrom::Field;
                page.state = if code.is_some() { State::Reading } else { State::Typing { unread: true } };
            } else {
                return;
            }
        }
        Act::Approve | Act::Refuse => {
            if let State::Asking { asked, kind, source, reader } = page.state.clone() {
                page.state = State::Deciding { approve: act == Act::Approve, asked, kind, source, reader };
            } else {
                return;
            }
        }
        Act::Type => {
            if page.state != State::Unknown {
                return;
            }
            page.code = None;
            page.words = Box::from([]);
            page.over = false;
            page.state = State::Typing { unread: false };
        }
        Act::Again => {
            if page.state != State::Busy {
                return;
            }
            page.state = if page.code.is_some() { State::Reading } else { State::Typing { unread: false } };
        }
    }
    changed(root);
}
pub(crate) fn progress(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(page) = &mut root.approval else { return };
    match root.client.link().state {
        jig_client::Link::SignedOut => {
            match page.state {
                State::SignedIn | State::Refused | State::Expired | State::Used | State::Unknown | State::SignedOut => {
                    return;
                }
                State::Typing { .. }
                | State::Reading
                | State::Asking { .. }
                | State::Deciding { .. }
                | State::Approved
                | State::Handing
                | State::Busy => {
                    page.state = State::SignedOut;
                    page.again = None;
                    page.loopback = None;
                    root.approving = Flight::Idle;
                    changed(root);
                }
            }
            return;
        }
        jig_client::Link::Live => {}
        jig_client::Link::Starting | jig_client::Link::Offline | jig_client::Link::OutOfDate => return,
    }
    if let Some(loopback) = page.loopback.take() {
        let to = hand_off(loopback);
        out.push(Request::Leave { to, replace: true });
    }
    if root.approving == Flight::Pending {
        return;
    }
    if root.client.admission().attempts >= env.limits.client.exchanges {
        return;
    }
    let Some(code) = page.code else { return };
    let event = match page.state {
        State::Reading => jig_client::Event::ReadApproval { code },
        State::Approved | State::Handing => {
            let Some(again) = page.again else { return };
            if env.now < again {
                return;
            }
            page.again = None;
            jig_client::Event::ReadApproval { code }
        }
        State::Deciding { approve, .. } => jig_client::Event::DecideApproval { code, approve },
        State::Typing { .. }
        | State::Asking { .. }
        | State::SignedIn
        | State::Refused
        | State::Expired
        | State::Used
        | State::Unknown
        | State::SignedOut
        | State::Busy => return,
    };
    root.approving = Flight::Pending;
    changed(root);
    crate::route::client(root, env, event, out);
}
pub(crate) fn answered(root: &mut Domain, env: &Env<Limits>, answer: jig_client::Approval) {
    let Some(page) = &mut root.approval else { return };
    let state = match answer {
        jig_client::Approval::Read { code, state } => {
            if page.code != Some(code)
                || (page.state != State::Reading && page.state != State::Approved && page.state != State::Handing)
            {
                return;
            }
            read_state(state, page.state.clone(), page.code_from)
        }
        jig_client::Approval::Decided { code, decided } => {
            if page.code != Some(code) {
                return;
            }
            if let State::Deciding { .. } = page.state {
                decided_state(decided, page)
            } else {
                return;
            }
        }
        jig_client::Approval::Asked { .. } | jig_client::Approval::NotAsked(_) => return,
    };
    if state == (State::Typing { unread: false }) {
        page.code = None;
        page.words = Box::from([]);
        page.over = false;
    }
    root.approving = Flight::Idle;
    page.again =
        if state == State::Approved || state == State::Handing { Some(env.now.saturating_add(AGAIN)) } else { None };
    page.state = state;
    changed(root);
}
fn read_state(state: jig_client::ApprovalState, from: State, code_from: CodeFrom) -> State {
    use jig_client::ApprovalState;
    match state {
        ApprovalState::Waiting { asked, kind, source, reader } => {
            if kind == ApprovalKind::Typed && code_from == CodeFrom::Address {
                State::Typing { unread: false }
            } else {
                State::Asking { asked, kind, source, reader }
            }
        }
        ApprovalState::Approved => {
            if from == State::Handing {
                State::Handing
            } else {
                State::Approved
            }
        }
        ApprovalState::Taken => State::SignedIn,
        ApprovalState::Refused => State::Refused,
        ApprovalState::Expired => State::Expired,
        ApprovalState::Unknown => State::Unknown,
        ApprovalState::NotABrowser | ApprovalState::SignedOut => State::SignedOut,
        ApprovalState::NotReady | ApprovalState::Busy | ApprovalState::Lost => {
            if from == State::Approved || from == State::Handing { from } else { State::Busy }
        }
    }
}
fn decided_state(decided: jig_client::Decided, page: &mut Page) -> State {
    use jig_client::Decided;
    match decided {
        Decided::Approved { loopback } => {
            let state = if loopback.is_some() { State::Handing } else { State::Approved };
            page.loopback = loopback;
            state
        }
        Decided::Refused => State::Refused,
        Decided::Used => State::Used,
        Decided::Expired => State::Expired,
        Decided::Unknown => State::Unknown,
        Decided::NotABrowser | Decided::SignedOut => State::SignedOut,
        Decided::NotReady | Decided::Busy | Decided::Lost => State::Busy,
    }
}
fn hand_off(loopback: Loopback) -> Box<[u8]> {
    let base = b"http://127.0.0.1:";
    let path = b"/approved?grant=";
    let port = Decimal::of(u64::from(loopback.port));
    assert!(loopback.grant.len() == 32, "the link's loopback grant has 32 bytes");
    let size = base.len().checked_add(port.as_bytes().len()).expect("bounded loopback origin");
    let size = size.checked_add(path.len()).expect("bounded hand-off path");
    let size = size.checked_add(43).expect("bounded hand-off grant");
    let mut out = Writer::new(size);
    out.put(base).expect("measured loopback host");
    out.put(port.as_bytes()).expect("measured loopback port");
    out.put(path).expect("measured loopback path");
    base64::encode(base64::Alphabet::Url, loopback.grant.expose(), &mut out).expect("measured grant");
    out.finish()
}
pub(crate) fn next_deadline(root: &Domain) -> Option<Time> {
    match &root.approval {
        Some(page) => page.again,
        None => None,
    }
}
