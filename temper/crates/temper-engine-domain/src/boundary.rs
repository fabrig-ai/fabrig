//! The engine boundary, wrapping its typed peers (jig's domain/root.md, 2).
//! Window pages carry whole or cut turns (temper/docs/design/domain/root.md, 7).
pub use crate::route::approvals::{ApprovalRow, Approvals, YourReview};
use crate::route::{HostRequest, escalation, forge_route, proposals};
use crate::{CallKey, Delivery, Key, Range, Record, RunPolicy, Write};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_core::{Delegate, ProcedureStep, RunCharter};
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_link::Grant;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{List, ReplyTo, Token, Wall};
use temper_engine_domain_forge as forge;

/// A complete claim's assignment, root to worker after durability; its
/// brief sections are bounded by brief limits (domain/root.md, 6.1 and 6.2).
/// The protocol layer encodes it as the hub's assignment, with the forge
/// workspace beside it as temper's application part (jig's
/// `protocol/channel/README.md`, 5.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Assignment {
    /// The change project's delivery checks' bound (domain/agent/charter.md, 4).
    pub check_timeout: Option<skein_lib::Duration>,
    /// Root-issued durable task number.
    pub task: u64,
    pub attempt: u64,
    /// The conversation's first attempt, shared by its resumes.
    pub conversation: u64,
    /// The core's ask for live content (jig's `domain/engine/executors.md`, 1).
    pub live: host::LiveAsk,
    /// Configured charter, never selected by the worker.
    pub charter: u32,
    /// Root-owned charter policy and task result contract for this activation.
    pub run: Box<RunCharter>,
    /// Owned task, attempt and transcript-tail sections, at most brief `sections` and
    /// `brief_bytes`. Task/deployment lineage awaits its root route.
    pub sections: Box<[BriefSection]>,
    /// Whole unread words offered to this attempt, oldest first, at most the run's inbox.
    pub messages: Box<[tasks::Word]>,
    /// Writable repository tags whose workspace starts at the task's saved-work branch.
    pub saved: Box<[u64]>,
    /// The task holding the workspace, whose runs may reuse it: the hub's workstream.
    pub workstream: u64,
    /// Concrete forge checkouts and their claimed writer branches.
    pub workspace: ForgeWorkspace,
    /// Ordered opaque committed turn bodies of this task, empty for a fresh run.
    pub transcript: Box<[Box<[u8]>]>,
    /// Named answers committed after the task's last accepted turn. The
    /// worker gives these to the new attempt before it can decide new calls.
    pub answered: Box<[crate::CallRecord]>,
    pub settled: Box<[jig_domain_core::Settled]>,
    /// Secret-free account and application credential grants; token bytes stay in the protocol.
    pub grants: Box<[Grant]>,
}

/// One root-owned section in a worker assignment (jig's domain/engine/briefs.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BriefSection {
    pub kind: BriefKind,
    pub body: BriefBody,
}

/// The core's typed section or a section owned by temper's forge connector.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BriefKind {
    Core(brief::Core),
    Forge(ForgeBriefKind),
}

/// The forge connector's brief section kinds.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ForgeBriefKind {
    Document(Box<[u8]>),
    Ci,
    Reviews,
    Pull,
    Home,
}

/// Bytes handed to the worker, or why a section was unavailable.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BriefBody {
    Text(Box<[u8]>),
    Missing(brief::GatherMissing),
}

/// A checkout's authoritative forge start, prepared afresh by the worker.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ForgeStart {
    Base(Box<[u8]>),
    Branch(Box<[u8]>),
    Saved(Box<[u8]>),
    Merge {
        branch: Box<[u8]>,
        base: skein_forge::api::Commit,
    },
    /// The landing branch with each head merged into it in order
    /// (temper/docs/design/domain/worker.md, 4.1).
    Merged {
        branch: Box<[u8]>,
        heads: Box<[skein_forge::api::Commit]>,
    },
}

/// One adopted repository translated for the worker's workspace.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForgeRepository {
    /// This repository delivers for a goal run (temper/docs/design/domain/forge/branches.md, 1).
    pub delivers: bool,
    /// This task's saved-work branch, under this repository's prefix.
    pub save: Option<Box<[u8]>>,
    pub tag: u64,
    pub provider: skein_forge::api::Repository,
    pub name: Box<[u8]>,
    pub host: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub start: ForgeStart,
    pub push: Option<Box<[u8]>>,
    pub landing: Option<Box<[u8]>>,
    pub identity: u32,
}

/// Concrete checkout roots for one assignment; the workstream that owns
/// their cache is the hub's (jig's `domain/hosts/README.md`, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForgeWorkspace {
    pub repositories: Box<[ForgeRepository]>,
}

/// A decoded value of temper's settings kinds, retained for its request's decision
/// (temper/docs/design/domain/routing.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct KindValue {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub value: forge::settings::SizedValue,
}

/// Whether a charter offers only the asking family's tools and optional settings
/// (temper/docs/design/domain/agent/tools.md).
#[must_use]
pub fn asking(tools: jig_domain_tasks::Tools) -> bool {
    let families = jig_domain_core::ToolFamilies::JIG;
    tools.0 & families.ask.0 != 0 && tools.0 & !(families.ask.0 | families.settings) == 0
}

