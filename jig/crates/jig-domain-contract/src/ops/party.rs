//! Party operations and their retained outcomes, distinct from task admission facts.

use super::Role;
use crate::lifecycle::Adoption;
use alloc::boxed::Box;
use skein_lib::{Duration, Token, Wall};

/// One act a connector makes in the deployment's name (domain/people/acts.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Act {
    MakeProject {
        maker: u64,
        name: Box<str>,
        policy: u32,
    },
    Close {
        project: u32,
        task: u64,
        grace: Option<Duration>,
        reason: Box<[u8]>,
    },
    /// The value travels beside the act, carried as a client's key is.
    AddKey {
        provider: u32,
        join: Option<u32>,
        digest: [u8; 32],
    },
    RemoveAccount {
        account: u32,
    },
    Adopt {
        project: u32,
        adoption: Adoption,
    },
    /// Delete a project: closed at once, its tasks ended and erased, then its rows
    /// (jig/docs/design/domain/people/acts.md, 2). Answered once the project is closed.
    RemoveProject {
        project: u32,
        grace: Option<Duration>,
    },
}

/// A task-derived recipient, retained only as a bounded inbox reference.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Whom {
    /// One authenticated deployment person.
    Person(u64),
    /// Every current holder of a project role may see the entry.
    Role { project: u32, role: u32 },
}

/// The reason a task currently appears in a person's inbox.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum EntryKind {
    /// A numbered question in a task requested by the person.
    Question { message: u64 },
    /// A pending action waiting for a decision.
    Proposal { proposer: Proposer, number: u64 },
    /// A held task waiting for a decision at this revision.
    Escalation { revision: u64 },
    /// An active person-executed task.
    PersonTask,
    /// A reply in a task requested by the person.
    Reply { message: u64 },
    /// A committed reply belonging to one conversation's independent read position.
    ChatReply { message: u64 },
}

/// Immutable proposal maker identity (client/domain/contract/requests.md, section 2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Proposer {
    Task(u64),
    Party(u64),
}

/// Committed semantic proposal choice in a keyed person answer.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProposalChoice {
    Accepted,
    Rejected,
    Passed,
    Withdrawn,
    Stale,
}

/// Semantic kind of a committed decision, returned to a caller or earlier
/// loser of a race; carries no second task state (domain/people/acts.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EscalationChoice {
    /// Hold released.
    Released,
    /// Task remains held with rejection reason.
    Rejected,
    /// Waiting moved to policy role.
    Passed,
}

/// The retry, refusal or retained fact carried by an answer (domain/people/acts.md, 1–3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutcomeFamily {
    Again(Again),
    SignedOut,
    Compound,
    Refused,
    CompoundRefused,
    ServiceMade(u64),
    RolesSet(u32),
    ProjectMade(u32),
    Settled,
    ServiceReissued,
}

