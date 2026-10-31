//! Typed host channel (domain/host.md, sections 2, 3, 6 and 7;
//! protocol/channel.md, sections 3 and 7).
//! Concrete parent values move; metadata controls sequence, spend, read fences and ACKs.
//! No framing, secrets or policy decoding. All payloads are
//! checked against receiving Limits before retained state changes. Durable decisions
//! belong to the parent, which scopes `CallName` by the same logical run after restart.
pub use crate::source_name::SourceName;

use crate::{Charter, Declared, Delivery, Fields, Transcript, TurnValue};
use alloc::boxed::Box;
use skein_lib::{Duration, Time, Token};
pub use smith_domain::run::{
    BudgetRefusal, CallName, Cap, Ceiling, CompletionEvidence, CompletionFailure, Exhausted, HostEffect as Effect,
    Invalid as RunInvalid, Limit, Message, Overflow, Part, Phase, Policy, Rate, ReceivingLimit, Refusal, Report,
    TranscriptRefusal,
};

/// The host asks for live content, one boolean per kind (protocol/channel.md, section 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LiveAsk {
    pub text: bool,
    pub reasoning: bool,
    pub input: bool,
}

/// Parent start moved to the first channel Send; process spawn has its own deadline.
#[derive(PartialEq, Eq, Debug)]
pub struct Start {
    pub live: Option<LiveAsk>,
    /// Run identity retained across activations, or absent for agent admission to draw.
    pub identity: Option<smith_domain::run::Identity>,
    /// At most the inbox, each fitting the rendered message bound.
    pub messages: Box<[Message]>,
    /// Stable parent run identity across restart, uninterpreted here.
    pub logical_run: Token,
    /// Positive parent-issued number unique for every activation of `logical_run`.
    pub activation: u64,
    /// Optional prepared workspace resolved by the lower process adapter.
    pub workspace: Option<Token>,
    /// Checked charter ownership at most `Limits::charter_bytes`.
    pub charter: Charter,
    /// Concrete saved history with aggregate `Limits::transcript_bytes`.
    pub transcript: Option<Transcript>,
    /// Calls answered after the last saved turn, bounded in aggregate.
    pub answered: Box<[AnsweredCall]>,
    /// At most `Limits::directories` unique named mounts.
    pub directories: Box<[Directory]>,
    /// At most `Limits::accounts` distinct credential names; no values.
    pub grants: Box<[Grant]>,
}

/// One durable host answer absent from the saved transcript.
#[derive(PartialEq, Eq, Debug)]
pub struct AnsweredCall {
    /// Original stable call name, preserved across activations.
    pub name: CallName,
    /// The host tool name from the call, kept as text bytes.
    pub tool: Box<[u8]>,
    /// The settled host or delivery answer, never a temporary refusal.
    pub reply: SavedReply,
}

/// One settled answer retained across activations.
#[derive(PartialEq, Eq, Debug)]
pub enum SavedReply {
    /// A host tool's result or error text.
    Host { error: bool, body: Box<[u8]> },
    /// A delivery's terminal disposition.
    Delivery(Box<Delivery>),
    /// The host decided the call but its answer exceeded the receiving bound.
    TooLarge,
}

/// Host credit sent with the start for turns awaiting durable acknowledgement.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Window {
    /// Maximum unacknowledged turns.
    pub turns: u32,
    /// Maximum unacknowledged turn bytes.
    pub bytes: u64,
}

/// Parent mount descriptor forwarded without filesystem or delivery policy.
#[derive(PartialEq, Eq, Debug)]
pub struct Directory {
    /// Opaque mounted IO root for the inline kind; spawned adapters attach their own.
    pub root: Token,
    /// Unique safe single component bounded by `Limits::name_bytes`; the host
    /// attests text encoding, and bytes move unchanged.
    pub name: Box<[u8]>,
    /// Absolute host path retained for inline workspace aliases, within `Limits::path_bytes`.
    pub host: Box<[u8]>,
    pub writable: bool,
    /// Parent declares a git working tree; plain directories have no conflicts.
    pub git: bool,
    /// At most `Limits::conflicts` unique bounded relative paths per git mount.
    /// Read-only git conflicts are informative and grant no writes. The host
    /// attests text encoding.
    pub conflicts: Box<[Box<[u8]>]>,
}

/// Credential name and value form forwarded to the protocol; never credential bytes
/// (domain/host.md, section 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Grant {
    /// Known parent credential account.
    pub account: u32,
    /// Positive increasing credential generation; rejected notices echo it.
    pub generation: u64,
    /// Relative lifetime; protocol resolves the secret.
    pub valid: Duration,
    pub form: Form,
}

