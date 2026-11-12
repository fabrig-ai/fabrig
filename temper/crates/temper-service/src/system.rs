//! One process, joined through io's workers channel (protocol/README.md, 6).
use crate::{
    AccountsRuntime, End, Stop, Worker, accounts, clients, engine, limits, route::box_link, settings, stop, store,
    values, workers,
};
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, What};
use skein_io::{
    self as io, Io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Env, List, Queue, Time, Token, Wall};
pub(crate) const ACCOUNT_APPLICATION: &[u8] = b"temper";

/// The engine root, its protocol children and the kernel queues.
#[expect(missing_debug_implementations, reason = "the service owns account token records")]
pub struct Service {
    pub(crate) engine: Option<engine::Engine>,
    pub(crate) worker: Option<Worker>,
    io: Io,
    io_env: Env<io::Limits>,
    pub(crate) completions: Queue<Complete>,
    pub(crate) submissions: Queue<Submit>,
    raw_submissions: Queue<Submit>,
    kernel: skein_io::kernel::Routes,
    pub(crate) told: Queue<io::Event>,
    box_pipe: Option<BoxPipe>,
    signals: Option<Token>,
    signals_closing: bool,
    // Keep the generational handles even after claim: a failed restore must
    // close any listener whose component never reached its Listen.
    held_listeners: List<Token>,
    listeners_closing: u32,
    links: Queue<Box<[u8]>>,
    local_links: Queue<Box<[u8]>>,
}
/// A refused adoption leaves both pipe descriptors with the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxRefusal {
    Sockets { limit: u32 },
    Adopted,
    NoEngine,
    ZeroLine,
}
struct BoxPipe {
    requests: Token,
    answers: Token,
    state: BoxPipeState,
    reading: bool,
    output: BoxOutput,
    read_bytes: u32,
}
#[derive(PartialEq, Eq)]
enum BoxPipeState {
    Open,
    Ended,
    Closing,
}
#[derive(PartialEq, Eq)]
enum BoxOutput {
    Idle,
    Waiting,
    Room,
}
impl Service {
    /// Payload-free closure and queue state at the shell's wait boundary.
    #[must_use]
    pub fn wait_state(&self) -> crate::diagnostics::Wait {
        use crate::diagnostics::{Wait, WaitEngine, WaitOwner};
        Wait {
            engine: match &self.engine {
                Some(engine) => Some(WaitEngine {
                    restart: engine.domain.start_phase(),
                    mark_durable: engine.store.host.mark_durable(),
                    store_closed: engine.store.closed(),
                    clients: WaitOwner { closed: engine.clients.closed, io: engine.clients.io.len() },
                    workers: WaitOwner { closed: engine.workers.host.closed(), io: engine.workers.host.io.len() },
                    setup: WaitOwner { closed: engine.setup.closed, io: engine.setup.io.len() },
                    setup_above: engine.setup.above.len(),
                    forge: WaitOwner { closed: engine.forge.closed, io: engine.forge.io.len() },
                    forge_above: engine.forge.above.len(),
                    accounts: match &engine.accounts {
                        Some(accounts) => Some(accounts::wait_state(accounts, engine.env.now)),
                        None => None,
                    },
                }),
                None => None,
            },
            worker_empty: match &self.worker {
                Some(worker) => worker.is_empty(),
                None => true,
            },
            held_listeners: self.held_listeners.len(),
            listener_closes: self.listeners_closing,
            signals_closing: self.signals_closing,
            operations: self.kernel.has_operations(),
            submissions: self.submissions.len(),
        }
    }

    /// Check the real ring's empty, unbounded wait shape without another iteration.
    pub fn check_wait_boundary(&self, now: Time) {
        assert!(
            self.is_empty()
                || self.work_pending(now)
                || self.next_deadline().is_some()
                || self.kernel.has_operations()
                || !self.submissions.is_empty(),
            "service would wait forever with nothing in flight: end={:?}; state={:?}; pending={:?}",
            self.end(),
            self.wait_state(),
            self.pending(now)
        );
    }

    /// The engine domain, for a world to inspect before requesting shutdown.
    #[must_use]
    pub fn engine_domain(&self) -> Option<&temper_engine_domain::Domain> {
        match &self.engine {
            Some(engine) => Some(&engine.domain),
            None => None,
        }
    }

    /// Keep matching workspace clones on the worker, when one is present.
    pub fn reuse_workspaces(&mut self) {
        if let Some(worker) = &mut self.worker {
            worker.reuse_workspaces();
        }
    }

    /// Give each agent the dump directory (temper/docs/design/draft/dogfood-hacks.md, 2.13).
    pub fn dump_llm(&mut self, directory: &[u8]) {
        if let Some(worker) = &mut self.worker {
            worker.dump_llm(directory);
        }
    }

