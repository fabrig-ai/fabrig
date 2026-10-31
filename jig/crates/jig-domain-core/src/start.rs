//! Load, link and sweep before durable opening (jig/docs/design/domain/engine.md, 6).

mod census;
mod cursor;
pub(crate) mod first;
mod inbox;
mod links;
pub(crate) mod projections;
mod restore;
mod runs;
mod script;
mod settings;

pub use census::{Census, ReadResults};
pub use cursor::restore_cursor;

use crate::{Core, Key, Wall, settings as core_settings};
use jig_domain_accounts as accounts;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
pub use restore::Restore;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    LoadCore,
    LoadInbox,
    RestoreConnector { connector: u16 },
    Link,
    Sweep,
    Switch,
    Wires,
    AdoptRuns,
    ReadAfresh { connector: u16 },
    SettleOutbox { connector: u16 },
    Open,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    Settings { failure: core_settings::Failure, changed: bool },
    ProjectPolicy { project: u32 },
    HeldPastLimit { which: HeldLimit, held: u32, limit: u32 },
    AccountsPastLimit { failure: accounts::RestoreLimit },
    LiveSet { set: LiveSet, key: Key, held: u64, limit: u64 },
    Accounting { key: Key },
    LlmChoice { key: Key },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LiveSet {
    Projects,
    ReadResults,
    RunProofs,
    Calls,
    Projections,
    Conversations,
    Census,
    PeopleBound(people::RestoreBound),
    TasksBound(tasks::RestoreBound),
    /// Restored worker claims past the attempts the fleet can adopt (`limits.fleet.attempts`).
    WorkerClaims,
}

/// A journal release identity, holding no row or output payload.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Continuation {
    sequence: u64,
    phase: Phase,
}

#[derive(Debug)]
pub enum Action {
    Step(Phase),
    Commit(Continuation),
    Idle,
    Refused(Failure),
}

/// Drive one queued sweep row or finish the walk.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SweepInput {
    Next,
    Finished,
}

/// The held settings bound that prevents a cold start (engine.md, 17.7–17.8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeldLimit {
    Revisions,
    Versions,
}

#[derive(Debug)]
pub(crate) struct Script {
    pub(crate) consumed: bool,
    pub(crate) phase: Option<Phase>,
    pub(crate) failure: Option<Failure>,
    pub(crate) missing: Option<(Key, Key)>,
    at: Wall,
    sequence: u64,
    waiting: Option<Continuation>,
    pub(crate) held: crate::Queue<crate::Request>,
    waiting_tasks: crate::Queue<tasks::Request>,
    pub(crate) sweep: crate::Queue<people::Stored>,
    pub(crate) projections: Option<skein_lib::Map<u64, projections::Assembly>>,
}

impl Script {
    pub(crate) fn new() -> Self {
        Self {
            consumed: false,
            phase: None,
            failure: None,
            missing: None,
            at: Wall::EPOCH,
            sequence: 0,
            waiting: None,
            held: crate::Queue::uncapped(),
            waiting_tasks: crate::Queue::uncapped(),
            projections: None,
            sweep: crate::Queue::uncapped(),
        }
    }
}

impl Core {
    pub(crate) fn start_loading(&self) -> bool {
        self.start.phase == Some(Phase::LoadCore) && self.start.waiting.is_none()
    }

    pub(crate) fn start_input(&mut self) {
        self.start.consumed = true;
    }

    pub(crate) fn start_runtime_input(&mut self) {
        self.start_input();
        match self.start.phase {
            Some(
                Phase::LoadCore
                | Phase::LoadInbox
                | Phase::RestoreConnector { .. }
                | Phase::Link
                | Phase::Sweep
                | Phase::Switch
                | Phase::Wires,
            ) => {
                unreachable!("runtime input requires durable Wires")
            }
            Some(Phase::AdoptRuns | Phase::ReadAfresh { .. } | Phase::SettleOutbox { .. } | Phase::Open) | None => {}
        }
    }

