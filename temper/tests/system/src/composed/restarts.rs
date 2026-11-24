//! Engine restarts during a worker's turn (temper/docs/design/testing/restarts.md, 3 and 7).
use skein_lib::{Rng, Time};
use skein_world::{
    faults::{FaultLedger, Injection, Kind, Struck},
    streams::{Stream, Streams},
};
use temper_faults::restarts;

use super::{Script, World};

#[derive(Clone, Copy, Debug)]
pub enum Point {
    Model(usize),
    HostCall(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum Restart {
    Settled,
    Crash,
}

impl Restart {
    #[must_use]
    pub const fn kind(self) -> Kind {
        match self {
            Self::Settled => restarts::ENGINE_RESTART_MID_TURN,
            Self::Crash => restarts::ENGINE_CRASH_MID_TURN,
        }
    }
}

/// A separate stream leaves the world's workload and scheduling draws untouched.
pub struct Injector {
    rng: Rng,
}

impl Injector {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { rng: Streams::new(seed).rng(Stream::Faults, restarts::PART) }
    }

    /// Select any query or host call from the story's script, then the stop's kind.
    pub fn draw(&mut self, queries: usize, host_calls: usize) -> (Point, Restart) {
        assert!(queries > 0, "a run has a model query");
        let count = queries.checked_add(host_calls).expect("script points");
        let position = self.rng.below(u64::try_from(count).expect("script point count"));
        let position = usize::try_from(position).expect("point within the script");
        let point =
            if position < queries { Point::Model(position + 1) } else { Point::HostCall(position - queries + 1) };
        let restart = if self.rng.chance(500) { Restart::Settled } else { Restart::Crash };
        (point, restart)
    }
}

impl World {
    /// Hold either a model response or a host call before the worker receives it.
    #[must_use]
    pub fn mid_turn_chat(seed: u64, script: Script, words: &[u8], point: Point) -> Self {
        match point {
            Point::Model(query) => Self::absence_chat(seed, script, words, query),
            Point::HostCall(call) => {
                assert!(call > 0, "host calls are numbered from one");
                let mut world = Self::absence_chat(seed, script, words, usize::MAX);
                world.release_model();
                world.process.held_host_call = Some(call);
                world
            }
        }
    }

    pub fn until_mid_turn(&mut self, point: Point) {
        match point {
            Point::Model(_) => self.until_model_held(),
            Point::HostCall(_) => {
                for _ in 0..10000 {
                    self.tick();
                    if self.process.host_call_held()
                        && self.store.idle()
                        && self.root_events.is_empty()
                        && self.worker_events.is_empty()
                    {
                        return;
                    }
                }
                panic!("host call not held: {:?}", self.observed);
            }
        }
    }

    pub fn release_mid_turn(&mut self, point: Point) {
        match point {
            Point::Model(_) => self.release_model(),
            Point::HostCall(_) => {
                assert!(self.process.host_call_held(), "the selected host call is held");
                self.process.held_host_call = None;
            }
        }
    }

    /// Continue past the injected point, holding a later model response for observation.
    pub fn model_after_mid_turn(&mut self, point: Point, query: usize) {
        self.release_mid_turn(point);
        self.process.held_query = Some(query);
        self.until_model_held();
    }

    #[must_use]
    pub fn restart_now(&self) -> Time {
        self.env.now
    }

    /// Stop the engine, not the worker. A settled stop says its word before the cut.
    pub fn inject_mid_turn(&mut self, restart: Restart, ledger: &mut FaultLedger) -> Injection {
        if let Restart::Settled = restart {
            self.settle_engine();
        }
        self.cut_link();
        self.stop_engine();
        ledger.inject(restart.kind(), self.env.now, Struck::default())
    }

    /// The interval ends at decisions opening, before the worker's link rejoins.
    pub fn open_after_mid_turn(&mut self, ledger: &mut FaultLedger, injection: Injection) {
        self.reopen_engine();
        for _ in 0..10000 {
            self.tick();
            if self.root.ready() {
                ledger.stop(injection, self.env.now).expect("one restart interval");
                return;
            }
        }
        panic!("the restarted engine did not open");
    }
}