    #[cfg(feature = "worker")]
    pub fn configure_program_sources(
        &mut self,
        resources: crate::program_sources::Resources,
    ) -> Result<(), crate::program_sources::Resources> {
        match &mut self.worker {
            Some(worker) => worker.configure_program_sources(resources),
            None => Err(resources),
        }
    }
    /// Construct the root, its deployment store and its configured accounts.
    pub fn engine(
        settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: List<crate::ForgeRuntime>,
    ) -> Result<Self, Problems> {
        let io = settings.jig.limits.io;
        Self::new(settings, seeds, accounts, forges, None, io)
    }
    /// Join the configured engine and worker in one io, on the same ring.
    pub(crate) fn new(
        settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: List<crate::ForgeRuntime>,
        worker: Option<Worker>,
        io_limits: io::Limits,
    ) -> Result<Self, Problems> {
        let forges = crate::forge::prepare(&settings, forges)?;
        Self::prepared(settings, seeds, accounts, forges, worker, io_limits, None, None)
    }
    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "startup keeps the checked settings and runtime facts together"
    )]
    pub(crate) fn prepared(
        mut settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: skein_forge_connection::Settings,
        worker: Option<Worker>,
        io_limits: io::Limits,
        box_line_bytes: Option<u32>,
        program_login: Option<Box<[u8]>>,
    ) -> Result<Self, Problems> {
        let mut limits =
            limits::engine(&settings.jig.limits, &settings.limits).ok_or(start_problem(b"limits", What::Bound))?;
        limits.memory_bytes = settings.jig.memory;
        limits.host_reply_bytes = settings.host_reply_bytes;
        if !temper_engine_domain::valid(&limits) {
            return Err(start_problem(b"memory_bytes", What::Bound));
        }
        let mut forge_settings = forges;
        let shared = crate::forge::shared(&settings.limits, &settings.jig.limits.io);
        settings.jig.limits.clients.wire.keyed_value_some =
            limits::client_value_bytes(&settings.jig.limits, &settings.limits);
        let store_limits = store::limits(&limits, &limits.forge, &settings.jig.limits.store)
            .ok_or(start_problem(b"limits", What::Bound))?;
        let worker_operations = match &worker {
            Some(worker) => worker.operations(),
            None => 0,
        };
        let operations =
            startup_operations(&io_limits, worker_operations).ok_or(start_problem(b"limits", What::Bound))?;
        let accounts = accounts_driver(&settings, seeds, accounts, &store_limits.secrets)?;
        let mut values =
            values::Values::new(settings.jig.limits.driver.accounts, values::credential_bytes(&settings.jig));
        values.fill(&settings);
        let (forge_config, identities) = crate::forge::config(&settings, seeds.deployment);
        let mut first_periods = List::with_capacity(limits.authority.projects);
        for project in &settings.jig.projects {
            first_periods
                .push((project.number, project.policy.period_spend))
                .or(Err(start_problem(b"projects", What::Bound)))?;
        }
        let mut clients = clients::Clients::new(
            &settings.directories,
            &settings.jig,
            &settings.clients,
            seeds,
            &limits.journal,
            settings.sign_in.as_ref(),
            settings.machine.owned.as_ref(),
            settings.files.entries.as_slice(),
        )?;
        clients.program_environment.clone_from(&settings.program_environment);
        clients.deployment = crate::family::Deployment {
            program_login: if worker.is_some() { program_login } else { None },
            welcome: settings.welcome,
            owned: deployment_owned(settings.machine.owned.is_some(), box_line_bytes.is_some()),
        };
        if let Some(sign_in) = &settings.sign_in {
            clients.configure_google(sign_in, settings.limits.sign_in, seeds, &settings.jig);
        }
        clients.configure_forge_sign_in(&settings);
        let setup = crate::setup::prepare(&settings, &mut forge_settings, &seeds.crypto, box_line_bytes.is_some())?;
        let mut forge = crate::forge::Layer::new(forge_settings, settings.limits.forge_component, &shared, &[])
            .expect("checked forge limits");
        forge.log_origin_limit = settings.limits.forges.log_origins;
        let notes = Queue::with_capacity(settings.limits.forge.forges);
        let workers = workers::Workers::configured(&mut settings, seeds.deployment)?;
        let mut core = core_config(&settings.jig, seeds);
        core.kinds = crate::stock::kinds(&limits, *core.connectors.first().expect("checked forge connector"));
        core.jobs = limits::jobs(&settings.jig.limits);
        let config = temper_engine_domain::Config {
            host_part: settings.host_part,
            kinds_stock: settings.kinds_stock,
            ways_stock: settings.ways_stock,
            period_span: crate::stock::PERIOD_SPAN,
            core,
            landings: settings.landings,
            forge: forge_config,
            identities,
            machine: Some(machine_config(&settings.machine)),
            forges: Box::new([]),
            under_box: box_line_bytes.is_some(),
            playground: playground_config(settings.demo.as_ref(), settings.welcome, settings.limits.setup.deadline),
        };
        let store = store::Store::new(&limits, &store_limits, &config.core.settings.models, seeds.installation);
        let demo = settings.demo;
        let own = settings.limits;
        let mut engine = engine::Engine::new(
            config,
            &limits,
            store,
            engine::Children { accounts, workers, clients, demo, forge, setup, notes, limits: own },
            values,
            first_periods,
        )?;
        if let Some(line_bytes) = box_line_bytes {
            engine.box_link = box_link::Link::new(line_bytes, false);
        }
        Ok(Self {
            worker,
            engine: Some(engine),
            io: Io::new(&io_limits),
            io_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: io_limits },
            // Every live kernel operation can complete in one reap batch.
            completions: Queue::with_capacity(operations),
            // The shell can publish the whole credited operation population at once.
            submissions: Queue::with_capacity(operations),
            // Staged successors and cancels share the aggregate operation credit.
            raw_submissions: Queue::with_capacity(operations),
            told: Queue::uncapped(),
            // Live routes and completed targets awaiting cancel publication use the same credit.
            kernel: skein_io::kernel::Routes::new(operations),
            box_pipe: None,
            signals: None,
            signals_closing: false,
            held_listeners: List::with_capacity(io_limits.sockets),
            listeners_closing: 0,
            // The binary-facing channel holds one admitted pending sign-in population.
            links: Queue::with_capacity(own.sign_in.pending),
            local_links: Queue::with_capacity(own.sign_in.pending),
        })
    }
    // The stock small profile's Prepared::room:
    // twice its worker root output bound of 5,099. These kernel-facing
    // process queues retain that capacity independently of the watermark.
    #[cfg(feature = "worker")]
    const WORKER_ALONE_QUEUE: u32 = 10_198;

    /// Build the worker and shared IO without allocating any engine children.
    #[cfg(feature = "worker")]
    pub(crate) fn worker_alone(
        settings: &settings::Worker,
        limits: &crate::worker_settings::Limits,
        agent: &settings::Agent,
        files: &settings::Files,
        runtime: crate::WorkerRuntime,
        seed: u64,
    ) -> Result<Self, Problems> {
        let io_limits = settings.jig.limits.io;
        crate::worker_settings::prepare(settings, agent, limits)?;
        let queue = Self::WORKER_ALONE_QUEUE;
        let worker_operations = crate::worker_settings::configured_operations(&limits.git_component)
            .ok_or(crate::worker_settings::problem(b"limits", What::Bound))?;
        let operations = worker_alone_operations(&io_limits, worker_operations)
            .ok_or(crate::worker_settings::problem(b"limits", What::Bound))?;
        Ok(Self {
            engine: None,
            worker: Some(Worker::new(settings, agent, limits, files, runtime, seed)?),
            io: Io::new(&io_limits),
            io_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: io_limits },
            completions: Queue::with_capacity(queue),
            submissions: Queue::with_capacity(queue),
            raw_submissions: Queue::with_capacity(operations),
            told: Queue::uncapped(),
            kernel: skein_io::kernel::Routes::new(operations),
            box_pipe: None,
            signals: None,
            signals_closing: false,
            held_listeners: List::with_capacity(io_limits.sockets),
            listeners_closing: 0,
            links: Queue::with_capacity(queue),
            local_links: Queue::with_capacity(queue),
        })
    }
    pub fn take_handover_read(&mut self) -> Option<crate::HandoverRead> {
        let engine = self.engine.as_mut()?;
        match engine.clients.playground_read() {
            Some(read) => Some(read),
            None => crate::setup::lifecycle::take_read(engine),
        }
    }
    pub fn handover_read(
        &mut self,
        forge: u16,
        run: Token,
        answer: Result<Option<skein_lib::Secret>, crate::HandoverUnread>,
    ) {
        if let Some(engine) = &mut self.engine {
            if engine.clients.owns_playground_read(run) {
                engine.clients.playground_read_answer(run, answer);
            } else {
                crate::setup::lifecycle::read(engine, forge, run, answer);
            }
        }
    }
    pub fn take_handover_removal(&mut self) -> Option<crate::HandoverRemoval> {
        crate::setup::lifecycle::take_removal(self.engine.as_mut()?)
    }
    pub fn handover_removed(&mut self, forge: u16, run: Token, result: Result<(), i32>) {
        if let Some(engine) = &mut self.engine {
            crate::setup::lifecycle::removed(engine, forge, run, result);
        }
    }
    /// A manifest page can be reloaded until its own setup clock or capacity ends it.
    #[must_use]
    pub fn setup_page(&self, forge: u16, state: &[u8], now: Time) -> Option<&[u8]> {
        self.engine.as_ref()?.setup.page(forge, state, now)
    }
    /// Adopt the binary's deployment directory.
    pub fn adopt_store(&mut self, root: Fd, effective_user: u32) -> Result<(), Fd> {
        match &mut self.engine {
            Some(engine) => engine.store.adopt(root, effective_user),
            None => Err(root),
        }
    }
    /// Hold the bundle the binary read at start, served on the clients' listener.
    pub fn adopt_bundle(&mut self, bundle: clients::bundle::Bundle) {
        self.engine.as_mut().expect("engine bundle").clients.adopt_bundle(bundle);
    }
    /// Earliest deadline of the root and its children.
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> {
        skein_lib::earliest(
            match &self.engine {
                Some(engine) => engine.next_deadline(),
                None => None,
            },
            match &self.worker {
                Some(worker) => worker.next_deadline(),
                None => None,
            },
        )
    }
    /// Whether an iteration has work on the supplied clock.
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        (match &self.engine {
            Some(engine) => engine.work_pending(now),
            None => false,
        }) || self.worker_runs()
            && match &self.worker {
                Some(worker) => worker.work_pending(now),
                None => false,
            }
            || self.io.is_ready()
            || self.io.is_due(now)
            || !self.completions.is_empty()
            || !self.told.is_empty()
            || !self.raw_submissions.is_empty()
            || self.end().is_some() && self.listeners_closing < self.held_listeners.len()
            || self.end().is_some() && self.signals.is_some() && !self.signals_closing
            || self.end().is_some()
                && match &self.box_pipe {
                    Some(pipe) => pipe.state != BoxPipeState::Closing,
                    None => false,
                }
    }
    /// Snapshot current readiness without advancing any component or exposing payloads.
    /// Only diagnostics callers pay for walking and copying the pending event tags.
    #[must_use]
    pub fn pending(&self, now: Time) -> crate::diagnostics::Pending {
        crate::diagnostics::Pending {
            work_pending: self.work_pending(now),
            worker_runs: self.worker_runs(),
            io_ready: self.io.is_ready(),
            io_due: self.io.is_due(now),
            completions: self.completions.len(),
            told: self.told.len(),
            raw_submissions: self.raw_submissions.len(),
            ended: self.end().is_some(),
            signals: self.signals.is_some(),
            signals_closing: self.signals_closing,
            box_pipe_pending: self.end().is_some()
                && match &self.box_pipe {
                    Some(pipe) => pipe.state != BoxPipeState::Closing,
                    None => false,
                },
            engine: match &self.engine {
                Some(engine) => Some(engine.pending(now)),
                None => None,
            },
            worker: match &self.worker {
                Some(worker) => Some(worker.pending(now)),
                None => None,
            },
        }
    }

    /// A first call drains; another aborts, keeping a failure's reason.
    pub fn shutdown(&mut self) {
        if let Some(engine) = &mut self.engine {
            stop::shutdown(engine);
            if engine.end.is_some()
                && let Some(worker) = &mut self.worker
            {
                worker.abort();
            }
        } else if let Some(worker) = &mut self.worker {
            if worker.stopping() {
                worker.abort();
                worker.set_end(End::Aborted);
            } else {
                worker.stop();
            }
        }
    }
    /// How this service ended, once it has.
    #[must_use]
    pub fn end(&self) -> Option<&End> {
        match &self.engine {
            Some(engine) => engine.end.as_ref(),
            None => match &self.worker {
                Some(worker) => worker.end(),
                None => None,
            },
        }
    }
    /// Sent forge entries left without their outcome when this stop reached its bound.
    #[must_use]
    pub fn unsettled(&self) -> &[temper_engine_domain::Unsettled] {
        if let Some(engine) = &self.engine
            && let Some(drain) = &engine.drain
        {
            drain.unsettled.as_slice()
        } else {
            &[]
        }
    }
    fn worker_runs(&self) -> bool {
        match &self.engine {
            Some(engine) => {
                engine.end.is_some()
                    || match &self.worker {
                        Some(worker) => worker.stopping(),
                        None => false,
                    }
                    || engine.store.opened && engine.workers.host.opened() && engine.clients.opened()
            }
            None => true,
        }
    }
    /// Adopt a held listener before the first down pass (protocol/setup/box.md, 1).
    /// A Listen for its address claims it; refusal leaves the descriptor with the caller.
    pub fn adopt_listener(&mut self, fd: Fd, addr: core::net::SocketAddr) -> Result<(), Fd> {
        if self.held_listeners.room() == 0 {
            return Err(fd);
        }
        match self.io.adopt_listener(fd, addr) {
            Ok(entity) => {
                self.held_listeners.push(entity).expect("one handle per admitted listener");
                Ok(())
            }
            Err(fd) => Err(fd),
        }
    }
    /// Adopt the box's lines and requests (protocol/setup/box.md, 1).
    /// io may keep a bounded read pending until close cancels it.
    /// On refusal the caller retains both descriptors.
    pub fn adopt_box(&mut self, requests: Fd, answers: Fd, line_bytes: u32) -> Result<(), BoxRefusal> {
        if self.box_pipe.is_some() {
            return Err(BoxRefusal::Adopted);
        }
        self.engine.as_ref().ok_or(BoxRefusal::NoEngine)?;
        if line_bytes == 0 {
            return Err(BoxRefusal::ZeroLine);
        }
        if self.io_env.limits.sockets.saturating_sub(self.io.sockets()) < 2 {
            return Err(BoxRefusal::Sockets { limit: self.io_env.limits.sockets });
        }
        self.engine.as_mut().expect("box engine").box_link = box_link::Link::new(line_bytes, true);
        let requests = self.io.adopt_write_pipe(requests, None).expect("two reserved pipe slots");
        let answers = self.io.adopt_read_pipe(answers).expect("second reserved pipe slot");
        self.box_pipe = Some(BoxPipe {
            requests,
            answers,
            state: BoxPipeState::Open,
            reading: false,
            output: BoxOutput::Idle,
            read_bytes: line_bytes.min(self.io_env.limits.intake),
        });
        Ok(())
    }
    /// Adopt the shell's termination-signal source.
    pub fn adopt_signals(&mut self, descriptor: Fd) -> Result<(), Fd> {
        if self.signals.is_some() {
            return Err(descriptor);
        }
        match self.io.adopt_signals_for(descriptor, Token::new(1)) {
            Ok(source) => {
                self.signals = Some(source);
                Ok(())
            }
            Err(descriptor) => Err(descriptor),
        }
    }
    /// Kernel completions, filled by the shell.
    pub const fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    /// Kernel submissions, taken by the shell.
    pub const fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    /// Printed sign-in links, taken by the binary.
    pub const fn links(&mut self) -> &mut Queue<Box<[u8]>> {
        &mut self.links
    }
    /// Local sign-in links, taken by the binary to keep in the engine's link file, never printed.
    pub const fn local_links(&mut self) -> &mut Queue<Box<[u8]>> {
        &mut self.local_links
    }
    /// Bound listeners, taken by the binary before other diagnostics.
    pub fn listening(&mut self) -> &mut Queue<crate::Note> {
        match &mut self.engine {
            Some(engine) => &mut engine.listening,
            None => self.worker.as_mut().expect("worker alone").notes(),
        }
    }
    /// Operator diagnostics, taken by the binary.
    pub fn notes(&mut self) -> &mut Queue<crate::Note> {
        match &mut self.engine {
            Some(engine) => {
                if let Some(worker) = &mut self.worker {
                    worker.drain_notes(&mut engine.notes);
                }
                &mut engine.notes
            }
            None => self.worker.as_mut().expect("worker alone").notes(),
        }
    }
    /// Root, component and I/O deadlines on the injected clock.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        skein_lib::earliest(self.next_policy_deadline(), self.io.next_deadline())
    }
    /// Maximum simultaneous kernel operations, for the shell's ring.
    #[must_use]
    pub fn operations(&self) -> u32 {
        self.raw_submissions.capacity()
    }
    /// Every owned child has closed and every queue and kernel route is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.end().is_some()
            && match &self.worker {
                Some(worker) => worker.is_empty(),
                None => true,
            }
            && match &self.engine {
                Some(engine) => {
                    engine.setup.closed
                        && engine.setup.io.is_empty()
                        && engine.setup.above.is_empty()
                        && engine.forge.closed
                        && engine.forge.io.is_empty()
                        && engine.forge.above.is_empty()
                        && engine.store.closed()
                        && engine.workers.host.closed()
                        && engine.workers.notes.is_empty()
                        && engine.clients.closed
                        && engine.clients.io.is_empty()
                        && match &engine.accounts {
                            Some(accounts) => accounts.idle(),
                            None => true,
                        }
                }
                None => true,
            }
            && self.io.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.raw_submissions.is_empty()
            && self.told.is_empty()
            && self.kernel.is_empty()
    }
    fn route_components(&mut self) {
        if let Some(engine) = &mut self.engine {
            for _ in 0..engine.store.submitted.len() {
                if self.raw_submissions.room() == 0 {
                    break;
                }
                self.raw_submissions.push(engine.store.submitted.pop().expect("store submissions"));
            }
            if engine.store.opened || engine.end.is_some() {
                for _ in 0..engine.setup.io.len() {
                    if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                        break;
                    }
                    let request = engine.setup.io.pop().expect("setup IO requests");
                    io::down(&mut self.io, &self.io_env, crate::setup::tag(request), &mut self.raw_submissions);
                }
                for _ in 0..engine.forge.io.len() {
                    if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                        break;
                    }
                    let request = engine.forge.io.pop().expect("forge io requests");
                    io::down(&mut self.io, &self.io_env, crate::forge::tag(request), &mut self.raw_submissions);
                }
                for _ in 0..engine.clients.io.len() {
                    if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                        break;
                    }
                    let request = engine.clients.io.pop().expect("client io requests");
                    io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
                }
                for _ in 0..engine.workers.host.io.len() {
                    if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                        break;
                    }
                    let request = engine.workers.host.io.pop().expect("worker io requests");
                    io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
                }
            }
            if (engine.store.opened || engine.end.is_some())
                && let Some(accounts) = &mut engine.accounts
            {
                for _ in 0..accounts.io.len() {
                    if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                        break;
                    }
                    let request = accounts::tag(accounts.io.pop().expect("account io requests"));
                    io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
                }
            }
        }
        if let Some(worker) = &mut self.worker {
            for _ in 0..worker.submitted().len() {
                if self.raw_submissions.room() == 0 {
                    break;
                }
                let submit = worker.submitted().pop().expect("worker kernel submissions");
                self.raw_submissions.push(submit);
            }
            for _ in 0..worker.io().len() {
                if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                    break;
                }
                let request = worker.io().pop().expect("worker io requests");
                io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
            }
        }
    }
    /// Format only the roots for a world's secrets check.
    pub fn fmt_roots(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut roots = formatter.debug_list();
        if let Some(engine) = &self.engine {
            roots.entry(&engine.domain);
        }
        #[cfg(feature = "worker")]
        if let Some(worker) = &self.worker {
            roots.entry(&worker.root);
        }
        roots.finish()
    }
    #[cfg(test)]
    pub(crate) fn publish_staged(&mut self) {
        self.kernel.publish(&mut self.raw_submissions, &mut self.submissions);
    }
    #[cfg(test)]
    pub(crate) fn complete_staged(&mut self, complete: Complete) -> Complete {
        self.kernel.complete(complete, &self.raw_submissions)
    }
    fn complete(&mut self, complete: Complete) {
        if store::owns(complete.op) {
            self.engine.as_mut().expect("engine store completion").store.complete(complete);
        } else if crate::tokens::prefix(complete.op) == 3 || crate::tokens::prefix(complete.op) == 6 {
            self.worker
                .as_mut()
                .expect("worker operation prefix")
                .complete(self.io_env.now, complete)
                .expect("worker operation owner");
        } else {
            io::up(&mut self.io, &self.io_env, complete, &mut self.told, &mut self.raw_submissions);
        }
    }
}
/// Core settings with the application's private tools (domain/agent/ask-temper.md, 4).
pub(crate) fn core_config(config: &jig_protocol_config::Engine, seeds: &settings::Seeds) -> jig_domain_core::Config {
    let mut core = jig_protocol_config::core(config, seeds.deployment, seeds.seed);
    let mut tools = List::with_capacity(1);
    for tool in temper_engine_smith::PRIVATE_TOOLS {
        tools.push(Box::from(tool)).expect("private tool names");
    }
    core.settings.private_tools = tools.into_boxed();
    core
}

