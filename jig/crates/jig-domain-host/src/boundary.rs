//! The host's boundary (domain/hosts/README.md, sections 4 to 5 and domain/hosts/hub.md and domain/hosts/README.md, section 6). Its root
//! translates core messages and the optional workspace and agent capabilities. A
//! request's `owner` names the hosted run or its call; a terminal echoes it.
//! The root owns application workspace items and interprets opaque tokens.

use alloc::boxed::Box;
pub(crate) use jig_domain_link::{Grant, GrantTable, Hosting, Phase};

use crate::JobKind;
use skein_lib::{Duration, Token};

/// What of the hub's start did not decode (jig/docs/design/protocol/service.md, 11.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Undecoded {
    Version(Mismatch),
    Charter,
    Transcript,
    CallName,
    /// A resumed delivery's evidence, which must decode and agree with its outcome.
    Evidence,
}

/// parent -> host
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    ListRead {
        read: u64,
        outcome: jig_domain_link::ListOutcome,
    },
    Agent(FromAgent),
    Job(FromJob),
    SourceAnswered {
        owner: Token,
        answer: crate::sources::Answer,
    },
    SourceLost,
    /// Terminal for a cancelled relay: its local delivery and wait have ended.
    RelayCancelled {
        run: Token,
        attempt: Token,
        call: Box<[u8]>,
    },
    /// The root asks the hub to drain (domain/hosts/hub.md, 7): admit no more
    /// work and ask live runs to stop at their turn's end. Jobs run on.
    Drain,
    /// From the top level: cancel every run hosted now, for `reason` (lost
    /// contact with the engine past its grace, or shutdown). The runs are
    /// cancelled one at a time, from the ready list. A root shutting down
    /// admits no more runs: assignments after it are refused as busy.
    CancelAll {
        reason: Reason,
    },
    /// The link opened; the root supplies the hello's configuration.
    Opened {
        stop_bound: Duration,
        workstreams: Box<[u64]>,
        bounds: crate::Bounds,
        jobs: Box<[crate::JobBound]>,
    },
    Down(crate::LinkDown),
    /// Terminal for `Prepare`: the workspace is ready, and `workspace` names it
    /// from now on.
    Prepared {
        owner: Token,
        workspace: Token,
    },
    /// Terminal for `Prepare`: the workspace could not be prepared, and
    /// nothing of it is held.
    Unprepared {
        owner: Token,
        failure: Preparation,
        detail: Box<[u8]>,
    },
    /// Terminal for a delivery, including the root's opaque left-work token.
    Delivered {
        owner: Token,
        delivery: Delivery,
    },
    /// Terminal for a save, including the root's opaque saved-work token.
    Saved {
        owner: Token,
        at: Option<Token>,
    },
}

/// host -> parent
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    ReadList {
        read: u64,
        endpoint: u32,
        grant: Grant,
    },
    CancelRead {
        read: u64,
    },
    Agent(ToAgent),
    Job(ToJob),
    DeliverJob {
        owner: Token,
        workspace: Token,
        delivery: Box<[u8]>,
    },
    Source {
        owner: Token,
        name: crate::SourceName,
        operation: crate::sources::Operation,
        deadline: skein_lib::Time,
    },
    SourceFault,
    Up(crate::LinkUp),
    Hello {
        hello: crate::Hello,
    },
    /// Deliver the unchecked tree with no result fields.
    DeliverUnchecked {
        owner: Token,
        workspace: Token,
    },
    Deliver {
        owner: Token,
        workspace: Token,
        title: Box<[u8]>,
        body: Box<[u8]>,
    },
    /// Cancel the local delivery and wait of a relay. Exactly one terminal
    /// follows: `RelayCancelled`, or `Relayed` if its answer won. Remote
    /// effects are not rolled back.
    CancelRelay {
        run: Token,
        attempt: Token,
        call: Box<[u8]>,
    },
    /// Prepare `workspace` for the hosted run `owner`.
    Prepare {
        owner: Token,
        workspace: Workspace,
    },
    /// Abandon the prepare in flight for `owner`, whose run is cancelled. Its
    /// terminal still comes: `Unprepared`, or `Prepared` if the prepare won
    /// the race, and then the workspace is released.
    Abort {
        owner: Token,
    },
    /// Ask the workspace to save unfinished work.
    Save {
        owner: Token,
        workspace: Token,
    },
    /// The run is done with `workspace`: it goes back to the cache.
    Release {
        workspace: Token,
    },
}

