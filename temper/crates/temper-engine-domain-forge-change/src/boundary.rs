//! Values owned by the connector top and passed to the change policy
//! (domain/forge/change/README.md and domain/forge/queues.md and domain/forge/README.md, section 6).
//!
//! The top keeps `Change`, including its pending phase, counters and clean
//! predecessor heads. This module keeps nothing and knows no API tokens.
//! `Facts` and `Heard` form a current snapshot, not a wake event.
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

/// A forge commit identifier, translated at the connector top.
pub type Commit = [u8; 32];

/// The status of a fact or requirement at the named head.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Status {
    /// No answer has been observed.
    Unknown,
    /// The answer remains in flight.
    Pending,
    /// The requirement holds.
    Passed,
    /// The requirement failed.
    Failed,
}

/// Whether a gate needs the current head or accepts clean predecessors.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Freshness {
    /// The verdict must name the current head.
    Exact,
    /// A verdict at a clean predecessor head remains valid.
    Clean,
}

/// How the top obtains a gate verdict.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GateKind {
    /// An agent reviews the change.
    Agent,
    /// A person approves the change.
    Person,
    /// A configured check supplies a verdict.
    Check,
}

/// One gate the change or its project asks for.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Gate {
    pub number: u64,
    pub kind: GateKind,
    pub blocking: bool,
    pub freshness: Freshness,
    pub eager: bool,
}

/// A gate's current verdict and the head it names.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GateReport {
    pub number: u64,
    pub head: Commit,
    pub status: Status,
}

/// Failures to obtain a gate's verdict at the change's last head.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GateFailure {
    pub number: u64,
    pub count: u32,
}

/// Why a repair delegate is needed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Repair {
    /// CI failed on the change's head.
    Ci,
    /// A blocking gate requested changes.
    Gate(u64),
    /// CI failed after a clean update from the base.
    Semantic,
    /// A landing's gate failed at its candidate (domain/landing.md, 7.3).
    Landing { gate: u16 },
    /// A person's request for changes on the forge, at the head they reviewed (domain/forge/change/gates.md, 1).
    Review { head: Commit },
}

/// The part of a change's authority its next delegate needs.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AuthorityLack {
    Tasks,
    Depth,
    Spend,
}

/// The amendment and release coordinates at the authority decision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AuthorityRecovery {
    pub task: u64,
    pub spec: u64,
    pub hold: u64,
    pub tasks: u64,
}

/// A precise, durable authority hold (domain/forge/change/repairs.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AuthorityHold {
    pub recovery: AuthorityRecovery,
    pub lack: AuthorityLack,
    pub had: u64,
    pub released: bool,
}

/// Why a change is held for its requester or a person.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Hold {
    /// A forge wait exceeded its stall bound.
    Stalled,
    /// Another repair would exceed the configured bound.
    Repairs,
    /// Another conflict resolution would exceed its bound.
    Resolutions,
    /// Another mechanical update would exceed its bound.
    Updates,
    /// The held branch moved outside its writer.
    BranchMoved,
    /// The held branch disappeared.
    BranchMissing,
    /// The pull request closed without a merge.
    PullClosed,
    /// The pull request was retargeted.
    Retargeted,
    /// A delegate or effect failed permanently.
    Failed,
    /// A person rejected a required gate.
    Rejected,
    /// The next delegate is beyond the change's remaining authority.
    Authority(AuthorityHold),
    /// Another question would exceed the change's bound.
    Questions,
}