/// Root's answer for one routed flight; people replies to each bounded waiter and saves
/// final outcomes with the parent's task decision. (domain/people/acts.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Outcome {
    WorkerPauseAsked {
        worker: u32,
    },
    /// The gathering keeps its concluding word.
    Concluded {
        task: u64,
    },
    AccountReading {
        count: u32,
    },
    /// A restore's proposal and omitted settings (jig/docs/design/domain/engine/settings.md, 10).
    Restored {
        proposal: u64,
        to: u32,
        left_out: Box<[Setting]>,
    },
    /// The project is closed and its deletion kept; the rest completes on its own
    /// (jig/docs/design/domain/people/acts.md, 2).
    ProjectRemoved {
        project: u32,
    },
    /// The sign-in took the address; its end shows on the accounts watch.
    AccountFinishing {
        source: u32,
    },
    /// One whole settings change committed at this revision.
    SettingsChanged {
        revision: u32,
    },
    /// Live agent tasks moved to the newest, and those that finish on their revision (domain/engine/settings.md, 4).
    MovedToNewest {
        newest: u32,
        moved: u32,
        stayed: u32,
    },
    /// All operations were made; their issued messages start here, or at zero for none.
    Compound {
        project: u32,
        operations: u32,
        first_message: u64,
    },
    /// No operation was made; the first failure has this zero-based place.
    CompoundRefused {
        at: u32,
        why: Refusal,
    },
    /// The batch was made in order.
    Created {
        first: u64,
        last: u64,
        given: Option<Given>,
    },
    /// The named read position was accepted; it never moves backwards.
    MarkedSeen {
        seen: Seen,
    },
    /// Connector-owned durable outcome named by token; the root asks its owner for the value.
    Applied {
        connector: u16,
        token: Token,
    },
    /// Unsaved retry answer; the request may be sent again.
    Again(Again),
    /// Unsaved expired-sign-in answer.
    SignedOut,
    /// A note correction or deletion was committed with its keyed answer.
    NoteEdited {
        name: u64,
    },
    /// A service party and its project role were committed together.
    ServiceMade {
        person: u64,
    },
    /// Resource adoption and its connector-known party seed committed together.
    Adopted {
        project: u32,
    },
    /// An explicit proposal awaits the named policy role.
    Proposed {
        proposal: u64,
        to: u32,
    },
    /// This party withdrew its pending proposal.
    Withdrawn {
        proposal: u64,
    },
    /// A person stopped one live task for a later release or decision.
    Left {
        task: u64,
    },
    Stopped {
        task: u64,
    },
    /// A person cancelled one task and its descendants.
    Cancelled {
        task: u64,
    },
    /// A person released a held task.
    Released {
        task: u64,
    },
    Lifted {
        hold: u64,
        released: u32,
    },
    /// One role-addressed task was durably claimed.
    PersonTaken {
        task: u64,
    },
    /// The claimant durably returned the task to its role.
    PersonHandedBack {
        task: u64,
    },
    /// An addressed person's result was durably admitted.
    PersonAnswered {
        task: u64,
    },
    /// The task now belongs to the requesting person, with its remaining funding transferred.
    Moved {
        task: u64,
    },
    /// Keyed answer to one current or ended proposal decision.
    ProposalDecided {
        proposer: Proposer,
        proposal: u64,
        by: u64,
        at: Wall,
        choice: ProposalChoice,
        repeated: bool,
    },
    /// Words were durably admitted under the root-issued message number.
    Said {
        task: u64,
        message: u64,
    },
    /// A named question was answered with a durable message.
    QuestionAnswered {
        task: u64,
        question: u64,
        message: u64,
    },
    /// The named task accepted this person's amendment.
    Amended {
        task: u64,
        released: bool,
    },
    /// Root accepted one roster and all affected `Waiting` recipients atomically; saved keyed
    /// replay does not apply the roster again.
    RolesSet {
        /// Project whose membership changed; no task authority or funding changed.
        project: u32,
    },
    /// The full committed policy now applies to later checks; existing grants remain.
    PolicyChanged {
        project: u32,
    },
    /// The project now carries its new name.
    Renamed {
        project: u32,
    },
    /// A configured policy became a new project owned by its maker.
    ProjectMade {
        project: u32,
    },
    /// One person's finite pool was opened or resized in its original period.
    PoolSet {
        project: u32,
        person: u64,
    },
    /// Root's committed current or ended decision for one task revision; first commit decides
    /// a race.
    EscalationDecided {
        /// Named held chat.
        task: u64,
        revision: u64,
        /// Authenticated winning person, including earlier races.
        by: u64,
        at: Wall,
        choice: EscalationChoice,
    },
    /// An account's sign-in is under way for this source; its end is the accounts child's fact.
    AccountSignInStarted {
        source: u32,
        address: Box<[u8]>,
    },
    /// The source was signed out, and whether its provider confirmed the revocation.
    AccountSignedOut {
        confirmed: bool,
        failed_hosts: Box<[AccountHostFailure]>,
    },
    /// The LLM key was kept at this source.
    AccountKeyKept {
        source: u32,
    },
    /// The holder declared this program source under its durable name.
    AccountLoginAsked {
        number: u32,
    },
    AccountProgramDeclared {
        source: u32,
        name: Box<[u8]>,
    },
    /// The holder's account change finished.
    AccountChanged,
    /// The move is under way; its record is taken from the source once, outside the answer.
    AccountMovingOut {
        account: u32,
        number: u32,
        source: u32,
    },
    /// A moved record was adopted, making or renewing this account.
    AccountMovedIn {
        account: u32,
        renewed: bool,
    },
    /// Every signed-in source answered its sign-out.
    AccountRemoved {
        confirmed: bool,
        failed_hosts: Box<[AccountHostFailure]>,
    },
    /// Final refusal, retained under the request key.
    Refused(/** Final refusal facts, retained for requests that keep an answer. */ Refusal),
    /// A service’s sign-ins ended; no answer is kept.
    ServiceReissued {
        party: u64,
    },
    /// The project is public or private from this decision (jig/docs/design/domain/people/public-reading.md, 2).
    VisibilitySet {
        project: u32,
        public: bool,
    },
}