/// A run of this policy is given no repository: an asking charter's, or one that
/// names none of inspect, modify and shell (temper/docs/design/domain/agent/charter.md, 2).
#[must_use]
pub fn no_workspace(policy: &RunPolicy) -> bool {
    asking(policy.tools) || !(policy.inspect || policy.modify || policy.shell)
}

/// What a `document` call does with its body (temper/docs/design/domain/agent/ask-temper.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DocumentAction {
    Write,
    Append,
    Replace,
}

/// What a `remember` call does to its person's list (domain/agent/ask-temper.md, 4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RememberAction {
    Add { text: Box<[u8]> },
    Change { line: u32, text: Box<[u8]> },
    Drop { line: u32 },
}

/// One typed engine tool request from the current worker claim
/// (domain/root.md, section 3, and domain/routing.md, section 2). Later routes extend this vocabulary.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Tool {
    /// Ask whether a goal may decide while its person is away (domain/forge/attention.md, 5).
    Decided(forge::DecidedCall),
    /// Read related task specifications or proven descendant reports.
    ReadTask { task: u64, field: jig_domain_core::reads::TaskTextField, from: u64 },
    /// Read settings without retaining a named-call record.
    Settings { read: jig_domain_core::SettingsRead, from: u32 },
    /// Read by one address line, from a byte offset of its answer (domain/agent/ask-temper.md, 1).
    See { line: Box<[u8]>, from: u32 },
    /// Write the chat's one document whole, or one section of it (domain/agent/ask-temper.md, 3).
    Document { action: DocumentAction, body: Box<[u8]> },
    /// Keep, change or drop a line about the chat's person (domain/agent/ask-temper.md, 4).
    Remember { action: RememberAction },
    /// Propose whole settings to the deployment's owners.
    ProposeSettings {
        against: u32,
        items: Box<[jig_domain_core::settings::Item]>,
        reason: Box<[u8]>,
        kinds: Box<[KindValue]>,
        /// Hold this call's part (domain/agent/ask-temper.md, 2).
        more: bool,
    },
    /// Propose a whole settings file held in an accessible note at this revision.
    ProposeSettingsNote { against: u32, note: u64, revision: u32, reason: Box<[u8]> },
    /// Write or correct one scoped note by the revision this run recalled.
    Note { entry: jig_domain_notes::New, recalled: Option<u32> },
    /// Replace a range of a note's body lines by the revision this run recalled.
    NoteLines { entry: jig_domain_notes::New, recalled: u32, range: jig_domain_notes::LineRange },
    /// Read one bounded page of notes by name or description.
    Recall { by: jig_domain_notes::Recall, page: u32 },
    /// Queue one checked forge write with a root-derived idempotency key.
    EffectForge { repository: skein_forge::api::Repository, resource: forge::What, write: Box<skein_forge::api::Write> },
    /// A host read resolved in its project before resource authority admission.
    /// A zero file/job `max_bytes` asks for the configured read limit.
    ReadNamed { repository: Option<Box<[u8]>>, read: skein_forge::api::Read },
    /// Read one adopted forge resource through the connector's bounded fresh lane.
    ReadForge { repository: skein_forge::api::Repository, read: skein_forge::api::Read },
    /// Ask a covering ancestor or person to carry one action the caller cannot take alone.
    Propose { action: ProposedAction, reason: Box<[u8]>, as_holder: bool },
    /// Decide one pending proposal currently addressed to this task.
    Decide { proposer: u64, proposal: u64, decision: ProposalChoice },
    /// Withdraw this task's still-pending proposal.
    Withdraw { proposal: u64 },
    /// Resolve one held delegate currently waiting at this task.
    DecideEscalation { task: u64, revision: u64, decision: EscalationChoice },
    /// Expand a goal in the run's words into one ordinary named delegation.
    Goal { words: Box<[u8]>, advisor: bool, agree: bool },
    /// Expand a project repository change into one ordinary named delegation.
    Change { changes: Box<[ChangeCall]> },
    /// Create one authorized batch of direct task delegates.
    Delegate { batch: Box<[Delegate]> },
    /// Amend one live delegate after authority fitting and finite source checks.
    Amend {
        target: u64,
        revision: u64,
        amendment: tasks::Amendment,
        /// The host omitted parameters; jig keeps the current task's before admission.
        keep_parameters: bool,
    },
    /// Cancel one live delegate's whole subtree with a bounded reason.
    Cancel { target: u64, reason: Box<[u8]> },
    /// Release one held delegate.
    Release { target: u64, revision: u64, again: Option<bool> },
    /// Send whole bounded words to a task the caller references.
    Message { target: u64, form: MessageForm, words: Box<[u8]> },
    /// Give two tasks referenced by the caller reciprocal references.
    Introduce { left: u64, right: u64 },
    /// Add one bounded task-state or timer interest.
    Subscribe { kind: tasks::SubscriptionKind },
    /// Subscribe this task to typed connector news, with overlap paths for landings.
    SubscribeForge { topic: forge::Topic, own_change: Option<u64>, paths: Box<[Box<[u8]>]> },
    /// Resolve an owner/name or the project's sole readable adoption before subscribing.
    SubscribeNamedForge {
        repository: Option<Box<[u8]>>,
        topic: forge::Topic,
        own_change: Option<u64>,
        paths: Box<[Box<[u8]>]>,
    },
    /// Remove one current interest by its root-issued number.
    Unsubscribe { subscription: u64 },
    /// Root-normalized whole-batch shape refusal after bounded ingress.
    Rejected(tasks::Refusal),
    /// Root-normalized message shape refusal before retaining its words.
    RejectedMessage(tasks::Refusal),
    /// Root-normalized control shape refusal before retaining its payload.
    RejectedControl(tasks::Refusal),
    /// Root-normalized proposal shape refusal before retaining its action.
    RejectedProposal(tasks::Refusal),
    /// A note or recall failed bounded ingress.
    RejectedNote(jig_domain_notes::Refusal),
    /// The engine records an unavailable answer for a route not yet installed.
    Unavailable,
}

