//! Admission order and cues on one checked real world.
pub(crate) mod diagnostics;
mod lifecycle;
use crate::{agent, calls::Caller, fixtures, forge, llm, service::ServiceHost};
use lifecycle::{Hosts, Lifecycle};
use skein_fake_peers::llm::Observation::{Answered, Query};
use skein_io::kernel::{Complete, Exit, ServiceSignal, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{
    HostedProgram, Referee,
    real::{CheckedOutcome, Controls, World},
};

pub struct Proc {
    pub(crate) kind: Kind,
    diagnostics: diagnostics::Operations,
}
impl Proc {
    pub(crate) fn new(kind: Kind) -> Self {
        Self { kind, diagnostics: diagnostics::Operations::default() }
    }
    pub(crate) fn observed_engine(&self) -> Option<(u64, &jig_domain_tasks::Domain, &diagnostics::Operations)> {
        let Kind::Service(host) = &self.kind else { return None };
        let engine = host.service.as_ref()?.engine_domain()?;
        Some((engine.revision(), engine.tasks(), &self.diagnostics))
    }
    pub(crate) fn code_exchanges(&self) -> usize {
        let Kind::Issuer(peer) = &self.kind else { panic!("issuer process") };
        peer.observations()
            .iter()
            .filter(|event| matches!(event, skein_fake_peers::oauth::Observation::Post { .. }))
            .count()
    }
}
impl Host for Proc {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if let Kind::Service(host) = &mut self.kind
            && let Some(service) = &mut host.service
            && service.engine_domain().is_some()
        {
            self.diagnostics.terminals.opened_logs(service.log_descriptors());
            self.diagnostics.complete(host.completions(), now, true);
        } else {
            self.diagnostics.complete(self.kind.completions(), now, false);
        }
        self.kind.iterate(now, wall);
    }
    fn drain(&mut self) {
        self.kind.drain();
        self.diagnostics.submit(self.kind.submissions());
        self.diagnostics.timers(self.kind.next_deadline(), self.kind.next_policy_deadline());
        if let Kind::Caller(caller) = &self.kind {
            self.diagnostics.step(caller.position());
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.kind.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.kind.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.kind.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.kind.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.kind.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.kind.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.kind.exit()
    }
    fn operations(&self) -> u32 {
        self.kind.operations()
    }
}

pub(crate) enum Kind {
    Llm(Box<Llm>),
    Forge(Box<ForgePeer>),
    Issuer(Box<skein_fake_peers::oauth::Peer>),
    Service(Box<ServiceHost>),
    Caller(Box<Caller>),
    Ctl(Box<crate::calls::Recorded>),
    Terminal(Box<temper_client_shell_terminal::terminal::Terminal>),
    Agent(Box<agent::Hosted>),
    Git(Box<temper_fake_git_hosted::Git<temper_fake_git_hosted::Typed>>),
}
impl Kind {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Llm(h) => &h.peer,
            Self::Forge(h) => &h.peer,
            Self::Issuer(h) => h.as_ref(),
            Self::Service(h) => h.as_ref(),
            Self::Caller(h) => h.as_ref(),
            Self::Ctl(h) => h.as_ref(),
            Self::Terminal(h) => h.as_ref(),
            Self::Agent(h) => h.as_ref(),
            Self::Git(h) => h.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Llm(h) => &mut h.peer,
            Self::Forge(h) => &mut h.peer,
            Self::Issuer(h) => h.as_mut(),
            Self::Service(h) => h.as_mut(),
            Self::Caller(h) => h.as_mut(),
            Self::Ctl(h) => h.as_mut(),
            Self::Terminal(h) => h.as_mut(),
            Self::Agent(h) => h.as_mut(),
            Self::Git(h) => h.as_mut(),
        }
    }
}
impl Host for Kind {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if !matches!(self, Self::Llm(llm) if llm.deferred()) {
            if let Self::Llm(llm) = self
                && std::mem::take(&mut llm.cue)
            {
                llm.peer.release(crate::cluster::PLACED).expect("placement hold");
            }
            if let Self::Llm(llm) = self
                && let Some(name) = llm.release.take()
            {
                llm.peer
                    .release(name)
                    .unwrap_or_else(|error| panic!("release LLM hold {}: {error:?}", String::from_utf8_lossy(name)));
                llm.released = Some(name);
            }
            let pending = self.host().work_pending(now);
            match self {
                Self::Llm(peer) => peer.pending_streak.observe(pending),
                Self::Service(service) => service.pending_streak.observe(pending),
                Self::Forge(peer) => peer.pending_streak.observe(pending),
                Self::Issuer(_)
                | Self::Caller(_)
                | Self::Ctl(_)
                | Self::Terminal(_)
                | Self::Agent(_)
                | Self::Git(_) => {}
            }
            let host = self.host_mut();
            if pending
                || !host.completions().is_empty()
                || host.next_deadline().is_some_and(|deadline| deadline <= now)
                || host.next_policy_deadline().is_some_and(|deadline| deadline <= now)
            {
                host.iterate(now, wall);
            }
        }
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        !matches!(self, Self::Llm(llm) if llm.deferred())
            && (matches!(self, Self::Llm(llm) if llm.release.is_some()) || self.host().work_pending(now))
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
// Register the world's own destructor-bearing thread-locals before any
// admission, without replacing settings or a transport a surrounding run kept.
fn warm_runtime() {
    fixtures::warm_runtime();
    crate::git::warm_runtime();
}

pub(crate) fn run(
    scratch: &skein_scratch::Scratch,
    recipe: impl FnOnce() -> llm::Recipe,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
    direct_key: bool,
    sign_in: bool,
) -> CheckedOutcome<Proc> {
    run_with_agent(scratch, recipe, steps, direct_key, sign_in, agent::agent)
}

pub(crate) fn run_stubborn(
    scratch: &skein_scratch::Scratch,
    recipe: impl FnOnce() -> llm::Recipe,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
) -> CheckedOutcome<Proc> {
    run_with_agent(scratch, recipe, steps, false, false, agent::stubborn)
}

fn run_with_agent(
    scratch: &skein_scratch::Scratch,
    recipe: impl FnOnce() -> llm::Recipe,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
    direct_key: bool,
    sign_in: bool,
    agent: agent::Factory,
) -> CheckedOutcome<Proc> {
    warm_runtime();
    let patience = crate::PATIENCE;
    // As the browser tier does, reserve a kernel port before the immutable
    // registration is made. Only this story pays for an issuer.
    let reservation =
        sign_in.then(|| std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("issuer port"));
    let issuer = reservation.as_ref().map(|listener| listener.local_addr().expect("issuer address"));
    let mut world = World::new_controlled_checked(|controls| Judge {
        checks: diagnostics::Judge::new(crate::patience::Patience::new(patience)),
        controls,
        workers: Vec::new(),
        lifecycle: Lifecycle::default(),
        failure: None,
        armed: false,
        wake: None,
        forge: None,
        github: None,
        llm_history: LlmHistory::default(),
        children: Children::default(),
    });
    world.spawn(|| {
        Proc::new(Kind::Llm(Box::new(Llm {
            pending_streak: crate::patience::PendingStreak::default(),
            peer: llm::peer(recipe()).expect("independent LLM peer"),
            scripts: None,
            rules: None,
            cue: false,
            placement_hold: true,
            holds: Vec::new(),
            release: None,
            released: None,
        })))
    });
    world.spawn_signals(|signals| {
        Proc::new(Kind::Service(Box::new(match issuer {
            Some(address) => ServiceHost::with_issuer(scratch.path().into(), signals, address),
            None => ServiceHost::new(scratch.path().into(), signals, direct_key),
        })))
    });
    host(&mut world, scratch.path(), steps, direct_key, fixtures::local_places(scratch.path()), agent);
    if let Some(listener) = reservation {
        use std::os::fd::{AsRawFd, IntoRawFd};
        let address = listener.local_addr().expect("issuer address");
        let fd = skein_io::kernel::Fd::new(listener.as_raw_fd());
        world.spawn_with_fds(vec![fd], || {
            let mut peer = crate::issuer::peer(address, &crate::issuer::registration(address));
            peer.adopt_listener(fd, address).expect("held issuer listener");
            let _ = listener.into_raw_fd();
            Proc::new(Kind::Issuer(Box::new(peer)))
        });
    }
    world.run(&skein_shell::Clock::new(), patience)
}
type Rules = fn() -> Box<[Box<[skein_fake_llm_domain::api::Rule]>]>;
type Scripts = fn(std::net::SocketAddr) -> Box<[skein_fake_llm_domain::api::Script]>;
pub(crate) struct ForgePeer {
    pub(crate) peer: skein_fake_peers::forge::Peer,
    pub(crate) pending_streak: crate::patience::PendingStreak,
}
impl ForgePeer {
    fn new(peer: skein_fake_peers::forge::Peer) -> Self {
        Self { peer, pending_streak: crate::patience::PendingStreak::default() }
    }
}
impl std::ops::Deref for ForgePeer {
    type Target = skein_fake_peers::forge::Peer;
    fn deref(&self) -> &Self::Target {
        &self.peer
    }
}
impl std::ops::DerefMut for ForgePeer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.peer
    }
}
pub(crate) struct Llm {
    pub(crate) pending_streak: crate::patience::PendingStreak,
    pub(crate) peer: skein_fake_peers::llm::Peer,
    scripts: Option<Scripts>,
    rules: Option<Rules>,
    cue: bool,
    placement_hold: bool,
    holds: Vec<Box<[u8]>>,
    pub(crate) release: Option<&'static [u8]>,
    pub(crate) released: Option<&'static [u8]>,
}
impl Llm {
    fn deferred(&self) -> bool {
        self.scripts.is_some()
    }
    fn release_hold(&mut self, caller: &Caller) {
        if let Some(name) = caller.releasing_hold() {
            if self.released == Some(name) {
                self.released = None;
                caller.hold_released.set(true);
            } else if !caller.hold_released.get() {
                self.release = Some(name);
            }
        }
    }
}
impl Llm {
    /// Plays the deferred scripts, by their turn rules where given, once the forge is bound.
    fn bind(&mut self, address: std::net::SocketAddr) -> Result<(), String> {
        let Some(scripts) = self.scripts.take() else { return Ok(()) };
        let recipe = match self.rules.take() {
            Some(rules) => llm::played_by_input(scripts(address), rules()),
            None => llm::scripted(scripts(address)),
        };
        self.holds = script_holds(&recipe);
        self.peer = llm::peer(recipe).map_err(|error| format!("bound forge scripts: {error:?}"))?;
        Ok(())
    }
}
impl std::ops::Deref for Llm {
    type Target = skein_fake_peers::llm::Peer;
    fn deref(&self) -> &Self::Target {
        &self.peer
    }
}
impl std::ops::DerefMut for Llm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.peer
    }
}
pub(crate) fn run_forge(
    scratch: &skein_scratch::Scratch,
    fixture: forge::Fixture,
    scripts: Scripts,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
) -> skein_world::real::Outcome<Proc> {
    warm_runtime();
    let patience = crate::PATIENCE;
    let state = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut world = World::new_controlled(|controls| Judge {
        checks: diagnostics::Judge::new(crate::patience::Patience::new(patience)),
        controls,
        workers: Vec::new(),
        lifecycle: Lifecycle::default(),
        failure: None,
        armed: false,
        wake: None,
        forge: Some(state.clone()),
        github: None,
        llm_history: LlmHistory::default(),
        children: Children::default(),
    });
    world.spawn(|| {
        Proc::new(Kind::Llm(Box::new(Llm {
            pending_streak: crate::patience::PendingStreak::default(),
            peer: llm::peer(llm::scripted(Box::new([]))).expect("deferred peer"),
            scripts: Some(scripts),
            rules: None,
            cue: false,
            placement_hold: true,
            holds: Vec::new(),
            release: None,
            released: None,
        })))
    });
    world.spawn_signals(|signals| {
        Proc::new(Kind::Service(Box::new(ServiceHost::configured(
            scratch.path().into(),
            signals,
            Some(fixture),
            false,
        ))))
    });
    let agents = fixtures::local_places(scratch.path());
    host(&mut world, scratch.path(), steps, false, agents, agent::agent);
    world.spawn(|| {
        let (forge, peer) = forge::State::new(fixture);
        *state.borrow_mut() = Some(forge);
        Proc::new(Kind::Forge(Box::new(ForgePeer::new(peer))))
    });
    let mut git = crate::git::program();
    git.instances = agents;
    world.host_roots(git, crate::git::roots);
    let outcome = world.run(&skein_shell::Clock::new(), patience);
    crate::git::share(None);
    outcome
}
fn host<R: Referee<Proc>>(
    world: &mut World<Proc, R>,
    path: &std::path::Path,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
    direct_key: bool,
    agents: u32,
    agent: agent::Factory,
) {
    let root = skein_shell::open_root(path).expect("caller launch root");
    world.spawn_with_fds(vec![root], || {
        Proc::new(Kind::Caller(Box::new(if direct_key {
            Caller::new(steps(), path.into(), root)
        } else {
            Caller::provisioned(steps(), path.into(), root)
        })))
    });
    let limits = temper_client_shell_cli::limits::native();
    world.host_roots(
        HostedProgram {
            program: fixtures::CTL.into(),
            make: fixtures::ctl,
            instances: fixtures::CALLS_AT_ONCE,
            operations: skein_io::operations(&limits.io).expect("tools operations") * 2 + 5,
        },
        fixtures::ctl_roots,
    );
    world.host_roots(
        HostedProgram {
            program: crate::terminal::OPEN.into(),
            make: crate::terminal::host,
            instances: 1,
            operations: crate::terminal::operations(),
        },
        crate::terminal::roots,
    );
    if agents != 0 {
        host_agent(world, path, agents, agent);
    }
}
fn host_agent<R: Referee<Proc>>(
    world: &mut World<Proc, R>,
    path: &std::path::Path,
    agents: u32,
    agent: agent::Factory,
) {
    use std::os::unix::ffi::OsStrExt;
    let agent_path =
        if path.join("agent.json").exists() { path.join("agent.json") } else { path.join("world-agent.json") };
    let configuration = smith_agent_shell::config::read(&agent_path).expect("agent operation bound");
    world.host_roots(
        HostedProgram {
            program: fixtures::pinned_agent_program(path).as_os_str().as_bytes().into(),
            make: agent,
            instances: agents,
            operations: smith_agent_service::program_operations(&configuration.service).expect("agent program routes")
                + 3,
        },
        |_| Vec::new(),
    );
}
pub struct Judge {
    checks: diagnostics::Judge,
    controls: Controls,
    workers: Vec<&'static str>,
    lifecycle: Lifecycle,
    armed: bool,
    failure: Option<Failure>,
    wake: Option<Time>,
    forge: Option<std::rc::Rc<std::cell::RefCell<Option<forge::State>>>>,
    github: Option<crate::github::State>,
    llm_history: LlmHistory,
    children: Children,
}

