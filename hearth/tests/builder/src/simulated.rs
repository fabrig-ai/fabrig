//! The shipped client and hosted tools over the fake machine (hearth/docs/design/testing/README.md, 2.6).
mod client;
use crate::{
    fixtures::{self, Scenario, Shared},
    hosted::{self, Hosted},
};
pub use client::Client;
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_sim::Config;
use skein_world::{HostedProgram, Inherited, Memory, Outcome, World};
use std::cell::RefCell;

#[derive(Debug)]
pub enum Proc {
    Client(Box<Client>),
    Git(Box<Hosted>),
    Mmdebstrap(Box<Hosted>),
}
impl Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Client(host) => host.as_ref(),
            Self::Git(host) | Self::Mmdebstrap(host) => host.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Client(host) => host.as_mut(),
            Self::Git(host) | Self::Mmdebstrap(host) => host.as_mut(),
        }
    }
}
impl Host for Proc {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.host().state_digest()
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
#[derive(Clone, Debug)]
struct Context {
    scenario: Scenario<'static>,
    shared: Shared,
    seed: u64,
    limits: hearth::Limits,
}
thread_local! { static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) }; }
fn context() -> Context {
    CONTEXT.with(|context| context.borrow().as_ref().expect("configured world factories").clone())
}
#[must_use]
pub fn client(inherited: &Inherited) -> Proc {
    let context = context();
    Proc::Client(Box::new(Client::new(inherited, &context.scenario, context.seed, context.limits)))
}
#[must_use]
pub fn git(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    let context = context();
    Proc::Git(Box::new(hosted::git::Git::host(spawn, inherited, &context.scenario, context.shared, context.seed)))
}
#[must_use]
pub fn mmdebstrap(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    let context = context();
    Proc::Mmdebstrap(Box::new(hosted::mmdebstrap::Mmdebstrap::host(
        spawn,
        inherited,
        &context.scenario,
        context.shared,
        context.seed,
    )))
}