/// One change of a `change` call (domain/agent/tools.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangeCall {
    pub words: Box<[u8]>,
    pub repository: Option<Box<[u8]>>,
    pub name: Option<Box<[u8]>>,
    pub landing: Option<ChangeLanding>,
    pub branch: Option<ChangeBranch>,
    pub merge: Option<[u8; 32]>,
    pub after: Box<[ChangeAfter]>,
    pub questions: Option<ChangeQuestions>,
    pub repairs: Option<skein_forge::api::Commit>,
    pub producer: Option<u32>,
    pub llm: ChangeLlm,
}

/// A change's LLM choices by role (domain/agent/tools.md, 1; domain/forge/change/README.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangeLlm {
    pub producer: Option<tasks::LlmChoice>,
    pub review: Option<tasks::LlmChoice>,
}

#[expect(clippy::derivable_impls, reason = "the Default derive is banned; a change names neither role by default")]
impl Default for ChangeLlm {
    fn default() -> Self {
        Self { producer: None, review: None }
    }
}

impl ChangeCall {
    #[must_use]
    pub fn words(words: Box<[u8]>, repository: Option<Box<[u8]>>) -> Self {
        Self {
            words,
            repository,
            name: None,
            landing: None,
            branch: None,
            merge: None,
            after: Box::new([]),
            producer: None,
            llm: ChangeLlm::default(),
            questions: None,
            repairs: None,
        }
    }
}

/// A change's landing choice (domain/agent/tools.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ChangeLanding {
    Goal,
    Branch(Box<[u8]>),
}

/// How a change takes its agents' questions (domain/agent/tools.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ChangeQuestions {
    None,
    Foreman,
    Up,
}

/// A change's branch choice (domain/agent/tools.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ChangeBranch {
    /// Its requester's held branch: the goal branch it holds in the change's repository.
    Mine,
}

/// A change's dependency within a call (domain/agent/tools.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ChangeAfter {
    Task(u64),
    Name(Box<[u8]>),
}

/// Action a worker can submit for an authority holder's decision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::large_enum_variant, reason = "the worker boundary carries jig's bounded amendment whole")
)]
pub enum ProposedAction {
    Effect { repository: skein_forge::api::Repository, resource: forge::What, write: Box<skein_forge::api::Write> },
    Batch(Box<[Delegate]>),
    Amend { task: u64, revision: u64, amendment: tasks::Amendment },
    Widen { task: u64, authority: tasks::Authority },
    Release { task: u64, hold: u64, revision: u64 },
}

/// A task holder's answer to one proposal.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProposalChoice {
    Accept,
    Reject { reason: Box<[u8]> },
    Pass,
}

/// A task ancestor's decision on one held descendant.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EscalationChoice {
    Release,
    Reject { reason: Box<[u8]> },
    Pass,
}

/// The sendable portion of the task inbox vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MessageForm {
    Words,
    Question,
    Answer { question: u64 },
}

/// A run-named tool call. The root fills the task and attempt from the
/// worker envelope before looking up its durable decision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Call {
    pub completion: u32,
    pub position: u32,
    pub tool: Tool,
}

/// A setup result contains no credential (protocol/credentials.md, 1).
#[derive(Hash, Debug)]
pub enum SetupOutcome {
    Done,
    Manifest { state: Box<[u8]> },
    Install { state: Box<[u8]>, url: Box<[u8]> },
}
#[derive(Hash, Debug)]
pub enum SetupEnd {
    Done { forge: u16, outcome: SetupOutcome },
    Stopped { forge: u16, reason: Box<[u8]> },
}
/// External inputs to the walking root (domain/root.md, 3). Admission is
/// `try_step`'s: an input the journal cannot take returns to its owner, and
/// store answers are always taken (jig's `domain/root.md`, 3).
/// The input union, with one variant for each typed peer.
#[derive(Hash, Debug)]
pub enum Event {
    /// Terminal of the protocol-owned plan named by `run`.
    Setup {
        run: Token,
        end: SetupEnd,
    },
    Store(StoreInput),
    Worker(WorkerInput),
    Party(PartyInput),
    Account(accounts::Event),
    /// A generation named by temper's own credential source for one numbered
    /// application credential, a repository's git identity (domain/root.md, 6.1).
    Credential {
        number: u32,
        generation: u64,
        valid: skein_lib::Duration,
    },
    Forge(Box<forge::Event>),
    Released(Released),
    Restart {
        pending: bool,
    },
    /// Iterate offers progress requested by ready owners.
    Resume,
    /// The service begins its drain (jig's `domain/engine/restart.md`, 1).
    Stop,
}
/// A window turn's text or its omitted body's length (temper/docs/design/domain/root.md, 7).
#[derive(Hash, Debug)]
pub enum TurnText {
    Rendered(Box<[u8]>),
    Unrendered,
    Cut(u32),
}
/// Protocol texts in the same order as a transcript load's rows.
pub type TranscriptTexts = Box<[TurnText]>;

