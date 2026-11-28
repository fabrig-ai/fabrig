//! Engine faults over the world's store and fake forge
//! (`temper/docs/design/testing/restarts.md`, 3–4). Existing stories opt out.
use super::{World, engine, fake};
use jig_domain_core::start::Phase;
use skein_lib::{Duration, Rng};
use skein_world::{
    faults::{FaultLedger, Injection, Kind, Struck},
    streams::{Stream, Streams},
};
use temper_faults::restarts::{self, ENGINE_CRASH, SECOND_SIGNAL, SETTLED_STOP, STOP_AFTER_OPENING, STOP_BEFORE_MARK};

/// One drawn fault and iteration. Prerequisite crashes and stops are faults too.
#[derive(Debug)]
pub struct Injector {
    random: Rng,
    kind: Kind,
    iteration: u64,
    ticks: u64,
    fired: bool,
    recovery_clock: bool,
    second_after: u64,
    opening: Vec<Injection>,
    pub ledger: FaultLedger,
}
impl Injector {
    #[must_use]
    pub fn new(seed: u64, configured: &[Kind], horizon: u64) -> Self {
        assert!(!configured.is_empty() && horizon > 0);
        assert!(configured.iter().all(|kind| restarts::ENGINE.contains(kind)));
        let mut random = Streams::new(seed).rng(Stream::Faults, restarts::PART);
        let index =
            usize::try_from(random.below(u64::try_from(configured.len()).expect("kind count"))).expect("kind index");
        let kind = configured[index];
        let iteration = random.below(horizon);
        let second_after = random.below(4);
        Self {
            random,
            kind,
            iteration,
            ticks: 0,
            fired: false,
            recovery_clock: false,
            second_after,
            opening: Vec::new(),
            ledger: FaultLedger::new(),
        }
    }

    /// Follows' synchronous helpers can wait out a durable restart hold without
    /// increasing their tick bound. Other workloads keep their own clock.
    #[must_use]
    pub fn with_recovery_clock(mut self) -> Self {
        self.recovery_clock = true;
        self
    }

    /// Named cuts use the same injection machinery as the seeded sweep.
    #[must_use]
    pub fn at(seed: u64, kind: Kind, iteration: u64) -> Self {
        let mut injector = Self::new(seed, &[kind], 1);
        injector.iteration = iteration;
        // The named cut supplies the plan; its first draw is a persistence outcome.
        injector.random = Streams::new(seed).rng(Stream::Faults, restarts::PART);
        injector
    }

    #[must_use]
    pub fn drawn(&self) -> (Kind, u64) {
        (self.kind, self.iteration)
    }

    pub(super) fn tick(&mut self, world: &mut World) {
        if !self.fired && self.ticks >= self.iteration {
            self.fall(world);
        }
        self.ticks = self.ticks.checked_add(1).expect("bounded run");
    }

    /// A short workload still injects its draw at its last iteration.
    pub fn last(&mut self, world: &mut World) {
        if !self.fired {
            self.fall(world);
        }
    }

    fn fall(&mut self, world: &mut World) {
        // Only an injection can strike an armed deadline. Snapshot just before
        // it changes the world, including a draw deferred until the last tick.
        let armed = world
            .workload_deadlines
            .as_ref()
            .map(|_| crate::outcome::Deadlines::armed(&world.root, &world.store, world.env.now));
        let before = self.ledger.entries().len();
        self.fired = true;
        eprintln!("inject {:?} at tick {} ({:?})", self.kind, self.ticks, world.env.now);
        self.inject(world);
        if self.recovery_clock {
            world.finish_restart_hold();
        }
        if let Some(armed) = armed {
            world
                .workload_deadlines
                .as_mut()
                .expect("workload observer")
                .struck(&armed, &self.ledger.entries()[before..]);
        }
    }

    fn record(&mut self, world: &World, kind: Kind) -> Injection {
        self.ledger.inject(kind, world.env.now, Struck::default())
    }

    /// Drop the engine now. Its interval stays open if this run ends before
    /// the next opening; focused cuts can inspect the store before opening it.
    pub fn crash(&mut self, world: &mut World) {
        let entry = self.record(world, ENGINE_CRASH);
        self.opening.push(entry);
        world.crash_engine(&mut self.random);
    }

    /// Observe the next start's opening for all faults it ended.
    pub fn opened(&mut self, world: &World) {
        assert!(world.root.ready());
        for entry in self.opening.drain(..) {
            self.ledger.stop(entry, world.env.now).expect("next opening");
        }
    }