    pub(crate) fn start_room(&mut self, set: LiveSet, key: Key, held: u32, limit: u32) -> bool {
        if held < limit {
            return true;
        }
        let _refused = self.start_refuse(Failure::LiveSet {
            set,
            key,
            held: u64::from(held).checked_add(1).expect("bounded table occupancy"),
            limit: u64::from(limit),
        });
        false
    }

    /// Begin the start once, capturing the expiry instant before loading.
    pub fn start_begin(&mut self, pending: bool, at: Wall) -> Action {
        assert!(!self.start.consumed, "the start begins before input once");
        self.start_input();
        self.people.start_restore(at);
        self.authority.start_restore();
        self.start.phase = Some(Phase::LoadCore);
        self.start.at = at;
        self.start_pending = pending;
        self.people.end_seeding();
        self.counters.begin_start();
        Action::Step(Phase::LoadCore)
    }

    #[must_use]
    pub fn start_phase(&self) -> Option<Phase> {
        self.start.phase
    }

    #[must_use]
    pub fn start_failure(&self) -> Option<Failure> {
        self.start.failure.clone()
    }

    /// Freeze the script with its first fit failure.
    pub fn start_refuse(&mut self, failure: Failure) -> Action {
        self.start_input();
        assert!(self.start.phase.is_some(), "start refusal requires the start");
        if self.start.failure.is_none() {
            self.start.failure = Some(failure);
        }
        self.start.waiting = None;
        Action::Refused(self.start_failure().expect("first fit failure"))
    }

    /// Retain the first externally discovered missing obligation for the Link pass.
    pub fn start_missing(&mut self, record: Key, target: Key) {
        self.start_input();
        assert!(self.start.phase.is_some(), "missing link belongs to the start");
        if self.start.missing.is_none() {
            self.start.missing = Some((record, target));
        }
    }
}

/// Ordered live ranges of the cold start (protocol/store.md, 4.3 and 7.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RestoreRange {
    /// The deployment header, first.
    Deployment,
    /// The live settings, before the children restore.
    Settings,
    /// The accounts' counter, accounts and sources, before the child starts.
    Accounts,
    /// All people's live families.
    People,
    /// The tasks child's live families.
    Tasks,
    /// The core's remaining live families.
    Core,
    /// Inbox owners named by durable indexes after read positions are restored.
    Census(Census),
}

impl RestoreRange {
    /// The fixed order of `LoadCore`'s ranges.
    pub const LIVE: [Self; 6] =
        [Self::Deployment, Self::Settings, Self::Accounts, Self::People, Self::Tasks, Self::Core];

    /// Predicate checked against the store's intervals (protocol/store.md, 4.3).
    #[must_use]
    pub fn admits(&self, key: &Key) -> bool {
        match self {
            Self::Census(census) => return census.admits(key),
            Self::Deployment | Self::Settings | Self::Accounts | Self::People | Self::Tasks | Self::Core => {}
        }
        match key {
            Key::Core(crate::CoreKey::Settings(key)) => {
                *self == Self::Settings
                    && match key {
                        crate::settings::Key::History(_) => false,
                        crate::settings::Key::Default(_)
                        | crate::settings::Key::Changed(_)
                        | crate::settings::Key::Held(_)
                        | crate::settings::Key::Charter { .. }
                        | crate::settings::Key::Proposal { .. }
                        | crate::settings::Key::Extent => true,
                    }
            }
            Key::Core(crate::CoreKey::Deployment) => *self == Self::Deployment,
            Key::Core(key) => *self == Self::Core && key.live(),
            Key::People(key) => *self == Self::People && key.live(),
            Key::Tasks(key) => *self == Self::Tasks && key.live(),
            Key::Accounts(_) => *self == Self::Accounts,
            Key::Notes(_) => false,
        }
    }
}

impl Core {
    pub(crate) fn ready(&self) -> bool {
        self.start.phase.is_none() && !self.start_stopped()
    }
}
