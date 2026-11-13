//! Routing (protocol/channel/README.md, sections 2, 3 and 7); Debug (jig/docs/design/protocol/service.md, 11.3).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_link::SourceName;
use jig_protocol_worker_wire::Part;
use skein_channel::{Lower, LowerEvent};
use skein_io::digest::Digest;
use skein_lib::{List, Secret, Token};
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
/// How a credential is presented (protocol/channel/README.md, 7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme {
    Bearer,
    Key,
}
/// `TurnBusy` is the fleet's busy answer, handled below the hub (protocol/channel/records.md, 2; domain/hosts/hub.md, 1).
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
/// Why a worker's connection ended (protocol/channel/README.md, 8 and 9.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Loss {
    /// Nothing heard for the silence deadline.
    Silence,
    /// Output queued with no frame drained for the stall deadline.
    Stall,
    /// The output cap filled.
    Full,
    /// The opening's handshake or the hello did not come within its deadline.
    Deadline,
    /// The stream ended or failed.
    Stream,
    /// A frame that does not parse, or a record that does not decode or
    /// comes outside its phase.
    Framing,
    /// Refused here, with the reason sent: jig's (`jig_protocol_worker_wire::Reason`),
    /// or skein's channel's (2, limits; 3, framing).
    Refused { reason: u16 },
    /// The authenticated worker's io.output differs from the engine's (protocol/channel/README.md, 4.1).
    OutputRoom { engine: u32, worker: u32 },
    /// The worker refused, with its reason.
    RefusedPeer { reason: u16 },
    /// No room for another connection: turned away at its accept.
    NoRoom,
    /// The engine closed it, stopping.
    Stopping,
}
/// A record the engine's side did not send (protocol/channel/README.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unsent {
    pub kind: jig_protocol_worker_wire::Kind,
    /// Its frame's bytes, the header included.
    pub bytes: u32,
    pub bound: Bound,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bound {
    /// Past the stream's room, in bytes.
    Room(u32),
    /// Refused at the channel's entrance; never `skein_channel::Unsent::Full`.
    Channel(skein_channel::Unsent),
}
pub enum Request {
    /// Opening accepted, with the worker's statement and agreed versions in its order.
    Opened {
        worker: u32,
        wires: List<jig_protocol_worker_wire::Agreed>,
        statement: crate::Statement,
    },
    /// A refusal sent at the opening, before a hello reaches the domain.
    Refused {
        worker: Option<u32>,
        reason: u16,
        text: Box<[u8]>,
    },
    /// Once for each record the component did not send.
    Unsent {
        worker: u32,
        channel: Token,
        unsent: Unsent,
    },
    Take {
        connection: Token,
    },
    TurnAway {
        connection: Token,
    },
    Lower {
        connection: Token,
        lower: Lower,
    },
    Close {
        connection: Token,
    },
    Abort {
        connection: Token,
    },
    Hello {
        channel: Token,
        worker: u32,
        hello: jig_domain_link::Hello,
    },
    Up {
        channel: Token,
        up: host::LinkUp,
        parts: Box<[Part]>,
    },
    Lost {
        channel: Token,
    },
    /// Once per accepted connection, when it ends, whatever ended it: the
    /// worker it authenticated as, the channel its hello opened, and why.
    Ended {
        connection: Token,
        worker: Option<u32>,
        channel: Option<Token>,
        why: Loss,
    },
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
            Self::Ended { connection, worker, channel, why } => {
                f.debug_tuple("Ended").field(connection).field(worker).field(channel).field(why).finish()
            }
            Self::Unsent { worker, channel, unsent } => f
                .debug_struct("Unsent")
                .field("worker", worker)
                .field("channel", channel)
                .field("unsent", unsent)
                .finish(),
            Self::Opened { worker, wires, statement } => {
                f.debug_tuple("Opened").field(worker).field(wires).field(statement).finish()
            }
            Self::Refused { worker, reason, text } => {
                f.debug_tuple("Refused").field(worker).field(reason).field(text).finish()
            }
            Self::Closed => f.write_str("Closed"),
        }
    }
}
