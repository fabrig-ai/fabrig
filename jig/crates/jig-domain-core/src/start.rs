//! Load, link and sweep before durable opening (jig/docs/design/domain/engine.md, 6).
mod links;
mod script;
use crate::{Core, HeldLimit, Key, Wall, settings};
use jig_domain_accounts as accounts;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

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
    Settings { failure: settings::Failure, changed: bool },
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
    People,
    SignIns,
    Roles,
    Answers,
    ReadResults,
    ReadConversations,
    Tasks,
    Stubs,
    Writers,
    Funders,
    Pools,
    PersonProposals,
    PlacedHolds,
    RunProofs,
    Calls,
    Projections,
    Conversations,
    Census,
    PeopleBound(people::RestoreBound),
    TasksBound(tasks::RestoreBound),
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

#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "the approved sweep input moves one decoded row without another allocation"
)]
pub enum SweepInput {
    Record(people::Stored),
    Finished,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StartMode {
    Default,
    LoadAndLink,
}

#[derive(Debug)]
pub(crate) struct Script {
    pub(crate) mode: StartMode,
    pub(crate) consumed: bool,
    pub(crate) phase: Option<Phase>,
    pub(crate) failure: Option<Failure>,
    pub(crate) missing: Option<(Key, Key)>,
    at: Wall,
    sequence: u64,
    waiting: Option<Continuation>,
    pub(crate) held: crate::Queue<crate::Request>,
    waiting_tasks: crate::Queue<tasks::Request>,
}

impl Script {
    pub(crate) fn new() -> Self {
        Self {
            mode: StartMode::Default,
            consumed: false,
            phase: None,
            failure: None,
            missing: None,
            at: Wall::EPOCH,
            sequence: 0,
            waiting: None,
            held: crate::Queue::uncapped(),
            waiting_tasks: crate::Queue::uncapped(),
        }
    }
}

impl Core {
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

    /// Choose the start interface once, capturing the expiry instant before loading.
    pub fn start_begin(&mut self, pending: bool, at: Wall) -> Action {
        assert!(!self.start.consumed && self.start.mode == StartMode::Default, "start selected before input once");
        self.start_input();
        self.people.start_restore(at);
        self.tasks.start_restore();
        self.accounts.start_restore();
        self.authority.start_restore();
        self.start.mode = StartMode::LoadAndLink;
        self.start.phase = Some(Phase::LoadCore);
        self.start.at = at;
        self.restart_pending = pending;
        self.people.end_seeding();
        self.counters.begin_restore();
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
        assert!(self.start.phase.is_some(), "start refusal requires a selected start");
        if self.start.failure.is_none() {
            self.start.failure = Some(failure);
        }
        self.start.waiting = None;
        Action::Refused(self.start_failure().expect("first fit failure"))
    }

    /// Retain the first externally discovered missing obligation for the Link pass.
    pub fn start_missing(&mut self, record: Key, target: Key) {
        self.start_input();
        assert!(self.start.phase.is_some(), "missing link belongs to a selected start");
        if self.start.missing.is_none() {
            self.start.missing = Some((record, target));
        }
    }
}
