//! The core's vocabulary toward its root (`domain/engine/README.md`, section 4).

use crate::{CallKey, CallPart, Delegate, EscalationDecisionRecord, Key, ProcedureStep, Record, RunCharter};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_fleet as fleet;
use jig_domain_link::{SignOutResult, SourceName};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{List, ReplyTo, Token, Wall};

/// The root's assembled, worker assignment.
#[derive(Hash, Debug)]
pub struct Assignment {
    pub live: crate::LiveAsk,
    /// Ordered sections selected by the brief child, typed until protocol routing.
    pub brief: Box<[jig_domain_brief::GatherPlaced]>,
    /// The conversation's first attempt, shared by its resumes.
    pub conversation: u64,
    /// Claimed task and activation fence.
    pub task: u64,
    pub attempt: u64,
    /// Charter and task-specific policy and contract.
    pub charter: u32,
    pub run: Box<RunCharter>,
    /// Whole words and prior committed conversation offered to the worker.
    pub messages: Box<[tasks::Word]>,
    pub transcript: Box<[Box<[u8]>]>,
    pub answered: Box<[crate::Settled]>,
    /// The selected account and application credential grants.
    pub grants: Box<[jig_domain_link::Grant]>,
}

/// A charter document's section in a core plan, for the root to give its connector
/// (domain/engine/settings.md, 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PlannedDocument {
    pub token: Token,
    pub document: crate::Document,
}

/// A value a client's request carried, kept in the secrets' store before
/// the core sees the request (`protocol/client/README.md`, section 3): the core
/// gets its number and digest, never the value.
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub struct Carried {
    pub number: u64,
    pub digest: [u8; 32],
}

/// A request from the core to its application root. Its variant fixes the
/// commit relationship before the root translates or wraps it.
#[derive(Hash, Debug)]
pub enum Request {
    /// A new project; the root holds this notification until the decision commits.
    ProjectMade { project: u32 },
    /// Store mutation in the current decision.
    Write(Write),
    /// Synchronous connector handoff within this decision.
    Ask { connector: u16, ask: Ask },
    /// Output held until this decision is durable.
    Held(Box<Held>),
    /// Read answer or refusal that decided nothing.
    Now(Box<Now>),
}

/// A question or retained-section command for a numbered connector.
#[derive(Hash, Debug)]
pub enum Ask {
    Catalogue {
        models: Box<[crate::settings::CatalogueModel]>,
    },
    /// Release every adoption this connector holds for a project being deleted
    /// (jig/docs/design/domain/people/acts.md, 2). Answered by
    /// `Event::ProjectReleased` once the release commits; at once when none are held.
    /// Sent again at each start until answered, so a connector releases idempotently.
    ReleaseProject {
        project: u32,
    },
    /// The complete procedure step committed with these spec revisions.
    StepMade {
        task: u64,
        step: u64,
        first: Option<u64>,
        revisions: Box<[(u64, u64)]>,
    },
    /// No part of the procedure step committed.
    StepRefused {
        task: u64,
        step: u64,
        why: StepRefusal,
    },

    /// Discard the connector-owned binding of a refused application watch or read.
    DropDocument {
        subject: Token,
    },
    /// Make a connector's current document, then complete the views' Make within this decision.
    Document {
        watcher: Token,
        scope: jig_domain_people::ApplicationScope,
        subject: Token,
        party: u64,
        window: crate::documents::Window,
    },
    /// Answer an admitted connector read; the root holds its typed answer in this decision.
    Read {
        reply: ReplyTo,
        scope: jig_domain_people::ApplicationScope,
        read: Token,
        party: u64,
        window: crate::documents::Window,
    },
    /// Decide the admitted connector-owned value in this decision and retain its outcome.
    Application {
        request: Token,
        token: Token,
        owner: Token,
        person: u64,
        scope: jig_domain_people::ApplicationScope,
        role: Option<people::Role>,
        retention: skein_lib::Duration,
    },
    /// Discard a prepared value after admission; deciding consumes it first.
    DropApplication {
        owner: Token,
    },
    /// The party's core part completed, with the same facts as a direct ask.
    ApplicationPartAnswered {
        request: Token,
        outcome: people::Outcome,
    },
    /// A core-selected effect handoff completed inside this decision.
    Effect(crate::effects::connector::Ask),
    /// Tell the owning connector that this task took a write hold.
    Hold {
        task: u64,
        resource: tasks::Name,
        /// The taking's own outbox entry, for the effects its connector's kind has at a taking
        /// (domain/connectors.md, 4.7).
        entry: u64,
    },
    /// Tell the owning connector to end a task's topic interest.
    EndTopic {
        task: u64,
        subscription: u64,
    },
    /// Gather a section within the brief's budget.
    Gather {
        section: Token,
        budget: u32,
    },
    /// Cut a retained section to a smaller bound.
    CutTo {
        section: Token,
        size: u32,
    },
    /// Release a retained section.
    Drop {
        section: Token,
    },
    /// Offer an authorized party's resource to its owning connector.
    Adopt {
        request: Token,
        project: u32,
        adoption: people::Adoption,
    },
    /// Settle a closing task's connector effects before its final release.
    Close {
        task: u64,
        root: u64,
        ending: tasks::Ending,
    },
    /// Release a closing task's connector holds through a numbered outbox entry.
    Release {
        task: u64,
        root: u64,
        ending: tasks::Ending,
        entry: u64,
    },
    /// Tell a connector that a fenced host attempt was lost.
    Lost {
        task: u64,
        attempt: u64,
    },
    /// Ask this connector which bounded task holdings its resources require.
    TaskHoldings {
        request: Token,
        project: u32,
        root: u64,
        number: u64,
        executor: tasks::Executor,
        spec: tasks::Spec,
        tracked: bool,
        /// This connector's symbols of the member, in the member's order.
        symbols: Box<[u16]>,
    },
    /// Ask one connector for the holdings of every member of a named delegate batch.
    DelegateHoldings {
        request: Token,
        from: u64,
        members: Box<[HoldingsNeed]>,
    },
    /// Ask one connector for a procedure's delegated batch holdings.
    ProcedureHoldings {
        task: u64,
        step: u64,
        members: Box<[HoldingsNeed]>,
    },
    /// Project the complete bounded tree and newly recorded history once per decision.
    ProjectGoal {
        feed: Box<crate::ProjectionFeed>,
    },
    /// Release a goal's projection state after every closing write has settled.
    ForgetProjection {
        goal: u64,
    },
    /// Complete a connector-owned subscription after the task hub accepts it.
    SubscriptionDone {
        request: Token,
        key: CallKey,
        subscription: u64,
    },
    /// Complete a connector-owned unsubscription after the task hub accepts it.
    UnsubscriptionDone {
        request: Token,
        key: CallKey,
    },
    /// Complete a prepared claim through the connector that owns its writes.
    ClaimDone {
        task: u64,
        attempt: u64,
        kinds: fleet::Kinds,
        version: fleet::Version,
    },
    /// Start a claimed job after its decision is durable (jig/docs/design/domain/engine/executors.md, 5).
    JobClaimed {
        task: u64,
        attempt: u64,
        kind: jig_domain_link::JobKind,
        spec: Box<[u8]>,
        deadline: skein_lib::Duration,
        result: u32,
        grants: Box<[jig_domain_link::Grant]>,
        workspace: Option<Box<[u8]>>,
    },
    /// Discard one rejected connector subscription flight.
    DropSubscription {
        request: Token,
    },
    /// Ask a connector whether a refused repair belongs to a held task.
    RepairRefused {
        repair: Option<u64>,
    },
    /// Let a connector settle a refused procedure delegation.
    DelegateRefused {
        task: Option<u64>,
    },
    /// Start one admitted procedure through its numbered connector.
    StartProcedure {
        context: Box<tasks::RunContext>,
        step: u64,
    },
    /// Ask a connector for a public document (jig/docs/design/domain/engine/views.md).
    Public {
        reply: ReplyTo,
        kind: u16,
        version: u16,
        project: u32,
        task: Option<u64>,
        public: Box<[u64]>,
    },
}

