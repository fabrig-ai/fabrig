//! Load, link and sweep before durable opening (jig/docs/design/domain/engine/restart.md;
//! jig/docs/design/domain/engine/settings-store.md, 1).

pub mod census;
pub mod cursor;
pub mod restore;

pub use census::{Census, ReadResults};
pub use cursor::restore_cursor;

use crate::{Key, settings as core_settings};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
pub use restore::Restore;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    LoadCore,
    LoadInbox,
    RestoreConnector { connector: u16 },
    Link,
    Sweep,
    Switch,
    Wires,
    AdoptRuns,
    Open,
}

/// A default or change outside its numbers (domain/engine/settings-store.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Numbered {
    pub setting: core_settings::Setting,
    pub changed: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Failure {
    Numbers {
        settings: Box<[Numbered]>,
    },
    /// A release's new defaults take numbers held by stored values (domain/engine/settings-store.md, 1).
    Taken {
        settings: Box<[core_settings::Setting]>,
    },
    Settings {
        failure: core_settings::Failure,
        changed: bool,
    },
    Application {
        kind: u32,
        scope: u32,
        number: u32,
        why: ApplicationWhy,
    },
    ProjectPolicy {
        project: u32,
    },
    HeldPastLimit {
        which: HeldLimit,
        held: u32,
        limit: u32,
    },
    NeedsPastLimit {
        held: u64,
        limit: u64,
    },
    AccountsPastLimit {
        failure: accounts::RestoreLimit,
    },
    LiveSet {
        set: LiveSet,
        key: Key,
        held: u64,
        limit: u64,
    },
    Accounting {
        key: Key,
    },
    LlmChoice {
        key: Key,
    },
    /// A record names a link the start cannot make (domain/engine/restart.md).
    Link {
        record: Key,
        linked: Linked,
    },
}

/// What a record links to that the start could not find (domain/engine/restart.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Linked {
    Record(Key),
    Charter {
        charter: u32,
        revision: u32,
    },
    Project(u32),
    Connector(u16),
    /// A proposal number no settings key can hold.
    SettingsProposal {
        proposer: people::Proposer,
        proposal: u64,
    },
    /// The hold that owns a writer's resource.
    Holder(tasks::Name),
}

/// A setting cannot be held under its application's declared kind (domain/engine/settings-application.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ApplicationWhy {
    Undeclared,
    Scope,
    Count,
    Shape,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LiveSet {
    Projects,
    ReadResults,
    RunProofs,
    Calls,
    Parts,
    Projections,
    Conversations,
    Census,
    PeopleBound(people::RestoreBound),
    TasksBound(tasks::RestoreBound),
    /// Restored worker claims past the attempts the fleet can adopt (`limits.fleet.attempts`).
    WorkerClaims,
}

/// A journal release identity, holding no row or output payload.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Continuation {
    pub sequence: u64,
    pub phase: Phase,
}

#[derive(Hash, Debug)]
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

/// The held settings bound that prevents a cold start (domain/engine/settings-store.md, 1, and domain/engine/settings.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HeldLimit {
    Revisions,
    Versions,
}

/// Ordered live ranges of the cold start (protocol/store/keys.md, 1.3, and protocol/store/loads.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RestoreRange {
    /// The deployment header, first.
    Deployment,
    /// The live settings, before the children restore.
    Settings,
    /// The accounts' counter, accounts, sources and facts, before the child starts.
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

    /// Predicate checked against the store's intervals (protocol/store/keys.md, 1.3).
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