struct Failure {
    message: String,
    settled: bool,
}

/// Hosted children append during a pass; killed children are removed between passes.
#[derive(Default)]
struct Children {
    seen: usize,
    finished: bool,
    terminals: Vec<usize>,
    calls: Vec<[CallRecords; 2]>,
    differing_calls: [usize; 2],
    lock_change: bool,
    last_setup: bool,
    locks: std::rc::Rc<std::cell::RefCell<crate::calls::recorded::LockChanges>>,
}
/// All retained attempts for one (call, setup), compared only when invalidated.
#[derive(Default)]
struct CallRecords {
    positions: Vec<usize>,
    differs: bool,
}
impl Children {
    fn observe(&mut self, children: &[Proc]) -> bool {
        let removed = children.len() < self.seen;
        if removed {
            self.seen = 0;
            self.terminals.clear();
            self.calls.clear();
            self.differing_calls = [0; 2];
            self.lock_change = false;
            self.locks.borrow_mut().clear();
        }
        for (offset, proc) in children[self.seen..].iter().enumerate() {
            if let Kind::Ctl(recorded) = &proc.kind {
                let index = self.seen + offset;
                if self.calls.len() <= recorded.call {
                    self.calls.resize_with(recorded.call + 1, Default::default);
                }
                let records = &mut self.calls[recorded.call][usize::from(recorded.setup)];
                records.positions.push(index);
                self.locks.borrow_mut().invalidate(recorded.call, recorded.setup);
                if !self.finished {
                    recorded.observe_with(&self.locks);
                }
            } else if matches!(&proc.kind, Kind::Terminal(_)) {
                self.terminals.push(self.seen + offset);
            }
        }
        self.seen = children.len();
        removed
    }
    fn apply(&mut self, children: &[Proc], caller: &mut Caller) {
        // Observe removals before this pass can append children with reused positions.
        if self.observe(children) {
            // Removal can reveal an older attempt at a reused call number. Restore
            // the retained hosts' admission order, just as the whole scan did.
            for proc in children {
                if let Kind::Ctl(recorded) = &proc.kind {
                    recorded.apply(caller);
                }
            }
        }
        self.locks.borrow_mut().apply(caller);
    }
    fn lock_changed(&mut self, children: &[Proc], caller: &Caller) -> Result<bool, String> {
        let mut locks = self.locks.borrow_mut();
        let setup = caller.setup();
        // Without an invalidation, neither the records nor the caller's lock
        // tuples changed; only leaving setup can change the answer.
        if locks.invalidated.is_empty() && self.last_setup == setup {
            return Ok(self.lock_change);
        }
        while let Some((call, setup)) = locks.invalidated.pop() {
            let phase = usize::from(setup);
            let records = &mut self.calls[call][phase];
            let mut differs = false;
            if records.positions.len() > 1 {
                for index in &records.positions {
                    let Kind::Ctl(recorded) = &children[*index].kind else { return Err("ctl position".into()) };
                    if recorded.differs(caller) {
                        differs = true;
                        break;
                    }
                }
            }
            if differs != records.differs {
                if differs {
                    self.differing_calls[phase] += 1;
                } else {
                    self.differing_calls[phase] -= 1;
                }
                records.differs = differs;
            }
        }
        // Ended disagreements persist after the queue drains. Setup records
        // cease to count when the caller reuses their positions for the story.
        self.last_setup = setup;
        self.lock_change =
            locks.changed(caller) || self.differing_calls[0] != 0 || setup && self.differing_calls[1] != 0;
        Ok(self.lock_change)
    }
    fn finish(&mut self, children: &[Proc]) {
        // The referee owns the queue's heap. Hosts must release their subscriptions
        // before the referee drops, so no host frees another owner's allocation.
        for proc in children {
            if let Kind::Ctl(recorded) = &proc.kind {
                recorded.stop_observing();
            }
        }
        self.finished = true;
    }
    fn settled(&self, children: &[Proc]) -> Result<bool, String> {
        for index in &self.terminals {
            let Kind::Terminal(terminal) = &children[*index].kind else { return Err("terminal position".into()) };
            if !terminal.is_empty() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Default)]
struct LlmHistory {
    seen: usize,
    pending: i64,
    queries: usize,
    visits: usize,
}
impl LlmHistory {
    fn observe(&mut self, observations: &[skein_fake_peers::llm::Observation]) -> Result<i64, String> {
        // Deferred forge/GitHub peers are replaced before they run; a driven peer only appends.
        let suffix = observations.get(self.seen..).ok_or("LLM observation history never shrinks")?;
        self.visits += suffix.len();
        for event in suffix {
            if matches!(event, Query { .. }) {
                self.queries += 1;
                self.pending += 1;
            } else if matches!(event, Answered { .. }) {
                self.pending -= 1;
            }
        }
        self.seen = observations.len();
        Ok(self.pending)
    }
}
impl Judge {
    fn shutdown_decisions(
        &mut self,
        caller: &Caller,
        service: &ServiceHost,
        workers: &[Proc],
        children: &[Proc],
    ) -> Result<lifecycle::Shutdown, String> {
        Ok(self.lifecycle.shutdown(
            caller.is_empty(),
            Hosts {
                service_empty: service.is_empty(),
                workers_empty: workers.iter().all(Host::is_empty),
                children_settled: self.children.settled(children)?,
            },
        ))
    }
    fn finish_peers(&mut self, peer: &mut Llm, children: &mut [Proc]) {
        peer.scripts = None;
        peer.shutdown();
        for proc in children.iter_mut() {
            match &mut proc.kind {
                Kind::Forge(forge) => forge.shutdown(),
                Kind::Issuer(issuer) => issuer.shutdown(),
                Kind::Llm(_)
                | Kind::Service(_)
                | Kind::Caller(_)
                | Kind::Ctl(_)
                | Kind::Terminal(_)
                | Kind::Agent(_)
                | Kind::Git(_) => {}
            }
        }
        self.children.finish(children);
    }
    fn fail_startup(&mut self, message: String, caller: &mut Caller, workers: &[Proc], caller_end: usize) {
        self.failure = Some(Failure { message, settled: false });
        caller.shutdown();
        for (position, worker) in workers.iter().enumerate() {
            if !worker.is_empty() {
                self.controls.signal(caller_end + position, ServiceSignal::Terminate);
            }
        }
    }
    fn worker_actions(&mut self, caller_end: usize, caller: &Caller, workers: &mut [Proc]) -> Result<(), String> {
        if let Some(name) = caller.adding_worker() {
            let position = worker_position(workers, name).ok_or("configured worker")?;
            let Kind::Service(worker) = &mut workers[position].kind else { return Err("worker root".into()) };
            let Some(crate::cluster::Shape::Worker { started, .. }) = &mut worker.cluster else {
                return Err("worker shape".into());
            };
            *started = true;
            caller.added.set(worker.linked() > 0);
        }
        if let Some(name) = caller.stopping_worker() {
            let position = worker_position(workers, name).ok_or("configured worker")?;
            let Kind::Service(worker) = &mut workers[position].kind else { return Err("worker root".into()) };
            if !worker.stopping() {
                worker.stop_for_restart()?;
                self.controls.signal(caller_end + position, ServiceSignal::Terminate);
            }
            caller.worker_ready.set(worker.settled());
        }
        if let Some(name) = caller.starting_worker() {
            let position = worker_position(workers, name).ok_or("configured worker")?;
            let Kind::Service(worker) = &mut workers[position].kind else { return Err("worker root".into()) };
            if worker.stopping() {
                worker.restart()?;
            }
            caller.worker_ready.set(worker.linked() > worker.lives().iter().map(|life| life.linked).sum());
        }
        Ok(())
    }
}
impl Judge {
    #[expect(clippy::too_many_lines, reason = "issuer admission cues join the other hosted-process story cues")]
    fn act_checked(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        self.wake = None;
        if self.failure.is_some() || self.checks.finding.is_some() {
            return Ok(());
        }
        let engine_index = procs.iter().position(|proc| matches!(&proc.kind, Kind::Service(_))).ok_or("engine root")?;
        let caller_end = 3;
        let (roots, children) = procs.split_at_mut(caller_end);
        let [
            Proc { kind: Kind::Llm(peer), .. },
            Proc { kind: Kind::Service(service), .. },
            Proc { kind: Kind::Caller(caller), .. },
        ] = roots
        else {
            return Err("root admission order".into());
        };
        let (workers, children) = children.split_at_mut(self.workers.len());
        // Open initial links in the fixture's order, independently of fetch timing.
        let mut preceding_linked = true;
        for proc in workers.iter_mut() {
            let Kind::Service(worker) = &mut proc.kind else { return Err("worker root".into()) };
            worker.address = peer.address();
            let opened = worker.started_at().is_some();
            let linked = worker.linked() > 0;
            if let Some(crate::cluster::Shape::Worker { engine, started, .. }) = &mut worker.cluster {
                if opened || preceding_linked {
                    *engine = service.workers_address;
                }
                preceding_linked &= !*started || linked;
            }
        }
        self.worker_actions(caller_end, caller, workers)?;
        peer.release_hold(caller);
        if placed(workers) && peer.placement_hold && !self.lifecycle.finished {
            peer.cue = true;
        }
        if !self.armed {
            for name in caller.held_names() {
                let name = temper_client_tools::jig::key::Name::new(name).ok_or("held request name")?;
                service.arm(temper_client_tools::key::engine(&name))?;
            }
            self.armed = true;
        }
        if caller.cancelling_run() {
            for proc in children.iter_mut() {
                if let Kind::Agent(agent) = &mut proc.kind {
                    agent.cancel();
                }
            }
        }
        self.children.apply(children, caller);
        if let Some(state) = &self.forge {
            let mut borrowed = state.borrow_mut();
            let state = borrowed.as_mut().ok_or("spawned forge")?;
            let Kind::Forge(forge) = &mut children[0].kind else { return Err("forge admission order".into()) };
            service.forge_address = forge.address();
            for proc in workers.iter_mut() {
                let Kind::Service(worker) = &mut proc.kind else { return Err("worker root".into()) };
                worker.forge_address = forge.address();
            }
            if let Some(address) = forge.address() {
                peer.bind(address)?;
            }
            if let Some(hooks) = service.hooks
                && !state.hooked
            {
                state.hand_hooks(hooks);
            }
            person_push(now, caller, state, &mut service.seen)?;
            service.seen.collect(now, forge, &state.handle, &state.repository)?;
            caller.pushed.set(state.moved && caller.pushing().is_some());
            if service.is_empty() {
                forge.shutdown();
            }
        }
        if self.github.is_some() {
            github_cues(peer, service, children)?;
        }
        service.address = peer.address();
        Lifecycle::release(
            caller,
            service.links(),
            service.fixture_ready() && (caller.setup() || story_ready(service, workers)),
        );
        if !caller.released && service.is_empty() {
            self.fail_startup(service.early_exit(engine_index)?, caller, workers, caller_end);
            self.finish_peers(peer, children);
            return Ok(());
        }
        if let Some(call) = caller.watching {
            service.arm_watch(call);
            caller.watched = service.delivered();
        }
        let shutdown = self.shutdown_decisions(caller, service, workers, children)?;
        if shutdown.workers {
            check_ending(service, workers, children)?;
            for (position, _) in self.workers.iter().enumerate() {
                self.controls.signal(caller_end + position, ServiceSignal::Terminate);
            }
        }
        if shutdown.engine {
            self.controls.signal(engine_index, ServiceSignal::Terminate);
        }
        if shutdown.peers {
            self.finish_peers(peer, children);
        }
        Ok(())
    }
}
impl Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        let context = diagnostics::Context {
            lifecycle: self.lifecycle,
            failure: self.failure.as_ref(),
            forge: self.forge.as_deref(),
            github: self.github.as_ref(),
        };
        if !self.checks.act(now, procs, context)
            && let Err(finding) = self.act_checked(now, procs)
        {
            self.checks.finding = Some(finding);
        }
    }
    fn watch(&mut self, now: Time, procs: &[Proc]) -> Result<(), String> {
        self.checks.watch(procs, procs.iter().filter_map(Proc::observed_engine))?;
        if let Some(failure) = &mut self.failure {
            failure.settled = procs.iter().all(Host::is_empty);
            return crate::patience::check_overdue(now, self.overdue(now));
        }
        let [
            Proc { kind: Kind::Llm(peer), .. },
            Proc { kind: Kind::Service(service), .. },
            Proc { kind: Kind::Caller(caller), .. },
            ..,
        ] = procs
        else {
            return Err("root admission order".into());
        };
        if let Some(engine) = service.service.as_ref().and_then(crate::service::Driven::engine_domain) {
            caller.observe_read_version(engine);
        }
        let caller_end = 3;
        let (workers, children) = procs[caller_end..].split_at(self.workers.len());
        let mut workers_changed = false;
        for proc in workers {
            let Kind::Service(worker) = &proc.kind else { return Err("worker root".into()) };
            workers_changed |= (peer.address().is_some() && worker.address.is_none())
                || (worker_started(worker)
                    && service.workers_address.is_some()
                    && worker.cluster.as_ref().is_some_and(|shape| !shape.ready()));
        }
        let _ = self.children.observe(children);
        let pending = self.llm_history.observe(peer.observations())?;
        let queries = self.llm_history.queries;
        caller.llm_queries.set(queries);
        caller.held.set(
            service.held_at(caller.held_index())
                || !workers.is_empty()
                    && caller.held_name().is_some_and(|name| peer.holds.iter().any(|hold| hold.as_ref() == name))
                    && queries > caller.held_queries.get()
                    && pending > 0,
        );
        let lock_changed = self.children.lock_changed(children, caller)?;
        let forge_changed = if let Some(state) = &self.forge {
            let borrowed = state.borrow();
            let state = borrowed.as_ref().ok_or("spawned forge")?;
            let Kind::Forge(forge) = &children[0].kind else { return Err("forge root".into()) };
            (forge.address().is_some() && service.forge_address.is_none())
                || (service.hooks.is_some() && !state.hooked)
                || (caller.pushing().is_some() && !state.moved)
        } else {
            false
        };
        let placing = placed(workers) && pending > 0 && peer.placement_hold;
        let named_worker = |name| {
            worker_position(workers, name).and_then(|position| {
                // The workers were checked above.
                if let Kind::Service(worker) = &workers[position].kind { Some(worker.as_ref()) } else { None }
            })
        };
        let worker_action = caller
            .stopping_worker()
            .and_then(named_worker)
            .is_some_and(|worker| !worker.stopping() || worker.settled() && !caller.worker_ready.get())
            || caller.starting_worker().and_then(named_worker).is_some_and(|worker| {
                worker.stopping()
                    || worker.linked() > worker.lives().iter().map(|life| life.linked).sum()
                        && !caller.worker_ready.get()
            });
        if worker_action || caller.adding_worker().and_then(named_worker).is_some_and(|worker| {
            !worker_started(worker) || worker.linked() > 0 && !caller.added.get()
        })
            || (caller.releasing_hold().is_some() && !caller.hold_released.get() && peer.release.is_none())
            || placing
            || workers_changed
            || (self.lifecycle.stopping && workers.iter().all(Host::is_empty) && !self.lifecycle.engine_stopping)
            || (self.github.is_some() && children.first().is_some_and(|proc|
                matches!(&proc.kind, Kind::Forge(forge) if forge.address().is_some() && service.forge_address.is_none())))
            || forge_changed
            || lock_changed
            || (peer.address().is_some() && service.address.is_none())
            || (service.links() != 0
                && !caller.released
                && service.fixture_ready() && (caller.setup() || story_ready(service, workers)))
            || (caller.watching.is_some() && (!service.watching() || service.delivered() && !caller.watched))
            || (caller.is_empty() && self.children.settled(children)? && !self.lifecycle.stopping)
            || (service.is_empty() && !self.lifecycle.finished && (!caller.released || caller.is_empty()))
        {
            self.wake = Some(now);
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.wake, self.checks.patience.deadline()].into_iter().flatten().min()
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        self.checks.overdue(self.failure.as_ref())
    }
    fn passed(&self) -> bool {
        self.lifecycle.passed(self.failure.is_some() || self.checks.finding.is_some())
    }
}