/// Store mutation issued by a child of the core.
#[derive(Hash, Debug)]
pub enum Write {
    /// Drop a task transcript whole with its places (jig/docs/design/protocol/store/transcripts.md, 5, jig/docs/design/protocol/store/README.md, 6.1;
    /// jig/docs/design/domain/people/acts.md, 2).
    DropTranscript { task: u64 },
    /// Copy the old held revision to history without materialising it in the core.
    CopySettingsRevision(u32),
    /// Save one authentic core or child record.
    Save(Record),
    /// Erase one authentic core or child key.
    Erase(Key),
}

/// Core output that must follow the decision's commit.
#[derive(Hash, Debug)]
pub enum Held {
    /// Terminal deployment act answer, released once its decision is durable.
    Acted { to: ReplyTo, outcome: people::Outcome },
    /// Every earlier commit is durable when the journal releases this terminal.
    Drained,
    /// A point load sent once every earlier commit is durable.
    Load(Box<Now>),
    /// Assemble the connector-owned document prepared for this delivery in the same decision.
    ApplicationDelivery { watcher: Token, connector: u16, subject: Token },
    /// Drop the connector-owned subject binding and deliver its paced ending.
    ApplicationWatchEnded { watcher: Token, connector: u16, subject: Token, why: views::End },
    /// Load an owner range only after every preceding commit is durable.
    ReadLoad { load: Token, range: Box<crate::reads::Range> },
    /// One authenticated read answer after every preceding commit is durable.
    Read { reply: ReplyTo, answer: crate::reads::Answer },
    /// The root assembles the outcome retained by its numbered connector.
    Applied { to: ReplyTo, connector: u16, token: Token },
    /// A request's carried value went unused (refused, sent again, joined
    /// or stopped): the clients' component erases it, if its own decode
    /// kept it, before the request's answer (`protocol/store/secrets.md`).
    CarriedUnused { number: u64 },
    /// Current subject document, held until every preceding commit is durable.
    Delivery { watcher: Token, document: Box<crate::documents::Document> },
    /// A watch has ended after every outstanding terminal.
    WatchEnded { watcher: Token, why: views::End },

    /// A connector may make this entry only after its keeping decision commits.
    MakeEffect { connector: u16, entry: u64 },
    /// A rendered answer and all opaque evidence have reached the store.
    SettledCall { to: ReplyTo, key: CallKey, call: crate::SettledCall },
    /// A claimed host receives the parent's conversation references.
    Assign { channel: Token, run: Token, attempt: Token, activation: u64, assignment: fleet::Assignment },
    /// Read an account's list on this channel (jig/docs/design/protocol/channel/records.md, 2).
    ReadList { channel: Token, read: u64, endpoint: u32, grant: jig_domain_link::Grant },
    /// Replace the connected worker's whole program-source list after commit.
    Sources { channel: Token, sources: Box<[SourceName]> },
    /// Request a fresh observation of one named program source on this channel.
    CheckSource { channel: Token, name: SourceName },
    /// Ask this channel to sign out one source, without retrying after loss.
    SignOutSource { channel: Token, name: SourceName },
    /// End one source sign-out; None means the channel could not confirm it.
    SourceSignedOut { to: ReplyTo, channel: Token, name: SourceName, result: Option<SignOutResult> },
    /// A grant follows every prior commit and its attempt's assignment (domain/engine/README.md, 5.2).
    Grant { channel: Token, run: Token, attempt: Token, grant: jig_domain_link::Grant },
    /// The host receives an answer under its original opaque call name.
    Relayed { channel: Token, run: Token, attempt: Token, call: Box<[u8]>, answer: Token },
    /// The host receives the same named label and words the root supplied.
    Inbound { channel: Token, run: Token, attempt: Token, message: fleet::Message },
    /// A saved word relayed to its fenced live attempt after commit.
    Relay { task: u64, attempt: u64, previous: Option<u64>, word: tasks::Word },
    /// A party's keyed terminal, released after its decision is durable.
    PeopleReply { to: ReplyTo, reply: people::Reply },
    /// A named core call answer, assembled and delivered after its decision commits.
    CallAnswer { to: ReplyTo, key: CallKey, part: CallPart },
    /// An ended task's result is delivered to its person requester.
    Result { person: u64, task: u64, words: Box<[u8]> },
    /// A host may forget its terminal only after the task decision commits.
    Acknowledge { channel: Token, run: Token, attempt: Token },
    /// The fleet may forget an accepted task terminal after its decision commits.
    TaskTerminalAcknowledged { task: u64, attempt: u64 },
    /// The fleet may forget one accepted turn after its decision commits.
    TaskTurnKept { task: u64, attempt: u64, turn: u32 },
    /// A host may forget its turn only after that turn commits.
    AcknowledgeTurn { channel: Token, run: Token, attempt: Token, turn: u32 },
    /// Fence and cancel one hosted attempt after the decision commits.
    Cancel { channel: Token, run: Token, attempt: Token },
    /// Refuse one host request after the decision commits.
    Refuse { channel: Token, hello: Option<fleet::HelloRefused> },
    /// Ask the host to retain and retry a turn that did not enter a decision.
    TurnBusy { channel: Token, run: Token, attempt: Token, turn: u32 },
    /// Cancel the current fenced run after its task decision commits.
    StopRun { task: u64, attempt: u64 },
    /// Store page requested by the notes child after earlier commits.
    NotesLoad { owner: Token, range: notes::Range },
    /// A note write's accepted revision.
    NotesWritten { owner: Token, name: u64, revision: u32 },
    /// A note deletion's accepted name.
    NotesDeleted { owner: Token, name: u64 },
    /// A public read's answer after durability (jig/docs/design/domain/people/public-reading.md, 10).
    PublicRead { reply: ReplyTo, answer: crate::PublicAnswer },
    /// Recheck a public read after preceding commits (jig/docs/design/domain/people/public-reading.md, 10).
    PublicWait { read: Token },
}

/// Why an unkeyed watch was not admitted (`client/domain/contract/README.md`, 5.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WatchRefusal {
    /// Its sign-in is absent or expired.
    SignedOut,
    /// Its party may not see the subject.
    NotAllowed,
    /// No live watch subject exists by that name.
    Unknown,
    /// The configured watch or document bound is full.
    Busy,
}

