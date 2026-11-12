//! Root/people messages and durable child records (domain/people/README.md, sections 3–4 and domain/people/acts.md). The root supplies authenticated identities, roles and keyed
//! requests; people returns bounded routes and persistence intentions. Every
//! routed flight ends with `Decided`; the root commits before releasing replies.
//! These values contain no credential secrets, task internals or protocol bytes.

use crate::{Refusal, ReplyRef, Seen};
use alloc::boxed::Box;
use skein_lib::{Duration, List, ReplyTo, Token, Wall};

/// Maximum address bytes retained in an account sign-in answer.
pub const ADDRESS: u32 = 2_048;

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
    Proposal { number: u64 },
    /// A held task waiting for a decision at this revision.
    Escalation { revision: u64 },
    /// An active person-executed task.
    PersonTask,
    /// A reply in a task requested by the person.
    Reply { message: u64 },
    /// A committed reply belonging to one conversation's independent read position.
    ChatReply { message: u64 },
}

/// One volatile reference derived from a live task; content remains in tasks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub task: u64,
    pub project: u32,
    pub whom: Whom,
    pub kind: EntryKind,
    /// Root-injected time of the change that made this entry visible.
    pub at: Wall,
}

/// A provider and its stable subject identify one party. Display fields are not keys.
/// The application numbers providers; no credential is retained (domain/people/README.md, section 3).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct IdentityKey {
    /// The application's configured sign-in provider number.
    pub provider: u16,
    /// The provider's bounded stable identifier, independent of its display login.
    pub subject: Box<[u8]>,
}

/// Authenticated identity and display data, admitted under one identity-byte limit.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Identity {
    /// Authenticated stable provider/subject pair.
    pub key: IdentityKey,
    /// Display login bytes; combined with the subject and name, at most `Limits::identity_bytes`.
    pub login: Box<[u8]>,
    /// Display-name bytes; combined with the subject and login, at most `Limits::identity_bytes`.
    pub name: Box<[u8]>,
}

/// Whether the authenticated party is a person or a deployment service.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    /// A person authenticated by an application provider.
    Person,
    /// A service made by a project owner and authenticated by the deployment.
    Service,
}

/// Whose LLM account a party's account request is for (domain/accounts.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountHolder {
    /// The asking person's own; a service holds no account.
    Own,
    /// The deployment's, decided by its first owners (domain/people/README.md, section 3).
    Deployment,
}

/// Root-configured first-owner grant used only when this identity's party record is first made;
/// bootstrap matches are bounded. (domain/people/README.md, section 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InitialOwner {
    /// Project whose roles must already be initialized before first sign-in.
    pub project: u32,
    /// Configured provider/subject pair to grant `Owner` on its first person creation.
    pub identity: IdentityKey,
}

/// People's project membership label; the root translates it to authority policy and performs the
/// action checks. (domain/people/README.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    /// `Owner` membership; may start chats here, while broader owner actions remain root policy.
    Owner,
    /// Maintainer membership; may start chats here, with authority checked by the root.
    Maintainer,
    /// Member membership; may start chats here within the root's policy and funding check.
    Member,
    /// Observer membership; creation is refused and its keyed role outcome is saved.
    Observer,
    /// A role numbered by this project's policy, beyond the four defaults.
    Policy { role: u32 },
}

impl Role {
    /// Number used by authority and role-addressed task entries.
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::Owner => 0,
            Self::Maintainer => 1,
            Self::Member => 2,
            Self::Observer => 3,
            Self::Policy { role } => role,
        }
    }

    /// Resolve a configured policy role to a membership label.
    #[must_use]
    pub const fn from_number(number: u32) -> Self {
        match number {
            0 => Self::Owner,
            1 => Self::Maintainer,
            2 => Self::Member,
            3 => Self::Observer,
            role => Self::Policy { role },
        }
    }
}

/// One person's authoritative project role; a roles record permits at most one holding per person.
/// (domain/people/README.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Holding {
    /// Deployment person number, checked for restored references when restoration finishes.
    pub person: u64,
    pub role: Role,
}
/// One connector-known party mapped to a project role at adoption.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Seed {
    pub identity: IdentityKey,
    pub candidate: u64,
    pub role: Role,
}

/// Exact addressed work by kind, beyond the recent inbox cache (client/domain/pages.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Needs {
    pub questions: u32,
    pub proposals: u32,
    pub escalations: u32,
    pub person_tasks: u32,
}

/// Exact needs and unread updates capped at the configured count limit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Counts {
    pub needs: u32,
    pub unread: u32,
    pub more_unread: bool,
}

/// A connector's resource path, opaque to people (domain/connectors.md, section 12).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ResourceName {
    pub connector: u16,
    pub path: Box<[Box<[u8]>]>,
}

