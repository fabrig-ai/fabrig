//! The engine boundary, wrapping its typed peers (jig's domain/root.md, 2).
//! Window pages carry whole or cut turns (temper/docs/design/domain/root.md, 7).
pub use crate::route::approvals::{ApprovalPerson, ApprovalRow, ApprovalSet, ApprovalSize, Approvals, YourReview};
use crate::route::{HostRequest, forge_route};
use crate::{CallKey, Delivery, Key, Range, Record, Write};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_core::engine::store as engine_store;
use jig_domain_core::{ProcedureStep, RunCharter};
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_link::Grant;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{ReplyTo, Token, Wall};
use temper_engine_domain_forge as forge;

/// A complete claim's assignment, root to worker after durability; its
/// brief sections are bounded by brief limits (domain/root-translation.md, 2.1 and 2.2).
/// The protocol layer encodes it as the hub's assignment, with the forge
/// workspace beside it as temper's application part (jig's
/// `protocol/channel/owners.md`, 1.2).
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
    pub parts: Box<[jig_domain_link::KeptPart]>,
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

pub use temper_engine_domain_forge::workspace::{ForgeRepository, ForgeStart, ForgeWorkspace};

pub use temper_engine_domain_store::{DocumentAction, FindingToolAnswer, FollowToolAnswer, RedToolAnswer};
pub use temper_engine_domain_translate::{
    Call, ChangeAfter, ChangeBranch, ChangeCall, ChangeLanding, ChangeLlm, ChangeQuestions, EscalationChoice,
    FindingCall, FollowCall, KindValue, MessageForm, ProposalChoice, ProposedAction, RedCall, RememberAction, Tool,
    asking, no_workspace,
};

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
    /// Take the checked at-once raises supplied by the service's environment.
    LimitsRaised,
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
    /// application credential, a repository's git identity (domain/root-translation.md, 2.1).
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
    /// The service begins its drain (jig's `domain/engine/restart.md`, 5).
    Stop,
}
#[derive(Hash, Debug)]
pub enum StoreInput {
    Checkpointed {
        token: Token,
        checkpoint: Checkpointed,
    },
    CheckpointRefused {
        token: Token,
        why: CheckpointWhy,
    },
    /// The named map census (jig's protocol/store/growth.md, 1.3).
    Census(Box<core::documents::StoreCensus>),
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
        /// The store's map bytes after this commit (jig's `protocol/store/growth.md`, 1.2).
        fill: u64,
    },
    Uncommitted {
        /// Store-echoed issued commit that failed; failures at/before durable progress or after
        /// stop are inert.
        number: u64,
    },
    Loaded {
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
    /// A joined milestone-delegates load owned by the root.
    PartLoaded {
        waiter: Token,
        result: engine_store::Answer,
    },
    /// An answer on the engine’s store port.
    Engine(engine_store::In),
}

/// The three inputs of a worker channel, in the hub's vocabulary (jig's
/// `domain/hosts/README.md`, 2, and `domain/hosts/hub.md`, 4).
#[derive(Hash, Debug)]
pub enum Link {
    Mismatch,
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
/// (jig's `protocol/channel/owners.md`, 1.2).
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

/// An observation of the root, which decides nothing (domain/root.md, 3, and domain/root-translation.md, 2.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Fact {
    /// A forge table full past its bound or free again (protocol/service-process.md, 1.3).
    ForgeTable {
        words: Box<[u8]>,
    },
    /// A start step past the report bound (protocol/service-process.md, 1.3).
    StartWaiting {
        step: jig_domain_core::start::Phase,
        age: skein_lib::Duration,
        waits: Box<[u8]>,
    },
    /// A forge's link changed or was refused (temper/docs/design/domain/forge/adoption-person.md, 1).
    ForgeLink {
        forge: u16,
        note: forge::LinkNote,
    },
    /// A train's task make was refused (domain/landing-forming.md, 3).
    LandingRefused {
        task: u64,
        repository: skein_forge::api::Repository,
        branch: Box<[u8]>,
        why: Box<[u8]>,
        changes: Box<[u64]>,
    },
    LandingTableRefused {
        repository: skein_forge::api::Repository,
        branch: Box<[u8]>,
        limit: u32,
        holders: Box<[u64]>,
        changes: Box<[u64]>,
    },
    LandingRead(temper_engine_domain_forge::LandingReadNote),
    LinkDisagreed {
        change: u64,
        named: u64,
        landing: Option<u64>,
    },
    ConversionAskDropped {
        change: u64,
        landing: u64,
        kind: &'static str,
    },
    /// Start's Link repairs (domain/landing-forming.md, 4).
    LandingUnmade {
        task: u64,
        repository: skein_forge::api::Repository,
        branch: Box<[u8]>,
        changes: Box<[u64]>,
        readback: bool,
        landed: Box<[u64]>,
        unrecorded: Box<[u64]>,
    },
    HoldDropped {
        name: forge::Name,
        task: u64,
    },
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
    /// A branch head did not fit (temper/docs/design/domain/forge/keeping-up.md, 1).
    HeadRefused {
        task: Option<u64>,
        words: Box<[u8]>,
    },
    /// A whole brief failed with its count and fixed reason (jig/docs/design/domain/engine/briefs.md, 1.1).
    BriefFailed {
        task: u64,
        counts: &'static str,
        reason: Box<[u8]>,
    },
    /// A decoded call refused at admission, before any tool runs.
    CallAdmissionRefused {
        task: u64,
        attempt: u64,
        tool: Box<[u8]>,
        layer: &'static str,
        reason: &'static str,
    },
    /// A fenced answer reported a permanent preparation failure.
    PreparationFailed {
        task: u64,
        attempt: u64,
        preparation: Preparation,
    },
    /// The core refused a procedure's step; the root decides nothing (domain/root.md, 3, and domain/root-translation.md, 2.4).
    StepRefused {
        task: u64,
        step: u64,
        problem: jig_domain_tasks::Problem,
    },
}