/// The core's outputs that are the link's (domain/root.md, 5).
#[derive(Hash, Debug)]
pub enum LinkNow {
    /// The host receives a read answer without waiting for a commit (domain/engine/README.md, 4.4).
    Relayed { channel: Token, run: Token, attempt: Token, call: Box<[u8]>, answer: Token },
    /// Forget a root-owned host payload that the fleet no longer needs.
    DropPayload { payload: Token },
    /// Read the root-owned turn payload before routing the charged turn.
    TurnPayload { run: Token, attempt: Token, turn: u32, body: Token },
    /// Read and translate the root-owned answer payload before the task terminal.
    AnswerPayload { run: Token, attempt: Token, payload: Token },
    /// Read the root-owned transcript after the task accepts its charge.
    AcceptedTurn { payload: Token, task: u64, attempt: u64, turn: u32, accepted: tasks::Accepted },
    /// Inspect a root-owned host payload after the task child refuses it.
    RefusedPayload { request: Token, problem: tasks::Problem },
}

/// Core output that follows no mutation.
#[derive(Hash, Debug)]
pub enum Now {
    /// The core's outputs for the link (domain/root.md, 5).
    Link(LinkNow),
    /// A bounded task read has no durable call record (domain/engine/executors.md, 3).
    TaskRead { to: ReplyTo, key: CallKey, answer: Box<Result<crate::reads::run_task::TaskRead, tasks::Refusal>> },
    /// A settings document leaves without a durable call record (domain/engine/executors.md, 3).
    SettingsAnswer { to: ReplyTo, key: CallKey, answer: Box<crate::reads::SettingsAnswer> },
    /// A run's read as its person answered: the party read's own answer, `None(Why::Unknown)`
    /// for what is not admitted, `None(Why::Busy)` where the loads are full (domain/engine/executors.md, 3).
    PersonAnswer { to: ReplyTo, key: CallKey, answer: Box<crate::reads::Answer> },
    /// A run's proposal read answered (domain/engine/executors.md, 3).
    ProposalRead { to: ReplyTo, key: CallKey, answer: Box<crate::reads::ProposalRead> },
    /// The root answers this host call with its own busy payload.
    CallBusy { to: ReplyTo },
    /// An effect wait or capacity refusal changed no durable state.
    EffectAnswer { to: ReplyTo, key: CallKey, part: CallPart },
    /// A settled answer exceeded its bounded record or no retained call owns it.
    SettledCallRefused { to: ReplyTo, key: CallKey, name: Box<[u8]>, tool: Box<[u8]> },
    /// The protocol layer decodes this attested tool and input at the root boundary.
    Call { to: ReplyTo, run: Token, attempt: Token, call: fleet::Call },
    /// A current host's call found no room; only its name crosses.
    RelayBusy { channel: Token, run: Token, attempt: Token, call: Box<[u8]> },
    /// A fenced call or an overlong name changed nothing.
    DropCall { call: fleet::Call },
    /// The host did not take this message; its durable inbox word remains unread.
    Undelivered { run: Token, attempt: Token, message: fleet::Message },
    /// A sign-in whose core-owned person or session counter is exhausted.
    SignInRefused { to: ReplyTo },
    /// A volatile watch refused before any durable decision.
    WatchRefused { watcher: Token, why: WatchRefusal },
    /// Secret-free account protocol output with no store decision.
    Account(accounts::Request),
    /// A provisional run suffix leaves through the journal door, without a commit.
    RunText {
        watcher: Token,
        turn: views::TurnName,
        from: u32,
        to: u32,
        text: Box<[u8]>,
        cut: bool,
        call: Option<views::Call>,
        writing: Option<views::WritingCall>,
    },
    /// One page of scope index lines.
    NotesIndexed { owner: Token, lines: List<notes::Line>, more: u32 },
    /// One page of recalled entries.
    NotesRecalled { owner: Token, entries: List<notes::Entry>, more: bool },
    /// A note request refused before changing state.
    NotesRefused { owner: Token, why: notes::Refusal },
    /// The notes child could not serve this named call yet; the host retries its name.
    NoteBusy { to: ReplyTo, key: CallKey },
    /// Forget the application's assignment for a run the host never started.
    DropAssignment { task: u64 },
    /// Ask the root for its current readiness before a due activation route.
    Activate { context: Box<tasks::RunContext> },
    /// Load the current transcript window in key order from its first turn.
    StartPreparation { task: u64, from: (u64, u32) },
    /// Load one proposal decision that has left live task memory.
    SeenConversation { request: Token, task: u64 },
    /// Load the durable position of one ended conversation for its mark.
    LoadEndedRead { request: Token, task: u64, party: u64 },
    /// Load one proposal decision that has left live task memory.
    EndedProposal { request: Token, person: u64, project: u32, proposer: people::Proposer, proposal: u64 },
    /// Load a task proposal from the proposer's retained history.
    DecidedProposal { to: ReplyTo, key: CallKey, proposal: u64 },
    /// Load the ended owner row for a person-task act whose live row is gone.
    EndedPersonTask { request: Token, person: u64, project: u32, task: u64 },
    /// Load an ended task row; the core retains why it needs the result.
    EndedTask { request: Token, task: u64 },
    /// Load the first input or ended tree root; later inputs return to the core between gets.
    PartyInputs { request: Token, person: u64, project: u32, tasks: Box<[u64]> },
    /// Assemble connector-held sections for a still-current brief.
    CompleteBrief { brief: Token, order: Box<[brief::GatherPlaced]> },
    /// Core-owned sections and charter documents are ready; the root may append its connector
    /// sections (domain/engine/settings.md, 9).
    BriefCorePlanned {
        task: u64,
        parent: Option<u64>,
        sections: List<brief::Planned>,
        documents: Box<[PlannedDocument]>,
    },
    /// The root assembles a numbered connector workspace for this core-selected attempt.
    WorkspaceRequest { task: u64, attempt: u64, context: Box<tasks::RunContext> },
    /// Core admission and claim preparation failed; discard root-owned assembly parts.
    RunPreparationFailed { task: u64 },
    /// A brief failed, for the application's log (`domain/engine/briefs.md`, 1.1).
    BriefFailed { task: u64, failure: brief::GatherFailure },
    /// Core-owned run parts are ready for root assembly with the connector workspace.
    RunPrepared {
        live: crate::LiveAsk,
        task: u64,
        attempt: u64,
        conversation: u64,
        charter: u32,
        run: Box<RunCharter>,
        messages: Box<[tasks::Word]>,
        saved: Box<[tasks::SavedResource]>,
        transcript: Box<[Box<[u8]>]>,
        answered: Box<[CallKey]>,
        grants: Box<[jig_domain_link::Grant]>,
    },
    /// Load one held-chat decision that has left live task memory.
    EndedEscalation { request: Token, person: u64, project: u32, task: u64, revision: u64 },
    /// The first child admitted from a connector procedure's batch, if any.
    /// A restored task row was refused; the root must fail startup.
    RestoreRefused,
}

