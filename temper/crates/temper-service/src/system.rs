//! One process, joined through io's workers channel (protocol/README.md, 6).
use crate::{AccountsRuntime, End, Worker, clients, settings, store};
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, What};
use skein_io::{
    self as io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{List, Queue, Time, Token, Wall};

/// The engine root, its protocol children and the kernel queues.
#[expect(missing_debug_implementations, reason = "the service owns account token records")]
pub struct Service {
    pub(crate) engine: Option<temper_engine_service::Engine>,
    pub(crate) worker: Option<Worker>,
    ending: Option<End>,
    notes: Queue<crate::Note>,
    listening: Queue<crate::Note>,
    pub(crate) process: temper_service_core::process::Process,
    box_pipe: Option<BoxPipe>,
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
        crate::diagnostics::Wait {
            engine: match &self.engine {
                Some(engine) => Some(engine.wait_state()),
                None => None,
            },
            worker_empty: match &self.worker {
                Some(worker) => worker.is_empty(),
                None => true,
            },
            held_listeners: self.held_listeners.len(),
            listener_closes: self.listeners_closing,
            signals_closing: self.process.signals_closing(),
            operations: self.process.has_operations(),
            submissions: self.process.submission_count(),
        }
    }

    /// Check the real ring's empty, unbounded wait shape without another iteration.
    pub fn check_wait_boundary(&self, now: Time) {
        assert!(
            self.is_empty()
                || self.work_pending(now)
                || self.next_deadline().is_some()
                || self.process.has_operations()
                || self.process.submission_count() != 0,
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
            Some(engine) => Some(engine.domain()),
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
    #[expect(clippy::too_many_arguments, reason = "startup keeps the checked settings and runtime facts together")]
    pub(crate) fn prepared(
        settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: skein_forge_connection::Settings,
        worker: Option<Worker>,
        io_limits: io::Limits,
        box_line_bytes: Option<u32>,
        program_login: Option<Box<[u8]>>,
    ) -> Result<Self, Problems> {
        let links = settings.limits.sign_in.pending;
        let notes = settings.limits.forge.forges;
        let sockets = io_limits.sockets;
        let operations = startup_operations(
            &io_limits,
            match &worker {
                Some(worker) => worker.operations(),
                None => 0,
            },
        )
        .ok_or(start_problem(b"limits", What::Bound))?;
        let engine = temper_engine_service::Engine::prepared(
            settings,
            seeds,
            accounts,
            forges,
            box_line_bytes,
            if worker.is_some() { program_login } else { None },
        )?;
        Ok(Self {
            engine: Some(engine),
            worker,
            ending: None,
            notes: Queue::with_capacity(notes),
            listening: Queue::with_capacity(3),
            process: temper_service_core::process::Process::new(io_limits, operations, operations),
            box_pipe: None,
            held_listeners: List::with_capacity(sockets),
            listeners_closing: 0,
            links: Queue::with_capacity(links),
            local_links: Queue::with_capacity(links),
        })
    }
    pub fn take_handover_read(&mut self) -> Option<crate::HandoverRead> {
        self.engine.as_mut().expect("composition engine").take_handover_read()
    }
    pub fn handover_read(
        &mut self,
        forge: u16,
        run: Token,
        answer: Result<Option<skein_lib::Secret>, crate::HandoverUnread>,
    ) {
        self.engine.as_mut().expect("composition engine").handover_read(forge, run, answer);
    }
    pub fn take_handover_removal(&mut self) -> Option<crate::HandoverRemoval> {
        self.engine.as_mut().expect("composition engine").take_handover_removal()
    }
    pub fn handover_removed(&mut self, forge: u16, run: Token, result: Result<(), i32>) {
        self.engine.as_mut().expect("composition engine").handover_removed(forge, run, result);
    }
    /// A manifest page can be reloaded until its own setup clock or capacity ends it.
    #[must_use]
    pub fn setup_page(&self, forge: u16, state: &[u8], now: Time) -> Option<&[u8]> {
        self.engine.as_ref().expect("composition engine").setup_page(forge, state, now)
    }
    /// Adopt the binary's deployment directory.
    pub fn adopt_store(&mut self, root: Fd, effective_user: u32) -> Result<(), Fd> {
        self.engine.as_mut().expect("composition engine").adopt_store(root, effective_user)
    }
    /// Hold the bundle the binary read at start, served on the clients' listener.
    pub fn adopt_bundle(&mut self, bundle: clients::bundle::Bundle) {
        self.engine.as_mut().expect("composition engine").adopt_bundle(bundle);
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
            Some(engine) => engine.work_pending_with_notes(now, self.notes.len()),
            None => false,
        }) || self.worker_runs()
            && match &self.worker {
                Some(worker) => worker.work_pending(now),
                None => false,
            }
            || self.process.work_pending(now)
            || self.end().is_some() && self.listeners_closing < self.held_listeners.len()
            || self.end().is_some() && self.process.signals_pending()
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
            io_ready: self.process.io().is_ready(),
            io_due: self.process.io().is_due(now),
            completions: self.process.completion_count(),
            told: self.process.told_count(),
            raw_submissions: self.process.raw_submission_count(),
            ended: self.end().is_some(),
            signals: self.process.has_signals(),
            signals_closing: self.process.signals_closing(),
            box_pipe_pending: self.end().is_some()
                && match &self.box_pipe {
                    Some(pipe) => pipe.state != BoxPipeState::Closing,
                    None => false,
                },
            engine: match &self.engine {
                Some(engine) => Some(engine.pending_with_notes(now, self.notes.len())),
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
            engine.shutdown();
            if engine.end().is_some()
                && let Some(worker) = &mut self.worker
            {
                worker.abort();
            }
        }
        self.join_end();
    }
    /// How this service ended, once it has.
    #[must_use]
    pub fn end(&self) -> Option<&End> {
        self.ending.as_ref()
    }
    /// Sent forge entries left without their outcome when this stop reached its bound.
    #[must_use]
    pub fn unsettled(&self) -> &[temper_engine_domain::Unsettled] {
        match &self.engine {
            Some(engine) => engine.unsettled(),
            None => &[],
        }
    }
    fn worker_runs(&self) -> bool {
        match &self.engine {
            Some(engine) => {
                engine.end().is_some()
                    || match &self.worker {
                        Some(worker) => worker.stopping(),
                        None => false,
                    }
                    || engine.ready_for_worker()
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
        match self.process.io_mut().adopt_listener(fd, addr) {
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
        if self.process.env().limits.sockets.saturating_sub(self.process.io().sockets()) < 2 {
            return Err(BoxRefusal::Sockets { limit: self.process.env().limits.sockets });
        }
        self.engine.as_mut().expect("box engine").box_link(line_bytes, true);
        let requests = self.process.io_mut().adopt_write_pipe(requests, None).expect("two reserved pipe slots");
        let answers = self.process.io_mut().adopt_read_pipe(answers).expect("second reserved pipe slot");
        self.box_pipe = Some(BoxPipe {
            requests,
            answers,
            state: BoxPipeState::Open,
            reading: false,
            output: BoxOutput::Idle,
            read_bytes: line_bytes.min(self.process.env().limits.intake),
        });
        Ok(())
    }
    /// Adopt the shell's termination-signal source.
    pub fn adopt_signals(&mut self, descriptor: Fd) -> Result<(), Fd> {
        self.process.adopt_signals(descriptor)
    }
    /// Kernel completions, filled by the shell.
    pub const fn completions(&mut self) -> &mut Queue<Complete> {
        self.process.completions()
    }
    /// Kernel submissions, taken by the shell.
    pub const fn submissions(&mut self) -> &mut Queue<Submit> {
        self.process.submissions()
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
        if let Some(engine) = &mut self.engine {
            for _ in 0..self.listening.room() {
                let Some(note) = engine.listening().pop() else { break };
                self.listening.push(crate::Note::Engine(note));
            }
        }
        if let Some(engine) = &mut self.engine {
            engine.held_listening(self.listening.len());
        }
        &mut self.listening
    }
    /// Operator diagnostics, taken by the binary.
    pub fn notes(&mut self) -> &mut Queue<crate::Note> {
        if let Some(engine) = &mut self.engine {
            for _ in 0..self.notes.room() {
                let Some(note) = engine.notes().pop() else { break };
                self.notes.push(crate::Note::Engine(note));
            }
        }
        if let Some(worker) = &mut self.worker {
            drain_worker_notes(worker, &mut self.notes);
        }
        if let Some(engine) = &mut self.engine {
            engine.held_notes(self.notes.len());
        }
        &mut self.notes
    }
    /// Root, component and I/O deadlines on the injected clock.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        skein_lib::earliest(self.next_policy_deadline(), self.process.io().next_deadline())
    }
    /// Maximum simultaneous kernel operations, for the shell's ring.
    #[must_use]
    pub fn operations(&self) -> u32 {
        self.process.operations()
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
                Some(engine) => engine.is_empty(),
                None => true,
            }
            && self.process.is_empty()
    }
    fn route_components(&mut self) {
        if let Some(engine) = &mut self.engine {
            engine.route(&mut self.process);
        }
        if let Some(worker) = &mut self.worker {
            for _ in 0..worker.submitted().len() {
                if self.process.room() == 0 {
                    break;
                }
                let submit = worker.submitted().pop().expect("worker kernel submissions");
                self.process.raw_submissions().push(submit);
            }
            for _ in 0..worker.io().len() {
                if self.process.room() < io::MAX_OUT_DOWN.submissions || !self.process.io().takes() {
                    break;
                }
                let request = worker.io().pop().expect("worker io requests");
                self.process.down(request);
            }
        }
    }
    /// Format only the roots for a world's secrets check.
    pub fn fmt_roots(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut roots = formatter.debug_list();
        if let Some(engine) = &self.engine {
            roots.entry(engine.domain());
        }
        if let Some(worker) = &self.worker {
            roots.entry(worker.domain());
        }
        roots.finish()
    }
    #[cfg(test)]
    pub(crate) fn publish_staged(&mut self) {
        self.process.publish();
    }
    #[cfg(test)]
    pub(crate) fn complete_staged(&mut self, complete: Complete) -> Complete {
        self.process.complete(complete)
    }
    fn complete(&mut self, complete: Complete) {
        if store::owns(complete.op) {
            self.engine.as_mut().expect("engine store completion").complete(complete).expect("store owner");
        } else if crate::tokens::prefix(complete.op) == 3 || crate::tokens::prefix(complete.op) == 6 {
            self.worker
                .as_mut()
                .expect("worker operation prefix")
                .complete(self.process.env().now, complete)
                .expect("worker operation owner");
        } else {
            self.process.up(complete);
        }
    }
}

fn start_problem(member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"engine").member(member), what);
    problems
}
/// Drive store terminals, account events, one root input and released outputs.
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    for stage in STAGES {
        iterate_stage(service, stage, now, wall);
    }
}
pub use temper_service_core::{STAGES, Stage};