/// A resource and its connector-specific options, opaque to people. The whole
/// path and options payload fits `Limits::amendment_bytes` at admission.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Adoption {
    pub resource: ResourceName,
    pub role: ResourceRole,
    /// Bounded connector-specific fields, interpreted by the application's root.
    pub options: Box<[Box<[u8]>]>,
}

/// How an adopted resource participates in the project.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceRole {
    Owned,
    Fork,
    Context,
}

/// Note scope named by a party; the parent translates it to the notes child.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum NoteScope {
    /// Shared deployment guidance.
    Deployment,
    /// Guidance for one project.
    Project,
    /// Guidance for one goal in the request's project.
    Goal { goal: u64 },
    /// Guidance for one connector resource pattern in the request's project.
    Resources { connector: u16, pattern: crate::Pattern },
    /// A person's own preferences, in no project (domain/engine/briefs.md, 2.1).
    Party { party: u64 },
}

/// A person's correction or deletion of a recalled note revision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum NoteChange {
    /// Replace an entry's bounded description, body and task references.
    Correct { description: Box<[u8]>, body: Box<[u8]>, references: Box<[u64]>, recalled: u32 },
    /// Delete the entry at the revision the person saw.
    Delete { recalled: u32 },
}

/// Authenticated keyed requests admitted by the people child and routed by
/// the root after role checks (domain/people/acts.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::large_enum_variant,
        reason = "an amendment stays inline, as `Event` carries a keyed request whole, and is priced by size_of"
    )
)]
pub enum Ask {
    /// Propose the settings as a revision left them (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/people/acts.md, 1).
    Restore { revision: u32, reason: Box<[u8]> },
    /// Delete an owned project (jig/docs/design/domain/people/acts.md, 2).
    RemoveProject { project: u32, grace: Option<Duration> },
    /// Deployment owner changes settings, whose value the core holds during admission.
    ChangeSettings { against: u32, digest: [u8; 32], projects: Box<[u32]> },
    /// Deployment owner moves live agent tasks to the newest (domain/engine/settings.md, 4; domain/people/acts.md, 1).
    MoveToNewest { revision: u32 },
    /// Several task operations committed together under one key.
    Compound { project: u32, operations: List<crate::Operation> },
    /// Make these roots together (domain/people/acts.md, 1).
    Create { project: u32, batch: Box<[crate::Member]> },
    /// Mark results or one conversation's replies seen under this party's key.
    MarkSeen { seen: Seen },
    /// Application-owned request metadata; the root carries the value, and the protocol supplies its digest.
    Application { scope: crate::ApplicationScope, connector: u16, digest: [u8; 32] },
    /// Correct or delete a note whose scope the party's project role permits.
    EditNote { project: u32, name: u64, scope: Box<NoteScope>, change: Box<NoteChange> },
    /// Make a deployment service in a project with a role and display name.
    MakeService { project: u32, name: Box<[u8]>, role: Role },
    /// Adopt a connector resource as a durable keyed owner request.
    Adopt { project: u32, adoption: Adoption },
    /// Propose a batch to the role named by the project's policy.
    Propose { project: u32, action: crate::Action, reason: Box<[u8]> },
    /// Withdraw this party's own pending proposal.
    Withdraw { project: u32, proposal: u64 },
    /// Hold a live run for this person's decision, cancelling its current claim.
    Stop { project: u32, task: u64 },
    /// Cancel a task and its descendants with a bounded reason, at once or after a grace.
    Cancel { project: u32, task: u64, reason: Box<[u8]>, grace: Option<Duration> },
    /// Lift a held task's cause and rejudge it.
    Release { project: u32, task: u64, revision: u64, again: Option<bool> },
    /// End a placed hold, optionally releasing its tasks (domain/tasks/lifecycle.md, 10).
    Lift { hold: u64, revision: u64, with_tasks: bool },
    /// Claim one role-addressed person task for this authenticated person.
    TakePerson { project: u32, task: u64 },
    /// Give this person's current role task claim back to its role.
    HandBackPerson { project: u32, task: u64 },
    /// Supply the bounded result required by an addressed person task.
    AnswerPerson { project: u32, task: u64, revision: u64, result: PersonResult },
    /// Adopt a live task as this authenticated person's own goal, retaining its old requester's reference.
    Move { project: u32, task: u64, reason: Box<[u8]> },
    /// Authenticated keyed decision for a proposal waiting at this person or a policy role.
    DecideProposal {
        project: u32,
        proposer: Proposer,
        proposal: u64,
        decision: ProposalDecision,
        running: Option<crate::Running>,
    },
    /// Authenticated requester's whole words to an existing chat.
    Say { project: u32, task: u64, words: Box<[u8]> },
    /// Answer a numbered question waiting in a task requested by this person.
    AnswerQuestion { project: u32, task: u64, question: u64, words: Box<[u8]> },
    /// Amend a live task this person may steer, after root policy and funding checks.
    /// A priority-only edit of a tracked task names spec revision zero.
    Amend { project: u32, task: u64, revision: u64, amendment: crate::Amendment, release: Option<u64> },
    /// Authenticated Owner's complete project roster replacement; root checks Policy permission and
    /// held-recipient preflight before applying it.
    SetRoles {
        /// Existing configured project; unknown projects refuse without mutation.
        project: u32,
        /// At most `Limits::holdings` unique positive existing people; validated by this child
        /// before replacement.
        holdings: Box<[Holding]>,
    },
    /// Owner changes one mutable part of the project policy for future decisions.
    ChangePolicy { project: u32, change: crate::PolicyChange },
    /// Owner replaces this project's name (domain/people/acts.md, 1).
    Rename { project: u32, name: Box<str> },
    /// Deployment owner makes a project from a configured policy (domain/people/acts.md, 2).
    MakeProject { name: Box<str>, policy: u32 },
    /// Owner changes one person's current-period pool, preserving already spent and reserved funds.
    SetPool { project: u32, person: u64, budget: u64 },
    /// Decide one held chat's exact semantic revision. Root verifies current waiting recipient and
    /// authority; admission authenticates the session and reserves keyed-answer room.
    DecideEscalation {
        /// Policy project, checked against the held task by root.
        project: u32,
        /// Positive held chat identity.
        task: u64,
        /// Positive task-local hold revision, unchanged by passing or rerouting.
        revision: u64,
        /// One bounded semantic choice; root authorizes before mutation.
        decision: EscalationDecision,
    },
    /// Sign an LLM provider's account in for this holder; admitted from the holder alone, never by
    /// a role (domain/authority.md, 8.4), and decided by the accounts child (domain/accounts.md, 3.3).
    AccountSignIn { holder: AccountHolder, provider: u32 },
    /// Sign this holder's account's source out; admitted like a sign-in.
    AccountSignOut { holder: AccountHolder, source: u32 },
    /// Change this holder's LLM accounts; carried keys and moved records retain only their digest.
    AccountChange { holder: AccountHolder, change: AccountChange },
    /// Set an owned project's visibility (jig/docs/design/domain/people/public-reading.md, 2).
    SetVisibility { project: u32, public: bool },
}