/// A configured worker that did not confirm the original operation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AccountHostFailure {
    pub name: Box<[u8]>,
    pub why: AccountHostWhy,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountHostWhy {
    NotInContact,
    Lost,
    Failed,
}

/// Semantic outcome of a deployment act (domain/people/acts.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActStatus {
    Closed,
    Again,
    NotOffered,
    NoOwner,
    Ended,
    Refused,
}

impl Proposer {
    /// The maker’s number in its own family.
    #[must_use]
    pub fn number(self) -> u64 {
        match self {
            Self::Task(number) | Self::Party(number) => number,
        }
    }
}

impl Outcome {
    /// The answer's disposition and the identities needed when linking retained facts.
    #[must_use]
    pub const fn family(&self) -> OutcomeFamily {
        match self {
            Self::Again(why) => OutcomeFamily::Again(*why),
            Self::SignedOut => OutcomeFamily::SignedOut,
            Self::Compound { .. } => OutcomeFamily::Compound,
            Self::Refused(_) => OutcomeFamily::Refused,
            Self::CompoundRefused { .. } => OutcomeFamily::CompoundRefused,
            Self::ServiceMade { person } => OutcomeFamily::ServiceMade(*person),
            Self::ServiceReissued { .. } => OutcomeFamily::ServiceReissued,
            Self::RolesSet { project } => OutcomeFamily::RolesSet(*project),
            Self::ProjectMade { project } => OutcomeFamily::ProjectMade(*project),
            Self::Created { .. }
            | Self::MarkedSeen { .. }
            | Self::Applied { .. }
            | Self::NoteEdited { .. }
            | Self::Adopted { .. }
            | Self::Restored { .. }
            | Self::Proposed { .. }
            | Self::Withdrawn { .. }
            | Self::Left { .. }
            | Self::Stopped { .. }
            | Self::Concluded { .. }
            | Self::Cancelled { .. }
            | Self::Lifted { .. }
            | Self::Released { .. }
            | Self::PersonTaken { .. }
            | Self::PersonHandedBack { .. }
            | Self::PersonAnswered { .. }
            | Self::Moved { .. }
            | Self::ProposalDecided { .. }
            | Self::Said { .. }
            | Self::QuestionAnswered { .. }
            | Self::Amended { .. }
            | Self::PolicyChanged { .. }
            | Self::PoolSet { .. }
            | Self::Renamed { .. }
            | Self::ProjectRemoved { .. }
            | Self::VisibilitySet { .. }
            | Self::SettingsChanged { .. }
            | Self::MovedToNewest { .. }
            | Self::EscalationDecided { .. }
            | Self::AccountSignInStarted { .. }
            | Self::AccountSignedOut { .. }
            | Self::AccountKeyKept { .. }
            | Self::AccountLoginAsked { .. }
            | Self::AccountProgramDeclared { .. }
            | Self::AccountReading { .. }
            | Self::WorkerPauseAsked { .. }
            | Self::AccountChanged
            | Self::AccountMovingOut { .. }
            | Self::AccountFinishing { .. }
            | Self::AccountMovedIn { .. }
            | Self::AccountRemoved { .. } => OutcomeFamily::Settled,
        }
    }