/// Limits of the core's child routes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub public: PublicIndexLimits,
    /// The most hand-offs one core decision routes, sibling continuations included: the
    /// bound of every drain within a step (skein's programming-model.md, 3 "Bounded"; 4.5).
    pub routes: u32,
    pub settings: crate::settings::Limits,
    /// Configured templates and overrides per project policy.
    pub templates: u32,
    /// Store map bytes at or above which new party work is refused (protocol/store/README.md, 6.2).
    pub fill_mark: u64,
    /// Number of numbered connectors retained by the core.
    pub connectors: u32,
    /// Bytes retained for one task's resumable transcript tail.
    pub resume_bytes: u32,
    /// Owned bytes of the configured run policy and its model alternatives.
    pub run_bytes: u32,
    /// Maximum encoded mutable policy row the application journal accepts.
    pub policy_bytes: u64,
    /// Bound on an ended person escalation rejection reason.
    pub escalation_reason_bytes: u32,
    /// Concurrent authenticated read loads, including retained result reads.
    pub load_slots: u32,
    /// Maximum owned bytes brought by one authenticated store read.
    pub load_bytes: u64,
    /// Named call replay and in-flight route slots.
    pub call_records: u32,
    /// Total bytes of a retained opaque call name, tool name and settled answer.
    pub call_answer_bytes: u32,
    /// The task hub's finite work.
    pub tasks: tasks::Limits,
    /// Policy tables and authority payload bounds owned by the core.
    pub authority: authority::Limits,
    /// Parties and their requests.
    pub people: people::Limits,
    /// Host slots and attempts.
    pub fleet: fleet::Limits,
    /// Brief gathering.
    pub brief: brief::Limits,
    /// Maximum fragments when rendering a core-owned brief section.
    pub brief_parts: u32,
    /// Core-owned section budgets; connector budgets stay with their owner.
    pub brief_core_budgets: CoreBriefBudgets,
    /// Secret-free account lifetimes.
    pub accounts: accounts::Limits,
    /// Scoped notes and their pages.
    pub notes: notes::Limits,
    /// Typed document construction bounds and served versions.
    pub documents: crate::documents::DocumentLimits,
    /// Live view streams.
    pub views: views::Limits,
}

/// Public index bounds (jig/docs/design/domain/people/public-reading.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PublicIndexLimits {
    pub goals: u16,
    pub ended: u16,
    pub lines: u16,
    pub depth: u16,
}
impl PublicIndexLimits {
    pub const OFF: Self = Self { goals: 0, ended: 0, lines: 0, depth: 0 };
}

/// Budgets for the core's own brief sections.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CoreBriefBudgets {
    /// Transcript tail text in a fresh run's brief.
    pub tail: u32,
    pub task: u32,
    pub dependencies: u32,
    pub attempts: u32,
    pub plan: u32,
    /// Maximum note index text in one brief.
    pub notes: u32,
    /// Whole preferences of the run's requester (domain/engine/briefs.md, 2.1).
    pub preferences: u32,
    /// A standing task's Now body (domain/tasks/standing.md, 6).
    pub now: u32,
}