/// The scope and admission family of a keyed request (domain/people/acts.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AskFamily {
    Project(u32),
    Application(crate::ApplicationScope),
    Restore,
    Deployment,
    Account,
    Lift,
    Seen,
}

impl AskFamily {
    /// The project named by this request, excluding deployment and party-only scopes.
    pub(crate) const fn named_project(self) -> Option<u32> {
        match self {
            Self::Project(project) | Self::Application(crate::ApplicationScope::Project(project)) => Some(project),
            Self::Application(crate::ApplicationScope::Own | crate::ApplicationScope::Deployment)
            | Self::Restore
            | Self::Deployment
            | Self::Account
            | Self::Lift
            | Self::Seen => None,
        }
    }
}

impl Ask {
    /// Keep the request's project distinct from deployment and party-only admission.
    #[must_use]
    pub const fn family(&self) -> AskFamily {
        match self {
            Self::Compound { project, .. }
            | Self::EditNote { project, .. }
            | Self::MakeService { project, .. }
            | Self::Adopt { project, .. }
            | Self::SetRoles { project, .. }
            | Self::RemoveProject { project, .. }
            | Self::SetVisibility { project, .. }
            | Self::Rename { project, .. }
            | Self::ChangePolicy { project, .. }
            | Self::SetPool { project, .. }
            | Self::DecideEscalation { project, .. }
            | Self::DecideProposal { project, .. }
            | Self::Say { project, .. }
            | Self::AnswerQuestion { project, .. }
            | Self::Amend { project, .. }
            | Self::Move { project, .. }
            | Self::TakePerson { project, .. }
            | Self::HandBackPerson { project, .. }
            | Self::AnswerPerson { project, .. }
            | Self::Stop { project, .. }
            | Self::Cancel { project, .. }
            | Self::Release { project, .. }
            | Self::Create { project, .. }
            | Self::Propose { project, .. }
            | Self::Withdraw { project, .. } => AskFamily::Project(*project),
            Self::Application { scope, .. } => AskFamily::Application(*scope),
            Self::Restore { .. } => AskFamily::Restore,
            Self::MakeProject { .. } | Self::ChangeSettings { .. } | Self::MoveToNewest { .. } => AskFamily::Deployment,
            Self::AccountSignIn { .. } | Self::AccountSignOut { .. } | Self::AccountChange { .. } => AskFamily::Account,
            Self::Lift { .. } => AskFamily::Lift,
            Self::MarkSeen { .. } => AskFamily::Seen,
        }
    }