#[derive(Hash, Debug)]
#[cfg_attr(
    target_pointer_width = "64",
    expect(clippy::large_enum_variant, reason = "decoded calls carry LLM choices with inline speed names")
)]
pub enum WorkerInput {
    /// A decoded push admission ask from jig's live attempt fence.
    Admit { asking: crate::wire::Asking, tag: u64, branch: Box<[u8]>, expected: Option<[u8; 32]>, commit: [u8; 32] },
    /// A decoded check report from a fenced worker ask (domain/forge/reds.md, 5.6).
    CheckResults { run: Token, attempt: Token, results: crate::wire::CheckResults },
    /// The protocol encoded the root's application answer, sent through the fleet's fence.
    Asked { channel: Token, run: Token, attempt: Token, number: u64, answer: jig_domain_link::AskAnswer },
    /// An authenticated opening, kept independently of the worker's hello (protocol/wires.md, 10.1).
    Opened { worker: Box<str>, statement: crate::WorkerStatement },
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
/// A row made by the root, joining the child's trial to jig's next-run wait
/// (temper/docs/design/domain/trial.md, 4 and 6).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct TrialRow {
    pub account: u32,
    pub task: Option<u64>,
    pub took: Option<u32>,
    pub step: temper_engine_domain_trials::Step,
    pub state: temper_engine_domain_trials::State,
    pub run_wait: Option<jig_domain_core::documents::TaskRunWait>,
}

