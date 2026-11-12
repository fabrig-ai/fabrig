//! Owner routing and shell exchanges (protocol/client/README.md, 2 and 3).
use alloc::boxed::Box;
use jig_client_domain::{Answered, Ask, Family, Readable, Readout, Retry, Watchable, contract};
pub use jig_client_domain::{StreamBound, StreamsFull};
pub use jig_client_protocol_wire::ApprovalCode;
use skein_lib::{List, RequestEnvelope, RequestKey, RequestRecord, Secret, Token, Wall};
/// A record the shell read with its 16-byte request key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub key: Box<[u8]>,
    pub bytes: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    pub version: u16,
    pub record: RequestRecord<jig_client_domain::Held>,
}
pub enum Event {
    SignIn {
        with: With,
    },
    SignOut,
    AskApproval,
    ReadApproval {
        code: ApprovalCode,
    },
    DecideApproval {
        code: ApprovalCode,
        approve: bool,
    },
    Authorize {
        query: Box<[u8]>,
    },
    Start {
        kept: List<Record>,
    },
    Save {
        record: Saved,
    },
    Erase {
        key: RequestKey,
    },
    /// Every write and removal of this step is durable.
    Stored,
    /// This step's writes failed; withhold and fail its held exchanges.
    NotStored,
    Send {
        attempt: Token,
        key: RequestKey,
        ask: Ask,
        /// A value the request carries once, put into `Keyed` and never kept (protocol/client/README.md, 3).
        value: Option<Secret>,
    },
    Read {
        read: Token,
        what: Readable,
    },
    Reply {
        exchange: Token,
        message: Box<[u8]>,
    },
    Lost {
        exchange: Token,
    },
    Open {
        watch: Token,
        subject: Watchable,
    },
    Continue {
        watch: Token,
        subject: contract::Subject,
    },
    Close {
        watch: Token,
    },
    Streamed {
        message: Streamed,
    },
    StreamGone,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    StreamsFull {
        full: StreamsFull,
    },
    Authorized(Authorized),
    Approval(Approval),
    Save {
        record: Record,
    },
    Erase {
        key: Box<[u8]>,
    },
    Kept {
        record: Saved,
    },
    Dropped,
    /// One message; its transport owns the mapping and deadline.
    Exchange {
        exchange: Token,
        credential: Option<Secret>,
        message: Message,
    },
    NotKept {
        attempt: Token,
    },
    Answer {
        attempt: Token,
        envelope: RequestEnvelope,
        answer: Option<Answered>,
        why: Option<Retry>,
    },
    /// The shell shows this once; it never enters the client's domain.
    ServiceCredential {
        party: u64,
        credential: Secret,
    },
    OpenStream {
        credential: Option<Secret>,
    },
    Signing {
        signing: Signing,
    },
    CloseStream,
    Watch {
        watch: Token,
        event: contract::WatchEvent,
    },
    FamilyDelivery {
        watch: Token,
        record: Family,
    },
    Ended {
        watch: Token,
        end: contract::WatchEnd,
    },
    Hello {
        family: Option<u16>,
        versions: Option<contract::VersionRange>,
    },
    /// One terminal per read.
    Read {
        read: Token,
        result: Readout,
    },
    Fact {
        fact: Fact,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    UndecodableAnswer,
    UndecodableDocument,
    UndecodableEvent,
    UndecodableWatchAnswer,
    StreamDropped,
    UndecodableStore,
}

/// Authentication held by the link (client/domain/client/README.md, 9).
#[derive(Debug, PartialEq, Eq)]
pub enum With {
    Service(Secret),
    Transport,
    Link { secret: Secret, held: Option<Secret> },
    Held(Secret),
    Approval { secret: Secret },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Signing {
    SignedIn { party: u64 },
    Again,
    Busy,
    Full { sign_ins: u32, earliest: Wall, link_until: Wall },
    Refused,
    LinkUsed,
    LinkExpired,
    LinkUnknown,
    Approval(Taking),
    Lost,
    SignedOut,
}

/// A protocol record, independent of its transport's route.
#[derive(PartialEq, Eq)]
pub enum Message {
    Authorize(Box<[u8]>),
    ApprovalAsk(Box<[u8]>),
    ApprovalRead(Box<[u8]>),
    ApprovalDecide(Box<[u8]>),
    SignInApproval(Box<[u8]>),
    Request(Box<[u8]>),
    Read(Box<[u8]>),
    Watch(Box<[u8]>),
    Unwatch(Box<[u8]>),
    StreamGone(Box<[u8]>),
    SignInLink(Box<[u8]>),
    SignInService,
    SignOut,
}
/// A transport exchange's terminal outcome.
#[derive(PartialEq, Eq)]
pub enum Heard {
    Reply { exchange: Token, message: Box<[u8]> },
    Lost { exchange: Token },
    Streamed { message: Streamed },
    StreamGone,
}

/// One whole message cut by the transport from its stream.
#[derive(Debug, PartialEq, Eq)]
pub enum Streamed {
    Full(StreamsFull),
    Record(Box<[u8]>),
    Heartbeat,
    /// The stream refused, its opening answered signed out.
    SignedOut,
}

/// Authorization's terminal answer (protocol/client/openid.md).
#[derive(Debug, PartialEq, Eq)]
pub enum Authorized {
    Go { location: Box<[u8]> },
    Refused(AuthorizeWhy),
    SignedOut,
    NotReady,
    Busy,
    Lost,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthorizeWhy {
    Malformed,
    UnknownClient,
    UnknownRedirect,
    NoProvider,
    NotAPerson,
}

/// Why an approval take installed nothing (jig/docs/design/client/domain/client/README.md, 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taking {
    Waiting,
    Refused,
    Expired,
    Used,
    Unknown,
}

/// An approval's single answer to its host (jig/docs/design/protocol/client/sign-in.md).
#[derive(Debug, PartialEq, Eq)]
pub enum Approval {
    Asked { secret: Secret, code: ApprovalCode, expires: Wall },
    NotAsked(NotAsked),
    Read { code: ApprovalCode, state: ApprovalState },
    Decided { code: ApprovalCode, decided: Decided },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotAsked {
    Refused,
    NotReady,
    Busy,
    Lost,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalState {
    Waiting { asked: Wall },
    Approved,
    Taken,
    Refused,
    Expired,
    Unknown,
    NotABrowser,
    SignedOut,
    NotReady,
    Busy,
    Lost,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decided {
    Approved,
    Refused,
    Used,
    Expired,
    Unknown,
    NotABrowser,
    SignedOut,
    NotReady,
    Busy,
    Lost,
}

// Encoded records can carry secrets and credentials (jig/docs/design/protocol/client/sign-in.md).
impl core::fmt::Debug for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Authorize(body) => f.debug_struct("Authorize").field("body_len", &body.len()).finish(),
            Self::ApprovalAsk(body) => f.debug_struct("ApprovalAsk").field("body_len", &body.len()).finish(),
            Self::ApprovalRead(body) => f.debug_struct("ApprovalRead").field("body_len", &body.len()).finish(),
            Self::ApprovalDecide(body) => f.debug_struct("ApprovalDecide").field("body_len", &body.len()).finish(),
            Self::SignInApproval(body) => f.debug_struct("SignInApproval").field("body_len", &body.len()).finish(),
            Self::Request(body) => f.debug_struct("Request").field("body_len", &body.len()).finish(),
            Self::Read(body) => f.debug_struct("Read").field("body_len", &body.len()).finish(),
            Self::Watch(body) => f.debug_struct("Watch").field("body_len", &body.len()).finish(),
            Self::Unwatch(body) => f.debug_struct("Unwatch").field("body_len", &body.len()).finish(),
            Self::StreamGone(body) => f.debug_struct("StreamGone").field("body_len", &body.len()).finish(),
            Self::SignInLink(body) => f.debug_struct("SignInLink").field("body_len", &body.len()).finish(),
            Self::SignInService => f.write_str("SignInService"),
            Self::SignOut => f.write_str("SignOut"),
        }
    }
}

// A reply may carry a credential (jig/docs/design/protocol/client/sign-in.md).
impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SignIn { with } => f.debug_struct("SignIn").field("with", with).finish(),
            Self::SignOut => f.write_str("SignOut"),
            Self::AskApproval => f.write_str("AskApproval"),
            Self::ReadApproval { code } => f.debug_struct("ReadApproval").field("code", code).finish(),
            Self::DecideApproval { code, approve } => {
                f.debug_struct("DecideApproval").field("code", code).field("approve", approve).finish()
            }
            Self::Authorize { query } => f.debug_struct("Authorize").field("query", query).finish(),
            Self::Start { kept } => f.debug_struct("Start").field("kept", kept).finish(),
            Self::Save { record } => f.debug_struct("Save").field("record", record).finish(),
            Self::Erase { key } => f.debug_struct("Erase").field("key", key).finish(),
            Self::Stored => f.write_str("Stored"),
            Self::NotStored => f.write_str("NotStored"),
            Self::Send { attempt, key, ask, value } => f
                .debug_struct("Send")
                .field("attempt", attempt)
                .field("key", key)
                .field("ask", ask)
                .field("value", value)
                .finish(),
            Self::Read { read, what } => f.debug_struct("Read").field("read", read).field("what", what).finish(),
            Self::Reply { exchange, message } => {
                f.debug_struct("Reply").field("exchange", exchange).field("body_len", &message.len()).finish()
            }
            Self::Lost { exchange } => f.debug_struct("Lost").field("exchange", exchange).finish(),
            Self::Open { watch, subject } => {
                f.debug_struct("Open").field("watch", watch).field("subject", subject).finish()
            }
            Self::Continue { watch, subject } => {
                f.debug_struct("Continue").field("watch", watch).field("subject", subject).finish()
            }
            Self::Close { watch } => f.debug_struct("Close").field("watch", watch).finish(),
            Self::Streamed { message } => f.debug_struct("Streamed").field("message", message).finish(),
            Self::StreamGone => f.write_str("StreamGone"),
        }
    }
}

// A reply may carry a credential (jig/docs/design/protocol/client/sign-in.md).
impl core::fmt::Debug for Heard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Reply { exchange, message } => {
                f.debug_struct("Reply").field("exchange", exchange).field("body_len", &message.len()).finish()
            }
            Self::Lost { exchange } => f.debug_struct("Lost").field("exchange", exchange).finish(),
            Self::Streamed { message } => f.debug_struct("Streamed").field("message", message).finish(),
            Self::StreamGone => f.write_str("StreamGone"),
        }
    }
}