    /// A roster replacement borrows the members checked and linked by people.
    #[must_use]
    pub fn roster(&self) -> Option<(u32, &[Holding])> {
        match self {
            Self::SetRoles { project, holdings } => Some((*project, holdings)),
            Self::Restore { .. }
            | Self::Compound { .. }
            | Self::MarkSeen { .. }
            | Self::Application { .. }
            | Self::MakeService { .. }
            | Self::EditNote { .. }
            | Self::Adopt { .. }
            | Self::ChangePolicy { .. }
            | Self::RemoveProject { .. }
            | Self::SetVisibility { .. }
            | Self::Rename { .. }
            | Self::MakeProject { .. }
            | Self::ChangeSettings { .. }
            | Self::MoveToNewest { .. }
            | Self::SetPool { .. }
            | Self::DecideEscalation { .. }
            | Self::DecideProposal { .. }
            | Self::Say { .. }
            | Self::AnswerQuestion { .. }
            | Self::Amend { .. }
            | Self::Move { .. }
            | Self::TakePerson { .. }
            | Self::HandBackPerson { .. }
            | Self::AnswerPerson { .. }
            | Self::Stop { .. }
            | Self::Cancel { .. }
            | Self::Lift { .. }
            | Self::Release { .. }
            | Self::Create { .. }
            | Self::Propose { .. }
            | Self::Withdraw { .. }
            | Self::AccountSignIn { .. }
            | Self::AccountSignOut { .. }
            | Self::AccountChange { .. } => None,
        }
    }
}

/// A person's answer to a report, choice or failure contract; root translates it for tasks.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PersonResult {
    /// Approve the exact opaque state shown by this person task contract.
    Approved { state: Box<[u8]>, words: Option<Box<[u8]>> },
    /// Freeform words for a report contract.
    Report { words: Box<[u8]> },
    /// One named choice with optional explanation.
    Verdict { code: u32, words: Box<[u8]> },
    /// The person could not complete the task.
    Failure { reason: Box<[u8]> },
}

/// Immutable proposal maker identity (client/domain/contract/requests.md, section 2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Proposer {
    Task(u64),
    Party(u64),
}

impl Proposer {
    pub(crate) fn number(self) -> u64 {
        match self {
            Self::Task(number) | Self::Party(number) => number,
        }
    }
}

/// Keyed person's choice for a pending proposal.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProposalDecision {
    Accept,
    Reject { reason: Box<[u8]> },
    Pass,
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

/// Authenticated person's held-chat choice. Rejection words are bounded by
/// people words; root also checks child/journal bounds (domain/people/acts.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EscalationDecision {
    /// Lift the held chat's cause and rejudge it under current conditions.
    Release,
    /// Decide once while preserving the hold.
    Reject {
        /// Bounded reason saved with the ask and held task.
        reason: Box<[u8]>,
    },
    /// Move from requester to final policy role; no further pass from that role.
    Pass,
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
    Again(crate::Again),
    SignedOut,
    Compound,
    Refused,
    CompoundRefused,
    ServiceMade(u64),
    RolesSet(u32),
    ProjectMade(u32),
    Settled,
}

/// Root's answer for one routed flight; people replies to each bounded waiter and saves
/// final outcomes with the parent's task decision. (domain/people/acts.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Outcome {
    AccountReading {
        count: u32,
    },
    /// A restore's proposal and omitted settings (jig/docs/design/domain/engine/settings.md, 10).
    Restored {
        proposal: u64,
        to: u32,
        left_out: Box<[crate::Setting]>,
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
        given: Option<crate::Given>,
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
    Again(crate::Again),
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
    Refused(/** Final refusal facts, retained under the request key. */ Refusal),
    /// The project is public or private from this decision (jig/docs/design/domain/people/public-reading.md, 2).
    VisibilitySet {
        project: u32,
        public: bool,
    },
}

