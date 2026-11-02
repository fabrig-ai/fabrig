//! Deployment-owned hosts, accounts and bounded diagnostics.
//! Core-owned facts; connector diagnostics are supplied independently.
//! client/domain/contract.md, 5 and 7.1;
//! client/domain/pages.md, 7.
use super::{AdmissionLimit, Chip, Funder, Proposer, Requester, SeenThrough, TaskLink};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};

/// Whole supplied deployment facts with independent bounded loss counters.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "independent aggregate and list loss facts")]
pub struct Deployment {
    pub store: StoreFill,
    pub hosts: List<Host>,
    pub attempts: List<HostRun>,
    pub accounts: List<Account>,
    pub tasks: Capacity,
    pub workers: Capacity,
    pub attempts_capacity: Capacity,
    pub waiting_attempts: u32,
    pub proposals: Capacity,
    pub projects: List<ProjectCapacity>,
    pub grace_bound: Duration,
    pub hello_refusals: List<HelloRefusalLine>,
    pub hosts_forgotten: u64,
    pub hello_refusals_forgotten: u64,
    pub person_room_refusals: List<PersonCapacityRefusal>,
    pub person_room_refusals_forgotten: u64,
    pub person_answer_retention: Duration,
    pub task_room_refusals: List<TaskCapacityRefusal>,
    pub task_room_refusals_forgotten: u64,
    pub cut: bool,
    /// Authentic configured connector numbers, in source configuration order.
    pub connectors: List<u16>,
    /// Independently supplied configured catalogue loss.
    pub connectors_cut: bool,
    /// Public configured provider offers within the document line bound.
    pub providers: List<super::ProviderOffer>,
    /// Whether additional provider offers were omitted.
    pub providers_cut: bool,
    /// Actual startup key ceiling, including the client carried-value bound.
    pub key_bytes: u32,
    /// Current observation from the engine selection and host checks.
    pub readiness: List<super::PayerReadiness>,
    /// Whether additional provider/payer rows were omitted.
    pub readiness_cut: bool,
}

/// Store map bytes at the last commit answer and its admission mark (store.md, 11.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StoreFill {
    pub bytes: u64,
    pub mark: u64,
}

/// One independently counted resource capacity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Capacity {
    pub used: u32,
    pub limit: u32,
}

/// A project task capacity and pending proposal count.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectCapacity {
    pub project: u32,
    pub tasks: Capacity,
    pub proposals: u32,
}

/// A host's current placement rule, independent of a run's payer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostRule {
    Open,
    Person(u64),
    Several,
    Unknown,
}

/// A connected or retained host and its supplied placement facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "independent host, inventory and document facts")]
pub struct Host {
    pub name: Option<Box<[u8]>>,
    pub awaiting_logins: bool,
    pub rule: HostRule,
    pub programs: List<ProgramLogin>,
    pub channel: u64,
    pub kind: HostKind,
    pub slots: u32,
    pub used: u32,
    pub free: u32,
    pub draining: bool,
    pub connected: bool,
    pub grace_until: Option<Wall>,
    pub grace_remaining: Option<Duration>,
    pub runs: List<HostRun>,
    pub cut: bool,
}

/// One attempt with its own identity, lifecycle and placement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HostRun {
    pub task: TaskLink,
    pub attempt: u64,
    pub kept: u32,
    pub listed: bool,
    pub state: AttemptState,
    pub location: RunLocation,
    pub grace_until: Option<Wall>,
}

/// Secret-free account state and independent attention and spend facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "independent account flags and owner list loss")]
pub struct Account {
    pub number: u32,
    pub place: u32,
    pub uses: Option<List<super::AccountUse>>,
    pub plan: bool,
    pub provider: Option<u32>,
    pub provider_name: Option<super::DocumentText>,
    pub manage_usage: Option<Box<[u8]>>,
    pub display: super::DocumentText,
    pub holder: super::AccountHolder,
    pub sources: List<super::AccountSource>,
    pub generation: Option<u64>,
    pub state: AccountState,
    pub usable: bool,
    pub expiring: bool,
    pub attention: bool,
    pub spent_until: Option<Wall>,
    pub runs: List<Chip>,
    pub waiting: List<Chip>,
    pub cut: bool,
}

/// One retained refused hello with its original clock and advertised stop bound.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HelloRefusalLine {
    pub channel: u64,
    pub at: Wall,
    pub slots: u32,
    pub stop_bound: Duration,
    pub why: HelloRefusal,
}

/// The person and opaque key of a retained request; the bytes are never interpreted.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RequestKey {
    pub person: u64,
    pub key: [u8; 16],
}

/// One person request refused for room, in source history order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersonCapacityRefusal {
    pub key: RequestKey,
    pub project: Option<u32>,
    pub action: PersonCapacityAction,
    pub at: Wall,
    pub cause: PersonCapacityCause,
}

