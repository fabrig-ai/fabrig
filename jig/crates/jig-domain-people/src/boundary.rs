//! Root/people messages and durable child records (domain/people/README.md, sections 3–4 and domain/people/acts.md). The root supplies authenticated identities, roles and keyed
//! requests; people returns bounded routes and persistence intentions. Every
//! routed flight ends with `Decided`; the root commits before releasing replies.
//! These values contain no credential secrets, task internals or protocol bytes.

pub use jig_domain_contract::lifecycle::{Adoption, ResourceName, ResourceRole};
pub use jig_domain_contract::ops::party::{
    AccountHostFailure, AccountHostWhy, Act, ActStatus, EntryKind, EscalationChoice, Outcome, OutcomeFamily,
    ProposalChoice, Proposer, Whom,
};

use crate::{Refusal, ReplyRef, Seen};
use alloc::boxed::Box;
pub use jig_domain_contract::ops::Role;
use skein_lib::{Duration, List, ReplyTo, Token, Wall};

/// Maximum address bytes retained in an account sign-in answer.
pub const ADDRESS: u32 = 2_048;

/// The source whose volatile needs a projection replaces (domain/people/README.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum NeedSource {
    /// A durable task: its questions, escalation, person task, replies and its own proposal.
    Task(u64),
    /// A party's proposal, whatever task it names.
    Proposal { party: u64, number: u64 },
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
#[expect(
    clippy::large_enum_variant,
    reason = "an amendment stays inline, as `Event` carries a keyed request whole, and is priced by size_of"
)]
pub enum Ask {
    /// Pause, drain or resume a linked worker (domain/engine/workers.md, 5).
    PauseWorker {
        worker: u32,
        mode: jig_domain_link::PauseMode,
    },
    /// Conclude a gathering (jig/docs/design/domain/tasks/gatherings.md, 3).
    Conclude {
        project: u32,
        task: u64,
    },
    /// Propose the settings as a revision left them (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/people/acts.md, 1).
    Restore {
        revision: u32,
        reason: Box<[u8]>,
    },
    /// Delete an owned project (jig/docs/design/domain/people/acts.md, 2).
    RemoveProject {
        project: u32,
        grace: Option<Duration>,
    },
    /// Deployment owner changes settings, whose value the core holds during admission.
    ChangeSettings {
        against: u32,
        digest: [u8; 32],
        projects: Box<[u32]>,
    },
    /// Deployment owner moves live agent tasks to the newest (domain/engine/settings.md, 4; domain/people/acts.md, 1).
    MoveToNewest {
        revision: u32,
    },
    /// Several task operations committed together under one key.
    Compound {
        project: u32,
        operations: List<crate::Operation>,
    },
    /// Make these roots together (domain/people/acts.md, 1).
    Create {
        project: u32,
        batch: Box<[crate::Member]>,
    },
    /// Make delegates beneath a task (jig/docs/design/domain/tasks/README.md, 4).
    Delegate {
        project: u32,
        task: u64,
        batch: Box<[crate::Member]>,
    },
    /// Mark results or one conversation's replies seen under this party's key.
    MarkSeen {
        seen: Seen,
    },
    /// Application-owned request metadata; the root carries the value, and the protocol supplies its digest.
    Application {
        scope: crate::ApplicationScope,
        connector: u16,
        digest: [u8; 32],
    },
    /// Correct or delete a note whose scope the party's project role permits.
    EditNote {
        project: u32,
        name: u64,
        scope: Box<NoteScope>,
        change: Box<NoteChange>,
    },
    /// Make a deployment service in a project with a role and display name.
    MakeService {
        project: u32,
        name: Box<[u8]>,
        role: Role,
    },
    /// Adopt a connector resource as a durable keyed owner request.
    Adopt {
        project: u32,
        adoption: Adoption,
    },
    /// Propose a batch to the role named by the project's policy.
    Propose {
        project: u32,
        action: crate::Action,
        reason: Box<[u8]>,
    },
    /// Withdraw this party's own pending proposal.
    Withdraw {
        project: u32,
        proposal: u64,
    },
    /// Hold a live run for this person's decision, cancelling its current claim.
    Stop {
        project: u32,
        task: u64,
        reason: Box<[u8]>,
    },
    /// Cancel a task and its descendants with a bounded reason, at once or after a grace.
    Cancel {
        project: u32,
        task: u64,
        reason: Box<[u8]>,
        grace: Option<Duration>,
    },
    /// Lift a held task's cause and rejudge it.
    Release {
        project: u32,
        task: u64,
        revision: u64,
        again: Option<bool>,
    },
    /// Lift a procedure's own hold and ask it to measure afresh.
    Measure {
        project: u32,
        task: u64,
        revision: u64,
    },
    /// End a placed hold, optionally releasing its tasks (domain/tasks/lifecycle-holds.md, 2).
    Lift {
        hold: u64,
        revision: u64,
        with_tasks: bool,
    },
    /// Claim one role-addressed person task for this authenticated person.
    TakePerson {
        project: u32,
        task: u64,
    },
    /// Give this person's current role task claim back to its role.
    HandBackPerson {
        project: u32,
        task: u64,
    },
    /// Supply the bounded result required by an addressed person task.
    AnswerPerson {
        project: u32,
        task: u64,
        revision: u64,
        result: PersonResult,
    },
    /// Adopt a live task as this authenticated person's own goal, retaining its old requester's reference.
    Move {
        project: u32,
        task: u64,
        reason: Box<[u8]>,
    },
    /// Authenticated keyed decision for a proposal waiting at this person or a policy role.
    DecideProposal {
        project: u32,
        proposer: Proposer,
        proposal: u64,
        decision: ProposalDecision,
        running: Option<crate::Running>,
    },
    /// Authenticated requester's whole words to an existing chat.
    Say {
        project: u32,
        task: u64,
        words: Box<[u8]>,
    },
    Talk {
        project: u32,
        task: u64,
        words: Box<[u8]>,
    },
    Leave {
        project: u32,
        task: u64,
    },
    /// Answer a numbered question waiting in a task requested by this person.
    AnswerQuestion {
        project: u32,
        task: u64,
        question: u64,
        words: Box<[u8]>,
    },
    /// Amend a live task this person may steer, after root policy and funding checks.
    /// A priority-only edit of a tracked task names spec revision zero.
    Amend {
        project: u32,
        task: u64,
        revision: u64,
        amendment: crate::Amendment,
        release: Option<u64>,
    },
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
    ChangePolicy {
        project: u32,
        change: crate::PolicyChange,
    },
    /// Owner replaces this project's name (domain/people/acts.md, 1).
    Rename {
        project: u32,
        name: Box<str>,
    },
    /// Deployment owner makes a project from a configured policy (domain/people/acts.md, 2).
    MakeProject {
        name: Box<str>,
        policy: u32,
    },
    /// Owner changes one person's current-period pool, preserving already spent and reserved funds.
    SetPool {
        project: u32,
        person: u64,
        budget: u64,
    },
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
    /// a role (domain/authority-checks.md, 1.4), and decided by the accounts child (domain/accounts.md, 3.3).
    AccountSignIn {
        holder: AccountHolder,
        provider: u32,
    },
    /// Sign this holder's account's source out; admitted like a sign-in.
    AccountSignOut {
        holder: AccountHolder,
        source: u32,
    },
    /// Change this holder's LLM accounts; carried keys and moved records retain only their digest.
    AccountChange {
        holder: AccountHolder,
        change: AccountChange,
    },
    /// Set an owned project's visibility (jig/docs/design/domain/people/public-reading.md, 2).
    SetVisibility {
        project: u32,
        public: bool,
    },
    /// End all sign-ins of a service without changing its party.
    ReissueService {
        party: u64,
    },
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
    ReissueService,
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
            | Self::Seen
            | Self::ReissueService => None,
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
            | Self::Talk { project, .. }
            | Self::Leave { project, .. }
            | Self::Say { project, .. }
            | Self::AnswerQuestion { project, .. }
            | Self::Amend { project, .. }
            | Self::Move { project, .. }
            | Self::TakePerson { project, .. }
            | Self::HandBackPerson { project, .. }
            | Self::AnswerPerson { project, .. }
            | Self::Stop { project, .. }
            | Self::Conclude { project, .. }
            | Self::Cancel { project, .. }
            | Self::Measure { project, .. }
            | Self::Release { project, .. }
            | Self::Create { project, .. }
            | Self::Delegate { project, .. }
            | Self::Propose { project, .. }
            | Self::Withdraw { project, .. } => AskFamily::Project(*project),
            Self::Application { scope, .. } => AskFamily::Application(*scope),
            Self::Restore { .. } => AskFamily::Restore,
            Self::PauseWorker { .. }
            | Self::MakeProject { .. }
            | Self::ChangeSettings { .. }
            | Self::MoveToNewest { .. } => AskFamily::Deployment,
            Self::AccountSignIn { .. } | Self::AccountSignOut { .. } | Self::AccountChange { .. } => AskFamily::Account,
            Self::Lift { .. } => AskFamily::Lift,
            Self::MarkSeen { .. } => AskFamily::Seen,
            Self::ReissueService { .. } => AskFamily::ReissueService,
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
            | Self::ReissueService { .. }
            | Self::EditNote { .. }
            | Self::Adopt { .. }
            | Self::ChangePolicy { .. }
            | Self::RemoveProject { .. }
            | Self::SetVisibility { .. }
            | Self::Rename { .. }
            | Self::MakeProject { .. }
            | Self::ChangeSettings { .. }
            | Self::MoveToNewest { .. }
            | Self::PauseWorker { .. }
            | Self::SetPool { .. }
            | Self::DecideEscalation { .. }
            | Self::DecideProposal { .. }
            | Self::Talk { .. }
            | Self::Leave { .. }
            | Self::Say { .. }
            | Self::AnswerQuestion { .. }
            | Self::Amend { .. }
            | Self::Move { .. }
            | Self::TakePerson { .. }
            | Self::HandBackPerson { .. }
            | Self::AnswerPerson { .. }
            | Self::Stop { .. }
            | Self::Conclude { .. }
            | Self::Cancel { .. }
            | Self::Lift { .. }
            | Self::Measure { .. }
            | Self::Release { .. }
            | Self::Create { .. }
            | Self::Delegate { .. }
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

/// Keyed person's choice for a pending proposal.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProposalDecision {
    Accept,
    Reject { reason: Box<[u8]> },
    Pass,
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
#[expect(
    clippy::large_enum_variant,
    reason = "the people boundary carries the complete request including its later choice"
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
    /// Admit a deployment act whose answer goes back under the connector’s key.
    ConnectorAct { key: ActKey, act: Box<Act> },
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
    /// Decide a live service reissue flight and answer its waiters.
    ReissueService { request: Token },
    /// Add previously unknown collaborators and their first project role in one decision.
    Seed { project: u32, collaborators: Box<[Seed]> },
    /// Replace the volatile inbox projection of one durable task after its row changes.
    Waiting { source: NeedSource, entries: Box<[Entry]> },
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
        /// Cap on a new sign-in's expiry; a live held sign-in keeps its saved expiry.
        until: Option<Wall>,
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
    /// Terminal connector answer held by the parent until durable.
    ConnectorActed { key: ActKey, outcome: Outcome },
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
    /// Parties omitted from a replaced project's roster, after its save.
    ReadingLost { project: u32, parties: Box<[u64]> },
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
    /// A new task-derived need reaches this party; the root holds it until durable
    /// (domain/people/README.md, section 8).
    NeedArrived { party: u64, project: u32, task: u64, needs: u32 },
}

/// One party's membership in one project, borrowed through a bounded read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProjectRole {
    pub project: u32,
    pub role: Role,
}

/// Maximum explicit account uses (domain/accounts-uses.md, 1.2).
pub const USES: u32 = 16;

/// A payer an LLM account may serve (domain/accounts-uses.md, 1.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Use {
    Party(u64),
    Project(u32),
}

/// Holder changes to LLM accounts (domain/accounts.md, section 5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AccountChange {
    LoginProgram {
        provider: u32,
        name: Box<[u8]>,
    },
    CancelLogin {
        number: u32,
    },
    LoginEnded {
        number: u32,
        exit: LoginExit,
    },
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

/// The live sign-in's party (domain/people/README.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Signer {
    pub identity: Identity,
    pub kind: Kind,
}

/// A program's terminal status (domain/terminal-login.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LoginExit {
    Exited { code: u8 },
    Killed { signal: u8 },
    NotStarted,
}