fn start_problem(member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"engine").member(member), what);
    problems
}
/// The accounts' driver, when the engine configures an account; its records
/// are kept in the secrets' store, at most `secret_bytes` each.
fn accounts_driver(
    settings: &settings::Engine,
    seeds: &settings::Seeds,
    runtime: Option<AccountsRuntime>,
    secrets: &skein_kv::Limits,
) -> Result<Option<accounts::Accounts>, Problems> {
    if settings.jig.llm.providers.is_empty() {
        return Ok(None);
    }
    Ok(Some(accounts::Accounts::new(
        ACCOUNT_APPLICATION.into(),
        &settings.jig,
        &seeds.crypto,
        &seeds.installation,
        runtime,
        secrets,
    )?))
}
/// Drive store terminals, account events, one root input and released outputs.
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    for stage in STAGES {
        iterate_stage(service, stage, now, wall);
    }
}
pub use temper_service_core::{STAGES, Stage};

/// Run one stage with the pass's times (protocol/README.md, 6.1).
#[expect(clippy::too_many_lines, reason = "the five ordered stages include supervised worker shutdown")]
pub fn iterate_stage(service: &mut Service, stage: Stage, now: Time, wall: Wall) {
    match stage {
        Stage::Io => {
            service.io_env.now = now;
            service.io_env.wall = wall;
            if let Some(engine) = &mut service.engine {
                engine::clock(engine, now, wall);
            }
            for _ in 0..service.io_env.limits.sockets.checked_add(service.io_env.limits.refusals).expect("io entries") {
                if !service.io.is_ready() || service.raw_submissions.room() < io::MAX_OUT_RESUME.submissions {
                    break;
                }
                io::resume(&mut service.io, &service.io_env, &mut service.told, &mut service.raw_submissions);
            }
            if !service.io.is_ready() {
                for _ in 0..service.completions.len() {
                    if service.raw_submissions.room() < io::MAX_OUT_UP.submissions {
                        break;
                    }
                    let complete = service.completions.pop().expect("completion count");
                    let complete = service.kernel.complete(complete, &service.raw_submissions);
                    service.complete(complete);
                }
                if service.completions.is_empty() {
                    for _ in 0..service.io_env.limits.sockets.checked_mul(2).expect("io timers") {
                        if !service.io.is_due(now) || service.raw_submissions.room() < io::MAX_OUT_FIRE.submissions {
                            break;
                        }
                        io::fire(&mut service.io, &service.io_env, &mut service.told, &mut service.raw_submissions);
                    }
                }
            }
        }
        Stage::Route => {
            route_events(service);
        }
        Stage::Engine => {
            if let Some(engine) = &mut service.engine {
                if let Some(drain) = &mut engine.drain
                    && let Some(worker) = &service.worker
                {
                    drain.agents_ended = worker.agents_ended();
                }
                engine::iterate(engine, now, wall);
            }
        }
        Stage::Worker => {
            let runs = service.worker_runs();
            if let Some(worker) = &mut service.worker {
                if let Some(engine) = &service.engine {
                    if engine.end.is_some() {
                        worker.abort();
                    } else if let Some(drain) = &engine.drain
                        && drain.closing
                    {
                        worker.stop();
                        if worker.agents_ended() {
                            worker.abort();
                        }
                    }
                }
                if runs {
                    worker.iterate(now, wall);
                }
                if let Some(engine) = &mut service.engine {
                    if let Some(problems) = worker.refused() {
                        if engine.drain.is_none() && engine.end.is_none() {
                            stop::shutdown(engine);
                        }
                        if let Some(End::Drained) = engine.end {
                            engine.end = Some(End::Stopped(Stop::Start(problems.clone())));
                        }
                    }
                } else {
                    if worker.end().is_none()
                        && let Some(problems) = worker.refused()
                    {
                        worker.set_end(End::Stopped(Stop::Start(problems.clone())));
                        worker.abort();
                    }
                    if worker.is_empty() && worker.end().is_none() {
                        worker.set_end(End::Drained);
                    }
                }
            }
        }
        Stage::Out => {
            close_sources(service);
            if let Some(engine) = &mut service.engine {
                engine.abort_if_ended();
            }
            box_io(service);
            service.route_components();
            if let Some(engine) = &mut service.engine
                && engine.clients.starting.is_none()
            {
                for _ in 0..engine.clients.links.len() {
                    if service.links.room() == 0 {
                        break;
                    }
                    let link = engine.clients.links.pop().expect("printed sign-in links");
                    service.links.push(link);
                }
                for _ in 0..engine.clients.local_links.len() {
                    if service.local_links.room() == 0 {
                        break;
                    }
                    let link = engine.clients.local_links.pop().expect("local sign-in links");
                    service.local_links.push(link);
                }
            }
            service.kernel.publish(&mut service.raw_submissions, &mut service.submissions);
            service.io.reclaim();
        }
    }
}
fn close_sources(service: &mut Service) {
    if service.end().is_some() {
        for _ in service.listeners_closing..service.held_listeners.len() {
            if service.raw_submissions.room() < io::MAX_OUT_DOWN.submissions {
                break;
            }
            let entity = *service.held_listeners.get(service.listeners_closing).expect("held listener");
            service.listeners_closing = service.listeners_closing.checked_add(1).expect("one close per held listener");
            io::down(&mut service.io, &service.io_env, io::Request::Close { entity }, &mut service.raw_submissions);
        }
    }
    if service.end().is_some()
        && !service.signals_closing
        && service.raw_submissions.room() >= io::MAX_OUT_DOWN.submissions
        && let Some(entity) = service.signals
    {
        service.signals_closing = true;
        io::down(&mut service.io, &service.io_env, io::Request::Close { entity }, &mut service.raw_submissions);
    }
    if service.end().is_some()
        && service.raw_submissions.room() >= io::MAX_OUT_DOWN.submissions.saturating_mul(2)
        && let Some(pipe) = &mut service.box_pipe
        && pipe.state != BoxPipeState::Closing
    {
        pipe.state = BoxPipeState::Closing;
        for entity in [pipe.requests, pipe.answers] {
            io::down(&mut service.io, &service.io_env, io::Request::Close { entity }, &mut service.raw_submissions);
        }
    }
}
fn held_listener_closed(service: &Service, event: &io::Event) -> bool {
    let io::Event::Closed { owner } = event else { return false };
    for entity in &service.held_listeners {
        if entity == owner {
            return true;
        }
    }
    false
}
fn route_events(service: &mut Service) {
    for _ in 0..service.told.len() {
        if box_notes_full(service) {
            break;
        }
        let event = service.told.pop().expect("io event count");
        if let io::Event::Shutdown { .. } = event {
            service.shutdown();
            continue;
        }
        let owner = match &event {
            io::Event::Listening { owner, .. }
            | io::Event::Accepted { owner, .. }
            | io::Event::Connecting { owner, .. }
            | io::Event::Connected { owner }
            | io::Event::Stream { owner, .. }
            | io::Event::Output { owner, .. }
            | io::Event::Spawned { owner, .. }
            | io::Event::Exited { owner, .. }
            | io::Event::Usage { owner, .. }
            | io::Event::Window { owner, .. }
            | io::Event::Failed { owner, .. }
            | io::Event::Closed { owner } => Some(*owner),
            io::Event::Resized | io::Event::Shutdown { .. } => None,
        };
        let error = if let io::Event::Failed { error, .. } = &event { Some(*error) } else { None };
        if owner == Some(Token::new(1)) {
            if let io::Event::Closed { .. } = event {
                service.signals = None;
            }
            continue;
        }
        if held_listener_closed(service, &event) {
            // An unclaimed listener's token owns Closed. Once claimed, its
            // component's owner receives that terminal through the normal route.
            continue;
        }
        if let Some(pipe) = &service.box_pipe
            && (owner == Some(pipe.requests) || owner == Some(pipe.answers))
        {
            box_event(service, event);
            continue;
        }
        if let Some(worker) = &mut service.worker
            && worker.accepts(&event)
        {
            worker.up(service.io_env.now, event);
        } else if let Some(engine) = &mut service.engine {
            if let Some(owner) = owner
                && crate::tokens::prefix(owner) == 11
            {
                engine.setup.up(event);
            } else if let Some(owner) = owner
                && crate::tokens::prefix(owner) == 5
            {
                engine.forge.up(event);
            } else if let Some(owner) = owner
                && clients::owns(owner)
            {
                if !engine.clients.up(event) && engine.end.is_none() {
                    engine.end = Some(End::Stopped(Stop::Listener {
                        listener: crate::Listening::Clients,
                        address: Some(engine.clients.address),
                        error,
                    }));
                }
            } else if let Some(owner) = owner
                && accounts::owns(owner)
            {
                if let Some(accounts) = &mut engine.accounts {
                    accounts.up(accounts::untag(event));
                }
            } else if !engine.workers.up(service.io_env.now, event, &mut engine.events) && engine.end.is_none() {
                let address = engine.workers.host.listener_address(owner);
                engine.workers.notes.push(crate::Note::WorkerListenerFailed { address, error });
                engine.end = Some(End::Stopped(Stop::Listener { listener: crate::Listening::Workers, address, error }));
            }
            engine.drain_worker_notes();
            for (listener, addr) in [
                (crate::Listening::Clients, engine.clients.listening.take()),
                (crate::Listening::Workers, engine.workers.host.listening.take()),
            ] {
                if let Some(addr) = addr {
                    engine
                        .listening
                        .try_push(crate::Note::Listening { listener, addr })
                        .expect("one note per listener bind");
                }
            }
        }
    }
}