/// Child-to-client answer routed by the parent; state-changing answers are withheld externally
/// until their decision is durable. (domain/people/README.md, section 3, and domain/people/acts.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Reply {
    /// The sign-in's party, or none when unknown, ended or past either clock.
    Signer(Option<Signer>),
    /// Sign-in succeeded or an existing active same-person sign-in was replayed.
    SignedIn {
        /** Stable deployment party number for the authenticated identity. */
        person: u64,
        /** Stable number of this saved sign-in. */
        sign_in: u64,
        /** Saved wall-time expiry for this sign-in; its monotonic deadline is retained internally. */
        expires: Wall,
    },
    /// Sign-out completed; an already absent number is also successful once ready.
    SignedOut,
    /// One terminal response for a keyed call or its duplicate waiter.
    Outcome(/** Terminal keyed result, or retryable root admission pressure for the current call. */ Outcome),
    /// Unsaved retry answer; no completed key is retained.
    Again(crate::Again),
    /// Unsaved sign-in answer while every slot is live; no identity or sign-in is written.
    Full {
        /** Configured deployment capacity for sign-ins. */
        sign_ins: u32,
        /** Earliest saved expiry, strictly after the answer's wall clock. */
        earliest: Wall,
    },
    /// Entrance refused before making a new answered-key record.
    Refused(/** Direct entrance refusal with no newly saved answered-key record. */ Refusal),
}

/// Person-scoped idempotency key shared across that person's sign-ins; a different ask under the
/// same key is a conflict. (domain/people/acts.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RequestKey {
    pub person: u64,
    /// Opaque web-selected 16-byte request key; the domain does not parse it.
    pub key: [u8; 16],
}

/// A connector's key space for the deployment's acts (domain/people/acts.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ActKey {
    pub connector: u16,
    pub key: [u8; 16],
}

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

/// Which acts the configuration grants a connector (domain/people/acts.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ActKind {
    Close,
    AddKey,
    RemoveAccount,
    Adopt,
    MakeProject,
    /// Delete a project (jig/docs/design/domain/people/acts.md, 2).
    RemoveProject,
}

impl ActKind {
    /// Every deployment act kind; also bounds a configured grant's owned entries.
    pub const ALL: &[Self] =
        &[Self::Close, Self::AddKey, Self::RemoveAccount, Self::Adopt, Self::MakeProject, Self::RemoveProject];
}

/// A connector's configured deployment authority (domain/people/acts.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ActGrant {
    pub connector: u16,
    pub may: Box<[ActKind]>,
}

/// Logical store key for a people record; the protocol handles bytes and secrets outside this
/// domain. (domain/people/README.md, sections 3–4 and domain/people/acts.md).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Key {
    /// Logical person-record key.
    Person(u64),
    /// One person's durable position in committed result order.
    ReadResults { party: u64 },
    /// One party's durable position over a conversation's replies.
    ReadConversation { party: u64, task: u64 },
    /// Logical secret-free sign-in-record key.
    SignIn(/** Root-issued deployment sign-in number; cookie secrets and token digests stay below the domain. */ u64),
    /// Logical per-project roles-record key.
    Roles(/** Project whose authoritative holdings are stored. */ u32),
    /// Root-owned current project policy, paged with people records.
    Policy(u32),
    /// Logical completed-key record.
    Answer(RequestKey),
    /// Ended positions are point reads, never restored at start.
    EndedRead { task: u64, party: u64 },
    /// One durable deployment act.
    Acted(ActKey),
    /// Live deletion, resumed at start (jig/docs/design/domain/people/acts.md, 2).
    Deletion(u32),
    /// Present while the project is public (jig/docs/design/domain/people/public-reading.md, 2).
    Public(u32),
    /// Number-only deletion mark, restored at start (domain/people/acts.md, 2).
    Deleted(u32),
}

impl Key {
    /// Ended positions are read on demand (protocol/store/loads.md, 2).
    #[must_use]
    pub const fn live(&self) -> bool {
        match self {
            Self::EndedRead { .. } => false,
            Self::Person(_)
            | Self::ReadResults { .. }
            | Self::ReadConversation { .. }
            | Self::SignIn(_)
            | Self::Roles(_)
            | Self::Policy(_)
            | Self::Answer(_)
            | Self::Acted(_)
            | Self::Deletion(_)
            | Self::Public(_)
            | Self::Deleted(_) => true,
        }
    }
}