/// A change's committed place through verification (temper/docs/design/domain/after-landing.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum State {
    /// A producing delegate has yet to push the branch.
    Producing { requested: bool },
    /// A pushed branch needs a pull request.
    Opening { requested: bool },
    /// A released change is recreating its deleted branch.
    Recreating { head: Commit },
    /// A released change is reopening or retargeting its pull request.
    Reopening { requested: bool, retarget: bool },
    /// CI is read on the pushed head.
    Checking { head: Commit },
    /// Required gates are sought on the checked head.
    Gating { head: Commit, asked: Box<[u64]> },
    /// Ready at `head`: its landing prepares, proves and lands it (domain/landing.md, 3).
    Ready { head: Commit, ready_since: Wall },
    /// The prepare's rebase onto `main` is outstanding (domain/landing.md, 5.2).
    Preparing { from: Commit, main: Commit, ready_since: Wall },
    /// CI failed; cancelled gate delegates must end before repair is funded.
    CancellingGates { head: Commit },
    /// The change is ready, waiting its turn.
    Queued { head: Commit, ready_since: Wall },
    /// The change owns the queue through an update and landing.
    First { head: Commit, ready_since: Wall },
    /// A mechanical update of the branch is outstanding.
    Updating { from: Commit, base: Commit, ready_since: Wall },
    /// A conflict keeps this base until every pushing child has ended.
    WaitingProducer { base: Commit },
    /// A conflict resolution delegate is outstanding.
    Resolving { requested: bool, base: Commit },
    /// A repair delegate is outstanding.
    Repairing { requested: bool, why: Repair },
    /// A merge at the named head is outstanding.
    Landing { head: Commit },
    /// Merged at this commit and verifying; `since` is when the top first saw the merge.
    Landed { merge: Commit, since: Wall },
    /// The producer concluded that the request does not apply, with no landing.
    NoChange { words: Box<[u8]> },
    /// The task ended after its outstanding effects and holdings settled.
    Ended,
    /// No write is made until a release lifts this cause.
    Held { was: Box<State>, why: Hold },
    /// A tier blames this landing; wait for its failures to be repaired.
    Red { merge: Commit, since: Wall },
    /// Every tier following its branch cleared this landing.
    Cleared { merge: Commit },
    /// The verification bound expired or the verifying change was cancelled.
    Uncleared { merge: Commit },
}

/// How the change's project lands (domain/landing.md, 1; domain/forge/change/README.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Way {
    Queue,
    Turn,
}

/// A pending dependency of the change as the top reads it each step
/// (temper/docs/design/domain/forge/change/questions.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pending {
    pub task: u64,
    pub kind: PendingKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PendingKind {
    /// A change into the same repository and landing branch: its pushed head, if any, and
    /// whether it is itself kept by a pending dependency.
    Change { head: Option<Commit>, kept: bool },
    /// A change into another repository: it orders the landings alone.
    Elsewhere,
    /// Anything else: no head to start from, so the producer waits for it to be done.
    Other,
}

/// One dependency's pushed head a producer started from.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StartedFrom {
    pub task: u64,
    pub head: Commit,
}

/// The change's early start (temper/docs/design/domain/forge/change/questions.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Early {
    /// No producer has started early.
    None,
    /// Its producer started from the landing branch's tip with these heads merged in, in order.
    Started(Box<[StartedFrom]>),
    /// Its early start is spent: refused for a conflict, or its early producer ended failed.
    Spent(Box<[StartedFrom]>),
}

/// The top-owned procedure record for one change task.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub early: Early,
    pub task: u64,
    pub way: Way,
    /// Its landing's task, once it is a member (domain/landing.md, 4.2).
    pub landing: Option<u64>,
    pub state: State,
    pub gates: Box<[Gate]>,
    pub clean: Box<[Commit]>,
    /// One bounded count per gate; cleared when the head changes or the task is released.
    pub gate_failures: Box<[GateFailure]>,
    /// A lifted hold awaiting the next fresh forge snapshot.
    pub released: Option<Hold>,
    pub repairs: u32,
    pub resolutions: u32,
    pub updates: u32,
    pub since: Wall,
    pub ready_since: Option<Wall>,
    pub owns_turn: bool,
    pub last_head: Option<Commit>,
}

/// Observed state of the change's pull request.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Pull {
    /// No pull request for the branch pair was found.
    Missing,
    /// The open pull request names this head and base.
    Open { head: Commit, base: Commit },
    /// Someone closed the pull request without merging it.
    Closed,
    /// The pull request merged at this commit.
    Merged { commit: Commit },
}

/// CI observed on a specific head.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ci {
    pub head: Commit,
    pub status: Status,
}

/// What its landing asks of the change (domain/landing.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Rebase the last known head onto the requested main tip.
    Prepare { head: Commit, main: Commit },
    /// Repair a real gate failure at the landing's candidate.
    Repair { gate: u16, candidate: Commit },
}

/// What the top reads of the landing's verification each step
/// (temper/docs/design/domain/after-landing.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Verification {
    /// No tier follows its landing branch, or it has not landed.
    Unfollowed,
    /// Some tier following its branch has not judged it yet.
    Verifying,
    /// A tier blames it and no later run has passed those failures.
    Red,
    /// Every tier following its branch cleared it.
    Cleared,
}

