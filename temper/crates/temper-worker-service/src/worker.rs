//! One worker root and its channels, with values below the domain (jig's
//! protocol/service.md, 6).
pub(crate) mod credentials;
mod diagnostics;
mod jobs;
#[cfg(test)]
#[path = "tests/worker_jobs.rs"]
mod tests;
pub mod translate;
use crate::worker_job_ending::JobEnding;
use crate::{
    identities::IdentityTable,
    settings::{Agent, Files},
    worker_route, worker_settings,
};
use alloc::boxed::Box;
use credentials::{Scheme, Table, Value};
use jig_domain_host as hub;
use jig_domain_link::SourceName;
use jig_protocol_config::{Problems, What};
use jig_protocol_worker_link as link;
use skein_channel::{Lower, LowerEvent};
use skein_io::{self as io, kernel::Fd};
use skein_lib::{Decimal, Env, List, Map, Queue, Time, Token, Wall, Writer, stream};
use smith_host_domain as smith;
use smith_host_protocol as host;
use temper_worker_domain as root;
use worker_settings::problem;

/// The worker's root directory and pinned agent, prepared by the shell
/// (domain/worker/README.md, 5).
#[derive(Debug)]
pub struct WorkerRuntime {
    pub root: Fd,
    pub path: Box<[u8]>,
    pub agent: jig_protocol_config::Agent,
}
/// The filesystem's absolute path envelope, including the worker root.
pub(crate) const WORKER_PATH_BYTES: usize = 4096;
/// smith-host-protocol's `Process::new` caps `channel_below` at 16 entries.
const PROCESS_CHANNEL_REQUESTS: u32 = 16;
#[derive(Debug)]
pub struct Worker {
    pub(super) root: root::Domain,
    pub(crate) env: Env<root::Limits>,
    pub(crate) git: crate::worker_git::Git,
    jobs: jobs::Jobs,
    agent: Agent,
    sources: crate::program_sources::Component,
    source_files: crate::program_sources::FileStage,
    link: link::Component,
    link_out: Queue<link::Request>,
    out: Queue<root::Request>,
    pub(crate) events: Queue<root::Event>,
    /// What the link brought from the engine, in its order, until the
    /// root takes it or holds it at the watermark.
    pub(crate) engine: Queue<root::Event>,
    /// The engine's new work that waits at the watermark, and the records
    /// behind it that may not pass it (W5), in their order
    /// (`domain/worker/README.md`, 3.1; skein's `programming-model.md`, 7.1, 7.2).
    pub(crate) held: Queue<root::Event>,
    process_events: Queue<host::ProcessEvent>,
    processes: Map<Token, host::Process>,
    owners: Map<Token, Token>,
    entities: Map<Token, Token>,
    retired: List<Token>,
    pub(crate) values: Table,
    pub(crate) identities: IdentityTable,
    pub(crate) undecodable_values: u64,
    payload: temper_protocol_worker_wire::v1::Limits,
    pub(super) io: Queue<io::Request>,
    connection: Connection,
    notes: Queue<crate::worker_alone::Note>,
    socket: Option<Token>,
    prefix: u64,
    next: u64,
    program: Box<[u8]>,
    configuration: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    environment: Box<[Box<[u8]>]>,
    directory: Option<Fd>,
    path: Box<[u8]>,
    root_close: Option<Token>,
    pub(super) submitted: Queue<io::kernel::Submit>,
    pub(super) started: bool,
    closed: bool,
    stopping: bool,
    pub(super) aborted: Option<()>,
}
// Socket workers own their endpoint and service ending (protocol/cluster.md).
#[derive(Debug)]
enum Connection {
    InProcess(io::Name),
    Socket(Box<Remote>),
}
#[derive(Debug)]
struct Remote {
    address: core::net::SocketAddr,
    worker: Box<[u8]>,
    end: Option<crate::worker_alone::End>,
    hello_pending: bool,
    linked: bool,
    loss_pending: bool,
    last_refusal: Option<(core::net::SocketAddr, crate::Refusal)>,
}
impl Worker {
    pub fn new(
        settings: &crate::settings::Worker,
        agent: &Agent,
        limits: &worker_settings::Limits,
        files: &Files,
        runtime: WorkerRuntime,
        seed: u64,
    ) -> Result<Self, Problems> {
        if runtime.path.len() > WORKER_PATH_BYTES {
            return Err(problem(b"root", What::Bound));
        }
        let prepared = worker_settings::prepare(settings, agent, limits)?;
        let config = &settings.jig;
        let secret = files.contents(&config.secret).ok_or(problem(b"secret", What::Unreadable))?;
        let bytes = secret.strip_suffix(b"\n").unwrap_or(secret);
        if bytes.len() > usize::try_from(config.limits.link.secret).expect("secret bound") {
            return Err(problem(b"secret", What::Bound));
        }
        let link = link::Component::new(&prepared.link, worker_settings::credential(settings, secret), seed)
            .or(Err(problem(b"limits", What::Bound)))?;
        let slots = config.slots;
        let git = crate::worker_git::Git::new(settings, limits, &runtime.path, runtime.root, seed)?;
        let operations = worker_settings::operations(git.component.limits()).ok_or(problem(b"limits", What::Bound))?;
        Ok(Self {
            git,
            jobs: jobs::Jobs::new(slots),
            sources: crate::program_sources::Component::new(config.limits.hub.detail_bytes),
            root: worker_root(&prepared.root),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: prepared.root },
            agent: Agent {
                run_limits: agent.run_limits,
                tool_payload: agent.tool_payload,
                programs: agent.programs.clone(),
                endpoints: agent.endpoints.clone(),
                limits: agent.limits,
                bounds: agent.bounds.clone(),
                host_limits: agent.host_limits,
            },
            source_files: crate::program_sources::FileStage::new(crate::tokens::worker(0).expect("worker prefix")),
            link,
            link_out: Queue::uncapped(),
            out: Queue::uncapped(),
            events: Queue::uncapped(),
            engine: Queue::uncapped(),
            held: Queue::with_capacity(
                jig_protocol_worker_wire::sizes::engine_cap(&prepared.link.sizes)
                    .ok_or(problem(b"limits", What::Bound))?
                    .frames
                    .checked_add(3)
                    .ok_or(problem(b"limits", What::Bound))?,
            ),
            process_events: Queue::uncapped(),
            processes: Map::with_capacity(slots),
            owners: Map::with_capacity(slots),
            entities: Map::with_capacity(slots.checked_mul(4).expect("process entities")),
            retired: List::with_capacity(slots),
            values: Table::new(
                config.limits.core.accounts.accounts.checked_add(1).expect("account slots"),
                worker_settings::credential_bytes(config.limits.token),
            ),
            identities: IdentityTable::new(limits.peer.forges),
            undecodable_values: 0,
            payload: prepared.payload,
            io: Queue::uncapped(),
            connection: match prepared.address {
                io::Addr::InProcess(name) => Connection::InProcess(name),
                io::Addr::Ip(address) => Connection::Socket(Box::new(Remote {
                    address,
                    worker: config.name.clone(),
                    end: None,
                    hello_pending: false,
                    linked: false,
                    loss_pending: false,
                    last_refusal: None,
                })),
            },
            notes: Queue::uncapped(),
            socket: None,
            prefix: prepared.prefix,
            next: 2,
            program: runtime.agent.program,
            configuration: runtime.agent.configuration,
            arguments: runtime.agent.arguments,
            environment: Box::default(),
            directory: Some(runtime.root),
            path: runtime.path,
            root_close: None,
            submitted: Queue::with_capacity(operations),
            started: false,
            closed: false,
            stopping: false,
            aborted: None,
        })
    }
    pub fn reuse_workspaces(&mut self) {
        self.root.reuse_workspaces();
        self.git.component.keep_workspaces();
    }

    /// Give each agent the dump directory (temper/docs/design/draft/dogfood-hacks.md, 2.13).
    pub fn dump_llm(&mut self, directory: &[u8]) {
        self.environment = Box::new([skein_lib::bytes::concat(&[b"TEMPER_INTERNAL_DUMP_LLM=", directory])]);
    }

    pub fn configure_program_sources(
        &mut self,
        resources: crate::program_sources::Resources,
    ) -> Result<(), crate::program_sources::Resources> {
        self.sources.configure(resources)
    }
    #[must_use]
    pub fn stopping(&self) -> bool {
        self.stopping
    }
    pub(crate) fn end(&self) -> Option<&crate::worker_alone::End> {
        match &self.connection {
            Connection::Socket(remote) => remote.end.as_ref(),
            Connection::InProcess(_) => None,
        }
    }
    pub(crate) fn set_end(&mut self, end: crate::worker_alone::End) {
        let Connection::Socket(remote) = &mut self.connection else { unreachable!("worker alone has a socket") };
        remote.end = Some(end);
    }
    pub fn notes(&mut self) -> &mut Queue<crate::worker_alone::Note> {
        &mut self.notes
    }
    /// Offer a domain entrance and dispatch its hand-outs in order.
    pub fn offer(&mut self, event: root::Event) {
        self.step(event);
    }
    /// Take the component's routed domain entrances.
    pub fn events(&mut self) -> &mut Queue<root::Event> {
        &mut self.events
    }
    /// Whether the git component's start has completed successfully.
    #[must_use]
    pub const fn git_ready(&self) -> bool {
        self.git.ready
    }
    /// Whether shutdown keeps the worker's checkouts.
    #[must_use]
    pub fn keeps_workspaces(&self) -> bool {
        self.git.component.keeps_workspaces()
    }
    /// The link's ordered entrances awaiting the worker root.
    pub fn engine_events(&mut self) -> &mut Queue<root::Event> {
        &mut self.engine
    }
    /// The domain owner of an io process request.
    #[must_use]
    pub fn process_owner(&self, owner: Token) -> Option<Token> {
        self.owners.get(&owner).copied()
    }
    /// The worker domain, inspected without exposing component fields.
    #[must_use]
    pub const fn domain(&self) -> &root::Domain {
        &self.root
    }
    /// One process: the engine has drained, so cancel at once, git closed first.
    pub fn stop(&mut self) {
        self.shutdown(false);
    }
    /// A worker alone keeps git open for its runs' terminal saves and pushes.
    pub fn drain(&mut self) {
        self.shutdown(true);
    }
    fn shutdown(&mut self, drain: bool) {
        if !self.stopping {
            self.stopping = true;
            if !drain {
                self.git_request(temper_worker_protocol_git::Request::Shutdown);
            }
            self.events.push(root::Event::Shutdown { drain });
        }
    }
    /// Whether every agent process the worker spawned has closed, one still
    /// spawning among them (protocol/README.md, section 6).
    #[must_use]
    pub fn agents_ended(&self) -> bool {
        self.processes.is_empty()
    }
    /// End immediately, dropping domain work and aborting every owned IO entity.
    pub fn abort(&mut self) {
        if self.aborted.is_some() {
            return;
        }
        self.aborted = Some(());
        self.stopping = true;
        self.jobs_fire(self.env.now, true);
        self.git.component.abort(&mut self.git.out);
        self.git_drain();
        for _ in 0..self.events.len() {
            let _event = self.events.pop();
        }
        for _ in 0..self.engine.len() {
            let _event = self.engine.pop();
        }
        for _ in 0..self.held.len() {
            let _event = self.held.pop();
        }
        for _ in 0..self.out.len() {
            let _request = self.out.pop();
        }
        self.link_down(self.env.now, link::Event::Abort);
        if let Some(at) = self.sources.next_deadline() {
            let settled = self.sources.fire(at, &mut self.io);
            self.answered(settled);
        }
        for (entity, _) in &self.entities {
            self.io.push(io::Request::Abort { entity: *entity });
        }
    }
    #[must_use]
    pub fn accepts(&self, event: &io::Event) -> bool {
        if let Some(owner) = event.owner()
            && self.jobs.accepts(owner)
        {
            return true;
        }
        if self.git.component.accepts(event) {
            return true;
        }
        match event.owner() {
            Some(owner) => {
                self.sources.owns(owner)
                    || owner == Token::new(self.prefix)
                    || self.owners.get(&owner).is_some()
                    || self.entities.get(&owner).is_some()
            }
            None => false,
        }
    }
    /// Entries waiting between the worker's components. The engine entrance
    /// and capped held queue are excluded so new work cannot hold itself at the watermark.
    fn backlog(&self) -> u32 {
        self.events
            .len()
            .saturating_add(self.out.len())
            .saturating_add(self.process_events.len())
            .saturating_add(self.io.len())
            .saturating_add(self.root.backlog())
    }
    /// Retry held work first, then take the engine's answers past it (W5).
    fn take_engine(&mut self) {
        for _ in 0..self.held.len().min(self.env.limits.watermark) {
            let Some(event) = self.held.iter().next() else { break };
            if root::is_new_work(event) && !root::admits(&self.env.limits, self.backlog()) {
                break;
            }
            let event = self.held.pop().expect("held head inspected");
            self.step(event);
        }
        for _ in 0..self.engine.len().min(self.env.limits.watermark) {
            let Some(event) = self.engine.iter().next() else { break };
            if self.held.room() == 0 && !root::passes(event) {
                break;
            }
            let hold = if self.held.is_empty() {
                root::is_new_work(event) && !root::admits(&self.env.limits, self.backlog())
            } else {
                !root::passes(event)
            };
            let event = self.engine.pop().expect("engine head inspected");
            if hold {
                self.held.push(event);
            } else {
                self.step(event);
            }
        }
    }
    pub fn up(&mut self, now: Time, mut event: io::Event) {
        if let Some(owner) = event.owner()
            && self.jobs.accepts(owner)
        {
            self.job_up(event);
            return;
        }
        if self.git.component.accepts(&event) {
            self.git.component.up(now, event, &mut self.git.out);
            self.git_drain();
            return;
        }
        let owner = event.owner().expect("routed owner");
        if self.sources.owns(owner) {
            let settled = self.sources.from_io(event, &mut self.io);
            self.answered(settled);
            return;
        }
        if owner == Token::new(self.prefix) {
            match event {
                io::Event::Connecting { socket, .. } => {
                    self.socket = Some(socket);
                    if self.aborted.is_some() {
                        self.io.push(io::Request::Abort { entity: socket });
                    }
                }
                io::Event::Connected { .. } => {
                    self.link_down(now, link::Event::Dialled { connection: Token::new(self.prefix) });
                }
                io::Event::Failed { .. } => self.link_down(now, link::Event::Unreachable),
                io::Event::Stream { up, .. } => self.link_down(
                    now,
                    link::Event::Lower { connection: Token::new(self.prefix), event: LowerEvent::Read(up) },
                ),
                io::Event::Output { up, .. } => self.link_down(
                    now,
                    link::Event::Lower { connection: Token::new(self.prefix), event: LowerEvent::Write(up) },
                ),
                io::Event::Closed { .. } => {
                    self.socket = None;
                    self.link_down(now, link::Event::Gone { connection: Token::new(self.prefix) });
                }
                io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Window { .. }
                | io::Event::Resized
                | io::Event::Shutdown { .. } => {}
            }
        } else {
            let agent = match self.owners.get(&owner) {
                Some(agent) => *agent,
                None => *self.entities.get(&owner).expect("owned pipe"),
            };
            if let io::Event::Spawned { child, pipes, .. } = &event {
                self.entities.insert(*child, agent).expect("child per process");
                if self.aborted.is_some() {
                    self.io.push(io::Request::Abort { entity: *child });
                }
                for pipe in pipes {
                    self.entities.insert(*pipe, agent).expect("three pipes per process");
                    if self.aborted.is_some() {
                        self.io.push(io::Request::Abort { entity: *pipe });
                    }
                }
            }
            let process = self.processes.get_mut(&agent).expect("owned process");
            if self.owners.get(&owner).is_some() {
                owner_of(&mut event, process.io_owner());
            }
            process.from_io(now, event, &mut self.process_events, &mut self.io);
        }
    }
    fn link_down(&mut self, now: Time, event: link::Event) {
        self.link.down(now, event, &mut self.link_out);
        self.drain_link(now);
    }
    fn drain_link(&mut self, now: Time) {
        // One link invocation's finite batch; dispatch does not invoke the link.
        let mut fault = false;
        for _ in 0..self.link_out.len() {
            let request = self.link_out.pop().expect("link output");
            let waiting = self.engine.len();
            self.link(now, request);
            if self.engine.len() > waiting
                && let Some(event) = self.engine.iter().next_back()
                && root::is_new_work(event)
                && self.held.room() <= waiting
            {
                // Reject only this new record, preserving the entrance's order.
                for _ in 0..waiting {
                    let event = self.engine.pop().expect("earlier engine records");
                    self.engine.push(event);
                }
                let _dropped = self.engine.pop().expect("new record past the entrance cap");
                fault = true;
            }
        }
        if fault {
            // The whole invocation is dispatched before the fault invokes the link.
            self.link.down(now, link::Event::SourceFault, &mut self.link_out);
            for _ in 0..self.link_out.len() {
                let request = self.link_out.pop().expect("source fault output");
                self.link(now, request);
            }
        }
        self.values.expire(now);
    }
    #[expect(clippy::too_many_lines, reason = "the worker link dispatch keeps every protocol request arm together")]
    pub fn link(&mut self, _now: Time, request: link::Request) {
        match request {
            link::Request::Ready => {
                if let Connection::Socket(remote) = &mut self.connection {
                    remote.hello_pending = true;
                }
                self.engine.push(root::Event::Ready);
            }
            link::Request::Lost => {
                match &mut self.connection {
                    Connection::InProcess(_) => {}
                    Connection::Socket(remote) => {
                        remote.hello_pending = false;
                        remote.loss_pending |= remote.linked;
                        remote.linked = false;
                    }
                }
                self.engine.push(root::Event::Lost);
            }
            link::Request::GracePassed => self.engine.push(root::Event::GracePassed),
            link::Request::Resend { run, attempt, turn } => {
                self.engine.push(root::Event::Resend { run, attempt, turn });
            }
            link::Request::RelayCancelled { run, attempt, call } => {
                self.engine.push(root::Event::RelayCancelled { run, attempt, call });
            }
            link::Request::Down { down: hub::LinkDown::Assign { assignment }, parts }
                if assignment.assignment.workspace.is_some() =>
            {
                let items = worker_route::items(&assignment, &parts, &self.payload);
                self.engine.push(root::Event::Assign { assignment, items });
            }
            link::Request::Down { down: hub::LinkDown::AssignJob { job }, parts } => {
                let items = worker_route::job_items(&job, &parts, &self.payload);
                let spec = worker_route::job_spec(&job.spec, &self.payload);
                self.engine.push(root::Event::AssignJob { job, items, spec });
            }
            link::Request::Down { down, .. } => self.engine.push(root::Event::Link(down)),
            link::Request::Value { account, generation, expires, value } => {
                let value = match value {
                    link::GrantValue::Credential { scheme: link::Scheme::Bearer, token } => {
                        Value { token, scheme: Scheme::Bearer }
                    }
                    link::GrantValue::Credential { scheme: link::Scheme::Key, token } => {
                        Value { token, scheme: Scheme::Key }
                    }
                    // An application's grant is a configured git identity.
                    link::GrantValue::Application { secret } => {
                        self.identity(account, generation, &secret, expires);
                        return;
                    }
                };
                self.values.granted(account, generation, value, expires).expect("wire values fit tables");
            }
            link::Request::Dial => {
                let addr = match &self.connection {
                    Connection::InProcess(name) => io::Addr::InProcess(*name),
                    Connection::Socket(remote) => io::Addr::Ip(remote.address),
                };
                self.io.push(io::Request::Connect { owner: Token::new(self.prefix), addr });
            }
            link::Request::Lower { lower, .. } => {
                if let Some(socket) = self.socket {
                    self.io.push(match lower {
                        Lower::Read(down) => io::Request::Stream { stream: socket, down },
                        Lower::Write(down) => io::Request::Output { stream: socket, down },
                        Lower::FinishWrite => io::Request::Stream { stream: socket, down: stream::Down::Finish },
                    });
                }
            }
            link::Request::Close { .. } => {
                if let Some(entity) = self.socket {
                    self.io.push(io::Request::Close { entity });
                }
            }
            link::Request::Abort { .. } => {
                if let Some(entity) = self.socket {
                    self.io.push(io::Request::Abort { entity });
                }
            }
            link::Request::Closed => self.closed = true,
            link::Request::Refused { reason, text } => {
                self.notes.push(crate::worker_alone::Note::OpeningRefused { reason, text });
                if let Connection::Socket(remote) = &mut self.connection {
                    remote.last_refusal = Some((remote.address, crate::Refusal::Peer(reason)));
                }
            }
            link::Request::Fact { why } => {
                match &mut self.connection {
                    Connection::InProcess(_) => {}
                    Connection::Socket(remote) => {
                        if remote.loss_pending {
                            remote.loss_pending = false;
                            self.notes.push(crate::worker_alone::Note::Lost {
                                worker: remote.worker.clone(),
                                address: Box::new(remote.address),
                                why,
                            });
                        }
                    }
                }
                let reason = match why {
                    link::Loss::Closed(skein_channel::Closed::RefusedPeer(code)) => Some(crate::Refusal::Peer(code)),
                    link::Loss::Closed(skein_channel::Closed::RefusedHere(
                        jig_protocol_worker_link::REFUSED_LIMITS,
                    )) => Some(crate::Refusal::Terms),
                    link::Loss::Closed(skein_channel::Closed::RefusedHere(
                        jig_protocol_worker_link::REFUSED_FRAMING,
                    )) => Some(crate::Refusal::Malformed),
                    link::Loss::Closed(
                        skein_channel::Closed::RefusedHere(_)
                        | skein_channel::Closed::Owner
                        | skein_channel::Closed::Framing
                        | skein_channel::Closed::Stream
                        | skein_channel::Closed::Truncated,
                    )
                    | link::Loss::Silence
                    | link::Loss::Stall
                    | link::Loss::Handshake
                    | link::Loss::Full
                    | link::Loss::Busy
                    | link::Loss::Fault
                    | link::Loss::Unreachable => None,
                };
                if let Some(reason) = reason
                    && let Connection::Socket(remote) = &mut self.connection
                    && remote.last_refusal != Some((remote.address, reason))
                {
                    remote.last_refusal = Some((remote.address, reason));
                    self.notes.push(crate::worker_alone::Note::Refused {
                        worker: remote.worker.clone(),
                        address: Box::new(remote.address),
                        reason,
                    });
                }
            }
        }
    }
    fn identity(&mut self, account: u32, generation: u64, secret: &skein_lib::Secret, expires: Time) {
        let value = worker_route::identity(secret, &self.payload);
        let undecodable = value.is_none();
        let unfit = self.identities.granted(account, generation, value, expires).is_err();
        if undecodable || unfit {
            self.undecodable_values = self.undecodable_values.saturating_add(1);
        }
    }
    pub(super) fn step(&mut self, event: root::Event) {
        root::step(&mut self.root, &self.env, event, &mut self.out);
        self.drain_root();
    }
    /// Follows its own pushes so an entrance's requests, its hand-outs answered,
    /// are dispatched before the next entrance (W3; `domain/worker/README.md`, 3.1).
    /// Within one step, hand-offs drain to their end (skein's
    /// `programming-model.md`, 2): an entry point's chain is finite, from a
    /// capability to the hub, the hub to a capability, at most once more back
    /// and forth.
    fn drain_root(&mut self) {
        while let Some(request) = self.out.pop() {
            self.route(request);
        }
    }
    fn run_answer(
        &mut self,
        run: Token,
        attempt: Token,
        answer: hub::RunAnswer,
        work: &root::wire::Work,
        preparation: Option<&root::wire::Preparation>,
    ) {
        let parts = worker_route::answer_parts(&answer, work, preparation);
        self.link_down(self.env.now, link::Event::Up { up: hub::LinkUp::Answer { run, attempt, answer }, parts });
    }
    fn job_answer(
        &mut self,
        run: Token,
        attempt: Token,
        answer: hub::JobEnding,
        work: &root::wire::Work,
        preparation: Option<&root::wire::Preparation>,
    ) {
        let kind = self.root.job_kind(run, attempt);
        let why = self.root.job_why(run, attempt);
        let ending = match &answer {
            hub::JobEnding::Failed { failure, detail, .. } => {
                Some(JobEnding::Failed { failure: *failure, detail: detail.clone() })
            }
            hub::JobEnding::Refused(reason) => Some(JobEnding::Refused(reason.clone())),
            hub::JobEnding::Finished { .. } => None,
        };
        if let Some((kind, verb)) = kind
            && let Some(ending) = ending
        {
            let why = why.expect("a failed or refused job keeps its operator evidence until taken");
            self.notes.push(crate::worker_alone::Note::WorkerJob {
                run: run.raw(),
                attempt: attempt.raw(),
                kind,
                verb,
                ending,
                why,
            });
        }
        let parts = worker_route::job_answer_parts(&answer, work, preparation);
        self.link_down(self.env.now, link::Event::Up { up: hub::LinkUp::JobAnswer { run, attempt, answer }, parts });
    }
    fn start_source(
        &mut self,
        owner: Token,
        name: SourceName,
        operation: jig_domain_host::sources::Operation,
        deadline: Time,
    ) {
        let io_owner = Token::new(self.prefix.checked_add(self.next).expect("source owner range"));
        self.next = self.next.checked_add(1).expect("source owner sequence");
        assert!(self.next < (1_u64 << 48_u32), "worker owner range");
        let root = self.directory.expect("worker root");
        let settled = self.sources.start(
            owner,
            io_owner,
            name,
            operation,
            root,
            deadline,
            &mut self.io,
            self.source_files.requests(),
        );
        self.answered(settled);
    }
    /// A list launch by the run's form (protocol/README.md, section 5).
    fn list_launch(&self) -> Option<host::ListLaunch> {
        let arguments: Box<[Box<[u8]>]> = match self.arguments.as_ref() {
            [internal, agent, configuration]
                if internal.as_ref() == b"internal"
                    && agent.as_ref() == b"agent"
                    && configuration.starts_with(b"/") =>
            {
                Box::new([
                    Box::from(b"internal".as_slice()),
                    Box::from(b"agent-list".as_slice()),
                    configuration.clone(),
                ])
            }
            [configuration] if configuration.starts_with(b"/") && *configuration == self.configuration => {
                Box::new([configuration.clone()])
            }
            _ => return None,
        };
        Some(host::ListLaunch {
            program: self.program.clone(),
            arguments,
            environment: self.environment.clone(),
            root: self.directory.expect("prepared worker root"),
            directory: Box::from(b".".as_slice()),
        })
    }
    fn read_list(&mut self, read: u64, endpoint: u32, grant: jig_domain_link::Grant) {
        let io_owner = Token::new(self.prefix.checked_add(self.next).expect("source owner range"));
        self.next = self.next.checked_add(1).expect("source owner sequence");
        assert!(self.next < (1_u64 << 48_u32), "worker owner range");
        let credential = list_credential(&self.values, grant, self.env.now);
        let deadline = self.env.now.saturating_add(self.env.limits.host.source_timeout);
        let launch = self.list_launch();
        self.sources.list_launch(launch);
        let settled = self.sources.read(read, io_owner, endpoint, credential, deadline, &mut self.io);
        self.answered(settled);
    }
    pub(crate) fn route(&mut self, request: root::Request) {
        match request {
            root::Request::ReadList { read, endpoint, grant } => self.read_list(read, endpoint, grant),
            root::Request::CancelRead { read } => {
                let settled = self.sources.cancel_read(read, &mut self.io);
                self.answered(settled);
            }
            root::Request::Source { owner, name, operation, deadline } => {
                self.start_source(owner, name, operation, deadline);
            }
            root::Request::SourceFault => self.link_down(self.env.now, link::Event::SourceFault),
            root::Request::Hello(hello) => {
                self.link_down(self.env.now, link::Event::Hello { hello });
                if let Connection::Socket(remote) = &mut self.connection
                    && remote.hello_pending
                {
                    remote.hello_pending = false;
                    remote.last_refusal = None;
                    remote.linked = true;
                    remote.loss_pending = false;
                    self.notes.push(crate::worker_alone::Note::Linked {
                        worker: remote.worker.clone(),
                        address: Box::new(remote.address),
                    });
                }
            }
            root::Request::Link(up) => self.link_down(self.env.now, link::Event::Up { up, parts: Box::default() }),
            root::Request::Answer { run, attempt, answer, work, preparation } => {
                self.run_answer(run, attempt, answer, &work, preparation.as_ref());
            }
            root::Request::JobAnswer { run, attempt, answer, work, preparation } => {
                self.job_answer(run, attempt, answer, &work, preparation.as_ref());
            }
            root::Request::Io { owner, op, deadline } => {
                let value = match crate::worker_git::identity(&op) {
                    Some(number) => self.identities.newest(number, self.env.now),
                    None => None,
                };
                match crate::worker_git::pick(value, &op) {
                    Ok(identity) => {
                        self.git_request(temper_worker_protocol_git::Request::Io { owner, op, deadline, identity });
                    }
                    Err(done) => {
                        self.git.out.events.push(temper_worker_protocol_git::Event::Done { owner, done });
                        self.git_drain();
                    }
                }
            }
            root::Request::CancelIo { owner } => {
                self.git_request(temper_worker_protocol_git::Request::CancelIo { owner });
            }
            root::Request::Job { owner, op } => {
                self.job_request(owner, op);
            }
            root::Request::CancelRelay { run, attempt, call } => {
                self.link_down(self.env.now, link::Event::CancelRelay { run, attempt, call });
            }
            root::Request::Stop => {
                self.git_request(temper_worker_protocol_git::Request::Shutdown);
                self.link_down(self.env.now, link::Event::Stop);
            }
            root::Request::Abandon => {
                self.notes.push(crate::worker_alone::Note::StopBoundPassed { abandoned: self.root.abandoned() });
                self.abort();
                if let Connection::Socket(remote) = &mut self.connection {
                    remote.end = Some(crate::worker_alone::End::Aborted);
                }
            }
            root::Request::Process(request) => self.process(*request),
            root::Request::Decode { agent, run, start } => {
                let event = worker_route::decoded(agent, run, *start, &self.agent);
                root::step(&mut self.root, &self.env, event, &mut self.out);
            }
            root::Request::Encode { owner, value } => {
                let encoded = worker_route::encode(value, &self.agent);
                root::step(&mut self.root, &self.env, root::Event::Encoded { owner, encoded }, &mut self.out);
            }
            root::Request::Fact { owner, body } => {
                let activity = worker_route::activity(&body, &self.agent.limits);
                root::step(&mut self.root, &self.env, root::Event::Activity { owner, activity }, &mut self.out);
            }
        }
    }
    fn process(&mut self, request: smith::process::Request) {
        match request {
            smith::process::Request::Spawn { owner, deadline, .. } => self.process_spawn(owner, deadline),
            smith::process::Request::Send { owner, message, .. } => self.process_send(owner, message),
            smith::process::Request::Signal { owner, signal, .. } => self.process_signal(owner, signal),
            smith::process::Request::Read { owner, .. } => self.process_read(owner),
            smith::process::Request::Wait { .. } => Self::process_wait(),
            smith::process::Request::Reap { .. } => Self::process_reap(),
        }
    }
    fn process_spawn(&mut self, owner: Token, deadline: Time) {
        let mut process = host::Process::new(
            owner,
            &self.agent.limits,
            self.env.limits.agent.detail_bytes,
            self.agent.endpoints.clone(),
        )
        .expect("startup checks process limits");
        let mut spawned = Queue::with_capacity(1);
        process.spawn(
            host::Launch {
                program: self.program.clone(),
                arguments: self.arguments.clone(),
                environment: self.environment.clone(),
                root: self.directory.expect("prepared worker root"),
                directory: b".".as_slice().into(),
            },
            deadline,
            &mut spawned,
        );
        let global = Token::new(self.prefix.checked_add(self.next).expect("worker owner range"));
        self.next = self.next.checked_add(1).expect("worker owner sequence");
        assert!(self.next < (1_u64 << 48_u32), "worker owner range");
        self.owners.insert(global, owner).expect("one owner per process");
        self.processes.insert(owner, process).expect("root bounds processes");
        let mut request = spawned.pop().expect("one spawn");
        if let io::Request::Spawn { owner, .. } = &mut request {
            *owner = global;
        }
        self.io.push(request);
    }
    fn process_send(&mut self, owner: Token, message: smith::Down) {
        let values = match &message {
            smith::Down::Start { start, .. } => start_values(&self.values, start, &self.path, self.env.now),
            smith::Down::Grant { grant } => grant_value(&self.values, *grant, self.env.now),
            smith::Down::Message { .. }
            | smith::Down::Answer { .. }
            | smith::Down::Acknowledge { .. }
            | smith::Down::Cancel
            | smith::Down::Drain => Some(host::Values { paths: Box::default(), credentials: Box::default() }),
        };
        let Some(values) = values else {
            self.events.push(root::Event::Process(Box::new(smith::process::Event::Unsent { owner })));
            return;
        };
        let message = sending(message, &values, &self.agent);
        let process = self.processes.get_mut(&owner).expect("root owns process");
        let token = owner;
        let sent = match message {
            smith::Down::Start { start, window } => {
                process.send_start(start, window, values, token, &mut self.process_events, &mut self.io)
            }
            smith::Down::Grant { grant } => process.send_grant(
                grant,
                values.credentials.into_iter().next().expect("one value"),
                token,
                &mut self.process_events,
                &mut self.io,
            ),
            smith::Down::Message { name, label, text, finish_requested } => process.send_message(
                smith::Message { name, label, text, finish_requested },
                token,
                &mut self.process_events,
                &mut self.io,
            ),
            smith::Down::Answer { call, reply } => {
                process.send_reply(call, reply, token, &mut self.process_events, &mut self.io)
            }
            smith::Down::Acknowledge { turn } => {
                process.send_acknowledge(turn, token, &mut self.process_events, &mut self.io)
            }
            smith::Down::Cancel => process.send_cancel(token, &mut self.process_events, &mut self.io),
            smith::Down::Drain => process.send_drain(token, &mut self.process_events, &mut self.io),
        };
        if sent.is_err() {
            self.events.push(root::Event::Process(Box::new(smith::process::Event::Unsent { owner })));
        }
    }
    fn process_signal(&mut self, owner: Token, signal: smith::Signal) {
        let signal = match signal {
            smith::Signal::Terminate => io::kernel::Signal::Terminate,
            smith::Signal::Kill => io::kernel::Signal::Kill,
        };
        self.processes.get(&owner).expect("root owns process").signal(signal, &mut self.io);
        self.events.push(root::Event::Process(Box::new(smith::process::Event::Signalled { owner })));
    }
    fn process_read(&mut self, owner: Token) {
        self.processes.get_mut(&owner).expect("root owns process").read(&mut self.process_events, &mut self.io);
    }
    // Smith's process adapter keeps exit waits and reaping active.
    fn process_wait() {}
    fn process_reap() {}
    fn answered(&mut self, settled: Option<(Token, jig_domain_host::sources::Answer)>) {
        if let Some((owner, answer)) = settled {
            let event = match answer {
                hub::sources::Answer::Read(outcome) => root::Event::ListRead { read: owner.raw(), outcome },
                answer @ (hub::sources::Answer::Checked { .. } | hub::sources::Answer::SignedOut(_)) => {
                    root::Event::SourceAnswered { owner, answer }
                }
            };
            self.events.push(event);
        }
    }
    pub fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        self.jobs_fire(now, false);
        let settled = self.sources.fire(now, &mut self.io);
        self.answered(settled);
        let settled = self.source_files.step(now, &mut self.sources, &mut self.io, &mut self.submitted);
        self.answered(settled);
        self.started = true;
        if !self.git.started && !self.stopping {
            self.git.started = true;
            self.git.component.start(now, &mut self.git.out);
            self.git_drain();
        }
        if self.git.component.is_due(now) {
            self.git.component.fire(now, &mut self.git.out);
            self.git_drain();
        }
        if self.git.ready {
            self.link.fire(now, &mut self.link_out);
            self.drain_link(now);
        }
        let mut agents = List::with_capacity(self.processes.len());
        for (agent, _) in &self.processes {
            agents.push(*agent).expect("process count");
        }
        for agent in &agents {
            self.processes.get_mut(agent).expect("owned process").fire(now, &mut self.process_events, &mut self.io);
        }
        for _ in 0..self.process_events.len() {
            let event = self.process_events.pop().expect("process events");
            match &event {
                host::ProcessEvent::Unspawned { agent, .. } | host::ProcessEvent::Reaped { agent, .. } => {
                    self.retired.push(*agent).expect("retired processes");
                }
                host::ProcessEvent::Spawned { .. }
                | host::ProcessEvent::Exited { .. }
                | host::ProcessEvent::Channel { .. } => {}
            }
            if self.aborted.is_none()
                && let Some(event) = worker_route::process(event)
            {
                self.events.push(event);
            }
        }
        if self.aborted.is_some() {
            for _ in 0..self.events.len() {
                let _event = self.events.pop();
            }
        }
        for _ in 0..self.events.len().min(self.env.limits.watermark) {
            let Some(event) = self.events.pop() else { break };
            self.step(event);
        }
        self.take_engine();
        // Progress taken in this iteration wins over an alarm due in it.
        if self.aborted.is_none() && self.events.is_empty() && self.root.is_due(now) {
            root::fire(&mut self.root, &self.env, &mut self.out);
            self.drain_root();
        }
        if self.aborted.is_none() && self.root.is_ready() {
            root::resume(&mut self.root, &self.env, &mut self.out);
            self.drain_root();
        }
        self.reclaim_processes();
        for agent in &agents {
            if let Some(process) = self.processes.get_mut(agent) {
                for _ in 0..PROCESS_CHANNEL_REQUESTS {
                    let Some(request) = process.next_channel_down() else { break };
                    process.channel_down(request, &mut self.io);
                }
            }
        }
        self.retired.clear();
        self.root.reclaim();
        self.git.component.reclaim();
        self.close_root();
    }
    fn reclaim_processes(&mut self) {
        for index in 0..self.retired.len() {
            let agent = *self.retired.get(index).expect("retired agent");
            drop(self.processes.remove(&agent));
            let mut owners = List::with_capacity(self.owners.len());
            for (owner, found) in &self.owners {
                if *found == agent {
                    owners.push(*owner).expect("owner count");
                }
            }
            for owner in &owners {
                let _owner = self.owners.remove(owner);
            }
            let mut entities = List::with_capacity(self.entities.len());
            for (owner, found) in &self.entities {
                if *found == agent {
                    entities.push(*owner).expect("entity count");
                }
            }
            for owner in &entities {
                let _entity = self.entities.remove(owner);
            }
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if !self.started || self.aborted.is_some() {
            return self.source_files.next_deadline();
        }
        let link = if self.git.ready { self.link.next_deadline() } else { None };
        let mut next = [
            self.root.next_deadline(),
            link,
            self.values.next_expiry(),
            self.git.component.next_deadline(),
            self.sources.next_deadline(),
            self.source_files.next_deadline(),
            self.jobs.deadline(),
        ]
        .into_iter()
        .flatten()
        .min();
        for (_, process) in &self.processes {
            if let Some(at) = process.next_deadline() {
                next = Some(match next {
                    Some(next) => next.min(at),
                    None => at,
                });
            }
        }
        next
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        !self.started
            || !self.io.is_empty()
            || !self.events.is_empty()
            || !self.engine.is_empty()
            || !self.held.is_empty()
            || !self.process_events.is_empty()
            || !self.submitted.is_empty()
            || self.source_files.work_pending(now)
            || (self.aborted.is_none() && self.root.is_ready())
            || match self.next_deadline() {
                Some(at) => at <= now,
                None => false,
            }
    }
    pub(super) fn closed(&self) -> bool {
        self.closed
            && self.jobs.empty()
            && self.sources.settled()
            && self.source_files.settled()
            && self.processes.is_empty()
            && self.io.is_empty()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.closed() && self.directory.is_none() && self.submitted.is_empty() && self.git.component.is_empty()
    }
    pub(super) fn close_root(&mut self) {
        if self.closed()
            && self.git.component.is_empty()
            && self.root_close.is_none()
            && let Some(fd) = self.directory
        {
            if let Some(parent) = self.sources.take_parent() {
                self.source_files.close_parent(parent);
            }
            self.root_close = Some(Token::new(self.prefix.checked_add(1).expect("root close")));
            self.submitted.push(io::kernel::Submit {
                op: Token::new(self.prefix.checked_add(1).expect("root close")),
                kind: io::kernel::Op::Close { fd },
            });
        }
    }
    pub fn complete(&mut self, now: Time, complete: io::kernel::Complete) -> Result<(), io::kernel::Complete> {
        let complete = match self.source_files.complete(complete) {
            Ok(()) => return Ok(()),
            Err(complete) => complete,
        };
        let complete = match self.job_complete(complete) {
            Ok(()) => return Ok(()),
            Err(complete) => complete,
        };
        if self.git.component.owns(complete.op) {
            self.git.component.complete(now, complete, &mut self.git.out);
            self.git_drain();
            return Ok(());
        }
        if complete.op == Token::new(self.prefix.checked_add(1).expect("root close")) {
            assert!(self.root_close.is_some(), "the worker closes its own descriptor");
            self.directory = None;
            self.root_close = None;
            Ok(())
        } else {
            Err(complete)
        }
    }
    pub fn io(&mut self) -> &mut Queue<io::Request> {
        &mut self.io
    }
    pub fn submitted(&mut self) -> &mut Queue<io::kernel::Submit> {
        &mut self.submitted
    }
    #[must_use]
    pub fn refused(&self) -> Option<&Problems> {
        self.git.refused.as_ref()
    }
    #[must_use]
    pub fn operations(&self) -> u32 {
        worker_settings::operations(self.git.component.limits()).expect("checked worker operations")
    }
}
fn list_credential(table: &Table, grant: jig_domain_link::Grant, now: Time) -> Option<smith_channel::CredentialValue> {
    let scheme = match grant.form {
        jig_domain_link::Form::Credential(jig_domain_link::Scheme::Bearer) => Scheme::Bearer,
        jig_domain_link::Form::Credential(jig_domain_link::Scheme::Key) => Scheme::Key,
        jig_domain_link::Form::Program(_) | jig_domain_link::Form::Application => return None,
    };
    let value = table.value(grant.account, grant.generation, now)?;
    if value.scheme != scheme {
        return None;
    }
    smith_channel::CredentialValue::new(
        &smith_channel::CEILINGS,
        smith_channel::CredentialValueParts {
            scheme: match scheme {
                Scheme::Bearer => smith_channel::Scheme::Bearer,
                Scheme::Key => smith_channel::Scheme::Key,
            },
            bytes: value.token.clone(),
        },
    )
    .ok()
}
pub(crate) fn grant_value(table: &Table, grant: smith::Grant, now: Time) -> Option<host::Values> {
    let value = account(table, grant, now)?;
    Some(host::Values { paths: Box::default(), credentials: Box::new([value]) })
}
fn account(table: &Table, grant: smith::Grant, now: Time) -> Option<smith_protocol_channel::GrantValue> {
    match grant.form {
        smith::Form::Program(name) => return Some(smith_protocol_channel::GrantValue::Program(name.as_bytes().into())),
        smith::Form::Credential(_) => {}
    }
    let value = table.value(grant.account, grant.generation, now)?;
    let actual = match &value.scheme {
        Scheme::Bearer => smith::Form::Credential(smith::Scheme::Bearer),
        Scheme::Key => smith::Form::Credential(smith::Scheme::Key),
    };
    if grant.form != actual {
        return None;
    }
    let token = value.token.expose().into();
    Some(match &value.scheme {
        Scheme::Bearer => smith_protocol_channel::GrantValue::Bearer(token),
        Scheme::Key => smith_protocol_channel::GrantValue::Key(token),
    })
}
fn sending(message: smith::Down, values: &host::Values, agent: &Agent) -> smith::Down {
    match message {
        smith::Down::Start { mut start, window } => {
            for (directory, path) in start.directories.iter_mut().zip(&values.paths) {
                directory.host.clone_from(path);
            }
            smith::Down::Start { start: fit_start(start, agent), window }
        }
        other @ (smith::Down::Grant { .. }
        | smith::Down::Message { .. }
        | smith::Down::Answer { .. }
        | smith::Down::Acknowledge { .. }
        | smith::Down::Cancel
        | smith::Down::Drain) => other,
    }
}