/// One event routed to a child of the core.
#[derive(Hash, Debug)]
pub enum Event {
    /// A connector's release of a deleted project's adoptions has committed
    /// (jig/docs/design/domain/people/acts.md, 2).
    ProjectReleased { connector: u16, project: u32 },
    /// One model act at the newest revision (domain/engine/settings.md, 11).
    ChangeModel {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        digest: [u8; 32],
        act: crate::settings::catalogue::ModelAct,
    },
    /// Owner settings value held only while people admits its keyed digest.
    ChangeSettings {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        digest: [u8; 32],
        change: Box<crate::settings::Change>,
    },
    /// A whole proposed set, held only while people admits its keyed digest.
    ProposeSettings {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        digest: [u8; 32],
        change: Box<crate::settings::Proposed>,
    },
    /// Commit a restored release before opening; `room` is the settings restore's write budget.
    SettingsCommit { room: u32 },
    /// A connector's deployment act, without a party or sign-in (domain/people/acts.md, 3).
    Act { reply_to: ReplyTo, connector: u16, key: [u8; 16], act: Box<people::Act>, carried: Option<Carried> },
    /// Store map bytes from each commit answer; needs no commit (protocol/store/README.md, 6.2).
    Fill { bytes: u64 },
    /// Refuse new decisions while the service drains (domain/engine/restart.md, 1).
    Stopping,
    /// A connector's state change names its opaque subjects in this decision.
    Marked { connector: u16, subjects: List<Token> },
    /// Exactly one terminal for a previously admitted read load.
    ReadLoaded { load: Token, result: crate::reads::Loaded },
    /// An unkeyed one-shot read on the supplied live sign-in.
    Read { reply: ReplyTo, sign_in: u64, read: crate::reads::Read },
    /// Admit an application request by metadata; its owner retains the value.
    /// A value the request carried is named by its number and digest only.
    Application {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        scope: jig_domain_people::ApplicationScope,
        connector: u16,
        digest: [u8; 32],
        owner: Token,
        carried: Option<Carried>,
    },
    /// An LLM key ask whose value is already in the secrets' store. People keeps
    /// only the ask's digest; the accounts child receives the carried number.
    Key { reply_to: ReplyTo, sign_in: u64, key: [u8; 16], ask: Box<people::Ask>, carried: Carried },
    /// A connector asks the core within the admitted party decision.
    ApplicationPart { request: Token, ask: Box<people::Ask> },
    /// The connector durably kept its outcome, or could not decide this time.
    ApplicationFinished { request: Token, outcome: Result<Token, people::Again> },
    /// A connector's final projection write has settled, or its project has no home there.
    ProjectionSettled { goal: u64, connector: u16 },
    /// A protocol-rendered answer; its live reply address is never retained (protocol/store/records.md, 1).
    SettledCall { to: ReplyTo, key: CallKey, name: Box<[u8]>, call: crate::SettledCall },
    /// A connector owns the decoded effect under this transient owner.
    EffectStart { owner: Token, connector: u16, origin: crate::EffectOrigin },
    /// Connector handoffs and outbox outcomes in the core vocabulary.
    EffectConnector(crate::effects::connector::Event),
    /// Subscribe to an offered connector topic after checking every read resource.
    SubscribeTopic { to: ReplyTo, key: CallKey, connector: u16, topic: u64, resources: Box<[authority::Effect]> },
    /// Release effect answers whose absolute answer deadline has passed.
    EffectDeadline,
    /// Allocate the person and session named by a new party sign-in.
    SignIn { reply_to: ReplyTo, identity: people::Identity, held: Option<u64> },
    /// Admit a visible subject on a live sign-in, without a request key.
    Watch { watcher: Token, sign_in: u64, subject: crate::documents::Watched, window: crate::documents::Window },
    /// Follow a task's live run from the client's acknowledged turn and offset.
    WatchRun { watcher: Token, sign_in: u64, task: u64, turn: Option<views::TurnName>, offset: u32 },
    /// The client leaves one volatile watch.
    Unwatch { watcher: Token },
    /// The protocol took a watch's held document.
    Delivered { watcher: Token },
    /// The protocol took one provisional stream delivery.
    StreamDelivered { watcher: Token },
    /// A content call with its opaque source identity, independent of text.
    RunCall { run: Token, attempt: Token, call: views::Call },
    /// A main completion's current tool input, before its turn commits.
    RunInput {
        run: Token,
        attempt: Token,
        turn: u32,
        retry: u32,
        offset: u32,
        id: Box<[u8]>,
        tool: Box<[u8]>,
        input: Box<[u8]>,
    },
    /// A live host's text or no-text start, named by its activation turn and supplied retry.
    RunText {
        run: Token,
        attempt: Token,
        turn: u32,
        retry: u32,
        offset: u32,
        text: Box<[u8]>,
        call: Option<views::Call>,
    },
    /// Task lifecycle or hub work.
    Tasks(tasks::Event),
    /// An application asks for one admitted recurring core procedure.
    StartRecurring { project: u32, authority: tasks::Authority, template: tasks::RecurringTemplate },
    /// The root assembled all selected connector sections, or found a missing owner.
    BriefAssembled { task: u64, failure: Option<brief::GatherFailure> },
    /// The root translated the selected connector's workspace writes.
    WorkspacePrepared {
        task: u64,
        attempt: u64,
        writes: Option<Box<[authority::Write]>>,
        /// A job's held names, translated by its connectors from workspace writes
        /// (jig/docs/design/domain/engine/executors.md, 1, step 3); empty for an agent.
        holds: Box<[tasks::Name]>,
        credentials: Box<[u32]>,
        /// A job's application-encoded workspace items, within `Limits::run_bytes`.
        workspace: Option<Box<[u8]>>,
    },
    /// The root returned translated workspace names after assembling the run assignment.
    ClaimPrepared { task: u64, attempt: u64, budget: u64, writes: Box<[tasks::Name]> },
    /// A connector refused the held names of a pending task claim.
    ClaimRefused { task: u64 },
    /// The root translated the holdings for a core-authorized connector repair.
    ConnectorRepair {
        seed: Box<crate::effects::connector::RepairSeed>,
        executor: tasks::Executor,
        spec: tasks::Spec,
        contract: tasks::Contract,
        holdings: Box<[tasks::Holding]>,
    },
    /// A person's task in no project (jig/docs/design/domain/authority.md, 6.1).
    PersonTask {
        seed: Box<crate::PersonSeed>,
        executor: tasks::Executor,
        /// None takes what a chat of theirs takes; `Core::account_llm` holds it to one account.
        llm: Option<tasks::LlmChoice>,
        spec: tasks::Spec,
        contract: tasks::Contract,
    },
    /// A new deployment funding period, checked against core policy.
    Period { project: u32, period: u64, budget: u64, next: Option<Wall> },
    /// One page or terminal outcome of the transcript load requested by the core.
    TranscriptLoaded { task: u64, load: TranscriptLoad },
    /// Party identity or request.
    People(people::Event),
    /// Host, run or attempt.
    Fleet(fleet::Event),
    /// Root-translated contents of a host turn payload.
    TurnPayload {
        run: Token,
        attempt: Token,
        turn: u32,
        body: Token,
        read: Option<u64>,
        cumulative: u64,
        refused: Box<[u64]>,
    },
    /// Root-translated contents of a host answer payload.
    AnswerPayload {
        run: Token,
        attempt: Token,
        payload: Token,
        cumulative: u64,
        end: tasks::End,
        saved: Option<Box<[tasks::SavedResource]>>,
        invalid_saved: bool,
        /// Unusable resume history retains its own typed reason (jig/docs/design/domain/hosts/hub.md, 3).
        unusable_history: bool,
        read: Option<u64>,
        refused: Box<[u64]>,
    },
    /// Root-translated accepted turn and its owned transcript.
    AcceptedTurn {
        task: u64,
        attempt: u64,
        turn: u32,
        accepted: tasks::Accepted,
        opens_window: bool,
        cumulative: u64,
        read: Option<u64>,
        transcript: Box<[u8]>,
    },
    /// Root-translated host payload for a refused task request.
    RefusedPayload { request: Token, problem: tasks::Problem, payload: Option<PayloadRefusal> },
    /// An activation with the root's readiness; `busy` when it has no room to read a transcript.
    Activate { context: Box<tasks::RunContext>, ready: bool, busy: bool },
    /// Return the ended owner header for an authenticated conversation mark.
    SeenConversationLoaded { request: Token, row: Option<Box<tasks::TaskRecord>>, busy: bool },
    /// The ended position loaded for an authenticated mark, zero if absent.
    EndedReadLoaded { request: Token, task: u64, party: u64, through: u64, busy: bool },
    /// The proposal loaded from its task proposer's durable history.
    DecidedProposal { to: ReplyTo, key: CallKey, proposal: u64, row: Option<tasks::Proposal>, busy: bool },
    /// One ended proposal decision, translated from the root's store row.
    EndedProposal {
        request: Token,
        person: u64,
        project: u32,
        proposer: people::Proposer,
        proposal: u64,
        row: Option<crate::ProposalDecisionRecord>,
        busy: bool,
    },
    /// One ended task row, loaded without root judgment.
    EndedTaskLoaded { request: Token, row: Option<Box<tasks::TaskRecord>>, busy: bool },
    /// One party input or its ended tree root, loaded without root judgment.
    PartyInputsLoaded { request: Token, row: Option<Box<tasks::TaskRecord>>, busy: bool },
    /// Ended owner row loaded for an authenticated person-task act.
    EndedPersonTask {
        request: Token,
        person: u64,
        project: u32,
        task: u64,
        row: Option<Box<tasks::TaskRecord>>,
        busy: bool,
    },
    /// One ended held-chat decision, translated from the root's store row.
    EndedEscalation {
        request: Token,
        person: u64,
        project: u32,
        task: u64,
        revision: u64,
        row: Option<EscalationDecisionRecord>,
        busy: bool,
    },
    /// One root-translated named-call answer that needs a durable core decision.
    NamedAnswer { to: ReplyTo, key: CallKey, part: CallPart },
    /// One generic named-call action translated from the root's host vocabulary.
    NamedAction { to: ReplyTo, key: CallKey, action: NamedAction },
    /// Retained inputs have been loaded and validated by the root's store transport.
    DelegateValidated { to: ReplyTo, key: CallKey, batch: Box<[Delegate]>, stubs: Box<[tasks::Stub]> },
    /// One retained input load failed validation before named delegation.
    DelegateInputFailed { to: ReplyTo, key: CallKey },
    /// One connector's batch holdings, in the same order as the core's request.
    DelegateHoldings {
        request: Token,
        connector: u16,
        holdings: Option<Box<[Box<[tasks::Holding]>]>>,
        /// A reason for refusal, set only when `holdings` is `None`.
        refused: Option<HoldingsRefusal>,
    },
    /// One connector's procedure batch holdings in requested order.
    ProcedureHoldings {
        task: u64,
        step: u64,
        connector: u16,
        holdings: Option<Box<[Box<[tasks::Holding]>]>>,
        /// A reason for refusal, set only when `holdings` is `None`.
        refused: Option<HoldingsRefusal>,
    },
    /// A numbered connector's procedure choice in the core's vocabulary.
    ProcedureStep { task: u64, step: u64, connector: u16, code: u32, decision: ProcedureStep },
    /// A gathered section.
    Brief(brief::GatherEvent),
    /// One numbered connector's answer to a task-holdings handoff.
    Holdings {
        request: Token,
        connector: u16,
        holdings: Option<Box<[tasks::Holding]>>,
        /// A reason for refusal, set only when `holdings` is `None`.
        refused: Option<HoldingsRefusal>,
    },
    /// An account refresh or grant.
    Account(accounts::Event),
    /// A live view.
    View(views::Event),
    /// A scoped note.
    Notes(notes::Event),
    /// An unsigned public read (jig/docs/design/protocol/client/public-reads.md).
    PublicRead { reply: ReplyTo, read: crate::PublicRead },
    /// A connector's public document (jig/docs/design/domain/engine/views.md).
    PublicAnswered { reply: ReplyTo, answer: Option<crate::PublicApplication> },
    /// The root released this read after preceding commits became durable.
    PublicDurable { read: Token },
}
/// Why a connector refuses a member's holdings (domain/tasks/holds.md, 1 and 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldingsRefusal {
    /// The member names a symbol this connector does not offer it.
    Symbol { symbol: u16 },
    /// The member would write a resource this task holds, outside its subtree.
    Held { holder: u64 },
}