/// Owned durable record submitted to the parent or restored from it; capacities and payload
/// bytes are checked when admitted. (domain/people/README.md, sections 3–4 and domain/people/acts.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Stored {
    /// Kept deletion (jig/docs/design/domain/people/acts.md, 2).
    Deletion { project: u32, step: DeletionStep, grace: Option<Duration> },
    /// Connector-scoped act and its permanent answer (domain/people/acts.md, 3).
    Acted { key: ActKey, act: Act, outcome: Outcome, at: Wall },
    /// Persistent party identity; provider/subject and party numbers must be unique on restore.
    Person {
        /** Stable person number, unique among restored people. */
        number: u64,
        /** Unique provider/subject identity and bounded display bytes. */
        identity: Identity,
    },
    /// Monotonic read position for the named person; no result is copied here.
    ReadResults { party: u64, through: u64 },
    /// Monotonic replies position, erased once an ended conversation and its replies are gone.
    ReadConversation { party: u64, task: u64, through: u64 },
    /// An ended conversation's position, joined on demand.
    EndedRead { task: u64, party: u64, through: u64 },
    /// Persistent secret-free sign-in; expired restored entries are erased when restore completes.
    SignIn {
        /** Root-issued sign-in number; at most `Limits::sign_ins` sign-in records are retained. */
        number: u64,
        /** Existing person reference, validated after all restore rows arrive. */
        person: u64,
        /** Persisted wall-time expiry; restoration projects it onto the supplied startup monotonic clock. */
        expires: Wall,
    },
    /// Persistent complete project holdings with bounded unique people.
    Roles {
        /** Project whose live role set this record replaces or restores. */
        project: u32,
        /** At most `Limits::holdings` distinct people; restored references are checked at completion. */
        holdings: Box<[Holding]>,
    },
    /// Root-owned full mutable project policy; the people page carries it to authority.
    Policy { project: u32, name: Box<str>, value: crate::PolicyValue },
    /// Persistent jig request and outcome, or application metadata and outcome token; replay never checks current
    /// roles anew.
    Answer {
        /** Person-scoped completed key; restoration validates its person reference. */
        key: RequestKey,
        /** Original request with words bounded by `Limits::words`, compared on every retry. */
        ask: Box<Ask>,
        /// Canonical original names referenced by the ask's tagged selectors (protocol/store/records.md, 1).
        models: Box<[Box<[u8]>]>,
        /** Retained permanent outcome returned without remaking the decision. */
        outcome: Outcome,
        /** Wall time the answer was made; completed keys are retained for `Limits::request_retention`. */
        at: Wall,
    },
    /// Present while the project is public (jig/docs/design/domain/people/public-reading.md, 2).
    Public { project: u32 },
    /// Number-only deletion mark (domain/people/acts.md, 2).
    Deleted { project: u32 },
}

impl Stored {
    /// Pure projection of this record's logical key; allocates nothing, emits no request and
    /// changes no state.
    #[must_use]
    pub const fn key(&self) -> Key {
        match self {
            Stored::Deletion { project, .. } => Key::Deletion(*project),
            Stored::Public { project } => Key::Public(*project),
            Stored::Deleted { project } => Key::Deleted(*project),
            Stored::Person { number, .. } => Key::Person(*number),
            Stored::ReadResults { party, .. } => Key::ReadResults { party: *party },
            Stored::ReadConversation { party, task, .. } => Key::ReadConversation { party: *party, task: *task },
            Stored::EndedRead { task, party, .. } => Key::EndedRead { task: *task, party: *party },
            Stored::SignIn { number, .. } => Key::SignIn(*number),
            Stored::Roles { project, .. } => Key::Roles(*project),
            Stored::Policy { project, .. } => Key::Policy(*project),
            Stored::Answer { key, .. } => Key::Answer(*key),
            Stored::Acted { key, .. } => Key::Acted(*key),
        }
    }
}

/// Where a kept deletion stands, resumed at start (jig/docs/design/domain/people/acts.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeletionStep {
    /// Its live tasks cancelled and ended, then each ended task erased whole.
    Tasks,
    /// Its own rows erased: policy, funders, notes, account uses, answers.
    Rows,
    /// Connectors release adoptions; the row goes when all answered.
    Release,
}

/// A kept deletion's step and the grace its conversations are cancelled with
/// (jig/docs/design/domain/people/acts.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct KeptDeletion {
    pub step: DeletionStep,
    pub grace: Option<Duration>,
}