#[derive(Hash, Debug)]
pub enum StoreInput {
    /// A startup page's missing owner, handed in before that page's Loaded.
    MissingLink {
        owner: Token,
        record: jig_domain_core::Key,
        target: jig_domain_core::Key,
    },
    Switched,
    Committed {
        /// Store-echoed positive commit number, at most the latest issued commit; older cumulative
        /// answers are inert.
        number: u64,
        /// The store's map bytes after this commit (jig's `protocol/store/README.md`, 6.2).
        fill: u64,
    },
    Uncommitted {
        /// Store-echoed issued commit that failed; failures at/before durable progress or after
        /// stop are inert.
        number: u64,
    },
    Loaded {
        /// Protocol-rendered text in row order, only for transcript window loads.
        /// Each rendering is at most the core tail budget; a cut turn is alone in its page.
        texts: Option<TranscriptTexts>,
        /// Echoed generational load token, fenced after its one terminal and reclaim.
        owner: Token,
        /// Owned decoded rows, at most `loads::Limits::rows` and `loads::Limits::reply_bytes`
        /// before restoration.
        rows: Box<[Record]>,
        /// Exclusive last-row continuation in the requested range, or terminal page.
        next: Option<Key>,
    },
    Unloaded {
        /// Echoed generational load token, fenced after its one terminal and reclaim.
        owner: Token,
    },
    /// The one terminal of a core read load (jig's `client/domain/contract/README.md`, 4).
    ReadLoaded {
        load: Token,
        result: core::reads::Loaded,
    },
}

/// The three inputs of a worker channel, in the hub's vocabulary (jig's
/// `domain/hosts/README.md`, 2, and `domain/hosts/hub.md`, 4).
#[derive(Hash, Debug)]
pub enum Link {
    Hello {
        worker: u32,
        hello: jig_domain_link::Hello,
    },
    /// One hub record; temper's parts ride beside an answer and are empty otherwise.
    Up {
        up: host::LinkUp,
        parts: Parts,
    },
    Lost,
}

/// Temper's application parts beside a hub answer, named by its work tokens
/// (jig's `protocol/channel/README.md`, 5.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Parts {
    /// Full set of repository tags with saved work after this terminal, when the worker made
    /// a save; absent preserves the previous set.
    pub saved: Option<Box<[u64]>>,
    /// Last heads actually pushed by the worker, indexed by deployment repository tag.
    pub pushed: Box<[forge::Pushed]>,
    /// A permanent failure to prepare, naming its repository by tag.
    pub preparation: Option<Preparation>,
}

impl Parts {
    /// A record that carries no application part.
    #[must_use]
    pub fn none() -> Parts {
        Parts { saved: None, pushed: Box::new([]), preparation: None }
    }
}

/// Why a repository could not be prepared, permanently (protocol/channel.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Preparation {
    /// The head that did not merge into this repository's base.
    Conflicted { tag: u64, head: skein_forge::api::Commit },
    /// The forge lacks what the assignment names for this repository.
    Missing { tag: u64, missing: Missing },
    /// The forge refused this repository's identity.
    Refused { tag: u64 },
}

/// What the forge lacks for a preparation (protocol/channel.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Missing {
    Repository,
    Branch,
    Commit,
}

/// An observation of the root, which decides nothing (domain/root.md, 3 and 6.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Fact {
    /// A forge's link changed or was refused (temper/docs/design/domain/forge/adoption.md, 6).
    ForgeLink { forge: u16, note: forge::LinkNote },
    /// A forge read refused locally or by the connector, for operator diagnostics.
    ReadRefused {
        task: u64,
        attempt: u64,
        tool: Box<[u8]>,
        layer: &'static str,
        reason: &'static str,
        findings: Box<[jig_domain_authority::Finding]>,
        error: Option<skein_forge::api::Error>,
        job: Option<skein_forge::api::JobAttempt>,
    },
    /// A brief section's forge read failed (temper/docs/design/domain/forge/change/repairs.md).
    BriefReadRefused {
        task: u64,
        section: &'static [u8],
        stage: &'static str,
        reason: &'static str,
        heads: Option<(skein_forge::api::Commit, skein_forge::api::Commit)>,
        error: Option<skein_forge::api::Error>,
    },
    /// A whole brief failed with its count and fixed reason (jig/docs/design/domain/engine/briefs.md, 1.1).
    BriefFailed { task: u64, counts: &'static str, reason: Box<[u8]> },
    /// A decoded call refused at admission, before any tool runs.
    CallAdmissionRefused { task: u64, attempt: u64, tool: Box<[u8]>, layer: &'static str, reason: &'static str },
    /// A fenced answer reported a permanent preparation failure.
    PreparationFailed { task: u64, attempt: u64, preparation: Preparation },
    /// The core refused a procedure's step; the root decides nothing (domain/root.md, 3 and 6.4).
    StepRefused { task: u64, step: u64, problem: jig_domain_tasks::Problem },
}