fn story_ready(service: &ServiceHost, workers: &[Proc]) -> bool {
    service.fixture_ready()
        && workers
            .iter()
            .all(|proc| matches!(&proc.kind, Kind::Service(worker) if !worker_started(worker) || worker.linked() > 0))
}

fn check_ending(service: &ServiceHost, workers: &[Proc], children: &[Proc]) -> Result<(), String> {
    let mut obligations = crate::ending::Obligations::default();
    obligations.read(service, &[])?;
    for worker in workers {
        let Kind::Service(worker) = &worker.kind else { return Err("worker root".into()) };
        obligations.read(worker, &[])?;
    }
    for (index, child) in children.iter().enumerate() {
        obligations.read_child(3 + workers.len() + index, child);
    }
    obligations.check(&[])
}

fn person_push(now: Time, caller: &Caller, state: &mut forge::State, seen: &mut forge::Seen) -> Result<(), String> {
    if let Some((path, words)) = caller.pushing()
        && !state.moved
    {
        state.push(now, skein_shell::Clock::new().now().wall, path, words)?;
        seen.pushed(&state.handle)?;
    }
    Ok(())
}

fn github_cues(peer: &mut Llm, service: &mut ServiceHost, children: &mut [Proc]) -> Result<(), String> {
    let Kind::Forge(forge) = &mut children[0].kind else { return Err("GitHub root".into()) };
    service.forge_address = forge.address();
    if service.blob_address != forge.blob_address() {
        service.blob_address = forge.blob_address();
        if let Some(blob) = service.blob_address {
            forge
                .github_layer_mut()
                .ok_or("GitHub layer")?
                .set_blob_origin(
                    format!("http://productionresultssa17.blob.core.windows.net:{}", blob.port()).into_bytes().into(),
                )
                .map_err(|error| format!("fake log origin: {error:?}"))?;
        }
    }
    if let Some(address) = forge.address() {
        peer.bind(address)?;
    }
    if service.is_empty() {
        forge.shutdown();
    }
    Ok(())
}