/// Root-to-child inputs; authenticated identity, authoritative roles and restored records
/// cross this boundary without protocol secrets. (domain/people/README.md, sections 3–4 and domain/people/acts.md).
#[derive(PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    target_pointer_width = "64",
    expect(
        clippy::large_enum_variant,
        reason = "the people boundary carries the complete request including its later choice"
    )
)]
pub enum Event {
    /// Keep or erase a deletion row (jig/docs/design/domain/people/acts.md, 2).
    Deleting { project: u32, step: Option<DeletionStep>, grace: Option<Duration> },
    /// Remove project roles from memory and storage (jig/docs/design/domain/people/acts.md, 2).
    CloseProject { project: u32 },
    /// Erase retained project answers and acts (jig/docs/design/domain/people/acts.md, 2).
    ForgetProject { project: u32 },
    /// Admit a connector's deployment act without a sign-in.
    Act { reply_to: ReplyTo, connector: u16, key: [u8; 16], act: Box<Act> },
    /// Move a live conversation and start its new requester with nothing read.
    MoveChat { task: u64, project: u32, from: u64, to: u64 },
    /// Authentic committed chat header; live headers replace their previous contribution.
    ChatState {
        party: u64,
        project: u32,
        task: u64,
        last_reply: u64,
        live: bool,
        seen: Option<u64>,
        replies: Box<[ReplyRef]>,
    },
    /// Authorized keyed conversation mark, with the owner's complete reply count.
    ConversationSeen { request: Token, project: u32, last_reply: u64 },
    /// The owner forgot an ended chat and its reply references; retire its positions.
    Forget { task: u64, project: u32, party: Option<u64>, last_reply: u64 },
    /// Apply an authorized owner request to make a service with a fresh party number.
    MakeService { request: Token, person: u64 },
    /// Add previously unknown collaborators and their first project role in one decision.
    Seed { project: u32, collaborators: Box<[Seed]> },
    /// Replace the volatile inbox projection of one durable task after its row changes.
    Waiting { task: u64, entries: Box<[Entry]> },
    /// Root's synchronous application after authority/revision/capacity preflight. Reads the
    /// original admitted roster, preserves membership on refusal and returns exactly one
    /// `RolesApplied`.
    ApplyRoles {
        /// Root's stage-local reply right, consumed by `RolesApplied`.
        reply_to: ReplyTo,
        /// Live `SetRoles` flight from `Route`; no synthetic or completed token is authorized.
        request: Token,
    },
    /// Root supplies a fresh person candidate and a fresh sign-in number. Only a new identity uses
    /// the person candidate; existing identities keep their durable person number, and unused
    /// person candidates leave gaps. Admit an authenticated sign-in after restoration; bootstrap
    /// first-owner, person and sign-in writes share one root decision.
    SignedIn {
        /// Opaque destination for the sign-in answer; the parent holds success until durability.
        reply_to: ReplyTo,
        /// Root-issued never-reused fresh candidate, used only for a new provider/subject identity;
        /// unused candidates leave gaps.
        person: u64,
        /// Root-issued fresh deployment sign-in number; replaying an active same-person number
        /// returns its existing expiry. Cookie secrets stay below the domain.
        sign_in: u64,
        /// Protocol-authenticated identity with bounded display bytes and no credential.
        identity: Identity,
        /// The protocol-authenticated party kind; a service must already exist.
        kind: Kind,
        /// Authenticated sign-in to keep if it is live and belongs to this identity.
        held: Option<u64>,
    },
    /// Read the live sign-in's identity and kind without writing or spending admission room.
    Whose { reply_to: ReplyTo, sign_in: u64 },
    /// End one sign-in after restoration, emitting erase if present and one reply.
    SignOut {
        /// Destination for the terminal `SignedOut` reply.
        reply_to: ReplyTo,
        /// Root-issued sign-in number to end; absence is an idempotent success once ready.
        sign_in: u64,
    },
    /// Apply an already authorized root roles replacement; invalid/full updates emit `RolesRefused`
    /// and preserve old holdings. Failed restoration ignores this event.
    Roles {
        project: u32,
        /// Complete bounded holdings, with no duplicate person; root already authorized this
        /// change.
        holdings: Box<[Holding]>,
    },
    /// Admit one keyed call after restoration; saved retries reply, pending duplicates join bounded
    /// waiters, and a new eligible flight routes once.
    Ask {
        /// Destination admitted into at most `Limits::waiters` replies per flight.
        reply_to: ReplyTo,
        /// Root-issued active authenticated sign-in number; wall or monotonic expiry refuses
        /// entrance.
        sign_in: u64,
        /// Opaque web key, scoped to the sign-in's stable person.
        key: [u8; 16],
        /// Request with payload bounds checked before lookup, route or mutation.
        ask: Ask,
        /// Canonical original names referenced by the ask's tagged selectors (protocol/store/records.md, 1).
        models: Box<[Box<[u8]>]>,
    },
    /// Complete a live `Route` exactly once, retire its flight and answer each waiter; permanent
    /// answer writes join the root's action commit.
    Decided {
        /// Opaque token from `Route`; the root must return this live flight exactly once, or the
        /// child panics.
        request: Token,
        /// Terminal root decision; persist permanent answers with task changes, but do not retain
        /// `Busy`/`NotReady` pressure.
        outcome: Outcome,
    },
    /// Load one decoded row during the start. A representation bound failure emits one
    /// `RestoreRefused` and permanently fails this instance.
    Restore {
        /// One saved row, admitted once during loading; duplicate keys panic.
        record: Stored,
    },
}