/// The host's agent capability, translated by its root (domain/hosts/hub.md
/// ). These requests carry no process or channel vocabulary.
#[derive(PartialEq, Eq, Debug)]
pub enum ToAgent {
    /// Pass a named message with its sender label and words.
    Message {
        agent: Token,
        name: Token,
        sender: Box<[u8]>,
        words: Box<[u8]>,
        finish_requested: bool,
    },
    /// Answer a call under the agent's opaque name.
    Answer {
        agent: Token,
        call: Box<[u8]>,
        answer: CallAnswer,
    },
    /// Start an agent with its activation and committed conversation state.
    Start {
        owner: Token,
        start: Box<Start>,
    },
    /// The hub holds this turn independently of the engine link.
    Acknowledge {
        agent: Token,
        turn: u32,
    },
    Grant {
        agent: Token,
        grant: Grant,
    },
    /// Ask the run to stop at the end of its current turn, then answer and
    /// report `Gone` as for a cancel (domain/hosts/hub.md, 7).
    Drain {
        agent: Token,
    },
    /// Cancel the run and end what remains past its grace. The start reports
    /// `Gone` once the agent and everything it started have ended.
    Cancel {
        agent: Token,
    },
}

/// What the agent capability reports back to the host. Its root translates
/// the application agent's own vocabulary into these events.
#[derive(PartialEq, Eq, Debug)]
pub enum FromAgent {
    /// A named host or delivery call with arguments.
    Called { owner: Token, call: Box<[u8]>, ask: Ask },
    /// The agent withdrew its named call.
    Withdrawn { owner: Token, call: Box<[u8]> },
    /// One completed conversation turn from the agent.
    Turn { owner: Token, turn: Turn },
    /// A piece of live content (domain/hosts/hub.md, 5).
    Piece { owner: Token, piece: crate::Piece },
    /// Typed agent activity, kept best effort for the engine.
    Activity { owner: Token, activity: crate::AgentActivity },
    /// The provider rejected the account's credential of `generation`.
    Unauthorized { owner: Token, account: u32, generation: u64 },
    /// The account reached its provider limit; `reset`, when the provider
    /// gave one, is how long until it lifts.
    Limited { owner: Token, account: u32, reset: Option<Duration> },
    /// The provider refused the account, with its code and reason.
    AccountRefused { owner: Token, account: u32, code: Box<[u8]>, reason: Box<[u8]> },
    /// Last word, with cumulative accounting.
    Finished { owner: Token, turns: u32, spent: u64, read: Option<Token>, finish: Finish },
    /// The agent started, and `agent` names it from now on.
    Started { owner: Token, agent: Token },
    /// The agent refused its start, before any run began.
    StartRefused { cause: StartCause, owner: Token, why: Box<[u8]> },
    /// The agent had no room for the run's start, before any run began.
    StartBusy { owner: Token },
    /// The start carries a wire version the worker cannot read.
    StartVersion { owner: Token, mismatch: Mismatch },
    /// The agent could not take an inbound event for the run.
    Refused { owner: Token, name: Token, reason: MessageRefusal },
    /// The run waits for its next inbound event.
    Yielded { owner: Token },
    /// The agent capability is stopping the agent for `fault`.
    Faulted { owner: Token, fault: AgentFailure },
    /// Terminal for `Start`: the agent and everything it started are gone.
    /// `detail` is for operators, such as the tail of its error output.
    Gone { owner: Token, detail: Box<[u8]> },
}