#[derive(Hash, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::large_enum_variant,
        reason = "the worker boundary carries the decoded call whole, its later choice with it"
    )
)]
pub enum WorkerInput {
    /// A worker channel's hello, record or loss.
    Link { channel: Token, link: Link },
    /// The protocol decoded a relayed call into the engine's tool vocabulary.
    DecodedHostCall { to: ReplyTo, body: Call },
    /// The protocol rendered a decided call's answer for its host.
    RenderedHostCall { to: ReplyTo, answer: jig_domain_core::SettledAnswer },
    /// The protocol read an ended answer's opaque outcome after the fleet fenced it.
    ResultRead { payload: Token, end: tasks::End },
}
/// A keyed request owned by the root, or forwarded to the forge (client/engine.md).
/// Trials: temper/docs/design/domain/trial.md, 6.
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum Application {
    Forge(forge::Ask),
    Try { account: u32, again: bool },
    Dismiss { what: forge::PersonFact },
    Looked { project: u32 },
    SetAway { project: u32, on: bool, until: Option<Wall> },
    Overturn { project: u32, number: u32 },
    FileFinding(forge::FileFinding),
    HandFinding(forge::HandFinding),
    TriageFinding(forge::TriageFinding),
}
/// The family's terminal answer (client/forge/records.md).
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum ApplicationAnswer {
    Make(forge::MakeAnswer),
    Adopt(forge::ApplicationOutcome),
    SetUp(forge::SetUpAnswer),
    RemoveForge(forge::RemoveAnswer),
    StartPlayground(forge::StartAnswer),
    EndPlayground(forge::EndAnswer),
    Connect(forge::ConnectAnswer),
    Install(forge::InstallAnswer),
    Accept(forge::AcceptAnswer),
    Reachable(forge::ReachableAnswer),
    Try(TryAnswer),
    Dismiss(forge::DismissAnswer),
    Looked(forge::LookedAnswer),
    SetAway(forge::SetAwayAnswer),
    Overturn(forge::OverturnAnswer),
    FileFinding(forge::FileFindingAnswer),
    HandFinding(forge::HandFindingAnswer),
    TriageFinding(forge::TriageFindingAnswer),
}
/// The trial's terminal answer (temper/docs/design/domain/trial.md, 6).
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum TryAnswer {
    Tried,
    NotTheirs,
}
pub use temper_engine_domain_forge::APPROVALS_SUBJECT;

/// A person's trials document (temper/docs/design/domain/trial.md, 6).
pub const TRIALS_SUBJECT: u64 = temper_engine_domain_trials::TRIALS_SUBJECT;

/// Forge reads through party authorization (temper/docs/design/client/forge/records.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ForgeRead {
    Finding {
        project: u32,
        number: u32,
    },
    /// A view block, read in the project's scope (temper/docs/design/client/forge/records.md).
    View {
        project: u32,
        address: Box<[u8]>,
    },
    /// The owner's program-login environment (temper/docs/design/client/forge/records.md).
    ProgramEnvironment,
    /// The explicit value set in a retained revision.
    RevisionSetting {
        kind: u32,
        scope: u32,
        number: u32,
        revision: u32,
    },
    Setting {
        kind: u32,
        scope: u32,
        number: u32,
    },
    ProposalItems {
        key: u64,
    },
}
/// Typed forge read answers, encoded at the clients' boundary.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ForgeReadAnswer {
    Finding(Option<forge::FindingAnswer>),
    View(Box<temper_engine_domain_ways::View>),
    ProgramEnvironment,
    Setting(Option<forge::settings::SizedValue>),
    /// The proposal's items and whether an enforcing approval covers an owned Forgejo (domain/forge/change/questions.md, 2).
    ProposalItems {
        items: Box<[(forge::settings::SettingKey, forge::settings::Edit)]>,
        owned_approval: bool,
    },
}
#[derive(Hash, Debug)]
pub enum PartyInput {
    /// A visitor read (jig/docs/design/domain/people/public-reading.md, 5).
    PublicRead {
        reply: ReplyTo,
        read: core::PublicRead,
    },
    /// Resolve a sign-in through the people child for provider authorization.
    Whose {
        reply_to: ReplyTo,
        sign_in: u64,
    },
    /// A settings proposal, admitted and decided by jig.
    ProposeSettings {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        digest: [u8; 32],
        change: Box<core::settings::Proposed>,
        kinds: Box<[KindValue]>,
    },
    /// An owner model act, decided by jig at its newest revision.
    ChangeModel {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        digest: [u8; 32],
        act: core::settings::catalogue::ModelAct,
    },
    /// A direct owner settings change, decided by jig.
    ChangeSettings {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        digest: [u8; 32],
        change: Box<core::settings::Change>,
        kinds: Box<[KindValue]>,
    },
    /// End one authenticated sign-in through the people child.
    SignOut {
        reply_to: ReplyTo,
        sign_in: u64,
    },
    Application {
        scope: people::ApplicationScope,
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        digest: [u8; 32],
        request: Application,
        /// The number and digest of a value the request carried, already kept
        /// in the secrets' store (jig's `protocol/client/README.md`, 3).
        carried: Option<core::Carried>,
    },
    StartRecurring {
        project: u32,
        authority: tasks::Authority,
        template: tasks::RecurringTemplate,
    },
    /// A project's period begins, from outside the root (jig's
    /// `protocol/configuration.md`, 5).
    Period {
        project: u32,
        period: u64,
        budget: u64,
        next: Option<Wall>,
    },
    ProcedureStep {
        task: u64,
        step: u64,
        connector: u16,
        code: u32,
        decision: ProcedureStep,
    },
    SignedIn {
        /// Authenticated sign-in offered beside the printed link.
        held: Option<u64>,
        /// Web-issued right to this one sign-in reply, moved through people to a durable terminal.
        reply_to: ReplyTo,
        /// Protocol-authenticated forge/user key and display bytes, bounded by people
        /// `identity_bytes`.
        identity: people::Identity,
    },
    Ask {
        /// Web-issued right to one keyed request terminal; duplicates can join bounded people
        /// waiters, each still replied to once.
        reply_to: ReplyTo,
        /// Root-issued durable secret-free session number; authenticated and expired by people.
        sign_in: u64,
        /// Web-issued fixed request key, scoped by person and persisted with the exact typed ask.
        key: [u8; 16],
        /// Typed chat request with words bounded by people words before routing.
        ask: people::Ask,
        models: Box<[Box<[u8]>]>,
    },
    /// A keyed account ask whose value is already kept in the secrets store.
    Key {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        ask: Box<people::Ask>,
        carried: core::Carried,
    },
    /// One unkeyed read on a live sign-in, answered after every preceding commit.
    Read {
        reply_to: ReplyTo,
        sign_in: u64,
        read: core::reads::Read,
    },
    /// The forge's typed read, answered after preceding commits (client/forge/served.md).
    ForgeRead {
        reply: ReplyTo,
        sign_in: u64,
        scope: people::ApplicationScope,
        query: ForgeRead,
    },
    /// Watch one subject's whole document on a live sign-in.
    /// A forge panel uses `Watched::Application` in its project's scope, the
    /// configured forge connector, and the change task's number as its token
    /// (domain/routing.md, 1; client/forge/served.md, 2). The core admits its project role.
    Watch {
        watcher: Token,
        sign_in: u64,
        subject: core::documents::Watched,
        window: core::documents::Window,
    },
    /// Follow a task's live run from the client's acknowledged turn and offset.
    WatchRun {
        watcher: Token,
        sign_in: u64,
        task: u64,
        turn: Option<views::TurnName>,
        offset: u32,
    },
    Unwatch {
        watcher: Token,
    },
    /// The protocol took a watch's held document.
    Delivered {
        watcher: Token,
    },
    /// The protocol took one provisional stream delivery.
    StreamDelivered {
        watcher: Token,
    },
}

