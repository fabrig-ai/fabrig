//! Routing (protocol/channel.md, sections 2, 3 and 8); Debug (jig/docs/design/protocol/service.md, 11.3).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_link::SourceName;
use jig_protocol_worker_wire::Part;
use skein_channel::{Lower, LowerEvent};
use skein_io::digest::Digest;
use skein_lib::{Secret, Token};
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Worker {
    pub name: Box<[u8]>,
    pub digest: Digest,
}
/// A grant's value: a credential with its scheme, an application's
/// secret, or a program source's name.
#[derive(Clone, Debug)]
pub enum GrantValue {
    Credential { scheme: Scheme, token: Secret },
    Application { secret: Secret },
    Program { name: SourceName },
}
/// How a credential is presented (protocol/channel.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme {
    Bearer,
    Key,
}
/// `TurnBusy` is the fleet's busy answer, handled below the hub (protocol/channel.md, 5.2; domain/hosts.md, 6.1).
pub enum Event {
    Accepted { connection: Token },
    Lower { connection: Token, event: LowerEvent },
    Gone { connection: Token },
    Down { channel: Token, down: host::LinkDown, parts: Box<[Part]>, values: Box<[Option<GrantValue>]> },
    TurnBusy { channel: Token, run: Token, attempt: Token, turn: u32 },
    NotAdmitted { channel: Token, text: Box<[u8]> },
    Stop,
    Close,
    Abort,
}
pub enum Request {
    Take { connection: Token },
    TurnAway { connection: Token },
    Lower { connection: Token, lower: Lower },
    Close { connection: Token },
    Abort { connection: Token },
    Hello { channel: Token, worker: u32, hello: jig_domain_link::Hello },
    Up { channel: Token, up: host::LinkUp, parts: Box<[Part]> },
    Lost { channel: Token },
    Closed,
}
impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Accepted { connection } => f.debug_tuple("Accepted").field(connection).finish(),
            Self::Gone { connection } => f.debug_tuple("Gone").field(connection).finish(),
            Self::Lower { connection, event } => match event {
                LowerEvent::Read(_) => f.debug_tuple("Lower::Read").field(connection).finish(),
                LowerEvent::Write(_) => f.debug_tuple("Lower::Write").field(connection).finish(),
                LowerEvent::WriteFailed(_) => f.debug_tuple("Lower::WriteFailed").field(connection).finish(),
            },
            Self::Down { channel, values, .. } => f.debug_tuple("Down/Secret").field(&(channel, values.len())).finish(),
            Self::TurnBusy { channel, run, attempt, turn } => {
                f.debug_tuple("TurnBusy").field(channel).field(run).field(attempt).field(turn).finish()
            }
            Self::NotAdmitted { channel, text } => f.debug_tuple("NotAdmitted").field(&(channel, text.len())).finish(),
            Self::Stop => f.write_str("Stop"),
            Self::Close => f.write_str("Close"),
            Self::Abort => f.write_str("Abort"),
        }
    }
}
impl core::fmt::Debug for Request {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Take { connection } => f.debug_tuple("Take").field(connection).finish(),
            Self::TurnAway { connection } => f.debug_tuple("TurnAway").field(connection).finish(),
            Self::Close { connection } => f.debug_tuple("Close").field(connection).finish(),
            Self::Abort { connection } => f.debug_tuple("Abort").field(connection).finish(),
            Self::Lower { connection, lower } => match lower {
                Lower::Read(_) => f.debug_tuple("Lower::Read").field(connection).finish(),
                Lower::Write(_) => f.debug_tuple("Lower::Write").field(connection).finish(),
                Lower::FinishWrite => f.debug_tuple("Lower::FinishWrite").field(connection).finish(),
            },
            Self::Hello { channel, worker, .. } => f.debug_tuple("Hello").field(channel).field(worker).finish(),
            Self::Up { channel, parts, .. } => f.debug_tuple("Up").field(channel).field(&parts.len()).finish(),
            Self::Lost { channel } => f.debug_tuple("Lost").field(channel).finish(),
            Self::Closed => f.write_str("Closed"),
        }
    }
}