/// The agent's complete start (domain/hosts/README.md, sections 2 and 7).
#[derive(PartialEq, Eq, Debug)]
pub struct Start {
    pub live: crate::LiveAsk,
    pub workspace: Option<Token>,
    pub charter: Box<[u8]>,
    pub activation: u64,
    pub identity: [u8; 16],
    pub turns: Box<[Box<[u8]>]>,
    pub answered: Box<[AnsweredCall]>,
    pub messages: Box<[StartMessage]>,
    pub grants: Box<[Grant]>,
}

/// A start's inbox message, oldest first (domain/hosts/README.md, section 7).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct StartMessage {
    pub name: Token,
    pub sender: Box<[u8]>,
    pub words: Box<[u8]>,
    pub finish_requested: bool,
}

/// What the core gives the host for one run (domain/hosts/README.md, section 2).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct RunAssignment {
    pub identity: [u8; 16],
    /// The engine's names for the run and for this attempt at it.
    pub run: Token,
    pub attempt: Token,
    /// Nothing when the run has no workspace items to prepare.
    pub workspace: Option<Workspace>,
    /// Whether the workspace must save unfinished work before release.
    pub save: bool,
    pub live: crate::LiveAsk,
    /// What the agent's run is given, passed through.
    pub charter: Box<[u8]>,
    pub grants: Box<[Grant]>,
}

/// An assignment with the conversation state the agent consumes (domain/hosts/README.md, 2).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Assignment {
    pub messages: Box<[StartMessage]>,
    pub assignment: RunAssignment,
    /// Ordered committed turn bodies. An empty list starts a fresh conversation.
    pub turns: Box<[Box<[u8]>]>,
    /// Calls settled after the last committed turn, in commit order.
    pub answered: Box<[AnsweredCall]>,
}

/// A call committed after the last turn, to tell the next activation.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct AnsweredCall {
    /// The run's opaque call name.
    pub name: Box<[u8]>,
    /// The host tool's name as declared by the charter.
    pub tool: Box<[u8]>,
    pub answer: SettledAnswer,
}

/// The committed answer to a host call or workspace delivery.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SettledAnswer {
    /// A host tool's answer, including whether it is an error.
    Host { error: bool, body: Box<[u8]> },
    /// A workspace delivery's outcome and opaque, versioned evidence for replay.
    Delivery { outcome: DeliveryOutcome, evidence: Box<[u8]> },
}

/// A turn's cumulative spend and last read message name are opaque accounting.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Turn {
    pub opens_window: bool,
    pub refused: Box<[Refused]>,
    pub turn: u32,
    pub spent: u64,
    pub read: Option<Token>,
    pub body: Box<[u8]>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Finish {
    Ended { outcome: Box<[u8]> },
    Parked,
    Failed { failure: RunFailure },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RunAnswer {
    pub read: Option<Token>,
    pub refused: Box<[Refused]>,
    pub turns: u32,
    pub spent: u64,
    pub ending: Ending,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ending {
    Refused(Refusal),
    Ended { outcome: Box<[u8]>, work: Work },
    Parked { work: Work },
    Failed { failure: Failure, detail: Box<[u8]>, work: Work },
}

/// The application's workspace items, held by its root.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Workspace {
    /// The task whose runs may reuse the workspace.
    pub workstream: u64,
    /// The root's token for the application's bounded workspace items.
    pub items: Token,
}

/// A host call of a run.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Keep the workspace unchecked, with a delivery terminal.
    Save,
    /// A host tool call, carrying its charter name, write flag, input and time left.
    Relay {
        tool: Box<[u8]>,
        writes: bool,
        input: Box<[u8]>,
        deadline: Duration,
    },
    Deliver {
        title: Box<[u8]>,
        body: Box<[u8]>,
    },
}

/// The answer to a host call.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum CallAnswer {
    /// The engine's answer to a relayed call, as it is.
    Relayed { error: bool, text: Box<[u8]> },
    /// How the delivery went.
    Delivered(Delivery),
    /// The run is cancelled or ending: nothing was done. A delivery in flight as
    /// the run leaves live is waited for, and answered with how it went.
    Unavailable,
    /// The run withdrew the relayed call: nothing more is done for it, and
    /// the engine's answer, if one comes, is dropped.
    Withdrawn,
    /// The run has as many calls in flight as it may, or a delivery in flight
    /// already: nothing was done. Calls answered within the loop's current
    /// iteration keep their slots until its reclaim point, so a busy call may
    /// find room in the next.
    Busy,
}