/// One member's concrete identity passed to a connector for its own holdings.
#[derive(Clone, Hash, Debug)]
pub struct HoldingsNeed {
    pub project: u32,
    pub root: u64,
    pub number: u64,
    pub executor: tasks::Executor,
    pub spec: tasks::Spec,
    pub tracked: bool,
    /// This connector's symbols of the member, in the member's order.
    pub symbols: Box<[u16]>,
}
/// Connector-free shape of a root-owned host payload after refusal.
#[derive(Hash, Debug)]
pub enum PayloadRefusal {
    /// One rejected host turn.
    Turn { task: u64, attempt: u64, turn: u32 },
    /// One rejected host answer.
    Answer { task: u64, attempt: u64 },
}

/// Connector-free work a named host call asks the core to route.
#[derive(Hash, Debug)]
pub enum NamedAction {
    /// Read related task specifications or proven descendant reports without deciding a call.
    ReadTask {
        field: crate::reads::TaskTextField,
        task: u64,
        from: u64,
        /// The most words this answer holds, named by its application (domain/engine/executors.md, 3).
        room: u32,
    },
    /// A settings read decides nothing and keeps no call record (domain/engine/executors.md, 3).
    Settings(crate::reads::SettingsRead),
    /// A run reads as its task's requester, a person, of its task's project alone
    /// (domain/engine/executors.md, 3, *A run's read as its person*). Decides nothing, keeps no call record.
    PersonRead(crate::reads::Read),
    /// A run's read of one proposal of settings, by the `settings` family (domain/engine/executors.md, 3).
    ReadProposal {
        proposer: jig_domain_people::Proposer,
        number: u64,
    },
    /// A run proposes settings against the revision it read (domain/engine/settings.md, 6).
    ProposeSettings {
        against: u32,
        items: Box<[crate::settings::Item]>,
        reason: Box<[u8]>,
    },
    Message {
        target: u64,
        kind: tasks::MessageKind,
        words: Box<[u8]>,
    },
    Introduce {
        left: u64,
        right: u64,
    },
    Subscribe {
        kind: tasks::SubscriptionKind,
    },
    Unsubscribe {
        subscription: u64,
    },
    Control {
        cancel: tasks::Cancel,
    },
    /// Release a delegate naming the hold the caller saw.
    Release {
        target: u64,
        revision: u64,
        again: Option<bool>,
    },
    Amend {
        target: u64,
        revision: u64,
        amendment: tasks::Amendment,
        /// Resolve omitted host specification parameters from the current task before admission.
        keep_parameters: bool,
    },
    DecideEscalation {
        task: u64,
        revision: u64,
        choice: EscalationChoice,
    },
    WithdrawProposal {
        proposal: u64,
    },
    DecideProposal {
        proposer: u64,
        proposal: u64,
        choice: ProposalChoice,
    },
    Propose {
        action: tasks::ProposalAction,
        reason: Box<[u8]>,
        as_holder: bool,
    },
    ProposeBatch {
        batch: Box<[Delegate]>,
        reason: Box<[u8]>,
        as_holder: bool,
    },
    /// Propose one tracked goal, requested and funded by its accepting holder.
    ProposeGoal {
        goal: Box<Delegate>,
        priority: u32,
        reason: Box<[u8]>,
    },
    /// Save one scoped note under a revision seen by the run.
    Note {
        entry: notes::New,
        recalled: Option<u32>,
    },
    /// Replace a range of a note's body lines under a revision seen by the run.
    NoteLines {
        entry: notes::New,
        recalled: u32,
        range: notes::LineRange,
    },
    /// Drop one scoped note at a revision seen by the run (domain/engine/briefs.md, 2.1).
    NoteDelete {
        scope: notes::Scope,
        name: u64,
        recalled: u32,
    },
    /// Recall one page of notes by name or description search.
    Recall {
        by: notes::Recall,
        page: u32,
    },
}

/// One engine-tool family and any condition the authority check needs.
#[derive(Debug)]
pub enum ToolKind {
    Settings,
    /// An application's asking tool, decided by the asking family alone.
    Ask,
    Delegate,
    Message {
        target: u64,
    },
    Control,
    Decide,
    Propose,
    Subscribe,
    Effect,
    Note {
        scope: notes::Scope,
    },
    Recall {
        by: notes::Recall,
    },
    Read,
}
/// A task holder's choice on one pending proposal.
#[derive(Hash, Debug)]
pub enum ProposalChoice {
    Accept,
    Reject { reason: Box<[u8]> },
    Pass,
}

/// A task holder's choice on one waiting escalation.
#[derive(Hash, Debug)]
pub enum EscalationChoice {
    Release,
    Reject { reason: Box<[u8]> },
    Pass,
}

/// One due child timer, classed before root admission (programming-model.md, 7.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Timer {
    /// Sign-in and retained keyed-answer expiration.
    People,
    /// Task deadlines and standing work.
    Tasks,
    /// Host, call and attempt deadlines.
    Fleet,
    /// Brief gathering deadline.
    Brief,
    /// Account refresh or retry.
    Account,
}

/// Why an entire connector-owned procedure step was refused.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum StepRefusal {
    Moved { task: u64, now: people::Moved },
    Refused(tasks::Problem),
    Denied { answer: authority::Answer, findings: Box<[authority::Finding]> },
}

impl Event {
    /// Whether this event brings new work, which waits at a root's
    /// watermark (programming-model.md, 7.1). Other events finish or
    /// advance admitted work and are taken at any backlog.
    #[must_use]
    pub fn new_work(&self) -> bool {
        match self {
            Self::EffectConnector(event) => effectconnector_new_work(event),
            Self::Tasks(event) => tasks_new_work(event),
            Self::People(event) => people_new_work(event),
            Self::Fleet(event) => fleet_new_work(event),
            Self::Account(event) => account_new_work(event),
            Self::View(event) => view_new_work(event),
            Self::Notes(event) => notes_new_work(event),
            Self::ChangeModel { .. }
            | Self::ChangeSettings { .. }
            | Self::ProposeSettings { .. }
            | Self::Act { .. }
            | Self::Read { .. }
            | Self::PublicRead { .. }
            | Self::Application { .. }
            | Self::Key { .. }
            | Self::EffectStart { .. }
            | Self::SubscribeTopic { .. }
            | Self::SignIn { .. }
            | Self::Watch { .. }
            | Self::WatchRun { .. }
            | Self::StartRecurring { .. }
            | Self::Period { .. }
            | Self::NamedAction { .. } => true,
            Self::SettingsCommit { .. }
            | Self::Fill { .. }
            | Self::Stopping
            | Self::Marked { .. }
            | Self::ReadLoaded { .. }
            | Self::PublicAnswered { .. }
            | Self::PublicDurable { .. }
            | Self::ApplicationPart { .. }
            | Self::ApplicationFinished { .. }
            | Self::ProjectionSettled { .. }
            | Self::SettledCall { .. }
            | Self::EffectDeadline
            | Self::Unwatch { .. }
            | Self::Delivered { .. }
            | Self::StreamDelivered { .. }
            | Self::RunCall { .. }
            | Self::RunInput { .. }
            | Self::RunText { .. }
            | Self::BriefAssembled { .. }
            | Self::WorkspacePrepared { .. }
            | Self::ClaimPrepared { .. }
            | Self::ClaimRefused { .. }
            | Self::ConnectorRepair { .. }
            | Self::PersonTask { .. }
            | Self::TranscriptLoaded { .. }
            | Self::TurnPayload { .. }
            | Self::AnswerPayload { .. }
            | Self::AcceptedTurn { .. }
            | Self::RefusedPayload { .. }
            | Self::Activate { .. }
            | Self::SeenConversationLoaded { .. }
            | Self::EndedReadLoaded { .. }
            | Self::DecidedProposal { .. }
            | Self::EndedProposal { .. }
            | Self::EndedTaskLoaded { .. }
            | Self::PartyInputsLoaded { .. }
            | Self::EndedPersonTask { .. }
            | Self::EndedEscalation { .. }
            | Self::NamedAnswer { .. }
            | Self::DelegateValidated { .. }
            | Self::DelegateInputFailed { .. }
            | Self::DelegateHoldings { .. }
            | Self::ProcedureHoldings { .. }
            | Self::ProcedureStep { .. }
            | Self::Brief(..)
            | Self::Holdings { .. }
            | Self::ProjectReleased { .. } => false,
        }
    }
}