pub(crate) fn fit_start(start: smith::Start, agent: &Agent) -> smith::Start {
    worker_route::fit_start(start, &agent.run_limits, agent.host_limits.charter_bytes)
}

pub(crate) fn start_values(table: &Table, start: &smith::Start, root: &[u8], now: Time) -> Option<host::Values> {
    let mut values = List::with_capacity(u32::try_from(start.grants.len()).ok()?);
    for grant in &start.grants {
        values.push(account(table, *grant, now)?).ok()?;
    }
    let mut paths = List::with_capacity(u32::try_from(start.directories.len()).ok()?);
    for directory in &start.directories {
        let token = Decimal::of(directory.root.raw());
        let capacity = root
            .len()
            .checked_add(b"/workspaces/".len())?
            .checked_add(1)?
            .checked_add(token.as_bytes().len())?
            .checked_add(directory.name.len())?;
        let mut path = Writer::new(capacity);
        path.put(root).ok()?;
        path.put(b"/workspaces/").ok()?;
        path.put(token.as_bytes()).ok()?;
        path.put(b"/").ok()?;
        path.put(&directory.name).ok()?;
        paths.push(path.finish()).ok()?;
    }
    Some(host::Values { paths: paths.into_boxed(), credentials: values.into_boxed() })
}
fn owner_of(event: &mut io::Event, new: Token) {
    match event {
        io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => *owner = new,
        io::Event::Listening { .. }
        | io::Event::Accepted { .. }
        | io::Event::Connecting { .. }
        | io::Event::Connected { .. }
        | io::Event::Stream { .. }
        | io::Event::Output { .. }
        | io::Event::Usage { .. }
        | io::Event::Window { .. }
        | io::Event::Resized
        | io::Event::Shutdown { .. } => {}
    }
}

