//! The agent capability's parent face (domain/host.md, sections 2, 3 and 9).
//! This stateless vocabulary owns commands and notices, without process
//! mechanics. A parent sends Spawn and receives Gone after retained rights
//! settle; Started binds an agent independently of Admitted.

use crate::{Answer, Ask, CallName, End, Fault, Grant, MessageRefusal, Reply, Start, Turn};
use alloc::boxed::Box;
use skein_lib::{Duration, Time, Token};

/// A termination signal heard by the agent, left to its parent to decide.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Signal {
    /// The agent heard SIGINT.
    Interrupt,
    /// The agent heard SIGTERM.
    Terminate,
}

/// Commands sent by a parent to either kind of agent capability.
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    /// Parent cannot renew grants, or can again; forwarded to each live run.
    Renewals { held: bool },
    /// Parent requests one agent capability; exactly one Gone.
    Spawn {
        client: Token,
        /// Bounded start admitted before process IO.
        start: Start,
    },
    /// Parent message, ending refused if the ingress cannot admit it.
    Message {
        /// Admitted Open messages request a final reply; Finite ignores this bit.
        finish_requested: bool,
        agent: Token,
        name: Token,
        /// Sender's bounded label, forwarded without interpretation.
        label: Box<[u8]>,
        /// Sender's bounded text, forwarded without interpretation.
        text: Box<[u8]>,
    },
    /// Parent consumes one outstanding operation right, even after channel shutdown.
    Answer {
        agent: Token,
        call: Token,
        /// Exactly one matching parent terminal; never abandons delivery.
        reply: Reply,
    },
    /// Parent committed exactly this forwarded turn.
    Acknowledge {
        agent: Token,
        /// Exact forwarded turn number; duplicate kept ACK is inert.
        turn: u32,
    },
    /// Parent replaces a known credential name.
    Grant {
        agent: Token,
        /// Positive generation and relative validity; an equal generation is resent without renewal.
        grant: Grant,
    },
    /// Parent politely stops once; no deadline reset.
    Stop { agent: Token },
    /// From the parent: stop after the current turn, without a cancel grace.
    Drain { agent: Token, grace: Option<Duration> },
    /// Parent committed exactly this forwarded part.
    AcknowledgePart { agent: Token, number: u32 },
    /// Parent drops the run now, releasing held turns; Gone waits for all operation terminals.
    Drop { agent: Token },
}

/// Notices sent by an agent capability to its parent; Spawn ends in Gone.
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    /// Capability started; parent may now address its agent.
    Started { client: Token, agent: Token },
    /// Agent independently accepted run.
    Admitted { client: Token },
    /// The live agent heard a signal; no terminal or stop is owed.
    Signalled { host_run: Token, signal: Signal, from: Option<u32> },
    /// One parent operation; exact Answer required.
    Called {
        client: Token,
        /// Stable parent scope, not interpreted.
        logical_run: Token,
        call: Token,
        /// Positive completion and assistant-block position, scoped by logical run.
        name: CallName,
        /// Operation deadline and bounded pause.
        deadline: Time,
        ask: Ask,
    },
    /// Agent/process withdrew call once; parent terminal still required.
    Withdrawn { client: Token, call: Token },
    /// Parent takes owned payload; kit retains exact ACK metadata.
    Turn { client: Token, turn: Turn },
    /// Parent takes a part in the order told, interleaved with turns.
    Part { client: Token, part: crate::channel::ToldPart },
    /// Validated read fence; crossed queued message prevents pause.
    Waiting {
        client: Token,
        /// Last sent message read.
        read: Option<Token>,
    },
    /// A bounded long operation began; the parent may show its notice.
    Long { client: Token, operation: crate::Operation, span: Duration },
    /// One prior long operation ended.
    LongDone { client: Token },
    /// Known-account credential rejection notice.
    Unauthorized { client: Token, account: u32, generation: u64 },
    /// One program allowance observation for the run's account.
    Usage { client: Token, account: u32, window: crate::UsageWindow },
    /// Known-account quota notice.
    Limited { client: Token, account: u32, reset: Option<Duration> },
    /// The provider refused a known account, retaining its diagnostic fields.
    Refused { client: Token, account: u32, code: Box<[u8]>, reason: Box<[u8]> },
    /// Metadata about the guides this run read, once at preparation.
    Prepared { client: Token, guides: Box<[smith_domain::run::facts::Guide]> },
    /// Best-effort fact moved to parent.
    Told {
        client: Token,
        /// Bounded opaque agent fact.
        body: Box<[u8]>,
    },
    /// One run answer; process cleanup still owes Gone.
    Answered {
        client: Token,
        /// Validated last word.
        answer: Answer,
        /// Inline diagnostic; spawned agents report their standard-error tail.
        detail: Box<[u8]>,
    },
    /// Capability names its remaining accepted messages unread, once after the answer or before failure/end.
    Unread { client: Token, names: Box<[Token]> },
    /// One typed failure before the run answer.
    Faulted {
        client: Token,
        /// Process/channel failure, not a fabricated run result.
        fault: Fault,
    },
    /// Message rejected before queue mutation.
    MessageRefused { client: Token, name: Token, reason: MessageRefusal },
    /// Spawn terminal; all process and parent rights settled.
    Gone {
        client: Token,
        end: End,
        /// Bounded operator detail.
        detail: Box<[u8]>,
    },
}
