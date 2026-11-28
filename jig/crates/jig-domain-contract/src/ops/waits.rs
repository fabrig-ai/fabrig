//! Wait facts shared by procedure reports and task document parts.

use super::party::EntryKind;
use super::{AccountWhy, EscalationHolder};
use crate::resources::Name;
use skein_lib::{Duration, List, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WaitKind {
    Party,
    Question,
    Dependency,
    Resource,
    Account,
    Host,
    Delegate,
    News,
    Step,
    Room,
    Turn,
    SignOff,
}

impl WaitKind {
    pub const ALL: [Self; 12] = [
        Self::Party,
        Self::Question,
        Self::Dependency,
        Self::Resource,
        Self::Account,
        Self::Host,
        Self::Delegate,
        Self::News,
        Self::Step,
        Self::Room,
        Self::Turn,
        Self::SignOff,
    ];

    /// This kind’s position in `ALL`.
    #[must_use]
    pub const fn slot(self) -> u32 {
        match self {
            Self::Party => 0,
            Self::Question => 1,
            Self::Dependency => 2,
            Self::Resource => 3,
            Self::Account => 4,
            Self::Host => 5,
            Self::Delegate => 6,
            Self::News => 7,
            Self::Step => 8,
            Self::Room => 9,
            Self::Turn => 10,
            Self::SignOff => 11,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum WaitObject {
    Entry { kind: EntryKind, at: EscalationHolder },
    Question { number: u64, answerer: u64 },
    Task(u64),
    Resource { name: Name, holder: Option<u64> },
    Run(TaskRunWait),
    News { subscriptions: List<u64>, timer: Option<Wall> },
    Procedure { step: u32, things: List<Thing> },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Thing {
    pub what: u32,
    pub number: u64,
    pub owner: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AtBound {
    Hold,
    PassUp,
    Overdue,
    GiveUp,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unbounded {
    NoDeadline,
    Standing,
    Behind(u64),
    OperatorHold,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskRunWait {
    Account { account: Option<u32>, source: Option<u32>, why: TaskWaitWhy },
    Credential,
    Host,
    Worker { worker: u32, until: Wall },
    Refused(TaskWaitRefused),
}

/// A derived explanation, never a durable task state (domain/engine/fleet.md).
/// Hello keeps the fleet's observed receipt time and cause until admission or restart.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskWaitRefused {
    Hello { kind: HostKind, at: Wall, why: HelloRefusal, figures: Option<Figures> },
    Bounds { kind: HostKind, why: Option<AgentShortfall>, figures: Option<Figures> },
    Job { worker: Option<u32>, kind: u32, why: JobWhy, figures: Option<Figures> },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskWaitWhy {
    Source(TaskSourceState),
    Refreshing,
    Host,
    Empty,
    Invalid(AccountWhy),
    Untooled { worker: u32 },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JobWhy {
    Short(JobShort),
    Untranslated,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskSourceState {
    Moving,
    Asking,
    Granted,
    Later { until: Option<Wall> },
    Limited { until: Option<Wall> },
    Refused,
    NeedsSignIn,
    Misconfigured,
    Unloaded { at: Place, why: Unread },
    Cancelled,
    Closing,
}

/// A kind of host that can run an attempt.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HostKind {
    /// A worker connected on a channel.
    Worker,
    /// The engine's own host.
    Engine,
}

/// The owner's reason for refusing a worker's hello.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HelloRefusal {
    /// Channel zero belongs to the engine's configured host.
    ReservedChannel,
    /// The hello's job kinds are not ascending and distinct.
    JobKinds,
    /// The agent cannot hold a configured charter for this host kind.
    Agent { why: AgentShortfall },
    /// The connected worker set has no admission room.
    Workers { limit: u32 },
    /// The position is unconfigured or already held by a live channel.
    Position { worker: u32 },
    /// A worker must stop strictly before the engine's grace expires.
    StopBound { grace: Duration },
}

/// What each side holds of an agent's shortfall (domain/engine/fleet.md):
/// in the bound's own unit, a duration in milliseconds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Figures {
    pub required: u64,
    pub offered: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AgentShortfall {
    Model { charter: u32, model: u32 },
    Window { charter: u32, model: u32 },
    Output { charter: u32, model: u32 },
    Turns { charter: u32 },
    Spend,
    Time { charter: u32 },
    Waiting { charter: u32 },
    Checks { charter: u32 },
    Message,
    Inbox,
    Unacknowledged,
}

/// Jobs use declared bounds and room alone (jig/docs/design/domain/engine/executors.md, 5).
/// Why a host cannot cover an application's job kind (domain/engine/fleet.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JobShort {
    Kind,
    Spec,
    Result,
    Deadline,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Place {
    Directory,
    File,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unread {
    Unreadable,
    Refused,
    Failed,
    Stalled,
    TooLarge,
}