/// What a grant's value lends, named by the host (protocol/channel.md, section 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Form {
    /// A credential, sent as its provider takes this scheme.
    Credential(Scheme),
    /// A program source, by its name; the run takes the program backend.
    Program(SourceName),
    /// The withdrawn id-and-bearer form, removed by the second part of LLM accounts once its last host has moved.
    Withdrawn,
}

/// A credential's scheme, named by the host (protocol/channel.md, section 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Scheme {
    /// A bearer credential sent as the provider takes it.
    Bearer,
    /// A key credential sent as the provider takes it.
    Key,
}

/// Agent turn moved once to parent, which owns payload until exact commitment ACK.
#[derive(PartialEq, Eq, Debug)]
pub struct Turn {
    /// Begins the provider-history suffix a host sends when resuming.
    pub opens_window: bool,
    /// Positive consecutive turn number from one; checked increment.
    pub number: u32,
    /// Global activation spend supplied by the agent in the host's unit; must
    /// not fall;
    /// the opaque body separately retains the session's inclusive child bill.
    pub spent: u64,

    /// Last named message sent and read; never a queued or unknown name.
    pub read: Option<Token>,
    /// Concrete transcript turn owning at most `Limits::turn_bytes`.
    pub body: TurnValue,
}

/// Agent operation forwarded once to parent and answered once even after withdrawal.
#[derive(PartialEq, Eq, Debug)]
pub enum Ask {
    /// Generic host tool; kit never decodes schema, body or policy.
    Host {
        /// Nonempty tool label bounded by `Limits::name_bytes`.
        tool: Box<[u8]>,
        effect: Effect,
        /// Opaque owned arguments bounded by `Limits::call_bytes`.
        body: Box<[u8]>,
    },
    /// Delivery of checked workspace; no cancellation after submission.
    Deliver {
        /// Checked generic fields bounded by `Limits::call_bytes`.
        fields: Fields,
    },
}

/// Parent terminal forwarded through Send; delivery keeps its evidence.
#[derive(PartialEq, Eq, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "sealed host diagnostic keeps a fixed inline tail; queue/state bounds price the full variant"
)]
pub enum Reply {
    /// Text tool result or error.
    Host {
        /// Parent classified error flag; body remains opaque.
        error: bool,
        /// Owned text result bounded by `Limits::answer_bytes`.
        body: Box<[u8]>,
    },
    /// One full delivered/nothing/refused/failed/stale terminal.
    Delivery(
        /// Sealed typed boundary value; no hidden policy.
        Delivery,
    ),
    /// Predecision capacity refusal; callback still gets exactly one response.
    Busy,
    /// Transport could not provide an answer; durable effects are unknown here.
    Unavailable,
    /// Transport withdrawal terminal for generic tools; durable effects remain parent-owned.
    Withdrawn,
    /// Ordinary tool response exceeded receiving bytes.
    TooLarge,
}

/// What kept an LLM from going on.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ModelFault {
    /// Full neutral failure and evidence after session retry policy.
    /// This is distinct from local model stopping rules and requested run Cancel.
    Completion {
        /// Exact content-free shared-client classification and cooldown.
        failure: CompletionFailure,

        /// Exact transport evidence, including across interrupted Delivery.
        evidence: CompletionEvidence,
        /// Charter LLM index, resolved by `Charter::model`.
        model: u32,
        attempts: u32,
        report: Report,
    },

    /// The account spent its provider allowance, with a reset only when supplied.
    Limited { account: u32, reset: Option<Duration> },
    /// The provider refused this account.
    Refused { account: u32, code: Box<[u8]> },
    /// The account lost its login.
    Unauthorized { account: u32 },
    /// The account's credential program could not run.
    Program { name: Box<[u8]>, exit: Option<u32>, detail: Box<[u8]> },
    /// Its provider failed for good: unreachable, overloaded past the
    /// retries, or refusing the call or its credentials.
    Provider,
    /// It kept declining to answer.
    Declined,
    /// It kept running out of tokens mid-answer.
    Truncated,
    /// It kept asking for tools and naming none.
    Malformed,
}

/// Why a run ended without an outcome: what the host parent acts on.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RunFailure {
    /// Exact transient history refusal before unsupported work starts.
    Transcript(
        /// Exact content-free history-admission classification sent by the agent
        /// to the host; no fresh-start fallback or provider effect is implied.
        TranscriptRefusal,
    ),

    /// The LLM could not do the work.
    Model(ModelFault),
    /// The run's budget ran out.
    Budget {
        exhausted: Exhausted,
        /// Turns, host spend units, or milliseconds of run wall time; absent for receiving/overflow failures.
        used: Option<u64>,
        bound: Option<u64>,
    },
    /// The LLM did not keep to the run's rules.
    Policy(Policy),
    /// The host parent cancelled the run.
    Cancelled,
    /// The host's delivery context moved: no later delivery from this run can
    /// land. The host decides what context a new run receives.
    Stale,
}