    /// Stop a start already restarted by a focused scenario, without adding a
    /// prerequisite fault to that scenario's ledger. Return the phase at the
    /// signal, commits at the drain, and whether the forge was still catching up.
    pub fn stop_in_class(&mut self, world: &mut World) -> (Option<Phase>, u64, bool) {
        world.hold_start(self.kind);
        if self.kind == STOP_AFTER_OPENING {
            world.restart_clock();
        }
        let phase = world.root.start_phase();
        let catching_up = !world.root.caught_up(0);
        let entry = self.record(world, self.kind);
        self.opening.push(entry);
        world.stop_engine();
        let committed = world.store.applied;
        world.restart_same();
        world.open_after_fault();
        self.opened(world);
        (phase, committed, catching_up)
    }

    fn inject(&mut self, world: &mut World) {
        match self.kind {
            ENGINE_CRASH => {
                self.crash(world);
                world.open_after_fault();
                self.opened(world);
            }
            SETTLED_STOP => {
                let entry = self.record(world, SETTLED_STOP);
                self.opening.push(entry);
                world.stop_engine();
                world.restart_same();
                world.open_after_fault();
                self.opened(world);
            }
            STOP_BEFORE_MARK | STOP_AFTER_OPENING => {
                self.crash(world);
                world.hold_start(self.kind);
                if self.kind == STOP_AFTER_OPENING {
                    self.opened(world);
                }
                let _stopped = self.stop_in_class(world);
            }
            SECOND_SIGNAL => {
                let stop = self.record(world, SETTLED_STOP);
                self.opening.push(stop);
                world.send(engine::Event::Stop);
                // A signal may arrive on the same loop pass, before a commit's
                // answer. Draw among the still-settling passes, never after Drained.
                for _ in 0..self.second_after {
                    if world.store.pending.is_empty() {
                        break;
                    }
                    world.restart_clock();
                    engine::release(&mut world.root, &world.env, &mut world.out);
                    world.collect();
                }
                assert!(world.drained.is_none(), "second signal precedes Drained");
                let second = self.record(world, SECOND_SIGNAL);
                self.ledger.stop(second, world.env.now).expect("instant");
                world.crash_engine(&mut self.random);
                world.open_after_fault();
                self.opened(world);
            }
            _ => unreachable!("configured engine kind"),
        }
    }
}

impl World {
    /// Enable deferred forge submissions before the first injected iteration.
    pub fn inject_restarts(&mut self, injector: Injector) {
        assert!(self.restarts.is_none());
        assert!(self.pending.is_empty(), "enable before the run's first forge call");
        self.restart_calls = Some(Vec::new());
        self.restarts = Some(injector);
    }

    /// Complete the final draw and return this run's ledger.
    pub fn finish_restarts(&mut self) -> FaultLedger {
        let mut injector = self.restarts.take().expect("enabled restart injector");
        injector.last(self);
        injector.ledger
    }

    /// A same-process hello proves only listed claims. An unlisted durable
    /// claim waits out the stored restart hold (jig's domain/engine/fleet.md).
    /// With no runnable input, jump to the next existing alarm, not a new bound.
    pub fn advance_to_recovery_deadline(&mut self) {
        if self.root.ready()
            && !self.root.quiescent()
            && !self.root.forge_work_ready()
            && self.root.backlog() == 0
            && self.events.is_empty()
            && self.immediate.is_empty()
            && self.out.is_empty()
            && self.fake_out.is_empty()
            && self.pending.is_empty()
            && self.store.idle()
            && self.restart_calls.as_ref().is_none_or(Vec::is_empty)
            && let Some(due) = self.root.next_deadline(&self.env)
        {
            self.env.now = self.env.now.max(due);
            self.sync_clock();
        }
    }

    fn finish_restart_hold(&mut self) {
        let wait = Duration::from_millis(self.store.header().restart_wait_ms.unwrap_or(0));
        let span = wait.max(self.env.limits.fleet.restart_wait).saturating_add(self.env.limits.fleet.grace);
        let passes = span.as_nanos() / self.tick_interval.as_nanos();
        for _ in 0..passes.saturating_add(200) {
            if self.root.quiescent() && self.events.is_empty() && self.store.idle() {
                return;
            }
            self.advance_to_recovery_deadline();
            self.tick();
        }
        panic!("restart hold did not settle within its configured wait and grace");
    }

    pub(super) fn restart_clock(&mut self) {
        self.env.now = self.env.now.saturating_add(self.tick_interval);
        self.sync_clock();
    }