/// Root to shell/worker/web. Commit and load requests each have one store
/// terminal; deliveries are notices or consume a `ReplyTo` (domain/root.md, 3
/// and 4). Every output crosses the journal door in its peer's vocabulary.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Request {
    /// A making's copy mark has crossed the root (temper/docs/design/domain/forge/adoption.md, 5).
    MakingCopied {
        key: forge::MakingKey,
    },
    /// Playground routing (temper/docs/design/protocol/setup/playground.md).
    PlaygroundActs {
        request: Token,
        forge: u16,
    },
    ResetAct {
        key: [u8; 16],
        act: forge::ResetAct,
    },
    /// Forge runtime routing (temper/docs/design/protocol/forge/engine.md, 2 and 3).
    ForgeKept(forge::ForgeRecord),
    ForgeGone {
        forge: u16,
    },
    BoxGroup {
        forge: u16,
        kind: forge::BoxKind,
        request: Token,
    },
    BoxRemove {
        forge: u16,
    },
    /// A sent forge entry left without its outcome at the stop bound (domain/root.md, 8.2).
    Unsettled(Unsettled),
    Bound {
        request: Token,
        answer: forge::BindAnswer,
    },
    /// Routed to the protocol layer's box link; nothing committed (domain/routing.md, 1).
    SetUp {
        forge: u16,
        request: Token,
        first: forge::SetUpFirst,
    },
    Connect {
        forge: u16,
        request: Token,
        how: forge::How,
        carried: jig_domain_core::Carried,
    },
    /// Run a Replace: Check on a connected Forgejo, Paste with the kept App id on GitHub
    /// (protocol/forge/engine.md, 2).
    Replace {
        forge: u16,
        request: Token,
        carried: jig_domain_core::Carried,
    },
    Manifest {
        forge: u16,
        request: Token,
        back: Option<Box<[u8]>>,
    },
    AdoptAccess {
        forge: u16,
        request: Token,
        owner: Box<[u8]>,
        name: Box<[u8]>,
    },
    Reachable {
        forge: u16,
        request: Token,
        installation: Option<u64>,
        page: u32,
    },
    Accept {
        forge: u16,
        request: Token,
        installation: ::core::num::NonZeroU64,
    },
    Install {
        forge: u16,
        request: Token,
        back: Option<Box<[u8]>>,
    },
    EraseCarried {
        number: u64,
    },
    /// Run the staged plan named by this token; credential values stay in routing.
    Setup {
        run: Token,
    },
    Store(StoreRequest),
    Worker(WorkerRequest),
    Party(PartyRequest),
    Forge {
        call: Token,
        repository: skein_forge::api::Repository,
        op: skein_forge::api::Op,
    },
    Account(
        /// Fixed-size accounts request/notice; token values are filled only by the protocol and
        /// never retained here.
        accounts::Request,
    ),
    ToChild(Released),
    /// A forge input the drain does not decide, returned to its owner (jig's
    /// `domain/engine/restart.md`, 1).
    Busy(Box<forge::Event>),
    /// Every commit before the drain is durable (jig's `domain/engine/restart.md`, 1).
    Drained,
    Stop,
}
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum StoreRequest {
    /// Replace the names before the catalogue decision commits (jig jig/docs/design/domain/engine/settings.md, 11).
    Catalogue {
        names: Box<[core::settings::ModelName]>,
    },
    Switch,
    Commit {
        /// Positive ordered commit number allocated by the journal, echoed by the store terminal;
        /// checked `u64` exhaustion stops admission.
        number: u64,
        /// Owned unique-key transaction, at most journal writes including its deployment header;
        /// applied atomically.
        writes: Box<[Write]>,
    },
    Load {
        /// Fresh generational load identity issued by root loads; store echoes it once through
        /// `Loaded` or `Unloaded`.
        owner: Token,
        /// Closed store-key family; every page key is checked for membership.
        range: Range,
        /// Exclusive cursor in this range, or its beginning.
        after: Option<Key>,
        /// Positive page demand, at most the configured load row bound.
        most: u32,
        /// Hard decoded-page byte limit, enforced before and after protocol decoding.
        bytes: u32,
    },
    /// One core read load after every preceding commit, answered by `ReadLoaded`.
    Read {
        load: Token,
        range: Box<core::reads::Range>,
    },
}
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum WorkerRequest {
    Host(Box<HostRequest>),
    /// Read an ended answer's opaque outcome in the protocol routing.
    ReadResult {
        payload: Token,
        outcome: Box<[u8]>,
    },
    Deliver(Delivery),
}
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum PartyRequest {
    /// Replace the clients' catalogue before its decision commits (jig jig/docs/design/domain/engine/settings.md, 11).
    Catalogue {
        models: Box<[core::settings::CatalogueModel]>,
    },
    /// A sign-in the core refused before any decision: a counter is
    /// exhausted, or the store is at its fill mark.
    SignInRefused {
        to: ReplyTo,
    },
    /// A watch refused before any durable decision.
    WatchRefused {
        watcher: Token,
        why: core::WatchRefusal,
    },
    /// A run's provisional suffix for one follower, with no commit.
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
    Deliver(Delivery),
}