/// One task caller refused for room, in source history order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskCapacityRefusal {
    pub request: TaskCapacityRequest,
    pub at: Wall,
}

/// The caller, action and room cause supplied for a task request.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskCapacityRequest {
    pub project: Option<u32>,
    pub by: Requester,
    pub action: TaskCapacityAction,
    pub cause: TaskCapacityCause,
}

/// The supplied closed hostkind vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostKind {
    /// Worker as supplied by its owner.
    Worker,
    /// Engine as supplied by its owner.
    Engine,
}

/// The supplied closed attemptstate vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttemptState {
    /// Waiting as supplied by its owner.
    Waiting,
    /// Adopted as supplied by its owner.
    Adopted,
    /// Claimed as supplied by its owner.
    Claimed,
    /// Cancelled as supplied by its owner.
    Cancelled,
    /// Handed as supplied by its owner.
    Handed,
    /// Acknowledged as supplied by its owner.
    Acknowledged,
    /// Stray as supplied by its owner.
    Stray,
    /// Kept as supplied by its owner.
    Kept,
    /// Fenced as supplied by its owner.
    Fenced,
    /// Closed as supplied by its owner.
    Closed,
}

/// The supplied closed runlocation vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RunLocation {
    /// None as supplied by its owner.
    None,
    /// Connected as supplied by its owner.
    Connected { channel: u64, kind: HostKind },
    /// Adrift as supplied by its owner.
    Adrift { channel: Option<u64>, until: Option<Wall>, remaining: Option<Duration> },
}

/// The supplied closed accountstate vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountState {
    Asking,
    Granted,
    Later { until: Option<Wall> },
    SignInAgain,
    Misconfigured,
    Unloaded { at: AccountPlace, why: AccountUnread },
    Cancelled,
    Closing,
    Moving,
}

/// The supplied closed hellorefusal vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HelloRefusal {
    /// Its job kinds are not ascending and distinct.
    JobKinds,
    Agent {
        why: AgentShortfall,
    },
    /// `ReservedChannel` as supplied by its owner.
    ReservedChannel,
    /// An unconfigured or occupied worker position, supplied by its owner.
    Position {
        worker: u32,
    },
    /// Workers as supplied by its owner.
    Workers {
        limit: u32,
    },
    /// `StopBound` as supplied by its owner.
    StopBound {
        grace: Duration,
    },
}

/// The supplied closed personcapacityaction vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PersonCapacityAction {
    Lift {
        hold: u64,
    },
    /// Creating tasks as one batch.
    Create,
    /// Applying several task operations together.
    Compound,
    /// Application as supplied by its owner.
    Application {
        connector: u16,
    },
    /// Note as supplied by its owner.
    Note {
        name: u64,
    },
    /// Service as supplied by its owner.
    Service,
    /// Adoption as supplied by its owner.
    Adoption,
    /// Roles as supplied by its owner.
    Roles,
    /// Policy as supplied by its owner.
    Policy,
    /// Pool as supplied by its owner.
    Pool {
        person: u64,
    },
    /// Escalation as supplied by its owner.
    Escalation {
        task: u64,
    },
    /// Proposal as supplied by its owner.
    Proposal {
        proposer: Proposer,
        number: u64,
    },
    /// Words as supplied by its owner.
    Words {
        task: u64,
    },
    /// Question as supplied by its owner.
    Question {
        task: u64,
        question: u64,
    },
    /// Amend as supplied by its owner.
    Amend {
        task: u64,
    },
    /// Move as supplied by its owner.
    Move {
        task: u64,
    },
    /// `TakePerson` as supplied by its owner.
    TakePerson {
        task: u64,
    },
    /// `HandBackPerson` as supplied by its owner.
    HandBackPerson {
        task: u64,
    },
    /// `AnswerPerson` as supplied by its owner.
    AnswerPerson {
        task: u64,
    },
    /// Stop as supplied by its owner.
    Stop {
        task: u64,
    },
    /// Cancel as supplied by its owner.
    Cancel {
        task: u64,
    },
    /// Release as supplied by its owner.
    Release {
        task: u64,
    },
    /// `MarkSeen` as supplied by its owner.
    MarkSeen {
        seen: SeenThrough,
    },
    /// An account sign-in, without the request's provider or holder.
    AccountSignIn,
    /// An account sign-out, without the request's source or holder.
    AccountSignOut,
    /// An account change, without its credential or subject payload.
    AccountChange,
}

/// The supplied closed personcapacitycause vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PersonCapacityCause {
    /// Requests as supplied by its owner.
    Requests { limit: u32 },
    /// Pending as supplied by its owner.
    Pending { limit: u32 },
    /// Waiters as supplied by its owner.
    Waiters { limit: u32 },
    /// Limit as supplied by its owner.
    Limit { which: AdmissionLimit },
}