    /// Refusal facts, including the failed operation of a compound.
    #[must_use]
    pub const fn refusal(&self) -> Option<&Refusal> {
        match self {
            Self::Refused(why) | Self::CompoundRefused { why, .. } => Some(why),
            Self::Again(_)
            | Self::SignedOut
            | Self::Compound { .. }
            | Self::ServiceMade { .. }
            | Self::ServiceReissued { .. }
            | Self::RolesSet { .. }
            | Self::ProjectMade { .. }
            | Self::Created { .. }
            | Self::MarkedSeen { .. }
            | Self::Applied { .. }
            | Self::NoteEdited { .. }
            | Self::Adopted { .. }
            | Self::Restored { .. }
            | Self::Proposed { .. }
            | Self::Withdrawn { .. }
            | Self::Left { .. }
            | Self::Stopped { .. }
            | Self::Concluded { .. }
            | Self::Cancelled { .. }
            | Self::Lifted { .. }
            | Self::Released { .. }
            | Self::PersonTaken { .. }
            | Self::PersonHandedBack { .. }
            | Self::PersonAnswered { .. }
            | Self::Moved { .. }
            | Self::ProposalDecided { .. }
            | Self::Said { .. }
            | Self::QuestionAnswered { .. }
            | Self::Amended { .. }
            | Self::PolicyChanged { .. }
            | Self::PoolSet { .. }
            | Self::Renamed { .. }
            | Self::ProjectRemoved { .. }
            | Self::VisibilitySet { .. }
            | Self::SettingsChanged { .. }
            | Self::MovedToNewest { .. }
            | Self::EscalationDecided { .. }
            | Self::AccountSignInStarted { .. }
            | Self::AccountSignedOut { .. }
            | Self::AccountKeyKept { .. }
            | Self::AccountLoginAsked { .. }
            | Self::AccountProgramDeclared { .. }
            | Self::AccountReading { .. }
            | Self::WorkerPauseAsked { .. }
            | Self::AccountChanged
            | Self::AccountMovingOut { .. }
            | Self::AccountFinishing { .. }
            | Self::AccountMovedIn { .. }
            | Self::AccountRemoved { .. } => None,
        }
    }

    /// Classify an act's answer without interpreting it in an application's root.
    #[must_use]
    pub const fn act_status(&self) -> ActStatus {
        match self.family() {
            OutcomeFamily::Compound => ActStatus::Closed,
            OutcomeFamily::Again(_) => ActStatus::Again,
            OutcomeFamily::Refused => match self.refusal() {
                Some(Refusal::NotOffered) => ActStatus::NotOffered,
                Some(Refusal::Role { held: None }) => ActStatus::NoOwner,
                Some(Refusal::Ended { .. }) => ActStatus::Ended,
                Some(_) | None => ActStatus::Refused,
            },
            OutcomeFamily::SignedOut
            | OutcomeFamily::CompoundRefused
            | OutcomeFamily::ServiceMade(_)
            | OutcomeFamily::ServiceReissued
            | OutcomeFamily::RolesSet(_)
            | OutcomeFamily::ProjectMade(_)
            | OutcomeFamily::Settled => ActStatus::Refused,
        }
    }
}

/// Who may accept a proposal of a refused request (client/domain/contract/requests.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Accepter {
    /// No proposal of this request could be accepted.
    Nobody,
    /// The lowest-numbered policy role deciding this kind.
    Role(u32),
}

/// The task or person that settled a question.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum QuestionAnswerer {
    Task(u64),
    Person(u64),
}