/// Trials: temper/docs/design/domain/trial.md, 6.
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum Application {
    Checkpoint,
    OpenChange(forge::OpenChange),
    Tier(forge::TierActRequest),
    Follow { project: u32, follow: u32, act: forge::follows::FollowAct },
    Forge(forge::Ask),
    Try { account: u32, again: bool },
    Dismiss { what: forge::PersonFact },
    Looked { project: u32 },
    SetAway { project: u32, on: bool, until: Option<Wall> },
    Overturn { project: u32, number: u32 },
    AddDevice(forge::AddDevice),
    RemoveDevice { number: u32 },
    FileFinding(forge::FileFinding),
    HandFinding(forge::HandFinding),
    TriageFinding(forge::TriageFinding),
}
/// The family's terminal answer (client/forge/records.md).
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum ApplicationAnswer {
    Checkpoint(Result<Checkpointed, CheckpointWhy>),
    Tier(Result<(), forge::TierRefusal>),
    Follow(Option<Box<[u8]>>),
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
    SetDraft(forge::SetDraftAnswer),
    Overturn(forge::OverturnAnswer),
    Device(forge::DeviceAnswer),
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
    /// The service supplies rows after the root authorizes the read and counts held bytes.
    Limits {
        after: Option<Box<[u8]>>,
    },
    LimitHolds {
        after: Option<u64>,
    },
    Milestone {
        project: u32,
        task: u64,
    },
    History(forge::history::Query),
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
    Limits {
        after: Option<Box<[u8]>>,
        held: u64,
    },
    LimitHolds {
        rows: Box<[forge::LimitHold]>,
        next: Option<u64>,
    },
    Milestone(Option<forge::MilestoneTotals>),
    History(forge::history::Reply),
    /// Keep finding detail out of every journal delivery slot's inline size.
    Finding(Option<Box<forge::FindingAnswer>>),
    View(Box<temper_engine_domain_ways::View>),
    ProgramEnvironment,
    /// Keep the setting's payload out of every journal delivery slot's inline size.
    Setting {
        value: Option<Box<forge::settings::SizedValue>>,
        revision: Option<u32>,
    },
    /// The proposal's items and whether an enforcing approval covers an owned Forgejo (domain/forge/change/questions.md, 2).
    ProposalItems {
        items: Box<[(forge::settings::SettingKey, forge::settings::Edit)]>,
        owned_approval: bool,
    },
}
#[derive(Hash, Debug)]
#[cfg_attr(
    target_pointer_width = "64",
    expect(clippy::large_enum_variant, reason = "typed asks carry LLM choices with inline speed names")
)]
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
        /// Cap on a new sign-in's expiry, handed to people unchanged.
        until: Option<Wall>,
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
        read: Box<core::reads::Read>,
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
    /// Both limit-setting lists at this durable revision (configuration/limit-settings.md, 4).
    LimitSettingsCommitted {
        revision: u32,
        jig: Box<[jig_domain_core::settings::Override]>,
        temper: Box<[jig_domain_core::settings::Override]>,
    },
    ChoosePlace {
        request: Token,
        kind: forge::BoxKind,
        login: Box<[u8]>,
    },
    /// A making's copy mark has crossed the root (temper/docs/design/domain/forge/adoption.md, 9).
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
    /// A sent forge entry left without its outcome at the stop bound (domain/root-restart.md, 1.2).
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
    /// `domain/engine/restart.md`, 5).
    Busy(Box<forge::Event>),
    /// Tell workers after the word is durable and before closing their links (domain/root-restart.md, 1.2).
    Restarting {
        wait: skein_lib::Duration,
    },
    /// Every commit before the drain is durable (jig's `domain/engine/restart.md`, 5).
    Drained,
    Stop,
}
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum StoreRequest {
    Checkpoint {
        token: Token,
    },
    /// Replace the names before the catalogue decision commits (jig jig/docs/design/domain/engine/settings-catalogue.md, 1).
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
    /// Joined rows for a root-owned milestone part, answered by `PartLoaded`.
    PartRead {
        waiter: Token,
        range: Box<core::reads::Range>,
    },
    /// A held engine load after every preceding commit.
    Engine(engine_store::Out),
}
#[derive(PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::large_enum_variant, reason = "deliveries with a workspace stay inline in the bounded route queue")
)]
pub enum WorkerRequest {
    Settings(Box<[core::settings::WorkerSetting]>),
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
    /// Replace the clients' catalogue before its decision commits (jig jig/docs/design/domain/engine/settings-catalogue.md, 1).
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
    PartsLoaded { waiter: Token, result: engine_store::Answer },
    PartsFailed { waiter: Token },
    CheckResults { task: u64, attempt: u64, results: crate::wire::CheckResults },
    StartDone(jig_domain_core::start::Phase),
    Start(jig_domain_core::start::Action),
    Decoded { to: ReplyTo, body: Call },
    ForgeRead { reply: ReplyTo, sign_in: u64, scope: people::ApplicationScope, query: ForgeRead },
    Core(jig_domain_core::Event),
    Connector(jig_domain_contract::Connector, jig_domain_contract::Up),
    EngineStore(engine_store::In),
    Tasks(tasks::Event),
    RepairFindings { goal: u64, writes: Box<[forge::FindingWrite]> },
    People(people::Event),
    Fleet(fleet::Event),
    Forge(forge::Event),
    LandingAdmission { row: forge::LandingRow, names: Box<[forge::Name]> },
    ForgeSettings(skein_lib::Queue<forge::Request>),
    ForgeRestored,
    ForgeLimitHold { row: forge::LimitHold, words: Box<[u8]> },
    ForgeLimitRelease { task: u64, revision: u64 },
    StatusAccount { procedure: u64 },
    ProjectGoal(Box<jig_domain_core::ProjectionFeed>),
    GoalSubscribe(forge::Subscriber),
    AgreedGoal(Box<GoalWords>),
    Activate { context: Box<tasks::RunContext>, ready: bool },
    CloserReportReady { task: u64 },
    ChangeFromLoaded { task: u64, row: Option<Box<jig_domain_tasks::TaskRecord>> },
}

#[derive(Hash, Debug)]
pub(crate) struct BriefConnector {
    pub(crate) task: u64,
    pub(crate) kind: u16,
    pub(crate) gatherable: bool,
}

/// Preparation waits for the goal branch before its brief and workspace
/// (temper/docs/design/domain/forge/branches.md, 3).
#[derive(Hash, Debug)]
pub(crate) enum WorkspacePreparation {
    Ready(PreparedWorkspace),
    Brief(Box<jig_domain_contract::parts::Preparation>),
    Workspace { attempt: u64, context: Box<jig_domain_tasks::RunContext> },
}

impl WorkspacePreparation {
    pub(crate) fn prepared(&self) -> &PreparedWorkspace {
        match self {
            Self::Ready(prepared) => prepared,
            Self::Brief(_) | Self::Workspace { .. } => unreachable!("the workspace was answered before claiming"),
        }
    }