/// A workspace's delivery outcome and the root's token for what the run left.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Delivery {
    pub outcome: DeliveryOutcome,
    pub left: Token,
    /// Whether the workspace changed even if the delivery reported a failure.
    pub changed: bool,
}

/// The result of making a delivery through an application's workspace.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeliveryOutcome {
    Delivered,
    Nothing,
    Stale,
    Refused,
    Failed,
}

/// Why an inbound event was not passed on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MessageRefusal {
    /// It holds more bytes than the limits allow.
    TooLarge,
    /// The run is not live yet, and holds as many events as it may.
    Full,
    NameInUse,
    /// The run is ending.
    Ending,
}

/// The root's tokens for what a run delivered and saved on its workspace.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Work {
    pub left: Option<Token>,
    pub saved: Option<Token>,
}

/// Why an assignment was refused.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    /// A payload carries a wire version this worker cannot read.
    Version(Mismatch),
    /// Every slot is taken (by a run hosted, or one whose answer the engine
    /// has yet to acknowledge), the run is hosted already under another
    /// attempt that has not answered yet, or the host is shutting down. A
    /// later retry, on this host or another, may find room.
    Busy,
    /// The assignment does not fit the limits.
    Invalid(Invalid),
}

/// The wire and both sides' versions (protocol/channel/wires.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Mismatch {
    pub wire: Box<[u8]>,
    pub version: u16,
    pub lowest: u16,
    pub highest: u16,
}

impl Mismatch {
    /// The operator reason shared by the retry notice and the engine's log.
    #[must_use]
    pub fn reason(&self) -> Box<[u8]> {
        let version = skein_lib::Decimal::of(u64::from(self.version));
        let lowest = skein_lib::Decimal::of(u64::from(self.lowest));
        let highest = skein_lib::Decimal::of(u64::from(self.highest));
        let parts: [&[u8]; 8] = [
            b"version mismatch: ",
            &self.wire,
            b" ",
            version.as_bytes(),
            b", the worker reads ",
            lowest.as_bytes(),
            b"..",
            highest.as_bytes(),
        ];
        let mut length = 0_usize;
        for part in parts {
            length = length.checked_add(part.len()).expect("bounded mismatch reason");
        }
        let mut writer = skein_lib::Writer::new(length);
        for part in parts {
            writer.put(part).expect("measured mismatch reason");
        }
        writer.finish()
    }
}

/// What about an assignment does not fit the limits.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Invalid {
    /// The agent refused the start; its words are for operators.
    Start {
        cause: StartCause,
        why: Box<[u8]>,
    },
    /// The charter holds more bytes than a run may.
    Charter,
    Transcript,
    Grants,
    Messages,
}

/// Why a hosted run failed (domain/hosts/hub.md): what the engine acts on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Failure {
    /// Its workspace could not be prepared.
    Unprepared(Preparation),
    /// The run failed, as it reports it.
    Run(RunFailure),
    /// Its agent failed.
    Agent(AgentFailure),
    /// It was cancelled.
    Cancelled(Reason),
}

/// Why a workspace could not be prepared.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Preparation {
    /// A later attempt may work.
    Transient,
    /// A resource needs a change before another attempt can work.
    Permanent { resource: Option<Token> },
}

/// Why a run failed, as it reports it (domain/hosts/hub.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RunFailure {
    /// A configured bound refused the run's model call (domain/hosts/hub.md, 3).
    Exceeded(Exceeded),
    /// The LLM could not do the work.
    Model,
    /// The run's budget ran out.
    Budget,
    /// The LLM did not keep to the run's rules.
    Policy,
    /// The run was cancelled.
    Cancelled,
    /// The run's target changed since it started.
    Stale,
    /// The run's account reached its provider limit; `reset`, when the
    /// provider gave one, is how long until it lifts.
    Limited { account: u32, reset: Option<Duration> },
    /// The provider refused the run's account.
    Refused { account: u32 },
    /// The saved history cannot be resumed; prepare afresh with its tail.
    Transcript,
}