/// A final reason a keyed request was refused.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    QuestionAnswered {
        question: u64,
        by: QuestionAnswerer,
    },
    /// The named worker has no admitted channel (domain/engine/workers.md, 5).
    Unlinked {
        worker: u32,
    },
    /// No delegate joins a concluded gathering (domain/tasks/gatherings.md, 3).
    Concluded {
        at: Wall,
    },
    NoTerminal,
    LoginUnderWay,
    NotLending,
    /// A revision below the history cutoff (jig/docs/design/domain/engine/settings.md, 10).
    Forgotten {
        oldest: u32,
    },
    Judged {
        at: u32,
        setting: Setting,
        why: Box<[u8]>,
    },
    /// A finish refused while the sign-in may still wait (accounts.md, section 5).
    SignInRedirect {
        why: SignInRedirectWhy,
    },
    Stale {
        newest: u32,
    },
    Full {
        revision: u32,
        tasks: Box<[u64]>,
        more: u32,
    },
    /// Another project holds this name under simple case folding, or a program source's name a declaration gives is taken (domain/accounts-sources.md, 1).
    NameTaken,
    /// A holder's account move could not proceed (domain/accounts.md, section 6).
    AccountMove {
        why: AccountMoveWhy,
    },
    /// The bounded wait circle in traversal order (domain/tasks/holds.md, 7).
    Circle {
        tasks: Box<[u64]>,
        cut: bool,
    },
    /// An added prerequisite already failed or was cancelled (domain/tasks/changes.md).
    Dependency {
        dependency: u64,
        how: DependencyStatus,
    },
    /// An answer is kept by its pending prerequisites (domain/tasks/lifecycle-endings.md, 1).
    Kept {
        pending: Box<[u64]>,
    },
    /// An explicit account preference was refused before the task changes.
    Account {
        account: u32,
        why: AccountWhy,
    },
    /// The named batch member is not well formed.
    NotWellFormed {
        at: u32,
        why: Malformed,
    },
    /// The held role does not permit this request here.
    Role {
        held: Option<Role>,
    },
    /// The request lacks the stated authority.
    Authority {
        lacking: Lack,
        /// Who could accept a proposal of this request.
        /// None preserves refusals that make no claim about a proposal.
        proposable: Option<Accepter>,
    },
    /// The stated admission bound was reached.
    Limit {
        which: LimitKind,
    },
    /// The decided subject changed since it was observed.
    Moved {
        now: Moved,
    },
    /// The subject ended with the stated outcome.
    Ended {
        how: Ending,
    },
    /// Another party holds the role-addressed person task.
    Taken {
        by: u64,
    },
    /// The proposal or escalation waits for someone else.
    NotWaiting,
    /// The final holder cannot pass further.
    NoFurther,
    /// No visible subject has that name.
    Unknown,
    /// The deployment does not offer this operation.
    NotOffered,
    /// The program source name remains reserved after its account is forgotten.
    ProgramNameReserved,
    /// A plan's uses name another person, directly or through a project
    /// (domain/accounts-uses.md, 1.2; domain/providers.md, 2.6 and 3.4).
    Provider,
    /// This party's key names another ask.
    KeyConflict,
    /// The root's requester has nobody to hold its escalations (domain/people/acts.md, 1).
    NoEscalationHolder,
    /// The task's LLM choice names this configured model, refused for `why` (domain/tasks/README.md, 3).
    Llm {
        model: u8,
        why: LlmWhy,
    },
    /// An owner must end the project's live roots before deleting it.
    LiveTasks {
        tasks: Box<[u64]>,
        more: u32,
    },
}

/// Why a batch member is not well formed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Malformed {
    Settings {
        setting: Setting,
        fault: SettingFault,
        detail: Option<SettingDetail>,
    },
    SettingsUnchanged,
    /// Running work is chosen only on accepting a settings proposal.
    Running,
    Name(NameRule),
    /// A proposed batch must name every budget.
    ProposedAsAllowed,
    Cycle,
    AsAllowedTwice,
    Conversation,
    EndedBy,
}