/// Build a restartable client over the scenario's machine and installed tools.
#[must_use]
pub fn world(scenario: &Scenario<'_>, seed: u64, config: Config) -> World<Proc, Referee, Shared> {
    build(scenario, seed, config, &Options::default())
}
#[derive(Clone, Copy, Debug)]
struct Options {
    release: Option<Time>,
    output_release: Option<Time>,
    memory: Memory,
    limits: hearth::Limits,
    instances: u32,
}
impl Default for Options {
    fn default() -> Self {
        Self { release: None, output_release: None, memory: Memory::Unchecked, limits: hearth::limits(), instances: 1 }
    }
}
fn build(scenario: &Scenario<'_>, seed: u64, config: Config, options: &Options) -> World<Proc, Referee, Shared> {
    let shared = Shared::for_scenario(scenario, seed, fixtures::WORLD_PIPE);
    if matches!(
        scenario.call,
        hearth::Call::Render { resolve: true, .. }
            | hearth::Call::Diff { resolve: true, .. }
            | hearth::Call::Build { .. }
    ) {
        fixtures::serve(&shared);
    }
    fixtures::lay_calls(&shared);
    if options.memory != Memory::Unchecked {
        fixtures::lay_cache_pages(&shared);
    }
    let lock = options.release.map(|at| (fixtures::HeldLock::new(&shared), at));
    if options.output_release.is_some() {
        shared.hold_output();
    }
    CONTEXT.with(|context| {
        *context.borrow_mut() = Some(Context {
            scenario: Scenario {
                files: &[],
                laid: &[],
                call: scenario.call.clone(),
                form: scenario.form,
                fault: scenario.fault,
                pace: scenario.pace,
            },
            shared: shared.clone(),
            seed,
            limits: options.limits,
        });
    });
    let mut world = World::new_controlled(seed, config, options.memory, |_| Referee {
        shared: shared.clone(),
        done: false,
        lock,
        output_release: options.output_release,
        seen: 0,
        finding: None,
    })
    .with_machine(shared);
    world.host_roots(
        HostedProgram {
            program: b"/usr/bin/git".as_slice().into(),
            make: git,
            instances: options.instances,
            operations: fixtures::HOST_OPERATIONS,
        },
        fixtures::tool_roots,
    );
    world.host_roots(
        HostedProgram {
            program: b"/usr/bin/mmdebstrap".as_slice().into(),
            make: mmdebstrap,
            instances: options.instances,
            operations: fixtures::HOST_OPERATIONS,
        },
        fixtures::tool_roots,
    );
    world.spawn_restartable_startup(fixtures::startup_roots(), fixtures::startup_appends(false), client);
    world
}
#[must_use]
pub fn run(scenario: &Scenario<'_>, seed: u64, config: Config) -> Outcome<Proc, Shared> {
    world(scenario, seed, config).run()
}
/// The call at the named limits, unmetered.
#[must_use]
pub fn with_limits(scenario: &Scenario<'_>, seed: u64, limits: hearth::Limits) -> Outcome<Proc, Shared> {
    build(scenario, seed, Config::calm(), &Options { limits, ..Options::default() }).run()
}
#[must_use]
pub fn two(scenario: &Scenario<'_>, seed: u64, config: Config) -> Outcome<Proc, Shared> {
    let mut world = world(scenario, seed, config);
    world.spawn_restartable_startup(fixtures::startup_roots(), fixtures::startup_appends(true), client);
    world.run()
}
#[must_use]
pub fn with_lock(scenario: &Scenario<'_>, seed: u64, config: Config, release: Time) -> Outcome<Proc, Shared> {
    build(scenario, seed, config, &Options { release: Some(release), ..Options::default() }).run()
}
#[derive(Debug)]
pub struct Referee {
    shared: Shared,
    done: bool,
    lock: Option<(fixtures::HeldLock, Time)>,
    output_release: Option<Time>,
    seen: usize,
    finding: Option<String>,
}
impl skein_world::Referee<Proc> for Referee {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_some() {
            return;
        }
        if self.lock.as_ref().is_some_and(|(_, at)| now >= *at)
            && let Some((lock, _)) = self.lock.take()
        {
            lock.release(&self.shared);
        }
        if self.output_release.is_some_and(|at| now >= at) {
            for proc in procs.iter().filter(|proc| matches!(proc, Proc::Client(_))) {
                if proc.work_pending(now) {
                    self.finding = Some("render has stopped behind the held output".to_owned());
                    return;
                }
            }
            self.output_release = None;
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(finding) = &self.finding {
            return Err(finding.clone());
        }
        let mmdebstraps: Vec<_> = procs.iter().filter(|proc| matches!(proc, Proc::Mmdebstrap(_))).collect();
        if mmdebstraps.iter().filter(|proc| !proc.is_empty()).count() > 1 {
            return Err("one mmdebstrap alive at a time".to_owned());
        }
        if mmdebstraps.len() > self.seen {
            if self.shared.contents(fixtures::OUTPUT).as_deref() != Some(b"".as_slice()) {
                return Err("resolve finishes before the client prints".to_owned());
            }
            if fixtures::simulation_records(&self.shared).len() != self.seen {
                return Err("earlier runs are recorded before the next spawn".to_owned());
            }
            self.seen = mmdebstraps.len();
        }
        self.done = procs.iter().filter(|proc| matches!(proc, Proc::Client(_))).all(Host::is_empty);
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if let Some((_, at)) = self.lock.as_ref() {
            Some(*at)
        } else if let Some(at) = self.output_release {
            Some(at)
        } else {
            (!self.done).then_some(fixtures::SIMULATION_DEADLINE)
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.done && now >= fixtures::SIMULATION_DEADLINE).then(|| "the simulated builder must settle".to_owned())
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.done
    }
}