/// Fresh forge and queue facts gathered by the top.
#[expect(clippy::struct_excessive_bools, reason = "independent root-supplied facts")]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Facts {
    /// Pending dependencies in the order of the change's dependencies.
    pub pending: Box<[Pending]>,
    /// Its landing's current request; absent under queue.
    pub ask: Option<Ask>,
    pub branch: Option<Commit>,
    /// The change has a title or an existing pull request (domain/forge/change/README.md, 1).
    pub titled: bool,
    pub expected_base: Commit,
    pub pull: Pull,
    pub ci: Ci,
    pub base_ci: Ci,
    pub base_tip: Commit,
    /// Whether the head contains `Updating.base` while updating, otherwise `base_tip`.
    /// The asked base remains the update's even when its landing branch moves on.
    pub contains_base: Status,
    /// Whether the landing branch's tip contains the change's head (temper/docs/design/domain/forge/change/README.md, 2).
    pub contains_head: Status,
    /// Whether a merge job is configured, declared and authorized (temper/docs/design/domain/forge/change/repairs.md).
    pub merge_job: bool,
    /// The change's current delegate, as the forge's row keeps it (`ChangeRow::delegate`);
    /// `None` when it has none.
    pub delegate: Option<Delegate>,
    /// A catch-up's commit (`ChangeRow::catch_up`), `None` for every other change.
    pub catch_up: Option<Commit>,
    /// A close job produces this change (temper/docs/design/domain/forge/change/README.md, 1).
    pub close: bool,
    /// Whether the landing tip contains the catch-up commit (temper/docs/design/domain/forge/change/README.md, 2).
    pub contains_catch_up: Status,
    /// Whether its landing branch's rules ask up to date (domain/forge/change/gates.md, 2).
    pub updates: bool,
    pub mergeable: Status,
    pub gates: Box<[GateReport]>,
    /// The head of the top's outstanding gate delegate, until its terminal result.
    pub live_gate: Option<Commit>,
    pub writer_taken: bool,
    /// Its branch is its requester's held branch (domain/forge/change/README.md, 1): a move with no
    /// drift and no writer in the slot is its new head, checked again (8.5).
    pub requester_branch: bool,
    pub drift: Option<Hold>,
    pub first: bool,
    pub may_merge: bool,
    pub queue_repair_active: bool,
    /// The task has a pending dependency (jig's `domain/tasks/lifecycle.md`, 8), so it is not ready.
    pub kept: bool,
    /// An open question of the change's current agent stands (domain/forge/change/repairs.md and domain/forge/change/questions.md, 1).
    pub asking: bool,
    /// The end of the window Forgejo is still settling in, when open.
    pub settling: Option<Wall>,
    pub verification: Verification,
    /// The project's `verification_bound` (temper/docs/design/protocol/configuration/engine.md).
    pub verification_bound: Duration,
}

/// Committed delegate and effect results supplied by the top.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectResult {
    /// No effect result is outstanding.
    None,
    /// The requested effect is still outstanding.
    Pending,
    /// The effect was made or found again.
    Made,
    /// A mechanical update conflicted with the base.
    Conflict,
    /// An effect failed permanently.
    Failed,
}

/// Remaining spend and the reserves of the two agent charters.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DelegateSpend {
    pub recovery: AuthorityRecovery,
    pub left: u64,
    pub coding: u64,
    pub review: u64,
}

/// Current task and effect standing, independent of the wake trigger.
#[expect(clippy::struct_excessive_bools, reason = "independent task and effect standing")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Heard {
    /// A counted person's request refused the merge (temper/docs/design/domain/forge/change/gates.md, 1).
    pub review_request: Option<Commit>,
    /// The head its live producing delegate conflicted on, until that delegate ends.
    pub start_conflict: Option<Commit>,
    /// Current authority, translated from jig's live accounting.
    pub lacking: Option<AuthorityHold>,
    pub spend: Option<DelegateSpend>,
    pub delegate: Status,
    /// Its delegate has a claimed or running attempt, or is closing: it finishes
    /// as a run does (temper/docs/design/domain/forge/change/repairs.md).
    pub delegate_running: bool,
    /// Any pushing child (producer, repair, resolution or merge job) is still live,
    /// independent of the current delegate slot (domain/forge/change/repairs.md).
    pub delegate_live: bool,
    pub effect: EffectResult,
    pub released: bool,
    pub cancelled: bool,
}

/// The delegate the top should create in the same decision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Delegate {
    /// Produce and push the change's branch.
    Produce,
    /// Settle the close request at the landing branch tip.
    Close,
    /// Repair CI or a gate failure on the branch.
    Repair(Repair),
    /// Resolve a conflicting update from a merge in progress.
    Resolve { base: Commit },
    /// Obtain a gate's verdict at the current head.
    Gate { number: u64, head: Commit },
    /// Merge `base` into the head `from`, absent for a catch-up (temper/docs/design/domain/forge/change/repairs.md).
    Merge { base: Commit, from: Option<Commit> },
}