    /// Submissions wait one world iteration at this boundary. The fake is called
    /// only when their outcome is chosen, with no second reply-latency queue.
    pub(super) fn dispatch_restart_calls(&mut self, mut random: Option<&mut Rng>) {
        let Some(calls) = &mut self.restart_calls else { return };
        for (event, mut limits) in std::mem::take(calls) {
            if random.as_mut().is_some_and(|random| random.below(2) == 0) {
                continue;
            }
            limits.latency_min = Duration::ZERO;
            limits.latency_max = Duration::ZERO;
            let env = self.fake_env.with(limits);
            fake::step(&mut self.fake, &env, event, &mut self.fake_out);
            while self.fake.is_due(self.env.now) {
                fake::fire(&mut self.fake, &env, &mut self.fake_out);
            }
        }
    }

    /// Drop only the engine: keep the store's ordered durable prefix, the fake
    /// forge and the worker. No drain, no old terminal enters the replacement.
    pub fn crash_engine(&mut self, random: &mut Rng) {
        let mut kept = true;
        while !self.store.pending.is_empty() {
            // A later ordered transaction cannot be durable past a lost one.
            kept &= random.below(2) != 0;
            if kept {
                self.store.apply();
            } else {
                self.store.pending.pop_front();
            }
        }
        self.dispatch_restart_calls(Some(random));
        self.held_assignments = None;
        self.held_reads = None;
        self.held_comparisons = None;
        self.held_merge_calls = None;
        self.busy_reads = false;
        self.crash_restart(random.next_u64());
    }

    pub fn stop_engine(&mut self) {
        self.send(engine::Event::Stop);
        for _ in 0..200 {
            assert_eq!(self.root.start_stop(), None, "stop did not refuse its start");
            if self.drained.is_some() {
                return;
            }
            self.tick();
        }
        panic!("stop did not drain: {:?}", self.root.start_phase());
    }

    pub fn open_after_fault(&mut self) {
        // Use the opening referee's step bound for the paged restore and sweep.
        for _ in 0..crate::ends::OPENING.steps {
            assert_eq!(self.root.start_stop(), None, "restart refused");
            if self.root.ready() {
                return;
            }
            self.tick();
        }
        // Judge the opening after the last tick's release pass. The service
        // routes held start outputs at once; they need no extra store or clock tick.
        while !self.root.start_runtime()
            && let Some(index) = self.events.iter().position(|event| !crate::waits_for_start(&self.root, event))
            && matches!(self.events[index], engine::Event::Released(_))
        {
            let event = self.events.remove(index).expect("eligible start release");
            self.send(event);
        }
        if self.root.ready() {
            if self.ends.awaiting_open() {
                self.ends.opened(self.env.now);
            }
            return;
        }
        panic!(
            "restart did not open: {:?}; events={:?}; pending={:?}; deferred={}; store={:?}",
            self.root.start_phase(),
            self.events,
            self.pending,
            self.restart_calls.as_ref().map_or(0, Vec::len),
            self.store.pending
        );
    }

    /// One event at a time holds the class before the next continuation. No
    /// product-side phase hooks, held store answers or synthetic rows are needed.
    pub fn hold_start(&mut self, kind: Kind) {
        // Single-event passes replace a tick's many releases and commits.
        for _ in 0..2_000 {
            let phase = self.root.start_phase();
            let held = match kind {
                STOP_BEFORE_MARK => matches!(
                    phase,
                    Some(
                        Phase::LoadCore
                            | Phase::LoadInbox
                            | Phase::RestoreConnector { .. }
                            | Phase::Link
                            | Phase::Sweep
                            | Phase::Switch
                            | Phase::Wires
                    )
                ),
                STOP_AFTER_OPENING => self.root.ready(),
                _ => panic!("not a start class"),
            };
            if held {
                return;
            }
            assert!(!self.root.ready(), "missed start class {kind:?}");
            assert_eq!(self.root.start_stop(), None);
            self.start_iteration();
        }
        panic!("start never reached {kind:?}: {:?}", self.root.start_phase());
    }

    fn start_iteration(&mut self) {
        if let Some(index) = self.events.iter().position(|event| !crate::waits_for_start(&self.root, event)) {
            let event = self.events.remove(index).expect("eligible input");
            self.send(event);
        } else if !self.store.pending.is_empty() {
            let number = self.store.apply();
            self.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        } else {
            self.tick();
        }
    }
}