fn tasks_new_work(event: &tasks::Event) -> bool {
    match event {
        tasks::Event::Place { .. }
        | tasks::Event::Lift { .. }
        | tasks::Event::MoveToNewest { .. }
        | tasks::Event::Compound { .. }
        | tasks::Event::Resource { .. }
        | tasks::Event::ResourceAccess { .. }
        | tasks::Event::Kinds { .. }
        | tasks::Event::Slots { .. }
        | tasks::Event::AllocationGone { .. }
        | tasks::Event::ProposePerson { .. }
        | tasks::Event::WithdrawPersonProposal { .. }
        | tasks::Event::DecidePersonProposal { .. }
        | tasks::Event::TickRecurring { .. }
        | tasks::Event::RenewStanding { .. }
        | tasks::Event::TakePerson { .. }
        | tasks::Event::HandBackPerson { .. }
        | tasks::Event::AnswerPerson { .. }
        | tasks::Event::Propose { .. }
        | tasks::Event::DecideProposal { .. }
        | tasks::Event::WithdrawProposal { .. }
        | tasks::Event::Cancel { .. }
        | tasks::Event::Release { .. }
        | tasks::Event::Amend { .. }
        | tasks::Event::Move { .. }
        | tasks::Event::Subscribe { .. }
        | tasks::Event::SubscribeTopic { .. }
        | tasks::Event::Unsubscribe { .. }
        | tasks::Event::Notice { .. }
        | tasks::Event::Introduce { .. }
        | tasks::Event::Message { .. }
        | tasks::Event::PersonQuestion { .. }
        | tasks::Event::PersonQuestionAnswer { .. }
        | tasks::Event::Talk { .. }
        | tasks::Event::Leave { .. }
        | tasks::Event::AuthorizedMessage { .. }
        | tasks::Event::PeriodNext { .. }
        | tasks::Event::OpenPeriod { .. }
        | tasks::Event::CarvePool { .. }
        | tasks::Event::ResizePool { .. }
        | tasks::Event::ResizePeriod { .. }
        | tasks::Event::Make { .. }
        | tasks::Event::Hold { .. }
        | tasks::Event::HoldFrom { .. } => true,
        tasks::Event::CloseProject { .. }
        | tasks::Event::Charters { .. }
        | tasks::Event::WakeProcedure { .. }
        | tasks::Event::CloserBatch { .. }
        | tasks::Event::RecurringBatch { .. }
        | tasks::Event::Procedure { .. }
        | tasks::Event::DecideSettingsProposal { .. }
        | tasks::Event::StalledProposal { .. }
        | tasks::Event::DelegateResult { .. }
        | tasks::Event::RememberStub { .. }
        | tasks::Event::InspectEscalations { .. }
        | tasks::Event::RecheckEscalations { .. }
        | tasks::Event::RoutedEscalation { .. }
        | tasks::Event::DecideEscalation { .. }
        | tasks::Event::ChargeEffect { .. }
        | tasks::Event::Turn { .. }
        | tasks::Event::NowWritten { .. }
        | tasks::Event::Refusals { .. }
        | tasks::Event::MakeResultFollowups { .. }
        | tasks::Event::Prepare { .. }
        | tasks::Event::Claim { .. }
        | tasks::Event::ReadAfresh { .. }
        | tasks::Event::EffectInFlight { .. }
        | tasks::Event::EffectSettled { .. }
        | tasks::Event::Started { .. }
        | tasks::Event::Activation { .. }
        | tasks::Event::PreparationWaiting { .. }
        | tasks::Event::PreparationPaused { .. }
        | tasks::Event::PreparationCounted { .. }
        | tasks::Event::EffectsSettled { .. }
        | tasks::Event::Retained { .. }
        | tasks::Event::Settled { .. }
        | tasks::Event::PersonStallNotice { .. }
        | tasks::Event::MetNotice { .. }
        | tasks::Event::QuestionOwedNotice { .. }
        | tasks::Event::QuestionParked { .. }
        | tasks::Event::NothingAwaitedNotice { .. }
        | tasks::Event::QuestionEndedNotice { .. }
        | tasks::Event::CancelQuestion { .. }
        | tasks::Event::Restore { .. }
        | tasks::Event::Check
        | tasks::Event::Restored => false,
    }
}

fn fleet_new_work(event: &fleet::Event) -> bool {
    match event {
        fleet::Event::Needs(..)
        | fleet::Event::CleanupPrograms { .. }
        | fleet::Event::CheckProgram { .. }
        | fleet::Event::SignOutProgram { .. }
        | fleet::Event::ReadList { .. }
        | fleet::Event::Sources { .. }
        | fleet::Event::Check { .. }
        | fleet::Event::SignOut { .. }
        | fleet::Event::Relay { .. } => true,
        fleet::Event::ListRead { .. }
        | fleet::Event::Draining { .. }
        | fleet::Event::Logins { .. }
        | fleet::Event::SignedOut { .. }
        | fleet::Event::StartJob { .. }
        | fleet::Event::Start { .. }
        | fleet::Event::Inbound { .. }
        | fleet::Event::Adopt { .. }
        | fleet::Event::Cross { .. }
        | fleet::Event::Cancel { .. }
        | fleet::Event::Interrupt { .. }
        | fleet::Event::Relayed { .. }
        | fleet::Event::Acknowledge { .. }
        | fleet::Event::Loaded
        | fleet::Event::Grant { .. }
        | fleet::Event::Unauthorized { .. }
        | fleet::Event::Limited { .. }
        | fleet::Event::AccountRefused { .. }
        | fleet::Event::Hello { .. }
        | fleet::Event::Lost { .. }
        | fleet::Event::Answer { .. }
        | fleet::Event::Turn { .. }
        | fleet::Event::TurnKept { .. }
        | fleet::Event::TurnBusy { .. }
        | fleet::Event::Observe { .. } => false,
    }
}