/// Child-to-root outputs; persistence joins the root's decision, and the root withholds external
/// replies until required writes are durable. (domain/people/acts.md, section 1).
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    /// A deployment flight; the root owes one Decided.
    Act { request: Token, connector: u16, act: Box<Act> },
    /// Terminal answer held by the parent until durable.
    Acted { to: ReplyTo, outcome: Outcome },
    /// Terminal for one service creation after its person and role saves.
    ServiceMade { request: Token, outcome: Outcome },
    /// Terminal for `ApplyRoles`; success `Save` precedes this output, but the root completes the
    /// keyed flight only after task recheck.
    RolesApplied {
        /// Echoed root stage-local right, consumed exactly once.
        reply_to: ReplyTo,
        /// Echoed admitted flight; root returns `Decided` after the whole decision.
        request: Token,
        /// Success, or readiness/identity/roster/project/standing refusal before mutation; bounds
        /// use `Limits::holdings`.
        result: Result<(), Outcome>,
    },
    /// New eligible keyed flight for the root's authority/task decision; duplicate callers share
    /// this route.
    Route {
        /** Opaque live flight token; root owes exactly one matching `Decided`, including admission pressure. */
        request: Token,

        person: u64,
        /** Project selected by the request. */
        project: u32,
        /** Current membership at admission: Some is required for chat; an escalation decision may carry None so root can check named-person standing. Root still checks authority. */
        role: Option<Role>,
        /** Original bounded request, owned by the root for routing and atomic decision. */
        ask: Box<Ask>,
        /// Canonical original names referenced by the ask's tagged selectors (protocol/store/records.md, 1).
        models: Box<[Box<[u8]>]>,
    },
    /// Terminal client output; the parent enforces durability before exposure.
    Reply {
        /** Opaque client destination; each admitted call or waiter receives one terminal reply. */
        to: ReplyTo,
        /** Answer to deliver only after any state-changing decision is durable. */
        reply: Reply,
    },
    /// Persistence output to join the parent's atomic decision, including task creation and
    /// keyed answer together.
    Save {
        /** Owned replacement row for the parent's current atomic decision, not a standalone IO submission. */
        record: Stored,
    },
    /// Removal output for an ended or expired sign-in, with durability handled by the parent.
    Erase {
        /** Logical row to remove with the parent's decision; secret cleanup remains below the domain. */
        key: Key,
    },
    /// Terminal role-update refusal with no role replacement.
    RolesRefused {
        /** Project whose role update was refused without replacing holdings. */
        project: u32,
        /** Bound or capacity reason for the refused role update. */
        outcome: Outcome,
    },
    /// Terminal restoration failure; a failed instance cannot become ready through more rows or
    /// start completion.
    RestoreRefused {
        /** Record key that made restoration fail. */
        key: Key,
        /** Restore representation bound reason; this instance stays unready. */
        refusal: Refusal,
    },
}

/// One party's membership in one project, borrowed through a bounded read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProjectRole {
    pub project: u32,
    pub role: Role,
}

/// Maximum explicit account uses (domain/accounts.md, 4.2).
pub const USES: u32 = 16;

/// A payer an LLM account may serve (domain/accounts.md, 4.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Use {
    Party(u64),
    Project(u32),
}

/// Holder changes to LLM accounts (domain/accounts.md, section 5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AccountChange {
    ReadLists {
        provider: u32,
    },
    DeclareProgram {
        provider: u32,
        join: Option<u32>,
        name: Box<[u8]>,
    },
    CheckProgram {
        source: u32,
    },
    AddKey {
        provider: u32,
        join: Option<u32>,
        digest: [u8; 32],
    },
    ReplaceKey {
        source: u32,
        digest: [u8; 32],
    },
    RemoveKey {
        source: u32,
    },
    Resume {
        source: u32,
    },
    Remove {
        account: u32,
    },
    MoveAccount {
        account: u32,
        place: u32,
    },
    MoveSource {
        account: u32,
        source: u32,
        place: u32,
    },
    SetUses {
        account: u32,
        uses: Option<Box<[Use]>>,
    },
    MoveOut {
        account: u32,
    },
    Written {
        number: u32,
    },
    MoveIn {
        digest: [u8; 32],
    },
    /// Finish the party's sign-in under `source`, carrying its browser's address as a key is (domain/accounts.md, section 5).
    FinishSignIn {
        source: u32,
        digest: [u8; 32],
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

/// The live sign-in's party (domain/people/README.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Signer {
    pub identity: Identity,
    pub kind: Kind,
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
            | Self::Stopped { .. }
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
            | Self::AccountProgramDeclared { .. }
            | Self::AccountReading { .. }
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
            | Self::Stopped { .. }
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
            | Self::AccountProgramDeclared { .. }
            | Self::AccountReading { .. }
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
            | OutcomeFamily::RolesSet(_)
            | OutcomeFamily::ProjectMade(_)
            | OutcomeFamily::Settled => ActStatus::Refused,
        }
    }
}