/// The supplied closed taskcapacityaction vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskCapacityAction {
    /// Batch as supplied by its owner.
    Batch {
        first: Option<u64>,
        members: u32,
    },
    /// Proposal as supplied by its owner.
    Proposal {
        number: u64,
    },
    /// `PersonProposal` as supplied by its owner.
    PersonProposal {
        number: u64,
    },
    /// Message as supplied by its owner.
    Message {
        task: u64,
        number: u64,
    },
    /// Source as supplied by its owner.
    Source {
        funder: Funder,
    },
    /// Introduce as supplied by its owner.
    Introduce {
        left: u64,
        right: u64,
    },
    /// Subscription as supplied by its owner.
    Subscription {
        number: u64,
    },
    /// Effect as supplied by its owner.
    Effect {
        entry: u64,
    },
    /// Claim as supplied by its owner.
    Claim {
        attempt: u64,
    },
    /// Release as supplied by its owner.
    Release,
    /// Move as supplied by its owner.
    Move,
    Place {
        hold: u64,
    },
}

/// The supplied closed taskcapacitycause vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskCapacityCause {
    /// Tasks as supplied by its owner.
    Tasks {
        limit: u32,
    },
    /// `CreationFlights` as supplied by its owner.
    CreationFlights {
        limit: u32,
    },
    /// `ResourceReports` as supplied by its owner.
    ResourceReports {
        limit: u32,
    },
    /// `ProjectTasks` as supplied by its owner.
    ProjectTasks {
        limit: u32,
    },
    /// Tree as supplied by its owner.
    Tree {
        limit: u32,
    },
    /// Delegates as supplied by its owner.
    Delegates {
        limit: u32,
    },
    /// Inbox as supplied by its owner.
    Inbox {
        task: u64,
        messages: u32,
        bytes: u32,
    },
    /// Holdings as supplied by its owner.
    Holdings {
        limit: u32,
    },
    /// `HoldWaiters` as supplied by its owner.
    HoldWaiters {
        limit: u32,
    },
    /// Proposals as supplied by its owner.
    Proposals {
        limit: u32,
    },
    /// References as supplied by its owner.
    References {
        limit: u32,
    },
    /// Subscriptions as supplied by its owner.
    Subscriptions {
        limit: u32,
    },
    /// Funders as supplied by its owner.
    Funders {
        limit: u32,
    },
    /// Writers as supplied by its owner.
    Writers {
        limit: u32,
    },
    PlacedHolds {
        limit: u32,
    },
    ProjectPlacedHolds {
        limit: u32,
    },
    PartyPlacedHolds {
        limit: u32,
    },
    Exemptions {
        limit: u32,
    },
}

/// Where the account's record could not load.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountPlace {
    Directory,
    File,
}
/// Why the account's record could not load.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountUnread {
    Unreadable,
    Refused,
    Failed,
    Stalled,
    TooLarge,
}

/// Which advertised agent capability falls short of the deployment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AgentShortfall {
    Model { charter: u32, model: u32 },
    Window { charter: u32, model: u32 },
    Output { charter: u32, model: u32 },
    Turns,
    Spend,
    Time,
    Waiting,
    Checks,
    Message,
    Inbox,
    Unacknowledged,
}

/// Names are whole; diagnostic details are bounded independently.
pub const PROGRAM_HOST_NAME: u32 = 4_096;
pub const PROGRAM_NAME: u32 = 32;
pub const PROGRAM_DETAIL: u32 = 4_096;
#[must_use]
pub fn valid_program_name(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 32 {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}
#[must_use]
pub fn valid_program_method(method: &[u8]) -> bool {
    if method.is_empty() || method.len() > 32 {
        return false;
    }
    for byte in method {
        if !(0x21..=0x7e).contains(byte) {
            return false;
        }
    }
    true
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProgramFailure {
    Missing,
    Deadline,
    Unreadable,
    Exited,
    Unconfigured,
    Unknown,
    Closed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProgramState {
    Usable,
    NotSignedIn,
    SignedInOtherwise { method: Box<[u8]> },
    Failed { reason: ProgramFailure, detail: Box<[u8]> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProgramSignOut {
    SignedOut,
    Failed { reason: ProgramFailure, detail: Box<[u8]> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProgramLogin {
    pub name: Box<[u8]>,
    pub state: Option<ProgramState>,
    pub sign_out: Option<ProgramSignOut>,
    pub cleanup: bool,
    pub cut: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProgramHost {
    pub name: Box<[u8]>,
    pub channel: u64,
    pub connected: bool,
    pub awaiting_logins: bool,
    pub programs: List<ProgramLogin>,
    pub cut: bool,
}
