//! Load, link and sweep before durable opening (jig/docs/design/domain/engine/restart.md).

pub use jig_domain_core_vocabulary::start::*;

pub(crate) mod first;
mod inbox;
mod links;
pub(crate) mod projections;
pub mod restore;
mod runs;
mod script;
mod settings;

use crate::{Core, Key, Wall};
use jig_domain_people as people;
use jig_domain_tasks as tasks;

#[derive(Hash, Debug)]
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
    caught_up: skein_lib::Map<u16, (bool, bool)>,
    pub(crate) projections: Option<skein_lib::Map<u64, projections::Assembly>>,
}

impl Script {
    pub(crate) fn new() -> Self {
        Self {
            caught_up: skein_lib::Map::with_capacity(0),
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
            Some(Phase::AdoptRuns | Phase::Open) | None => {}
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

    pub(crate) fn start_floor_time(&self) -> Option<Wall> {
        if self.start.phase.is_some() { Some(self.start.at) } else { None }
    }

    /// Begin the start once, capturing the expiry instant before loading.
    pub fn start_begin(&mut self, pending: bool, at: Wall) -> Action {
        assert!(!self.start.consumed, "the start begins before input once");
        self.start_input();
        self.people.start_restore(at);
        self.authority.start_restore();
        self.start.phase = Some(Phase::LoadCore);
        self.start.at = at;
        self.start.caught_up =
            skein_lib::Map::with_capacity(u32::try_from(self.connectors.len()).expect("configured connectors"));
        for &connector in &self.connectors {
            self.start.caught_up.insert(connector, (false, false)).expect("one report pair per connector");
        }
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

    /// Freeze the script with its first failure.
    pub fn start_refuse(&mut self, failure: Failure) -> Action {
        self.start_input();
        assert!(self.start.phase.is_some(), "start refusal requires the start");
        if self.start.failure.is_none() {
            self.start.failure = Some(failure);
        }
        self.start.waiting = None;
        Action::Refused(self.start_failure().expect("first start failure"))
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

impl Core {
    #[must_use]
    pub fn ready(&self) -> bool {
        self.start.phase.is_none() && !self.start_stopped()
    }
}

impl Core {
    /// Record one half of catching up after decisions have opened.
    pub fn catch_up_done(&mut self, connector: u16, stage: crate::effects::connector::RestartStage) {
        use crate::effects::connector::RestartStage;
        assert!(self.start_opened(), "catching up follows the opening");
        let reports = self.start.caught_up.get_mut(&connector).expect("configured connector reports");
        match stage {
            RestartStage::Restored => unreachable!("restoration is not a half of catching up"),
            RestartStage::ReadAfresh => reports.0 = true,
            RestartStage::Settled => reports.1 = true,
        }
    }

    /// Both halves have reported since this start's opening.
    #[must_use]
    pub fn caught_up(&self, connector: u16) -> bool {
        self.start.caught_up.get(&connector) == Some(&(true, true))
    }
}