#[derive(Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "events stay inline in the bounded route queue priced at startup")]
pub(crate) enum Work {
    StartDone(jig_domain_core::start::Phase),
    Start(jig_domain_core::start::Action),
    Decoded { to: ReplyTo, body: Call },
    ForgeRead { reply: ReplyTo, sign_in: u64, scope: people::ApplicationScope, query: ForgeRead },
    Core(jig_domain_core::Event),
    Tasks(tasks::Event),
    People(people::Event),
    Fleet(fleet::Event),
    Brief(brief::GatherEvent),
    Forge(forge::Event),
    ForgeSettings(skein_lib::Queue<forge::Request>),
    ForgeRestored,
    ProjectGoal(Box<jig_domain_core::ProjectionFeed>),
    GoalSubscribe(forge::Subscriber),
    AgreedGoal(Box<GoalWords>),
    Activate { context: Box<tasks::RunContext>, ready: bool },
    EscalationLoaded { waiter: Token, rows: Box<[Record]> },
    EscalationFailed { waiter: Token },
    ProposalLoaded { waiter: Token, rows: Box<[Record]> },
    ProposalFailed { waiter: Token },
    TaskProposalLoaded { waiter: Token, rows: Option<Box<[Record]>> },
    EndedLoaded { waiter: Token, rows: Box<[Record]> },
    EndedFailed { waiter: Token },
}

#[derive(Hash, Debug)]
pub(crate) struct BriefConnector {
    pub(crate) deadline: skein_lib::Time,
    pub(crate) task: u64,
    pub(crate) kind: ForgeBriefKind,
    pub(crate) gatherable: bool,
    pub(crate) active: bool,
    pub(crate) cutting: bool,
}

/// Preparation waits for the goal branch before its brief and workspace
/// (temper/docs/design/domain/forge/branches.md, 3).
#[derive(Hash, Debug)]
pub(crate) enum WorkspacePreparation {
    Ready(PreparedWorkspace),
    Brief {
        parent: Option<u64>,
        sections: List<jig_domain_brief::Planned>,
        documents: Box<[jig_domain_core::PlannedDocument]>,
    },
    Workspace {
        attempt: u64,
        context: Box<jig_domain_tasks::RunContext>,
    },
}

impl WorkspacePreparation {
    pub(crate) fn prepared(&self) -> &PreparedWorkspace {
        match self {
            Self::Ready(prepared) => prepared,
            Self::Brief { .. } | Self::Workspace { .. } => unreachable!("the workspace was answered before claiming"),
        }
    }

    pub(crate) fn into_prepared(self) -> PreparedWorkspace {
        match self {
            Self::Ready(prepared) => prepared,
            Self::Brief { .. } | Self::Workspace { .. } => unreachable!("the workspace was answered before claiming"),
        }
    }
}

#[derive(Hash, Debug)]
pub(crate) struct PreparedWorkspace {
    pub(crate) workspace: forge_route::RunWorkspace,
    pub(crate) claim_names: Box<[forge::Name]>,
}

pub(crate) use jig_domain_hub_link::Payload;

/// Temper's connector parts beside the link's prepared assignment
/// (temper/docs/design/domain/root.md, 6.1).
#[derive(Hash, Debug)]
pub(crate) struct AssignmentParts {
    pub saved: Box<[u64]>,
    pub workstream: u64,
    pub workspace: ForgeWorkspace,
    pub answered: Box<[crate::CallRecord]>,
}

/// An ended owner row the core asked the root to load.
#[derive(Hash, Debug)]
pub(crate) enum EndedRead {
    EndedTask { request: Token, task: u64 },
    SeenConversation { request: Token, task: u64 },
    Position { request: Token, task: u64, party: u64 },
    PartyInput { request: Token, task: u64 },
    PersonTask { request: Token, person: u64, project: u32, task: u64 },
}