/// Builder chaos preserves progress while varying transport and cancellation
/// races: M1 gives git no deadline (ruling 186).
#[must_use]
pub fn chaos() -> Config {
    use skein_sim::Faults;
    Config {
        faults: Faults {
            no_space: 0,
            read_only: 0,
            io_error: 0,
            hung: 0,
            no_buffer: 0,
            reset: 0,
            refuse: 0,
            timed_out: 0,
            ..Faults::CHAOS
        },
        ..Config::chaos()
    }
}
/// Meter the shipped client over the paged-store scenario at the named limits.
#[must_use]
pub fn checked(seed: u64, ceiling: u64) -> Outcome<Proc, Shared> {
    checked_with(&fixtures::thin(), seed, fixtures::small_limits(), ceiling)
}
/// Meter every iteration, including decoding and writing resolution records.
#[must_use]
pub fn checked_with(scenario: &Scenario<'_>, seed: u64, limits: hearth::Limits, ceiling: u64) -> Outcome<Proc, Shared> {
    build(scenario, seed, Config::calm(), &Options { memory: Memory::Within(ceiling), limits, ..Options::default() })
        .run()
}

/// Meter the client with its output held from its first write until `release`.
#[must_use]
pub fn checked_stuck(
    scenario: &Scenario<'_>,
    seed: u64,
    limits: hearth::Limits,
    ceiling: u64,
    release: Time,
) -> Outcome<Proc, Shared> {
    let world = build(
        scenario,
        seed,
        Config::calm(),
        &Options { output_release: Some(release), memory: Memory::Within(ceiling), limits, ..Options::default() },
    );
    let shared = context().shared;
    let mut release = Some(release);
    let mut answers = Queue::uncapped();
    world.run_with_inputs(move |sim, _, _| {
        if release.is_some_and(|at| sim.now() >= at) {
            assert_eq!(shared.contents(fixtures::OUTPUT).as_deref(), Some(b"".as_slice()), "no output before release");
            shared.release_output(&mut answers);
            sim.answer(&mut answers);
            release = None;
        }
    })
}

/// A killed client's children run on while its cold replacement starts
/// another instance at the same pace (ruling 187).
#[must_use]
pub fn for_cut(scenario: &Scenario<'_>, seed: u64) -> World<Proc, Referee, Shared> {
    build(scenario, seed, Config::calm(), &Options { instances: fixtures::CUT_INSTANCES, ..Options::default() })
}

#[cfg(test)]
mod tests {
    use super::{Client, Proc, Referee};
    use crate::fixtures;
    use skein_io::kernel::Fd;
    use skein_lib::Time;
    use skein_shell::Host;
    use skein_world::{Inherited, Referee as _};

    fn judge(shared: fixtures::Shared) -> Referee {
        Referee { shared, done: false, lock: None, output_release: None, seen: 0, finding: None }
    }

    #[test]
    fn a_held_output_finding_survives_act_until_watch_and_prevents_passing() {
        let scenario = fixtures::thin();
        let shared = fixtures::Shared::for_scenario(&scenario, fixtures::SIMULATION_SEED, fixtures::WORLD_PIPE);
        let mut referee = judge(shared);
        let inherited = Inherited {
            roots: [(b"store".as_slice(), 1), (b"cache", 2), (b"resolutions", 3), (b"staging", 4)]
                .into_iter()
                .map(|(name, fd)| (name.into(), Fd::new(fd)))
                .collect(),
            appends: vec![(b"output".as_slice().into(), Fd::new(5)), (b"error".as_slice().into(), Fd::new(6))],
            signal: Fd::new(7),
            pipes: Vec::new(),
        };
        let client = Client::new(&inherited, &scenario, fixtures::SIMULATION_SEED, hearth::limits());
        let mut procs = [Proc::Client(Box::new(client))];
        assert!(procs[0].work_pending(Time::ZERO));
        referee.output_release = Some(Time::ZERO);
        referee.done = true;
        referee.act(Time::ZERO, &mut procs);
        assert!(!referee.passed());
        referee.act(Time::ZERO, &mut []);
        assert_eq!(referee.watch(Time::ZERO, &[]), Err("render has stopped behind the held output".to_owned()));
        assert!(!referee.passed());
    }

    #[test]
    fn watch_returns_the_finding_when_the_client_prints_before_resolution() {
        let outcome = fixtures::simulated_resolved();
        let mut referee = judge(outcome.machine.clone());
        assert_eq!(
            referee.watch(Time::ZERO, &outcome.procs),
            Err("resolve finishes before the client prints".to_owned())
        );
        assert!(!referee.passed());
        assert_eq!(referee.seen, 0, "the failing spawn was not accepted");
    }
}
