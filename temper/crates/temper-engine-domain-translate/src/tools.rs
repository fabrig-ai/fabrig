//! Typed host-tool vocabulary (temper/docs/design/domain/routing.md, 2).
use alloc::boxed::Box;
use jig_domain_core::{Delegate, RunPolicy};
use jig_domain_tasks as tasks;
use temper_engine_domain_forge as forge;

/// A decoded value of temper's settings kinds, retained for its request's decision
/// (temper/docs/design/domain/routing.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct KindValue {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub value: forge::settings::SizedValue,
}

pub use temper_engine_domain_room::asking;
pub use temper_engine_domain_store::{DocumentAction, FindingToolAnswer, FollowToolAnswer, RedToolAnswer};

/// A run of this policy is given no repository: an asking charter's, or one that
/// names none of inspect, modify and shell (temper/docs/design/domain/agent/charter.md, 2).
#[must_use]
pub fn no_workspace(policy: &RunPolicy) -> bool {
    asking(policy.tools) || !(policy.inspect || policy.modify || policy.shell)
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
    /// File, triage or read findings in the caller's project (domain/agent/tools.md).
    Finding(FindingCall),
    Red(RedCall),
    Follow(FollowCall),
    /// Read related task specifications or proven descendant reports.
    ReadTask {
        task: u64,
        field: jig_domain_core::reads::TaskTextField,
        from: u64,
    },
    /// Read settings without retaining a named-call record.
    Settings {
        read: jig_domain_core::SettingsRead,
        from: u32,
    },
    /// Read by one address line, from a byte offset of its answer (domain/agent/ask-temper.md, 1).
    See {
        line: Box<[u8]>,
        from: u32,
    },
    /// Write the chat's one document whole, or one section of it (domain/agent/ask-temper.md, 3).
    Document {
        action: DocumentAction,
        body: Box<[u8]>,
    },
    /// Keep, change or drop a line about the chat's person (domain/agent/ask-temper.md, 4).
    Remember {
        action: RememberAction,
    },
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
    ProposeSettingsNote {
        against: u32,
        note: u64,
        revision: u32,
        reason: Box<[u8]>,
    },
    /// Write or correct one scoped note by the revision this run recalled.
    Note {
        entry: jig_domain_notes::New,
        recalled: Option<u32>,
    },
    /// Replace a range of a note's body lines by the revision this run recalled.
    NoteLines {
        entry: jig_domain_notes::New,
        recalled: u32,
        range: jig_domain_notes::LineRange,
    },
    /// Read one bounded page of notes by name or description.
    Recall {
        by: jig_domain_notes::Recall,
        page: u32,
    },
    /// Queue one checked forge write with a root-derived idempotency key.
    EffectForge {
        repository: skein_forge::api::Repository,
        resource: forge::What,
        write: Box<skein_forge::api::Write>,
    },
    /// A host read resolved in its project before resource authority admission.
    /// A zero file/job `max_bytes` asks for the configured read limit.
    /// An empty `Branch` or `Protection` branch asks for the default branch.
    ReadNamed {
        repository: Option<Box<[u8]>>,
        read: skein_forge::api::Read,
    },
    /// A file at a branch's head: the branch, then the file at the head it answered.
    /// An empty `branch` asks for the default branch; a zero `max_bytes` the configured read limit.
    ReadBranchFile {
        repository: Option<Box<[u8]>>,
        branch: Box<[u8]>,
        path: Box<[u8]>,
        max_bytes: u32,
    },
    /// Read one adopted forge resource through the connector's bounded fresh lane.
    ReadForge {
        repository: skein_forge::api::Repository,
        read: skein_forge::api::Read,
    },
    /// Ask a covering ancestor or person to carry one action the caller cannot take alone.
    Propose {
        action: ProposedAction,
        reason: Box<[u8]>,
        as_holder: bool,
    },
    /// Decide one pending proposal currently addressed to this task.
    Decide {
        proposer: u64,
        proposal: u64,
        decision: ProposalChoice,
    },
    /// Withdraw this task's still-pending proposal.
    Withdraw {
        proposal: u64,
    },
    /// Resolve one held delegate currently waiting at this task.
    DecideEscalation {
        task: u64,
        revision: u64,
        decision: EscalationChoice,
    },
    /// Expand a goal in the run's words into one ordinary named delegation.
    Goal {
        words: Box<[u8]>,
        advisor: bool,
        agree: bool,
    },
    /// Expand a project repository change into one ordinary named delegation.
    Change {
        changes: Box<[ChangeCall]>,
    },
    /// Create one authorized batch of direct task delegates.
    Delegate {
        batch: Box<[Delegate]>,
    },
    /// Amend one live delegate after authority fitting and finite source checks.
    Amend {
        target: u64,
        revision: u64,
        amendment: tasks::Amendment,
        /// The host omitted parameters; jig keeps the current task's before admission.
        keep_parameters: bool,
    },
    /// Cancel one live delegate's whole subtree with a bounded reason.
    Cancel {
        target: u64,
        reason: Box<[u8]>,
    },
    /// Release one held delegate.
    Release {
        target: u64,
        revision: u64,
        again: Option<bool>,
    },
    /// Send whole bounded words to a task the caller references.
    Message {
        target: u64,
        form: MessageForm,
        words: Box<[u8]>,
    },
    /// Give two tasks referenced by the caller reciprocal references.
    Introduce {
        left: u64,
        right: u64,
    },
    /// Add one bounded task-state or timer interest.
    Subscribe {
        kind: tasks::SubscriptionKind,
    },
    /// Subscribe this task to typed connector news, with overlap paths for landings.
    SubscribeForge {
        topic: forge::Topic,
        own_change: Option<u64>,
        paths: Box<[Box<[u8]>]>,
    },
    /// Resolve an owner/name or the project's sole readable adoption before subscribing.
    SubscribeNamedForge {
        repository: Option<Box<[u8]>>,
        topic: forge::Topic,
        own_change: Option<u64>,
        paths: Box<[Box<[u8]>]>,
    },
    /// Remove one current interest by its root-issued number.
    Unsubscribe {
        subscription: u64,
    },
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

/// A task acts only on its own follows (domain/forge/follows.md, 5 and 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FollowCall {
    Act { follow: u32, act: forge::follows::FollowAct },
    List,
    Show { follow: u32 },
}

/// A goal's red acts; the caller's task is always the goal (forge/reds.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RedCall {
    Record { key: Box<[u8]>, at: Box<[u8]>, why: Box<[u8]> },
    End { key: Box<[u8]>, why: Box<[u8]> },
    List,
}

/// Finding acts never take a project or actor from the agent (domain/areas.md, 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FindingCall {
    File {
        kind: forge::FindingKind,
        severity: forge::Severity,
        area: Option<Box<[u8]>>,
        title: Box<[u8]>,
        body: Box<[u8]>,
    },
    Hand {
        number: u32,
        area: Box<[u8]>,
        why: Box<[u8]>,
    },
    Triage {
        number: u32,
        act: forge::Triage,
    },
    List {
        state: Option<forge::FindingState>,
        mine: bool,
    },
    Show {
        number: u32,
    },
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