/// Kernel work in a worker-only process, independent of its domain's room.
pub(crate) fn worker_alone_operations(io: &io::Limits, worker_operations: u32) -> Option<u32> {
    // Allow staged successors beside io's in-flight operations. Every io entry
    // point emits at most MAX_OUT_DOWN.submissions; the worker adds git's
    // operations and closes counted by worker_settings.
    io::operations(io)?.checked_mul(io::MAX_OUT_DOWN.submissions)?.checked_add(worker_operations)
}
fn startup_operations(io: &io::Limits, worker_operations: u32) -> Option<u32> {
    // Shared io successors, worker file operations and the three store drivers own the kernel credit.
    worker_alone_operations(io, worker_operations)?.checked_add(store::KERNEL_HANDOFF_ENTRIES)
}

fn box_notes_full(service: &Service) -> bool {
    let Some(engine) = &service.engine else { return false };
    let Some(pipe) = &service.box_pipe else { return false };
    pipe.state == BoxPipeState::Open
        && engine.notes.room() == 0
        && match service.told.iter().next() {
            Some(event) => box_owner(event, pipe),
            None => false,
        }
}
fn box_owner(event: &io::Event, pipe: &BoxPipe) -> bool {
    match event {
        io::Event::Stream { owner, .. } | io::Event::Failed { owner, .. } | io::Event::Closed { owner } => {
            *owner == pipe.requests || *owner == pipe.answers
        }
        io::Event::Listening { .. }
        | io::Event::Accepted { .. }
        | io::Event::Connecting { .. }
        | io::Event::Connected { .. }
        | io::Event::Output { .. }
        | io::Event::Spawned { .. }
        | io::Event::Exited { .. }
        | io::Event::Usage { .. }
        | io::Event::Window { .. }
        | io::Event::Resized
        | io::Event::Shutdown { .. } => false,
    }
}
fn box_event(service: &mut Service, event: io::Event) {
    use skein_lib::stream::Up;
    let pipe = service.box_pipe.as_mut().expect("box event");
    if pipe.state != BoxPipeState::Open {
        return;
    }
    let engine = service.engine.as_mut().expect("box engine");
    let mut out = Queue::with_capacity(3);
    match event {
        io::Event::Stream { owner, up } => match up {
            Up::Bytes(bytes) => {
                assert!(owner == pipe.answers, "box read owner");
                pipe.reading = false;
                engine.box_link.read(&bytes, &mut out);
            }
            Up::Room => {
                assert!(owner == pipe.requests, "box write owner");
                pipe.output = BoxOutput::Room;
            }
            Up::End | Up::Failed(_) => {
                pipe.state = BoxPipeState::Ended;
                engine.box_link.closed(&mut out);
            }
        },
        io::Event::Failed { .. } | io::Event::Closed { .. } => {
            pipe.state = BoxPipeState::Ended;
            engine.box_link.closed(&mut out);
        }
        io::Event::Listening { .. }
        | io::Event::Accepted { .. }
        | io::Event::Connecting { .. }
        | io::Event::Connected { .. }
        | io::Event::Output { .. }
        | io::Event::Spawned { .. }
        | io::Event::Exited { .. }
        | io::Event::Usage { .. }
        | io::Event::Window { .. }
        | io::Event::Resized
        | io::Event::Shutdown { .. } => unreachable!("box pipe event"),
    }
    box_link::drain(engine, &mut out);
}
fn box_io(service: &mut Service) {
    use skein_lib::stream::{Delimiter, Down, Read};
    let Some(pipe) = &mut service.box_pipe else { return };
    if pipe.state != BoxPipeState::Open {
        return;
    }
    if !pipe.reading && service.raw_submissions.room() >= io::MAX_OUT_DOWN.submissions && service.io.takes() {
        io::down(
            &mut service.io,
            &service.io_env,
            io::Request::Stream {
                stream: pipe.answers,
                down: Down::Demand { read: Read::Scan { until: Delimiter::LF, max: pipe.read_bytes }, room: 0 },
            },
            &mut service.raw_submissions,
        );
        pipe.reading = true;
    }
    let engine = service.engine.as_mut().expect("box engine");
    if engine.box_link.write.is_some()
        && service.raw_submissions.room() >= io::MAX_OUT_DOWN.submissions
        && service.io.takes()
    {
        if pipe.output == BoxOutput::Room {
            let line = engine.box_link.write.take().expect("box line ready");
            io::down(
                &mut service.io,
                &service.io_env,
                io::Request::Stream { stream: pipe.requests, down: Down::Send(line) },
                &mut service.raw_submissions,
            );
            pipe.output = BoxOutput::Idle;
        } else if pipe.output == BoxOutput::Idle {
            // Full output credit waits for the preceding line to leave io.
            io::down(
                &mut service.io,
                &service.io_env,
                io::Request::Stream {
                    stream: pipe.requests,
                    down: Down::Demand { read: Read::Nothing, room: service.io_env.limits.output },
                },
                &mut service.raw_submissions,
            );
            pipe.output = BoxOutput::Waiting;
        }
    }
}