fn people_new_work(event: &people::Event) -> bool {
    match event {
        people::Event::Act { .. }
        | people::Event::Seed { .. }
        | people::Event::SignedIn { .. }
        | people::Event::Whose { .. }
        | people::Event::SignOut { .. }
        | people::Event::Roles { .. }
        | people::Event::Ask { .. } => true,
        people::Event::Deleting { .. }
        | people::Event::CloseProject { .. }
        | people::Event::ForgetProject { .. }
        | people::Event::MoveChat { .. }
        | people::Event::ChatState { .. }
        | people::Event::ConversationSeen { .. }
        | people::Event::Forget { .. }
        | people::Event::MakeService { .. }
        | people::Event::Waiting { .. }
        | people::Event::ApplyRoles { .. }
        | people::Event::Decided { .. }
        | people::Event::Restore { .. } => false,
    }
}

fn account_new_work(event: &accounts::Event) -> bool {
    match event {
        accounts::Event::Want
        | accounts::Event::Change { .. }
        | accounts::Event::Credential { .. }
        | accounts::Event::SignIn { .. }
        | accounts::Event::SignOut { .. } => true,
        accounts::Event::ForgetProject { .. }
        | accounts::Event::Restore { .. }
        | accounts::Event::ProgramSaid { .. }
        | accounts::Event::ProgramTargets { .. }
        | accounts::Event::ProgramSignedOut { .. }
        | accounts::Event::Component(..)
        | accounts::Event::Unauthorized { .. }
        | accounts::Event::Limited { .. }
        | accounts::Event::Refused { .. }
        | accounts::Event::RunFailed { .. }
        | accounts::Event::Visited { .. }
        | accounts::Event::Finishing { .. }
        | accounts::Event::FinishRefused { .. }
        | accounts::Event::SignedIn { .. }
        | accounts::Event::SignInFailed { .. }
        | accounts::Event::SignedOut { .. }
        | accounts::Event::Withdrawn { .. }
        | accounts::Event::Opened { .. }
        | accounts::Event::NotOurs { .. }
        | accounts::Event::Adopted { .. }
        | accounts::Event::Listed { .. }
        | accounts::Event::ListFailed { .. }
        | accounts::Event::Close => false,
    }
}

fn view_new_work(event: &views::Event) -> bool {
    match event {
        views::Event::Watch { .. } => true,
        views::Event::EndWatch { .. }
        | views::Event::Changed { .. }
        | views::Event::Made { .. }
        | views::Event::Delivered { .. }
        | views::Event::StreamDelivered { .. }
        | views::Event::Text { .. }
        | views::Event::Input { .. }
        | views::Event::Called { .. }
        | views::Event::TurnCommitted { .. }
        | views::Event::RunEnded { .. }
        | views::Event::Gone { .. } => false,
    }
}

fn notes_new_work(event: &notes::Event) -> bool {
    match event {
        notes::Event::Index { .. }
        | notes::Event::Recall { .. }
        | notes::Event::Write { .. }
        | notes::Event::WriteLines { .. }
        | notes::Event::Edit { .. } => true,
        notes::Event::DropProject { .. } | notes::Event::LoadFailed { .. } | notes::Event::Loaded { .. } => false,
    }
}

#[expect(clippy::match_same_arms, reason = "application settings placeholders have their own arm")]
fn effectconnector_new_work(event: &crate::effects::connector::Event) -> bool {
    match event {
        crate::effects::connector::Event::SettingsRestored { .. } => false,
        crate::effects::connector::Event::Resource { .. }
        | crate::effects::connector::Event::PoolSlots { .. }
        | crate::effects::connector::Event::News { .. }
        | crate::effects::connector::Event::Drift { .. } => true,
        crate::effects::connector::Event::Described { .. }
        | crate::effects::connector::Event::DescribeRefused { .. }
        | crate::effects::connector::Event::DescribeBusy { .. }
        | crate::effects::connector::Event::Verdict { .. }
        | crate::effects::connector::Event::Procedure { .. }
        | crate::effects::connector::Event::Outbox { .. }
        | crate::effects::connector::Event::ProjectionOutbox { .. }
        | crate::effects::connector::Event::SectionReady { .. }
        | crate::effects::connector::Event::WorkspaceReady { .. }
        | crate::effects::connector::Event::Adopted { .. }
        | crate::effects::connector::Event::Closed { .. }
        | crate::effects::connector::Event::RestartDone { .. } => false,
        &crate::effects::connector::Event::SettingsJudged { .. }
        | &crate::effects::connector::Event::SettingsDefaults { .. } => false,
    }
}

impl Timer {
    /// Whether firing starts new work rather than advancing admitted work
    /// (programming-model.md, 7.1). Tasks also starts standing work.
    #[must_use]
    pub const fn new_work(self) -> bool {
        match self {
            Self::Tasks => true,
            Self::People | Self::Fleet | Self::Brief | Self::Account => false,
        }
    }
}

#[cfg(test)]
mod admission_tests {
    use super::{Event, Timer};
    use jig_domain_fleet as fleet;
    use jig_domain_people as people;
    use skein_lib::{ReplyTo, Token};

    #[test]
    fn completions_and_requests_have_distinct_admission() {
        let reply_to = ReplyTo::new(Token::new(1));
        for (event, new) in [
            (Event::Stopping, false),
            (Event::Fill { bytes: 0 }, false),
            (Event::Unwatch { watcher: Token::new(1) }, false),
            (Event::Fleet(fleet::Event::Lost { channel: Token::new(1) }), false),
            (
                Event::People(people::Event::Ask {
                    reply_to,
                    sign_in: 1,
                    key: [1; 16],
                    models: Box::new([]),
                    ask: people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } },
                }),
                true,
            ),
            (Event::Period { project: 1, period: 1, budget: 0, next: None }, true),
        ] {
            assert_eq!(event.new_work(), new, "the variant fixes its admission");
        }
    }

    #[test]
    fn only_the_tasks_timer_starts_work() {
        for (timer, new) in [
            (Timer::People, false),
            (Timer::Fleet, false),
            (Timer::Brief, false),
            (Timer::Account, false),
            (Timer::Tasks, true),
        ] {
            assert_eq!(timer.new_work(), new, "the timer's work fixes its admission");
        }
    }
}

/// Core-owned continuations drained within one decision (jig/docs/design/domain/engine/README.md, 4.3).
#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "public entrances move through the decision queue without a separate allocation"
)]
pub(crate) enum Work {
    Event(Event),
    /// A required preparation read or connector section could not be assembled.
    PreparationFailed {
        task: u64,
        failure: brief::GatherFailure,
    },
    /// Begin core-owned brief planning for one preparing task.
    StartBrief {
        task: u64,
    },
    /// Forget a fully read ended task and its read marker in one decision.
    Forget {
        owner: crate::reads::forget::Owner,
    },
    /// The notes child supplied the brief's bounded index before connector gathering.
    BriefNotes {
        task: u64,
        lines: List<notes::Line>,
        more: u32,
    },
}

/// The store transport's transcript answer (jig/docs/design/domain/engine/README.md, 4.1, and jig/docs/design/domain/engine/executors.md, 2).
#[derive(Hash, Debug)]
pub enum TranscriptLoad {
    Page {
        rows: Box<[crate::TranscriptRow]>,
        done: bool,
    },
    /// A window's turn past its store page's room, alone in its page: its
    /// fields, `turn.transcript` empty and its transcript's length
    /// (jig/docs/design/domain/engine/executors.md, 2).
    Cut {
        turn: crate::TurnRecord,
        bytes: u64,
        done: bool,
    },
    Unread,
    Failed,
}
