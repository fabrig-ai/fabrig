//! Unit signals and previous-release starts (temper/docs/design/testing/restarts.md, 3 and 8).
use crate::{fixtures, golden, referee, world};
use skein_io::kernel::{Complete, Done, Exit, ServiceSignal, Signalled};
use skein_lib::{Rng, Time};
use skein_shell::Host;
use skein_world::{
    Controls,
    faults::{FaultLedger, Injection, Struck},
    streams::{Stream, Streams},
};
use std::{cell::RefCell, path::Path, rc::Rc};
use temper_faults::restarts::{OLDER_LAYOUT, PART, UNIT_SIGNALLED};
use temper_playground::programs::{Harness, Process, Role};

/// A small horizon of iterations while the story holds its turn (or serves the kept person).
const HORIZON: u64 = 8;

#[derive(Debug)]
pub struct Signal {
    pub at: Time,
    pub iteration: u64,
    /// Every live member signalled in the same iteration, including the service.
    pub members: Vec<&'static str>,
    /// Signal completions read by the live service and agents, not their later channel stops.
    pub received: Vec<&'static str>,
}

#[derive(Debug)]
pub struct Outcome {
    pub ledger: FaultLedger,
    pub openings: Vec<Time>,
    pub signal: Option<Signal>,
    pub fresh: Option<world::Outcome>,
}

/// One run's independent injector. The story and simulator keep their original seeds.
pub(crate) struct Injector {
    rng: Rng,
    moment: Option<u64>,
    iteration: u64,
    armed: bool,
    signalled: bool,
    controls: Option<Controls>,
    offset: Time,
    opened: bool,
    ledger: FaultLedger,
    older_entry: Option<Injection>,
    unit_entry: Option<Injection>,
    openings: Vec<Time>,
    signal: Option<Signal>,
}

impl Injector {
    fn new(seed: u64) -> Self {
        Self {
            rng: Streams::new(seed).rng(Stream::Faults, PART),
            moment: None,
            iteration: 0,
            armed: false,
            signalled: false,
            controls: None,
            offset: Time::ZERO,
            opened: false,
            ledger: FaultLedger::new(),
            older_entry: None,
            unit_entry: None,
            openings: Vec::new(),
            signal: None,
        }
    }

    pub(crate) fn wants_signal(&self) -> bool {
        self.moment.is_some()
    }

    pub(crate) fn unit_active(&self) -> bool {
        self.unit_entry.is_some()
    }

    pub(crate) fn attach(&mut self, controls: Controls) {
        self.controls = Some(controls);
        self.opened = false;
    }

    pub(crate) fn opening(&mut self, now: Time) {
        if self.opened {
            return;
        }
        self.opened = true;
        let at = self.at(now);
        self.openings.push(at);
        for entry in [self.older_entry.take(), self.unit_entry.take()].into_iter().flatten() {
            self.ledger.stop(entry, at).expect("fault stops at the opening");
        }
    }

    pub(crate) fn end_phase(&mut self, now: Time) {
        self.offset = self.at(now);
    }

    fn at(&self, now: Time) -> Time {
        Time::from_nanos(self.offset.as_nanos().checked_add(now.as_nanos()).expect("bounded simulated run"))
    }

    /// Count only the horizon the story holds open. No workload draw is consumed.
    pub(crate) fn ready(&mut self) -> bool {
        let Some(moment) = self.moment else { return true };
        if self.signalled {
            return true;
        }
        if self.iteration == moment {
            self.armed = true;
        } else {
            self.iteration += 1;
        }
        false
    }

    pub(crate) fn waiting(&self) -> bool {
        self.moment.is_some() && !self.signalled && (self.iteration > 0 || self.armed)
    }