/// Authority missing from a party's request.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Lack {
    /// A policy edit passes this ceiling; scalar values use their dimension's unit.
    Ceiling { holder: CeilingHolder, dimension: CeilingDimension, value: Option<u64>, asked: Option<u64> },
    /// A placed hold waits for its owner (domain/tasks/lifecycle-holds.md, 2).
    HoldOwner { hold: u64, owner: HoldOwner },
    /// No policy role may accept this kind of party proposal.
    Approver,
    /// Requested spend exceeds what this source can supply.
    Budget { needs: u64, left: u64 },
    /// No grant covers this operation kind.
    Grant { kind: u16 },
    /// No valid funding relation supports the requested reservation.
    Funding,
    /// The requested executor is not allowed.
    TaskKind { executor: Option<TaskExecutor> },
    /// Descendant depth exceeds this maximum.
    Depth { most: u32 },
    /// The deadline is past or later than permitted.
    Deadline,
    /// Lifetime task capacity cannot supply the request.
    Tasks { left: u32 },
    /// The tool family is absent.
    Tool,
    /// The note scope is absent.
    Scope,
    /// A requested model is absent (domain/authority.md, sections 3 and 5).
    Models,
    /// No verified reference permits this action.
    Reference,
    /// Current resource access forbids this action.
    ResourceAccess,
    /// A required connector resource is unavailable.
    ResourceUnavailable,
    /// The authority value has an invalid shape.
    Shape,
    /// Whether an unrecoverable effect choice was required or unexpected.
    Again { required: bool },
    /// A required model account is unusable.
    Account,
    /// A written resource is held elsewhere.
    Writer,
    /// A required judge has no usable fact.
    Required { connector: u16, kind: u16 },
    /// A required judge refused the effect.
    Failed { connector: u16, kind: u16 },
    /// The effect cannot guard a required fact.
    Unguarded { connector: u16, kind: u16 },
}

/// The bound a final refusal identifies.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LimitKind {
    /// The deployment has no room for another pending settings set.
    SettingsProposals,
    /// The deployment has no room for another project.
    Projects,
    /// Empty or too many operations in one compound request.
    Operations,
    /// The words or other byte payload exceed their bound.
    Words,
    /// The identity payload exceeds its bound.
    Identity,
    /// The roster exceeds its bound or repeats a party.
    Roles,
    /// A checked identity or accounting counter is exhausted.
    Arithmetic,
    /// An identity is repeated in the admitted input.
    Duplicate,
    /// The required input is empty.
    Empty,
    /// The batch exceeds its bound.
    Batch,
    /// The project has no live task capacity.
    ProjectLiveTasks,
    /// The live task set is full.
    LiveTasks,
    /// The task tree exceeds its bound.
    Tree,
    /// The task tree exceeds its depth bound.
    Depth,
    /// The delegate list exceeds its bound.
    Delegates,
    /// The subscription list exceeds its bound.
    Subscription,
    /// The dependency list exceeds its bound.
    Dependencies,
    /// The dependency tree would contain a cycle.
    Cycle,
    /// The task spec exceeds its bounds.
    Spec,
    /// The result contract exceeds its bounds.
    Contract,
    /// The authority value exceeds its structural bounds.
    Authority,
    /// The shown input list exceeds its bound.
    Inputs,
    /// The attempt counter is exhausted.
    Attempts,
    /// The live delegate allowance is exhausted.
    LiveDelegates,
    /// The stored record is invalid or restoration is finished.
    Restore,
    /// The read payload exceeds its bounds.
    Read,
    /// The turn count exceeds its bound.
    Turn,
    /// The resource hold kind is invalid.
    HoldKind,
    /// The resource hold is already taken.
    HoldTaken,
    /// The hold list exceeds its bound.
    Holds,
    /// The inbox has no admission room.
    InboxFull,
    /// The notes set is full.
    Notes,
    /// The revision counter is exhausted.
    Revision,
    /// The question payload exceeds its bounds.
    Question,
    /// The priority batch exceeds its bounds.
    Priority,
    /// The cancel grace exceeds its span.
    Grace,
    /// The project has no room for another goal.
    Goals,
    /// The policy payload exceeds its bounds.
    Policy,
    /// The resource payload exceeds its bounds.
    Resource,
    /// The service payload exceeds its bounds.
    Service,
    /// The deployment has no room for another LLM account.
    Accounts,
    /// The holder has no room for another LLM account.
    HolderAccounts,
    /// The account, or the sources signing out, have no room for another source.
    Sources,
    /// As many sign-ins are under way as the deployment allows.
    SignIns,
    PlacedHolds,
    ProjectPlacedHolds,
    PartyPlacedHolds,
    Exemptions,
}

/// A party in an ending fact, independent of the tasks child's vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Party {
    /// An authenticated person or service.
    Person(u64),
    /// A task cancelled its descendant work.
    Task(u64),
    /// A deployment rule made the decision.
    Deployment { project: u32 },
}