impl Service {
    /// Start-time endpoints only; the browser and the issuer never choose a socket.
    pub fn configure_google_endpoints(&mut self, runtime: clients::fetch::Runtime) {
        if let Some(engine) = &mut self.engine {
            engine.clients.google_endpoints(runtime.endpoints);
        }
    }
}

/// Whether the deployment can set up an owned forge (temper/docs/design/client/forge/records.md).
pub(crate) const fn deployment_owned(document_owned: bool, under_box: bool) -> bool {
    document_owned && under_box
}

pub(crate) fn playground_config(
    demo: Option<&settings::Demo>,
    welcome: bool,
    deadline: skein_lib::Duration,
) -> Option<temper_engine_domain_forge::PlaygroundConfig> {
    if !welcome {
        return None;
    }
    match demo {
        Some(demo) => Some(temper_engine_domain_forge::PlaygroundConfig { provider: demo.provider, deadline }),
        None => None,
    }
}

/// github.com's remotes' base (temper/docs/design/protocol/configuration/engine.md).
pub(crate) const GITHUB_REMOTE: &[u8] = b"https://github.com";

/// Machine places passed to the root (temper/docs/design/protocol/configuration/engine.md).
pub(crate) fn machine_config(machine: &temper_forge_setup::machine::Machine) -> temper_engine_domain_forge::Machine {
    use temper_engine_domain_forge::{Address, Machine, Place};
    let github = match &machine.github {
        Some(github) => {
            let (host, port) = match &github.api {
                jig_protocol_config::Address::Host { name, port } => (name.clone(), *port),
                jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => {
                    (socket_host(*address), address.port())
                }
                jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => {
                    unreachable!("checked machine socket")
                }
            };
            Place {
                api: Address { tls: if github.tls { Some(host.clone()) } else { None }, host, port },
                git: github.remote.clone(),
                web: Some(github.web.clone()),
            }
        }
        None => Place {
            api: Address {
                host: skein_lib::bytes::copy_of(b"api.github.com"),
                port: 443,
                tls: Some(skein_lib::bytes::copy_of(b"api.github.com")),
            },
            git: skein_lib::bytes::copy_of(GITHUB_REMOTE),
            web: Some(skein_lib::bytes::copy_of(b"https://github.com")),
        },
    };
    Machine {
        owned_person: match &machine.owned {
            Some(owned) => owned.person.login.clone(),
            None => Box::new([]),
        },
        owned: match &machine.owned {
            Some(owned) => Some(local_place(owned.listen, &owned.remote)),
            None => None,
        },
        fake: match &machine.fake {
            Some(fake) => Some(local_place(fake.listen, &fake.remote)),
            None => None,
        },
        github,
        github_git_program: match &machine.github {
            Some(github) => github.git.clone(),
            None => None,
        },
    }
}
#[expect(clippy::disallowed_macros, reason = "startup renders a checked IP address for the domain place")]
pub(crate) fn socket_host(address: core::net::SocketAddr) -> Box<[u8]> {
    alloc::format!("{}", address.ip()).into_bytes().into_boxed_slice()
}
fn local_place(address: core::net::SocketAddr, remote: &[u8]) -> temper_engine_domain_forge::Place {
    temper_engine_domain_forge::Place {
        api: temper_engine_domain_forge::Address { host: socket_host(address), port: address.port(), tls: None },
        git: skein_lib::bytes::copy_of(remote),
        web: None,
    }
}