    /// Apply one batch before any process turns. Trapped signals go to each host's
    /// own source; ordinary shell/check/git programs take SIGTERM's default exit.
    pub(crate) fn inputs<H: Harness<Process = world::HarnessProcess>>(
        &mut self,
        sim: &mut skein_sim::Sim,
        pids: &[skein_sim::Pid],
        procs: &[Process<H>],
    ) {
        if !self.armed {
            return;
        }
        self.armed = false;
        self.signalled = true;
        let at = self.at(sim.now());
        let mut members = Vec::new();
        for (index, (pid, proc)) in pids.iter().zip(procs).enumerate() {
            if proc.is_empty() {
                continue;
            }
            let (name, trapped) = match proc {
                Process::Service(_) => ("service", true),
                Process::Agent(_) => ("agent", true),
                Process::List(_) => ("agent-list", true),
                Process::Git(_) | Process::GitRoute(_) => ("git", false),
                Process::Shell(_) => ("shell/check", false),
                Process::Other(world::HarnessProcess::ProgramSource(_)) => ("program", true),
                Process::Peer(_)
                | Process::Forge(_)
                | Process::Visitor(_)
                | Process::Issuer(_)
                | Process::Other(
                    world::HarnessProcess::Client(_)
                    | world::HarnessProcess::Ctl(_)
                    | world::HarnessProcess::Calls(_)
                    | world::HarnessProcess::Push(_),
                ) => continue,
            };
            members.push(name);
            if trapped {
                self.controls.as_ref().expect("attached injector").signal(index, ServiceSignal::Terminate);
            } else {
                sim.finish_service(*pid, Exit::Signal(15));
            }
        }
        self.unit_entry = Some(self.ledger.inject(UNIT_SIGNALLED, at, Struck::default()));
        self.signal = Some(Signal { at, iteration: self.iteration, members, received: Vec::new() });
    }

    fn outcome(&mut self, fresh: Option<world::Outcome>) -> Outcome {
        Outcome {
            ledger: std::mem::take(&mut self.ledger),
            openings: std::mem::take(&mut self.openings),
            signal: self.signal.take(),
            fresh,
        }
    }
}

thread_local! {
    static OBSERVER: RefCell<Option<Rc<RefCell<Injector>>>> = const { RefCell::new(None) };
}

fn observing<R>(injector: &Rc<RefCell<Injector>>, run: impl FnOnce() -> R) -> R {
    struct Reset(Option<Rc<RefCell<Injector>>>);
    impl Drop for Reset {
        fn drop(&mut self) {
            OBSERVER.with(|slot| *slot.borrow_mut() = self.0.take());
        }
    }
    let _reset = Reset(OBSERVER.with(|slot| slot.replace(Some(injector.clone()))));
    run()
}

pub(crate) fn observe(role: Role, complete: &Complete) {
    if !matches!(complete.result, Ok(Done::Signalled(Signalled::Termination { signal: ServiceSignal::Terminate, .. })))
    {
        return;
    }
    let name = match role {
        Role::Service => "service",
        Role::Agent => "agent",
        Role::Git | Role::Shell | Role::Provider | Role::Forge | Role::Visitor | Role::Other => return,
    };
    OBSERVER.with(|slot| {
        if let Some(injector) = slot.borrow().as_ref() {
            let mut injector = injector.borrow_mut();
            if injector.unit_active() {
                injector.signal.as_mut().expect("injected signal").received.push(name);
            }
        }
    });
}

/// Signal a chat's held turn, drain the whole unit, and reopen the same disk.
#[must_use]
pub fn unit_signalled(settings: world::Settings, iteration: u64) -> Outcome {
    assert!(iteration < HORIZON, "moment within the held-turn horizon");
    let mut injector = Injector::new(settings.seed);
    injector.moment = Some(iteration);
    fresh(settings, &Rc::new(RefCell::new(injector)))
}

fn fresh(settings: world::Settings, injector: &Rc<RefCell<Injector>>) -> Outcome {
    let run = Rc::new(RefCell::new(referee::Run::default()));
    run.borrow_mut().restart_faults = Some(injector.clone());
    let outcome = observing(injector, || {
        world::drain_run(settings, world::Story::ChatReadThenWait { words: b"Hello" }, true, &run)
    });
    injector.borrow_mut().outcome(Some(outcome))
}

/// Start this build over dev's immutable image and serve the kept person's history.
#[must_use]
pub fn older_layout(settings: world::Settings, directory: &Path) -> Outcome {
    let injector = Injector::new(settings.seed);
    image(settings, directory, &Rc::new(RefCell::new(injector)))
}

fn image(settings: world::Settings, directory: &Path, injector: &Rc<RefCell<Injector>>) -> Outcome {
    {
        let mut state = injector.borrow_mut();
        state.older_entry = Some(state.ledger.inject(OLDER_LAYOUT, Time::ZERO, Struck::default()));
    }
    observing(injector, || golden::dev::restart_faults(directory, settings, injector));
    injector.borrow_mut().outcome(None)
}

/// Draw the starting image and the signal's iteration, independently of the world's seed stream.
#[must_use]
pub fn drawn(seed: u64, directory: &Path) -> Outcome {
    let mut injector = Injector::new(seed);
    let older = injector.rng.below(2) == 0;
    injector.moment = Some(injector.rng.below(HORIZON));
    let injector = Rc::new(RefCell::new(injector));
    let settings = fixtures::calm(seed);
    if older { image(settings, directory, &injector) } else { fresh(settings, &injector) }
}