/// The effect the top should commit after checking authority.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Effect {
    /// Open a pull request for the pushed branch.
    Open,
    /// Recreate a deleted held branch at its last known head.
    CreateBranch { head: Commit },
    /// Reopen a pull request closed by another person.
    Reopen,
    /// Restore the change's landing branch after a retarget.
    Retarget,
    /// Merge the landing branch into the change's branch.
    Update { head: Commit, base: Commit },
    /// Rebase the pull request onto `main` (domain/landing.md, 5.2).
    Prepare { head: Commit, main: Commit },
    /// Merge the pull request at its checked head.
    Merge { head: Commit, base: Commit },
}

/// What the top does with the procedure's one decision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decision {
    /// Nothing is due in this snapshot.
    None,
    /// Wait for a fact or deadline to change.
    Wait { until: Option<Wall> },
    /// Create one bounded delegate.
    Delegate(Delegate),
    /// Commit one effect to the top's outbox.
    Effect(Effect),
    /// Cancel live gate delegates at the head whose CI failed.
    CancelGates { head: Commit },
    /// Cancel the live producing delegate: its early start conflicted (domain/forge/change/questions.md, 1).
    CancelProducer,
    /// Cancel the live CI repair: CI passed at this head and no run is in progress (8.4).
    CancelRepair { head: Commit },
    /// A CI repair ended at this green head. The top records it and steps on (8.4).
    RepairEnded { head: Commit },
    /// The change's CI hold ended; the root releases its task as the deployment's (8.4).
    EndHold,
    /// Tell the top this change is ready for its queue or landing.
    Ready,
    /// Ask the top for the branch's one queue repair.
    QueueRepair,
    /// Finish with the observed merge commit.
    Finish { merge: Commit },
    /// Finish successfully with the producer's no-change explanation.
    NoChange,
    /// Finish cancellation after the top settles pending effects.
    Cancel,
    /// Hold the task and tell its requester why.
    Hold(Hold),
}

/// A next state and its decision from one coherent snapshot.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Stepped {
    pub change: Change,
    pub decision: Decision,
}

/// Bounds on delegates, updates, stalls and supplied gate facts
/// (temper/docs/design/domain/agent/tools.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub repairs: u32,
    pub resolutions: u32,
    pub updates: u32,
    /// Questions a change takes from its making or its last release (domain/forge/change/questions.md, 1).
    pub questions: u32,
    /// Tasks a change's delegate may make beside its foremen.
    pub tasks: u32,
    pub stall: Duration,
    pub gates: u32,
    pub clean_heads: u32,
}

/// How a change takes its agents' questions (domain/forge/change/questions.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Route {
    #[default]
    None,
    Foreman,
    Up,
}

/// Open questions a change's record keeps at most (domain/forge/change/questions.md, 1).
pub const QUESTIONS_OPEN: usize = 128;
/// Bytes of a question's first line kept on the record and shown on the panel.
pub const QUESTION_LINE: usize = 256;

/// A change's relay of its agents' questions (domain/forge/change/questions.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Questions {
    pub route: Route,
    /// Questions taken since the change was made or last released.
    pub taken: u32,
    /// Open questions in the order taken; the first is the one being worked.
    pub open: Box<[Open]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Open {
    /// The asking task: the change's producing task, a repair or a resolution.
    pub asker: u64,
    /// The question's number in the change's inbox, which its answer names.
    pub number: u64,
    /// Its words' first line, at most `QUESTION_LINE` bytes, cut on a UTF-8 boundary, marked "…".
    pub line: Box<[u8]>,
    /// The question's whole words, within the deployment's message bound.
    pub words: Box<[u8]>,
    pub stands: Stands,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Stands {
    /// Waiting its turn, its foreman's delegation or its passing up.
    Taken,
    /// With its foreman: `None` while the delegate is asked, its task once made.
    Foreman { task: Option<u64> },
    /// Asked of the change's requester, the change kept by its own question.
    Asked,
    /// Answered by the requester and held while the change is kept by a pending dependency.
    Held { answer: Box<[u8]> },
}

/// One change ready for a landing branch's derived queue.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ready {
    pub task: u64,
    pub priority: i32,
    pub since: Wall,
}