    pub(crate) fn into_prepared(self) -> PreparedWorkspace {
        match self {
            Self::Ready(prepared) => prepared,
            Self::Brief(_) | Self::Workspace { .. } => unreachable!("the workspace was answered before claiming"),
        }
    }
}

#[derive(Hash, Debug)]
pub(crate) struct PreparedWorkspace {
    pub(crate) workspace: forge_route::RunWorkspace,
    pub(crate) claim_names: Box<[forge::Name]>,
}

/// A claimed job's workspace items, kept for every offer of its attempt: a
/// worker that answers busy has the same attempt placed again (jig's
/// `domain/engine/executors.md`, 5). They go when the attempt answers or its
/// assignment is dropped, as the link's job does.
#[derive(Hash, Debug)]
pub(crate) struct JobWorkspace {
    pub(crate) attempt: u64,
    pub(crate) workspace: ForgeWorkspace,
    /// Offered to a worker at least once, so it awaits a worker's answer.
    pub(crate) offered: bool,
}

pub(crate) use jig_domain_hub_link::Payload;

/// Temper's connector parts beside the link's prepared assignment
/// (temper/docs/design/domain/root-translation.md, 2.1).
#[derive(Hash, Debug)]
pub(crate) struct AssignmentParts {
    pub sections: Box<[BriefSection]>,
    pub saved: Box<[u64]>,
    pub workstream: u64,
    pub workspace: ForgeWorkspace,
    pub answered: Box<[crate::CallRecord]>,
}

#[derive(Hash, Debug)]
pub(crate) enum Read {
    ChangeDelegate { task: u64, child: u64, step: u64, revision: u64 },
    History { load: u64, most: u32 },
    ChangeFrom { task: u64 },
    GoalWords(Box<GoalWords>),
    SettingsNote { to: Token, call: Box<Call> },
    Parts(Box<crate::route::parts::Flight>),
    Document(Box<crate::route::document::DocumentRead>),
    CloserReport(Box<forge_route::goal_report::Flight>),
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
        | Delivery::FindingsOwned { .. }
        | Delivery::PublicWait { .. }
        | Delivery::ForgeCommitted { .. }
        | Delivery::Fleet(_)
        | Delivery::Relay { .. }
        | Delivery::Start(_)
        | Delivery::Load { .. }) => Request::ToChild(Released(delivery)),
        delivery @ (Delivery::Host(_)
        | Delivery::AssignedJob { .. }
        | Delivery::Link { .. }
        | Delivery::Procedure { .. }
        | Delivery::Assigned { .. }
        | Delivery::Refuse { .. }
        | Delivery::PartBusy { .. }
        | Delivery::TurnBusy { .. }) => Request::Worker(WorkerRequest::Deliver(delivery)),
        delivery @ (Delivery::Push(_)
        | Delivery::Providers(..)
        | Delivery::WorkerFigures { .. }
        | Delivery::PlaygroundNote { .. }
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
        Delivery::PartLoad { waiter, range } => Request::Store(StoreRequest::PartRead { waiter, range }),
        Delivery::EngineStore(load) => Request::Store(StoreRequest::Engine(load)),
        Delivery::Restarting { wait } => Request::Restarting { wait },
        Delivery::LimitSettingsCommitted { revision, jig, temper } => {
            Request::LimitSettingsCommitted { revision, jig, temper }
        }
        Delivery::Drained => Request::Drained,
        Delivery::Bound { request, answer } => Request::Bound { request, answer },
        Delivery::ForgeCall { call, repository, op } => Request::Forge { call, repository, op },
    }
}

/// A forge write a settling stop left without its outcome (domain/root-restart.md, 1.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Unsettled {
    pub entry: u64,
    pub task: u64,
}

/// A failed run's volatile operator evidence (temper/docs/design/domain/worker/hosted-runs.md, 1.3).
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

/// A refused assignment's volatile operator evidence (temper/docs/design/domain/worker/hosted-runs.md, 1.3).
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

/// An accepted job's volatile operator evidence (temper/docs/design/domain/worker/jobs.md).
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

/// The store's cut and figures (temper/docs/design/protocol/checkpoints.md, 3).
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct Checkpointed {
    pub name: Box<[u8]>,
    pub store: u64,
    pub secrets: u64,
    pub linked: u32,
    pub copied: u32,
    pub bytes: u64,
    pub paused: u64,
    pub took: u64,
    pub held: u64,
    pub removed: u32,
    pub skipped: u32,
}
/// Refusals a serving engine meets, plus deployment ownership (protocol/checkpoints.md, 3).
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum CheckpointWhy {
    NotOwner,
    InProgress,
    Unavailable,
    Files,
    Unrecognized { path: Box<str> },
    Vanished { path: Box<str> },
    Failed { path: Box<str>, error: Box<str> },
}