/// Run one stage with the pass's times (protocol/README.md, 6.1).
pub fn iterate_stage(service: &mut Service, stage: Stage, now: Time, wall: Wall) {
    if let Some(engine) = &mut service.engine {
        engine.held_notes(service.notes.len());
        engine.held_listening(service.listening.len());
    }
    match stage {
        Stage::Io => {
            service.process.clock(now, wall);
            if let Some(engine) = &mut service.engine {
                engine.clock(now, wall);
            }
            service.process.resume();
            if !service.process.io().is_ready() {
                for _ in 0..service.process.completion_count() {
                    let Some(complete) = service.process.take_completion() else { break };
                    service.complete(complete);
                }
                service.process.fire();
            }
        }
        Stage::Route => {
            route_events(service);
        }
        Stage::Engine => {
            if let Some(engine) = &mut service.engine {
                if let Some(worker) = &service.worker {
                    engine.agents_ended(worker.agents_ended());
                }
                engine.iterate(now, wall);
            }
        }
        Stage::Worker => {
            let runs = service.worker_runs();
            if let Some(worker) = &mut service.worker {
                if let Some(engine) = &service.engine {
                    if engine.end().is_some() {
                        worker.abort();
                    } else if engine.draining() {
                        worker.stop();
                        if worker.agents_ended() {
                            worker.abort();
                        }
                    }
                }
                if runs {
                    worker.iterate(now, wall);
                }
                if let Some(engine) = &mut service.engine
                    && worker.refused().is_some()
                    && !engine.stopping()
                {
                    engine.shutdown();
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
            if let Some(engine) = &mut service.engine {
                engine.take_links(&mut service.links, &mut service.local_links);
            }
            service.process.publish();
        }
    }
    service.join_end();
}
fn close_sources(service: &mut Service) {
    if service.end().is_some() {
        for _ in service.listeners_closing..service.held_listeners.len() {
            if service.process.room() < io::MAX_OUT_DOWN.submissions {
                break;
            }
            let entity = *service.held_listeners.get(service.listeners_closing).expect("held listener");
            service.listeners_closing = service.listeners_closing.checked_add(1).expect("one close per held listener");
            service.process.down(io::Request::Close { entity });
        }
    }
    if service.end().is_some() {
        service.process.close_signals();
    }
    if service.end().is_some()
        && service.process.room() >= io::MAX_OUT_DOWN.submissions.saturating_mul(2)
        && let Some(pipe) = &mut service.box_pipe
        && pipe.state != BoxPipeState::Closing
    {
        pipe.state = BoxPipeState::Closing;
        for entity in [pipe.requests, pipe.answers] {
            service.process.down(io::Request::Close { entity });
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
    for _ in 0..service.process.told_count() {
        if box_notes_full(service) {
            break;
        }
        let event = service.process.told().pop().expect("io event count");
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
        if owner == Some(Token::new(1)) {
            if let io::Event::Closed { .. } = event {
                service.process.signals_closed();
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
            worker.up(service.process.env().now, event);
        } else if let Some(engine) = &mut service.engine {
            engine.up(service.process.env().now, event);
        }
    }
}

fn startup_operations(io: &io::Limits, worker_operations: u32) -> Option<u32> {
    temper_engine_service::system::startup_operations(io)?.checked_add(worker_operations)
}

fn box_notes_full(service: &Service) -> bool {
    let Some(engine) = &service.engine else { return false };
    let Some(pipe) = &service.box_pipe else { return false };
    pipe.state == BoxPipeState::Open
        && engine.notes_room() == 0
        && match service.process.next_event() {
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
    match event {
        io::Event::Stream { owner, up } => match up {
            Up::Bytes(bytes) => {
                assert!(owner == pipe.answers, "box read owner");
                pipe.reading = false;
                engine.box_read(&bytes);
            }
            Up::Room => {
                assert!(owner == pipe.requests, "box write owner");
                pipe.output = BoxOutput::Room;
            }
            Up::End | Up::Failed(_) => {
                pipe.state = BoxPipeState::Ended;
                engine.box_closed();
            }
        },
        io::Event::Failed { .. } | io::Event::Closed { .. } => {
            pipe.state = BoxPipeState::Ended;
            engine.box_closed();
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
}
fn box_io(service: &mut Service) {
    use skein_lib::stream::{Delimiter, Down, Read};
    let Some(pipe) = &mut service.box_pipe else { return };
    if pipe.state != BoxPipeState::Open {
        return;
    }
    if !pipe.reading && service.process.room() >= io::MAX_OUT_DOWN.submissions && service.process.io().takes() {
        service.process.down(io::Request::Stream {
            stream: pipe.answers,
            down: Down::Demand { read: Read::Scan { until: Delimiter::LF, max: pipe.read_bytes }, room: 0 },
        });
        pipe.reading = true;
    }
    let engine = service.engine.as_mut().expect("box engine");
    if engine.box_writing() && service.process.room() >= io::MAX_OUT_DOWN.submissions && service.process.io().takes() {
        if pipe.output == BoxOutput::Room {
            let line = engine.box_write().expect("box line ready");
            service.process.down(io::Request::Stream { stream: pipe.requests, down: Down::Send(line) });
            pipe.output = BoxOutput::Idle;
        } else if pipe.output == BoxOutput::Idle {
            // Full output credit waits for the preceding line to leave io.
            service.process.down(io::Request::Stream {
                stream: pipe.requests,
                down: Down::Demand { read: Read::Nothing, room: service.process.env().limits.output },
            });
            pipe.output = BoxOutput::Waiting;
        }
    }
}

impl Service {
    /// Start-time endpoints only; the browser and the issuer never choose a socket.
    pub fn configure_google_endpoints(&mut self, runtime: clients::fetch::Runtime) {
        self.engine.as_mut().expect("composition engine").configure_google_endpoints(runtime);
    }
}

fn drain_worker_notes(worker: &mut Worker, sink: &mut Queue<crate::Note>) {
    for _ in 0..sink.room() {
        let Some(note) = worker.notes().pop() else { break };
        sink.push(note.into());
    }
}

impl Service {
    fn join_end(&mut self) {
        self.ending = match self.engine.as_ref().expect("composition engine").end() {
            Some(temper_engine_service::End::Drained) => match &self.worker {
                Some(worker) => match worker.refused() {
                    Some(problems) => Some(End::Worker(crate::worker_alone::End::Stopped(
                        crate::worker_alone::Stop::Start(problems.clone()),
                    ))),
                    None => Some(End::Engine(temper_engine_service::End::Drained)),
                },
                None => Some(End::Engine(temper_engine_service::End::Drained)),
            },
            Some(end) => Some(End::Engine(end.clone())),
            None => None,
        };
    }
}

impl Service {
    pub fn engine_component(&mut self) -> &mut temper_engine_service::Engine {
        self.engine.as_mut().expect("composition engine")
    }
}
pub use temper_engine_service::system::{machine_config, playground_config, socket_host};

#[cfg(test)]
mod box_tests {
    use super::BoxRefusal;
    use crate::tests::fixtures;
    use skein_io::kernel::Fd;
    #[test]
    fn box_adoption_names_each_refusal_and_keeps_both_descriptors() {
        for expected in
            [BoxRefusal::Sockets { limit: 1 }, BoxRefusal::Adopted, BoxRefusal::NoEngine, BoxRefusal::ZeroLine]
        {
            let mut service = fixtures::started();
            let mut line_bytes = 256;
            match expected {
                BoxRefusal::Sockets { limit } => service.process.env_mut().limits.sockets = limit,
                BoxRefusal::Adopted => service.adopt_box(Fd::new(3), Fd::new(4), line_bytes).unwrap(),
                BoxRefusal::NoEngine => service.engine = None,
                BoxRefusal::ZeroLine => line_bytes = 0,
            }
            let before = (service.process.io().sockets(), service.process.submission_count());
            assert_eq!(service.adopt_box(Fd::new(5), Fd::new(6), line_bytes), Err(expected));
            assert_eq!((service.process.io().sockets(), service.process.submission_count()), before);
            // In particular, no close or adoption was submitted for either fd.
        }
    }
}