/// The final outcome of a subject that cannot be decided again.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ending {
    /// The task completed successfully.
    Done,
    /// The task failed.
    Failed,
    /// The named party cancelled it at this wall time.
    Cancelled { by: Party, at: Wall },
    /// The named party decided the immutable subject at this wall time.
    Decided { by: u64, at: Wall },
    /// The corrected or deleted note is no longer present.
    Deleted,
}

/// An unsaved answer: nothing was made and the key may be retried.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Again {
    /// Temporary admission pressure.
    Busy,
    /// The shared retained-answer table has no unreserved place.
    Full { held: u32, room_at: Wall },
    /// Restoration or a required neighbour is not ready.
    NotReady,
}

/// The terminal phase of an added prerequisite (client/domain/contract/answers.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DependencyStatus {
    Done,
    Failed,
    Cancelled,
}

/// Who placed a hold, independent of the tasks child (domain/tasks/lifecycle-holds.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldOwner {
    Party { person: u64 },
    Task { task: u64 },
}

/// The holder of a ceiling a policy edit passes (jig/docs/design/domain/authority.md, 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CeilingHolder {
    Rules,
    Policy,
    Role(u32),
}

/// A ceiling's dimension; list dimensions carry no contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CeilingDimension {
    PeriodSpend,
    Spend,
    Tools,
    Tasks,
    Depth,
    Executors,
    Grants,
    Deadline,
    Notes,
}

/// A core or application setting (jig/docs/design/domain/engine/settings.md, 1, jig/docs/design/domain/engine/settings-application.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Setting {
    Charter(u32),
    Template(u32),
    Policy(u32),
    Rules,
    /// Scope 0 is the deployment; otherwise it is a project number.
    Application {
        kind: u32,
        scope: u32,
        number: u32,
    },
    /// A project's own policy, an item of its scope (jig/docs/design/domain/engine/settings.md, 2).
    ProjectPolicy {
        project: u32,
    },
    Model(u32),
    Worker(u32),
    Provider(u32),
    Limits,
}

/// A check fault, including a connector's judgment (jig/docs/design/domain/engine/settings.md, 1, jig/docs/design/domain/engine/settings-application.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingFault {
    Shape,
    Count,
    Reference,
    Rules,
    Budget,
    Grace,
    Owner,
    Judged,
    Linked,
    Accounts,
    /// A charter, template, policy or model numbered out of its range.
    Range,
}

/// The charter member rejected by the settings check (domain/engine/settings.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingMember {
    /// The charter's document count.
    Documents,
    /// A document resource name's segment count.
    NameSegments,
    /// One segment's byte length in a document resource name.
    NameSegmentBytes,
    /// The document path's byte length.
    Path,
    /// The optional heading's byte length.
    Heading,
    /// The document connector's registration.
    Connector,
    /// The charter's stored policy and parts byte length.
    StoredPolicy,
}

/// The failed setting rule; scalar bounds are carried beside it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingRule {
    /// The supplied count or length exceeds the configured maximum.
    Maximum,
    /// The supplied length falls below the required minimum.
    Minimum,
    /// The supplied connector number is not registered.
    Registered,
}

/// Bounded refusal facts, without copying any of the rejected charter's bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SettingDetail {
    /// The member relative to the charter.
    pub member: SettingMember,
    /// Document index; zero for documents or stored policy.
    pub document: u32,
    /// Name segment index; zero for members other than `NameSegmentBytes`.
    pub segment: u32,
    /// The failed check, interpreted with the member and bound.
    pub rule: SettingRule,
    /// The supplied count, byte length or connector number.
    pub supplied: u32,
    /// The minimum or maximum; zero for `Registered`.
    pub bound: u32,
}

/// Who carries out a member.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskExecutor {
    Agent { charter: u32 },
    Procedure { connector: u16, code: u32 },
    Person(PersonAddress),
}

/// Who may take or answer the task in its project.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PersonAddress {
    /// One named party, without a role claim.
    Person(u64),
    /// Any holder of this project role.
    Role(u32),
}