pub(crate) fn run_cluster_forge(
    workers: &[&'static str],
    late: &[&str],
    scratch: &[skein_scratch::Scratch],
    fixture: forge::Fixture,
    scripts: Scripts,
    rules: Rules,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
) -> skein_world::real::Outcome<Proc> {
    let patience = crate::PATIENCE;
    let state = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut world = World::new_controlled(|controls| cluster_judge(controls, workers, Some(state.clone()), patience));
    cluster_roots(
        &mut world,
        workers,
        late,
        skein_lib::Duration::from_millis(10),
        scratch,
        || llm::scripted(Box::new([])),
        Some((fixture, scripts, rules)),
        steps,
    );
    world.spawn(|| {
        let (forge, peer) = forge::State::new(fixture);
        *state.borrow_mut() = Some(forge);
        Proc::new(Kind::Forge(Box::new(ForgePeer::new(peer))))
    });
    world.run(&skein_shell::Clock::new(), patience)
}

#[must_use]
pub fn cluster_judge(
    controls: Controls,
    workers: &[&'static str],
    forge: Option<std::rc::Rc<std::cell::RefCell<Option<forge::State>>>>,
    patience: skein_lib::Duration,
) -> Judge {
    warm_runtime();
    Judge {
        checks: diagnostics::Judge::new(crate::patience::Patience::new(patience)),
        controls,
        workers: workers.to_vec(),
        lifecycle: Lifecycle::default(),
        failure: None,
        armed: false,
        wake: None,
        forge,
        github: None,
        llm_history: LlmHistory::default(),
        children: Children::default(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "cluster roots share the story's worker grace alongside its peers and scripts"
)]
pub fn cluster_roots<R: Referee<Proc>>(
    world: &mut World<Proc, R>,
    workers: &[&'static str],
    late: &[&str],
    grace: skein_lib::Duration,
    scratch: &[skein_scratch::Scratch],
    recipe: impl FnOnce() -> llm::Recipe,
    forge: Option<(forge::Fixture, Scripts, Rules)>,
    steps: impl FnOnce() -> Vec<crate::calls::Step>,
) {
    let (engine, worker_scratch) = scratch.split_first().expect("engine scratch");
    world.spawn(|| {
        let recipe = recipe();
        let holds = script_holds(&recipe);
        let placement_hold = holds.iter().any(|name| name.as_ref() == crate::cluster::PLACED);
        Proc::new(Kind::Llm(Box::new(Llm {
            pending_streak: crate::patience::PendingStreak::default(),
            peer: llm::peer(recipe).expect("cluster LLM"),
            scripts: forge.map(|(_, scripts, _)| scripts),
            rules: forge.map(|(_, _, rules)| rules),
            cue: false,
            placement_hold: forge.is_some() || placement_hold,
            holds,
            release: None,
            released: None,
        })))
    });
    world.spawn_signals(|signals| {
        Proc::new(Kind::Service(Box::new(ServiceHost::cluster(
            engine.path(),
            signals,
            crate::cluster::Shape::Engine { workers: workers.to_vec(), grace },
            forge.map(|(fixture, _, _)| fixture),
        ))))
    });
    host(world, &engine.path().join("data"), steps, false, 0, agent::agent);
    for (name, scratch) in workers.iter().zip(worker_scratch) {
        world.spawn_signals(|signals| {
            Proc::new(Kind::Service(Box::new(ServiceHost::cluster(
                scratch.path(),
                signals,
                crate::cluster::Shape::Worker { name, engine: None, started: !late.contains(name), grace },
                forge.map(|(fixture, _, _)| fixture),
            ))))
        });
        host_agent(world, &scratch.path().join("data"), 1, agent::agent);
    }
    let mut git = crate::git::program();
    git.instances = u32::try_from(workers.len()).expect("worker git count");
    world.host_roots(git, crate::git::roots);
}

fn worker_position(workers: &[Proc], name: &str) -> Option<usize> {
    workers.iter().position(|proc| matches!(&proc.kind, Kind::Service(worker)
        if matches!(worker.cluster, Some(crate::cluster::Shape::Worker { name: worker_name, .. }) if worker_name == name)))
}

fn script_holds(recipe: &llm::Recipe) -> Vec<Box<[u8]>> {
    let mut names = Vec::new();
    for script in recipe.domain.scripts() {
        for turn in &script.turns {
            for line in &turn.lines {
                if let skein_fake_llm_domain::api::Line::Pieces { pieces } = line {
                    names.extend(pieces.iter().filter_map(|piece| piece.hold.clone()));
                }
            }
        }
    }
    names
}

fn worker_started(worker: &ServiceHost) -> bool {
    !matches!(worker.cluster, Some(crate::cluster::Shape::Worker { started: false, .. }))
}

fn placed(workers: &[Proc]) -> bool {
    !workers.is_empty()
        && workers.iter().all(
            |proc| matches!(&proc.kind, Kind::Service(worker) if !worker_started(worker) || worker.runs.assigned()),
        )
}

#[cfg(test)]
mod tests {
    use super::LlmHistory;
    use skein_fake_llm_domain::api;
    use skein_fake_peers::llm::Observation::{self, Accepted, Answered, Closed, Credential, Head, Query};
    use skein_lib::Token;

    fn caller() -> crate::calls::Caller {
        let mut caller = crate::calls::Caller::new(vec![], std::path::PathBuf::new(), skein_io::kernel::Fd::new(0));
        caller.calls.push(crate::calls::Called {
            locked: None,
            released: None,
            busy: 0,
            exit: None,
            stdout: vec![],
            stderr: vec![],
            spawned: skein_lib::Time::ZERO,
            ended: skein_lib::Time::ZERO,
            observed_at: None,
            tries: 0,
            early: 0,
            answer: std::cell::RefCell::new(None),
            timeout: std::cell::RefCell::new(None),
        });
        caller
    }

    pub(super) fn ctl(setup: bool) -> super::Proc {
        use skein_io::kernel::Fd;
        use temper_client_shell_cli::host::{Invocation, OneShot, Startup};
        super::Proc::new(super::Kind::Ctl(Box::new(crate::calls::Recorded::new(
            OneShot::new(
                Invocation {
                    directory: None,
                    parsed: temper_client_shell_cli::arguments::parse(&[b"help"]).expect("help"),
                },
                Startup {
                    parent: None,
                    file_parent: None,
                    stdin: Fd::new(0),
                    stdout: Fd::new(1),
                    stderr: Fd::new(2),
                    signals: Fd::new(3),
                    user: 0,
                    seed: 7,
                },
            )
            .expect("shipped tools limits"),
            0,
            setup,
        ))))
    }

    fn lock_event(procs: &mut [super::Proc], index: usize, locked: Option<u64>, released: Option<u64>) {
        let super::Kind::Ctl(recorded) = &mut procs[index].kind else { panic!("ctl fixture") };
        recorded.locked = locked.map(skein_lib::Time::from_nanos);
        recorded.released = released.map(skein_lib::Time::from_nanos);
        recorded.record_change();
    }

    #[test]
    fn late_lock_completion_of_an_older_attempt_is_observed() {
        let mut caller = caller();
        let mut procs = vec![ctl(false), ctl(false)];
        let mut children = super::Children::default();
        children.apply(&procs, &mut caller);
        assert!(!children.lock_changed(&procs, &caller).expect("ctl records"));

        lock_event(&mut procs, 0, Some(1), None);
        assert!(
            children.lock_changed(&procs, &caller).expect("ctl records"),
            "the older attempt completed after its retry"
        );
        children.apply(&procs, &mut caller);
        assert_eq!(caller.calls[0].locked, Some(skein_lib::Time::from_nanos(1)));
        assert!(
            children.lock_changed(&procs, &caller).expect("ctl records"),
            "the newer empty attempt still disagrees"
        );
        assert!(children.lock_changed(&procs, &caller).expect("ctl records"), "the disagreement needs no new event");
    }

    #[test]
    fn setup_and_story_reuse_a_call_position_without_sharing_records() {
        let mut setup = crate::calls::Caller::provisioned(
            vec![crate::calls::sign_in(crate::calls::D1)],
            std::path::PathBuf::new(),
            skein_io::kernel::Fd::new(0),
        );
        setup.calls = caller().calls;
        let mut procs = vec![ctl(true), ctl(true)];
        lock_event(&mut procs, 0, Some(1), None);
        let mut children = super::Children::default();
        children.apply(&procs, &mut setup);
        assert!(children.lock_changed(&procs, &setup).expect("setup records"), "setup's older attempt disagrees");

        // The story's slot now has a fresh host, while the setup hosts remain.
        let mut caller = caller();
        assert!(
            !children.lock_changed(&procs, &caller).expect("ctl records"),
            "finished setup's cached disagreement is ignored"
        );
        procs.push(ctl(false));
        children.apply(&procs, &mut caller);
        assert!(
            !children.lock_changed(&procs, &caller).expect("ctl records"),
            "a setup attempt is not a retry of the story's call"
        );
        assert_eq!(caller.calls[0].locked, None);
    }

    #[test]
    fn removing_a_killed_attempt_rebuilds_the_lock_comparison() {
        let mut caller = caller();
        let mut procs = vec![ctl(false), ctl(false)];
        lock_event(&mut procs, 1, Some(1), None);
        let mut children = super::Children::default();
        children.apply(&procs, &mut caller);
        assert!(children.lock_changed(&procs, &caller).expect("ctl records"));

        procs.pop().expect("killed retry");
        children.apply(&procs, &mut caller);
        assert_eq!(caller.calls[0].locked, None, "the retained attempt replaces the killed one");
        assert!(
            !children.lock_changed(&procs, &caller).expect("ctl records"),
            "the removed attempt cannot keep the judge awake"
        );
    }

    #[test]
    fn removing_all_killed_attempts_clears_the_cached_answer() {
        let mut caller = caller();
        let mut procs = vec![ctl(false), ctl(false)];
        lock_event(&mut procs, 1, Some(1), None);
        let mut children = super::Children::default();
        children.apply(&procs, &mut caller);
        assert!(children.lock_changed(&procs, &caller).expect("ctl records"));

        procs.clear();
        children.apply(&procs, &mut caller);
        assert!(
            !children.lock_changed(&procs, &caller).expect("ctl records"),
            "no attempt remains to disagree with the caller"
        );
    }

    #[test]
    fn an_older_attempts_release_can_resolve_a_cached_disagreement() {
        let mut caller = caller();
        let mut procs = vec![ctl(false), ctl(false)];
        lock_event(&mut procs, 0, Some(1), None);
        lock_event(&mut procs, 1, Some(1), Some(2));
        let mut children = super::Children::default();
        children.apply(&procs, &mut caller);
        assert!(
            children.lock_changed(&procs, &caller).expect("ctl records"),
            "the older attempt has not released its lock"
        );

        lock_event(&mut procs, 0, Some(1), Some(2));
        assert!(!children.lock_changed(&procs, &caller).expect("ctl records"), "both attempts now match the caller");
        children.apply(&procs, &mut caller);
        assert!(!children.lock_changed(&procs, &caller).expect("ctl records"));
    }

    #[test]
    fn llm_history_matches_every_prefix_and_visits_each_record_once() {
        let connection = Token::new(1);
        let query = || Query {
            connection,
            at: skein_lib::Time::ZERO,
            query: api::Query {
                credential: None,
                caching: api::Caching::Unscoped,
                model: b"fake".as_slice().into(),
                system: Box::new([]),
                tools: Box::new([]),
                choice: api::ToolChoice::None,
                messages: Box::new([]),
                max_tokens: 1,
            },
        };
        let answered = || Answered { connection: Some(connection), result: Ok(()) };
        let detached = || Answered { connection: None, result: Err(api::Error::Unavailable) };
        let rows: [(&str, Vec<Observation>); 7] = [
            ("empty", vec![]),
            ("balanced", vec![query(), answered()]),
            ("duplicate queries and waves", vec![query(), query(), answered(), detached(), query(), answered()]),
            ("closed route", vec![query(), Closed { connection }, detached()]),
            ("negative prefix", vec![detached(), query(), query(), answered()]),
            ("pending", vec![query(), answered(), query()]),
            (
                "other records",
                vec![
                    Accepted { connection },
                    Head { connection, headers: Box::new([]) },
                    Credential {
                        connection,
                        route: skein_llm::Provider::OpenAi,
                        scheme: Some(skein_llm::Scheme::Key),
                        index: Some(0),
                    },
                    Closed { connection },
                ],
            ),
        ];
        for (name, records) in rows {
            let mut history = LlmHistory::default();
            let mut reference_visits = 0;
            for length in 0..=records.len() {
                for _ in 0..3 {
                    let prefix = &records[..length];
                    let expected = prefix.iter().fold(0_i64, |n, event| {
                        n + i64::from(matches!(event, Query { .. })) - i64::from(matches!(event, Answered { .. }))
                    });
                    reference_visits += prefix.len();
                    let pending = history.observe(prefix).expect("growing history");
                    assert_eq!(pending, expected, "{name}, prefix {length}");
                    assert_eq!(history.queries, prefix.iter().filter(|event| matches!(event, Query { .. })).count());
                    for placed in [false, true] {
                        assert_eq!(placed && pending > 0, placed && expected > 0, "{name}, prefix {length}");
                    }
                    assert_eq!(history.visits, length, "{name}, prefix {length}");
                }
            }
            assert_eq!(history.seen, records.len(), "{name}");
            assert_eq!(history.visits, records.len(), "{name}");
            assert_eq!(reference_visits > history.visits, !records.is_empty(), "{name}");
        }
    }
}

pub(crate) mod teardown;