/// Agent last-word result; the kit does not judge the accepted declaration.
#[derive(PartialEq, Eq, Debug)]
pub enum RunResult {
    /// Run refused before Admitted; requires zero turns/spend.
    Refused { refusal: Refusal },
    /// Agent-declared result allowed by its contract.
    Accepted {
        /// Concrete declared result bounded by `Limits::outcome_bytes`.
        outcome: Declared,
    },
    /// Parking resumes from transcript, with no snapshot.
    Parked,
    /// Typed run failure.
    Failed { failure: RunFailure },
}

/// Agent final word forwarded once; process still owes Gone.
#[derive(PartialEq, Eq, Debug)]
pub struct Answer {
    /// Last message covered by a told turn, echoed at the terminal.
    pub read: Option<Token>,

    /// Exactly the observed numbered turn count.
    pub turns: u32,

    /// Final global host-unit spend, at least the previous Turn's spend;
    /// refused starts have zero.
    pub spent: u64,

    pub result: RunResult,
}

/// Decoded agent records; one record terminates each Read.
#[derive(PartialEq, Eq, Debug)]
pub enum Up {
    /// Independent run admission after process Started, exactly once.
    Admitted,
    /// The run refused one sent message before a fence read it.
    MessageRefused { name: Token, reason: crate::MessageRefusal },
    /// Generic named host operation; parent owes terminal.
    Call {
        call: Token,
        /// Positive completion and assistant-block position, scoped by logical run.
        name: CallName,
        /// Agent deadline bounds watchdog pause, not abandonment.
        deadline: Time,
        ask: Ask,
    },
    /// Call withdrawn once; parent still owes terminal.
    Withdraw { call: Token },
    /// Validated numbered turn moved to parent.
    Turn {
        /// Bounded turn metadata and owned transcript body.
        turn: Turn,
    },
    /// Best-effort observation; valid receipt counts as progress.
    Fact {
        /// Opaque fact bounded by `Limits::fact_bytes`.
        body: Box<[u8]>,
    },
    /// Bounded long operation stretches progress clock.
    Long {
        operation: crate::Operation,
        /// At most `Limits::long_span`.
        span: Duration,
    },
    /// Ends the announced progress stretch.
    LongDone,
    /// Waiting with a known read prefix pauses the watchdog until a message arrives.
    Waiting {
        /// None before first message; known sent message thereafter.
        read: Option<Token>,
    },
    /// Protocol rejected a named grant.
    Unauthorized {
        /// Known credential account.
        account: u32,
        /// Known positive generation.
        generation: u64,
    },
    /// Protocol exhausted a known account.
    Limited {
        /// Known credential account.
        account: u32,
        /// Parent refresh/retry hint.
        reset: Option<Duration>,
    },
    /// The provider refused a known account, retaining its diagnostic fields.
    Refused { account: u32, code: Box<[u8]>, reason: Box<[u8]> },
    /// Agent last word; no further records may follow before EOF.
    Answer { answer: Answer },
}

/// Owned downlink; Start first, ordered controls and at most one Cancel.
#[derive(PartialEq, Eq, Debug)]
pub enum Down {
    /// First and exactly once; concrete values moved from Spawn.
    Start {
        /// Validated parent start including post-transcript answers.
        start: Start,
        /// Credit derived from the host kit's limits.
        window: Window,
    },
    /// Named ordered parent message.
    Message {
        /// Admitted Open messages request a final reply; Finite ignores this bit.
        finish_requested: bool,
        /// Opaque parent name; reserved until read fence covers it.
        name: Token,
        /// Opaque sender label; label and text together fit `Limits::message_bytes`.
        label: Box<[u8]>,
        /// Opaque text; label and text together fit `Limits::message_bytes`.
        text: Box<[u8]>,
    },
    /// One response to agent callback.
    Answer {
        call: Token,
        /// Exactly one matching parent terminal; never abandons delivery.
        reply: Reply,
    },
    /// Exact parent commitment of one forwarded turn.
    Acknowledge {
        /// Outstanding exact numbered turn, never an inferred prefix.
        turn: u32,
    },
    /// Coalesced known-account credential refresh.
    Grant {
        /// Names only; generation increases.
        grant: Grant,
    },
    /// First polite stop, once; operations retain terminal rights.
    Cancel,
}