fn worker_root(limits: &root::Limits) -> root::Domain {
    root::Domain::new(limits, worker_settings::job_kinds())
}

#[cfg(test)]
mod stopping_tests;

#[cfg(test)]
mod merge_tests {
    use jig_domain_link::{JobBound, JobKind};

    /// temper/docs/design/domain/worker/jobs.md.
    #[test]
    fn the_production_worker_declares_the_merge_kind() {
        let mut worker = crate::tests::fixtures::worker();
        let mut out = skein_lib::Queue::uncapped();
        temper_worker_domain::step(&mut worker.root, &worker.env, temper_worker_domain::Event::Ready, &mut out);
        let temper_worker_domain::Request::Hello(hello) = out.pop().expect("worker hello") else {
            panic!("ready declares capabilities");
        };
        assert!(hello.jobs.contains(&JobBound {
            kind: JobKind(3),
            spec: 4_096,
            result: 512,
            deadline: skein_lib::Duration::from_secs(600)
        }));
    }

    /// temper/docs/design/domain/worker/jobs.md.
    #[test]
    fn the_production_worker_declares_the_close_kind() {
        let mut worker = crate::tests::fixtures::worker();
        let mut out = skein_lib::Queue::uncapped();
        temper_worker_domain::step(&mut worker.root, &worker.env, temper_worker_domain::Event::Ready, &mut out);
        let temper_worker_domain::Request::Hello(hello) = out.pop().expect("worker hello") else {
            panic!("ready declares capabilities");
        };
        let close =
            JobBound { kind: JobKind(4), spec: 16_384, result: 512, deadline: skein_lib::Duration::from_secs(600) };
        assert!(hello.jobs.contains(&close));
        let kinds = crate::worker_settings::job_kinds();
        let kind = kinds.last().expect("close declared");
        assert_eq!(kind.bound, close);
        assert_eq!(kind.verb.as_ref(), b"close");
        assert!(kind.delivers);
    }
    #[test]
    fn prepare_is_declared_on_the_worker() {
        let bound =
            JobBound { kind: JobKind(1), spec: 16_384, result: 4_096, deadline: skein_lib::Duration::from_secs(600) };
        let mut worker = crate::tests::fixtures::worker();
        let mut out = skein_lib::Queue::uncapped();
        temper_worker_domain::step(&mut worker.root, &worker.env, temper_worker_domain::Event::Ready, &mut out);
        let temper_worker_domain::Request::Hello(hello) = out.pop().expect("worker hello") else {
            panic!("hello");
        };
        assert_eq!(hello.jobs.first(), Some(&bound));
        let kinds = crate::worker_settings::job_kinds();
        let kind = kinds.first().expect("prepare");
        assert_eq!(kind.bound, bound);
        assert_eq!(kind.verb.as_ref(), b"prepare");
        assert!(kind.delivers);
    }
}
