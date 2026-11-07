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
pub struct Page { code: Option<ApprovalCode>, words: Box<[u8]>, state: State }
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
pub(crate) fn went(_root: &mut Domain, _place: &Place) {}
pub(crate) fn act(_root: &mut Domain, _act: Act) {}
pub(crate) fn progress(_root: &mut Domain, _env: &Env<Limits>, _out: &mut Queue<Request>) {}
pub(crate) fn answered(_root: &mut Domain, _env: &Env<Limits>, _answer: jig_client::Approval) {}
pub(crate) fn next_deadline(_root: &Domain) -> Option<Time> { None }