/// How an agent failed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AgentFailure {
    /// It could not be started.
    Unstarted,
    /// It exited without saying how its run finishes.
    Exited,
    /// It broke the channel's rules, or said more than the limits allow.
    Rules,
    /// The watchdog stopped it: no progress.
    NoProgress,
    /// The watchdog stopped it: past its wall time.
    WallTime,
}

/// Why a run was cancelled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Reason {
    /// The engine cancelled it.
    Engine,
    /// A worker root lost contact with the core for longer than the grace.
    Contact,
    /// The root is shutting down.
    Shutdown,
}

/// A message refusal carried by its next turn or answer (domain/hosts/hub.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Refused {
    pub name: Token,
    pub reason: MessageRefusal,
}

/// Why the agent refused a start, independent of its diagnostic bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StartCause {
    Other,
    Grants,
    /// Smith refused the saved history before activation.
    Transcript,
}

/// A configured bound a run's model call met, and that bound's configured
/// value in its own unit: never a measured size.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Exceeded {
    pub bound: Bound,
    pub value: u64,
}

/// A configured bound on a model call.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Bound {
    /// The model's reasoning, per item or in all: bytes.
    Reasoning,
    /// The model's other replay metadata: bytes.
    Metadata,
    /// The items of the model's answer: a count.
    Items,
    /// The model's answer: bytes.
    Answer,
    /// The tools offered to the model: a count.
    Tools,
    /// The messages and blocks one request carries: a count of both.
    HistoryItems,
    /// The run's retained history, which compaction could not bring under
    /// one of its configured ceilings: on messages, on blocks or on bytes.
    /// The value is that ceiling, and the bound doesn't name its unit.
    History,
    /// The model's window: tokens.
    Window,
    /// The model's declared output ceiling: tokens.
    Output,
}

impl Event {
    /// Whether this input starts work, rather than settling admitted work (domain/hosts/hub.md).
    #[must_use]
    pub fn new_work(&self) -> bool {
        match self {
            Event::Agent(event) => event.new_work(),
            Event::Opened { .. } => true,
            Event::Down(event) => match event {
                crate::LinkDown::ReadList { .. }
                | crate::LinkDown::Sources { .. }
                | crate::LinkDown::Check { .. }
                | crate::LinkDown::SignOut { .. }
                | crate::LinkDown::AssignJob { .. }
                | crate::LinkDown::Assign { .. }
                | crate::LinkDown::Message { .. } => true,
                crate::LinkDown::Relayed { .. }
                | crate::LinkDown::TurnKept { .. }
                | crate::LinkDown::Acknowledge { .. }
                | crate::LinkDown::Cancel { .. }
                | crate::LinkDown::Grant { .. } => false,
            },
            Event::ListRead { .. }
            | Event::Job(_)
            | Event::SourceAnswered { .. }
            | Event::SourceLost
            | Event::RelayCancelled { .. }
            | Event::Drain
            | Event::CancelAll { .. }
            | Event::Prepared { .. }
            | Event::Unprepared { .. }
            | Event::Delivered { .. }
            | Event::Saved { .. } => false,
        }
    }
}
impl FromAgent {
    /// Calls start new work; other reports settle hosted work (domain/hosts/hub.md).
    #[must_use]
    pub const fn new_work(&self) -> bool {
        match self {
            FromAgent::Called { .. } => true,
            FromAgent::Withdrawn { .. }
            | FromAgent::Turn { .. }
            | FromAgent::Piece { .. }
            | FromAgent::Activity { .. }
            | FromAgent::Unauthorized { .. }
            | FromAgent::Limited { .. }
            | FromAgent::AccountRefused { .. }
            | FromAgent::Finished { .. }
            | FromAgent::Started { .. }
            | FromAgent::StartRefused { .. }
            | FromAgent::StartBusy { .. }
            | FromAgent::StartVersion { .. }
            | FromAgent::Refused { .. }
            | FromAgent::Yielded { .. }
            | FromAgent::Faulted { .. }
            | FromAgent::Gone { .. } => false,
        }
    }
}