#[derive(Hash, Debug)]
pub(crate) enum Read {
    Escalation(escalation::Query),
    Proposal(proposals::Query),
    TaskProposal { to: ReplyTo, key: CallKey, proposal: u64 },
    Ended(EndedRead),
    Transcript { task: u64 },
    InputCheck(InputCheck),
    GoalWords(Box<GoalWords>),
    Notes { owner: Token },
    SettingsNote { to: Token, call: Box<Call> },
    Document(Box<crate::route::document::DocumentRead>),
    Remember(Box<crate::route::remember::RememberRead>),
}

#[derive(Hash, Debug)]
pub(crate) struct GoalWords {
    pub(crate) to: Token,
    pub(crate) key: CallKey,
    pub(crate) words: Box<[u8]>,
    pub(crate) person: u64,
    pub(crate) position: u64,
    pub(crate) person_words: Box<[u8]>,
}

#[derive(Hash, Debug)]
pub(crate) struct InputCheck {
    pub(crate) to: Token,
    pub(crate) key: CallKey,
    pub(crate) batch: Box<[Delegate]>,
    pub(crate) ids: Box<[u64]>,
    pub(crate) at: u32,
    pub(crate) project: u32,
    pub(crate) stubs: List<tasks::Stub>,
}

/// A child continuation that has crossed its journal barrier. Its payload is
/// private; iterate may only return it as `Event::Released`.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Released(pub(crate) Delivery);

/// Put an owner-produced held output in its destination envelope, by variant only.
pub(crate) fn delivery_output(delivery: Delivery) -> Request {
    match delivery {
        delivery @ (Delivery::ProjectMade { .. }
        | Delivery::ProjectReleased { .. }
        | Delivery::MoveStanding { .. }
        | Delivery::PublicWait { .. }
        | Delivery::ForgeCommitted { .. }
        | Delivery::Fleet(_)
        | Delivery::Relay { .. }
        | Delivery::NotesLoad { .. }
        | Delivery::Start(_)
        | Delivery::Load { .. }
        | Delivery::ReadEscalationDecision { .. }) => Request::ToChild(Released(delivery)),
        delivery @ (Delivery::Host(_)
        | Delivery::AssignedJob { .. }
        | Delivery::Link { .. }
        | Delivery::Procedure { .. }
        | Delivery::Assigned { .. }
        | Delivery::Refuse { .. }
        | Delivery::TurnBusy { .. }) => Request::Worker(WorkerRequest::Deliver(delivery)),
        delivery @ (Delivery::PlaygroundNote { .. }
        | Delivery::Forges { .. }
        | Delivery::ForgeDocument { .. }
        | Delivery::Trials { .. }
        | Delivery::Ways { .. }
        | Delivery::Approvals { .. }
        | Delivery::Panel { .. }
        | Delivery::Application { .. }
        | Delivery::Public { .. }
        | Delivery::WebReply { .. }
        | Delivery::CarriedUnused { .. }
        | Delivery::ForgeRead { .. }
        | Delivery::Read { .. }
        | Delivery::Document { .. }
        | Delivery::WatchEnded { .. }
        | Delivery::SourceSignedOut { .. }
        | Delivery::Result { .. }) => Request::Party(PartyRequest::Deliver(delivery)),
        Delivery::ReadLoad { load, range } => Request::Store(StoreRequest::Read { load, range }),
        Delivery::Drained => Request::Drained,
        Delivery::Bound { request, answer } => Request::Bound { request, answer },
        Delivery::ForgeCall { call, repository, op } => Request::Forge { call, repository, op },
    }
}

/// A forge write a settling stop left without its outcome (domain/root.md, 8.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Unsettled {
    pub entry: u64,
    pub task: u64,
}

/// A failed run's volatile operator evidence (temper/docs/design/domain/worker.md, 4.3).
/// It is neither a task result nor a fact, and is never journaled or shown to an LLM.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct RunDiagnostic {
    /// The task whose accepted answer failed.
    pub task: u64,
    /// The worker run token.
    pub run: u64,
    /// The fenced activation attempt.
    pub attempt: u64,
    /// The failure classification reported by the worker.
    pub failure: jig_domain_host::Failure,
    /// Worker-bounded evidence for the operator sink alone.
    pub detail: Box<[u8]>,
}

/// A refused assignment's volatile operator evidence (temper/docs/design/domain/worker.md, 4.3).
/// It is neither a task result nor a fact, and is never journaled or shown to an LLM.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct RunRefusalDiagnostic {
    /// The task whose accepted assignment was refused.
    pub task: u64,
    /// The worker run token.
    pub run: u64,
    /// The fenced activation attempt.
    pub attempt: u64,
    /// The worker's typed refusal, including bounded start words for operators.
    pub reason: jig_domain_host::Refusal,
}

/// An accepted job's volatile operator evidence (temper/docs/design/domain/worker.md, 4.4).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct JobDiagnostic {
    pub task: u64,
    pub run: u64,
    pub attempt: u64,
    pub kind: u32,
    pub ending: JobDiagnosticEnding,
}

/// Failed and refused job answers only; no result or work reaches the operator sink.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum JobDiagnosticEnding {
    Failed { failure: jig_domain_host::JobFailure, detail: Box<[u8]> },
    Refused(jig_domain_host::JobRefusal),
}