#[cfg(test)]
mod box_tests {
    use super::BoxRefusal;
    use crate::tests::fixtures;
    use skein_io::kernel::Fd;

    #[test]
    fn machine_context_carries_only_the_owned_persons_login() {
        let mut machine = fixtures::machine();
        assert!(super::machine_config(&machine).owned_person.is_empty());
        let temper_forge_setup::setup::Mode::Owned(owned) = fixtures::owned_setup(1).entry.mode else {
            panic!("owned machine fixture")
        };
        machine.owned = Some(temper_forge_setup::machine::OwnedMachine {
            sign_in: None,
            listen: owned.listen,
            remote: b"http://forge.example".as_slice().into(),
            person: owned.person,
            public: owned.public,
            fetch: owned.fetch,
            forgejo: owned.forgejo,
            runner: owned.runner,
        });
        assert_eq!(super::machine_config(&machine).owned_person.as_ref(), b"owner");
    }

    #[test]
    fn box_adoption_names_each_refusal_and_keeps_both_descriptors() {
        for expected in
            [BoxRefusal::Sockets { limit: 1 }, BoxRefusal::Adopted, BoxRefusal::NoEngine, BoxRefusal::ZeroLine]
        {
            let mut service = fixtures::started();
            let mut line_bytes = 256;
            match expected {
                BoxRefusal::Sockets { limit } => service.io_env.limits.sockets = limit,
                BoxRefusal::Adopted => service.adopt_box(Fd::new(3), Fd::new(4), line_bytes).unwrap(),
                BoxRefusal::NoEngine => service.engine = None,
                BoxRefusal::ZeroLine => line_bytes = 0,
            }
            let before = (service.io.sockets(), service.submissions.len());
            assert_eq!(service.adopt_box(Fd::new(5), Fd::new(6), line_bytes), Err(expected));
            assert_eq!((service.io.sockets(), service.submissions.len()), before);
            // In particular, no close or adoption was submitted for either fd.
        }
    }
}
