//! Retry rule for conversation activity (temper/docs/design/client/README.md, 3.5).
use super::{Request, task::ReadEnd};
use jig_client::{Client, Handle, Readable, Readout, contract as c};
use skein_lib::{Duration, Queue, Time};

pub(super) const READ_FIRST: Duration = Duration::from_secs(1);
pub(super) const READ_MOST: Duration = Duration::from_secs(30);

/// A conversation's activity read, asked again after a transient end
/// while its page is open (temper/docs/design/client/README.md, 3.5).
#[derive(Debug)]
pub(crate) struct Asking {
    asked: Option<c::Read>,
    wait: Duration,
    retry_at: Option<Time>,
}
impl Asking {
    pub(crate) const fn new() -> Self {
        Self { asked: None, wait: READ_FIRST, retry_at: None }
    }
    /// Sends `read` on `handle` and remembers it as the one to ask again.
    pub(crate) fn ask(&mut self, handle: Handle, read: c::Read, out: &mut Queue<Request>) {
        self.asked = Some(read.clone());
        self.retry_at = None;
        out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }));
    }
    /// A page arrived: no retry, the wait back to its first.
    pub(crate) fn answered(&mut self) {
        self.asked = None;
        self.retry_at = None;
        self.wait = READ_FIRST;
    }
    /// A transient end: the same read at `now + wait`, the wait doubling to its most.
    pub(crate) fn ended(&mut self, now: Time) {
        if self.asked.is_some() {
            self.retry_at = Some(now.saturating_add(self.wait));
            self.wait = self.wait.saturating_mul(2).min(READ_MOST);
        }
    }
    /// When due, asks the remembered read again.
    pub(crate) fn fire(&mut self, now: Time, handle: Handle, out: &mut Queue<Request>) {
        if let Some(at) = self.retry_at
            && at <= now
            && let Some(read) = self.asked.take()
        {
            self.ask(handle, read, out);
        }
    }
    pub(crate) const fn next_deadline(&self) -> Option<Time> {
        self.retry_at
    }
}

pub(super) const fn earliest(left: Option<Time>, right: Option<Time>) -> Option<Time> {
    match (left, right) {
        (Some(left), Some(right)) => Some(if left.as_nanos() <= right.as_nanos() { left } else { right }),
        (Some(at), None) | (None, Some(at)) => Some(at),
        (None, None) => None,
    }
}

pub(super) fn read_end(client: &Client, page: &Readout) -> ReadEnd {
    match page {
        Readout::Limit => ReadEnd::Limit,
        Readout::NotServed { .. } => ReadEnd::NotServed,
        Readout::Jig(c::ReadResult::None(why)) => match why {
            c::ReadNone::Busy => ReadEnd::Busy,
            c::ReadNone::Unreachable => ReadEnd::Unreachable,
            c::ReadNone::SignedOut => signed_out(client),
            c::ReadNone::Unknown => ReadEnd::Unknown,
            c::ReadNone::Refused => ReadEnd::NotAllowed,
            c::ReadNone::Gone => ReadEnd::Gone,
            c::ReadNone::TooLinked { joins } => ReadEnd::TooLinked { joins: *joins },
            c::ReadNone::TooLarge { .. } => ReadEnd::TooLarge,
            c::ReadNone::Malformed => ReadEnd::Malformed,
        },
        Readout::Jig(c::ReadResult::Value(_)) | Readout::Family { .. } => ReadEnd::Malformed,
    }
}
pub(super) fn signed_out(client: &Client) -> ReadEnd {
    match client.link().state {
        jig_client::Link::SignedOut => ReadEnd::SignedOut,
        jig_client::Link::Starting | jig_client::Link::Live | jig_client::Link::Offline => ReadEnd::SignIn,
        jig_client::Link::OutOfDate => ReadEnd::NotServed,
    }
}