/// What the core gives the host for one job (jig/docs/design/domain/hosts/hub.md, 9).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct JobAssignment {
    /// The engine's names for the job task and this attempt at it.
    pub run: Token,
    pub attempt: Token,
    pub kind: JobKind,
    /// The application's bytes, carried and never read.
    pub spec: Box<[u8]>,
    pub deadline: Duration,
    /// Nothing when the job has no workspace items.
    pub workspace: Option<Workspace>,
    /// The application's credentials its workspace needs: `Form::Application` only.
    pub grants: Box<[Grant]>,
}

/// How a hosted job ended, as the hub answers it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum JobEnding {
    Refused(JobRefusal),
    Finished { result: Box<[u8]>, work: Work },
    Failed { failure: JobFailure, detail: Box<[u8]>, work: Work },
}

/// Why a hosted job failed (jig/docs/design/domain/hosts/hub.md, 9): what the engine acts on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JobFailure {
    /// Its workspace could not be prepared.
    Unprepared(Preparation),
    /// It ended and said it failed.
    Failed,
    /// Its deadline passed: it was stopped and its processes killed.
    Deadline,
    /// Its result holds more bytes than its kind's bound.
    Result,
    /// Its processes could not be started, or ended without its capability's word.
    Unstarted,
    /// What it made was not delivered: `Stale`, `Refused` or `Failed`, as the workspace said.
    Undelivered(DeliveryOutcome),
    Cancelled(Reason),
}

/// Why a job's assignment was refused at the entrance; a run's `Refusal` is unchanged.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum JobRefusal {
    /// As a run's `Refusal::Busy`.
    Busy,
    Invalid(JobInvalid),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum JobInvalid {
    /// A kind this host does not run.
    Kind,
    /// More spec bytes than its kind's bound.
    Spec,
    /// A deadline longer than its kind's bound.
    Deadline,
    /// Its grants are not the application's, or more than the limits allow.
    Grants,
    /// The job capability refused its start; its words are for operators.
    Start { why: Box<[u8]> },
}

/// host -> parent, inside `Request::Job(ToJob)`.
#[derive(PartialEq, Eq, Debug)]
pub enum ToJob {
    /// Run the job until it ends or its deadline passes, in `workspace` if it has one.
    Run { owner: Token, kind: JobKind, spec: Box<[u8]>, deadline: Duration, workspace: Option<Token> },
    /// Stop it at once and kill its processes. `Gone` follows.
    Stop { owner: Token },
}

impl ToJob {
    /// What a root without a job capability answers (domain/hosts/hub.md, 9).
    #[must_use]
    pub fn refused(self) -> FromJob {
        match self {
            Self::Run { owner, .. } => FromJob::StartRefused { owner, why: Box::from(&b"no job capability"[..]) },
            Self::Stop { owner } => FromJob::Gone { owner, detail: Box::new([]) },
        }
    }
}

/// parent -> host, inside `Event::Job(FromJob)`.
#[derive(PartialEq, Eq, Debug)]
pub enum FromJob {
    /// Its first process started.
    Started { owner: Token },
    /// The capability refused to start it, before anything ran; nothing of it is live.
    StartRefused { owner: Token, why: Box<[u8]> },
    /// How it ended. At most once per job; a stopped job may end with nothing said.
    Ended { owner: Token, end: JobEnd },
    /// Terminal for `Run`: every process of the job is gone. `detail` is for operators.
    Gone { owner: Token, detail: Box<[u8]> },
}

#[derive(PartialEq, Eq, Debug)]
pub enum JobEnd {
    /// It ended well, with its result and, for a kind that delivers, what to deliver: the
    /// application's bytes, which the hub hands the workspace as they are.
    Finished { result: Box<[u8]>, delivery: Option<Box<[u8]>> },
    /// It ended and said it failed; `detail` is for operators.
    Failed { detail: Box<[u8]> },
    /// Its deadline passed and it was killed.
    Deadline,
}