/// The resolved amount of a member whose budget was `AsAllowed`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Given {
    pub task: u64,
    pub amount: u64,
}

/// A party's requested read position (domain/people/README.md, section 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Seen {
    /// Results committed through this position.
    Results { through: u64 },
    /// Replies of one conversation through this position.
    Conversation { task: u64, through: u64 },
}

/// The current revision of a changed decision subject.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Moved {
    /// A separately numbered placed hold changed.
    PlacedHold { revision: u64 },
    /// A hold began anew or its reason changed.
    Hold { revision: u64, reason: HoldReason },
    /// A task's words, parameters, dependencies or policy changed.
    Spec { revision: u64 },
    /// A person task's question, options or shown results changed.
    Person { revision: u64 },
    /// A note changed after it was recalled.
    Note { revision: u32 },
}

/// The reason a task is held, in the party's vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldReason {
    /// A configured bound refused the run's model call (domain/hosts/hub.md, 3).
    Exceeded(Exceeded),
    /// Its retry allowance was exhausted.
    Failures { class: FailureClass },
    /// Its executor stopped progressing.
    Stalled,
    /// An input failed or was cancelled.
    Dependency { task: u64 },
    /// An authorized stop has no named party.
    Stopped,
    /// A separately numbered placed hold keeps the task held.
    Placed { hold: u64 },
    /// A resource drifted away from what the task needs.
    Drift,
    /// Its effects need a decision.
    Effects,
    /// An effect failed permanently.
    EffectFailed,
    /// An effect's result is uncertain.
    Uncertain { entry: u64 },
    /// It lacks budget for its next activation.
    Budget,
    /// Its deadline has passed.
    Deadline,
    /// It waited too long for its resource holds.
    HoldsWaited,
    /// Joining a queue would close a circle of waits (domain/tasks/holds.md, 7).
    Circle,
    /// One of its waits reached its bound.
    Waited,
    /// A required resource is unavailable.
    ResourceUnavailable,
    /// Adoption no longer permits its workspace writes.
    ResourceAccess,
    /// Its allocation in a pool disappeared.
    PoolLost { pool: u64 },
    /// Its procedure asked for a decision.
    Procedure,
}

/// The class of failures that exhausted a task's retry allowance.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FailureClass {
    /// A temporary neighbour or read failure.
    Transient,
    /// A permanent failure awaiting a change outside the engine.
    Permanent,
    /// The executor reported failure.
    Run,
    /// The agent crashed, stalled or broke its channel.
    Agent,
    /// The host disappeared.
    Lost,
    /// The result broke its contract.
    Invalid,
    /// A workspace or required brief section could not be prepared for now.
    Infrastructure,
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

/// Why the root refused a named model, the tasks child's `LlmFault`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LlmWhy {
    /// The model is not configured.
    Unconfigured,
    /// The effort is not one the model declares, or none is named for a model that declares some.
    Effort,
    /// The model is not one of the task's charter's.
    Charter,
    /// The task's authority does not allow the model.
    Authority,
    /// The task's executor runs no model.
    Procedure,
    /// The sub-agents name the model twice.
    Repeated,
    /// The speed is not one the model declares, nor `standard`.
    Speed,
}

/// Why an explicit account cannot fund the prospective task (domain/accounts-uses.md, 1.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountWhy {
    Unknown,
    Provider,
    Uses,
    Limited,
}

/// Why a holder's account move cannot proceed (domain/accounts.md, section 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountMoveWhy {
    NotSignedIn,
    NotOurs,
    Held,
    Cancelled,
    Moving,
}

/// Why a finish was refused (client/domain/contract/answers.md, section 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignInRedirectWhy {
    /// No sign-in of this party's waits for its redirect under that number.
    NotWaiting,
    /// The address does not match the sign-in's redirect or its required fields.
    Address,
    /// The state belongs to another sign-in; this sign-in still waits.
    State,
    /// Too many states from other sign-ins; it still waits for its browser.
    TooMany,
}

/// Why a project's name is out of shape (domain/model.md, 6.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NameRule {
    Empty,
    Long,
    Control,
    Space,
    Digits,
}
