//! One worker root and its channels, with values below the domain (service.md, 6).
use crate::{Agent, Files, worker_route};
use alloc::boxed::Box;
use jig_protocol_accounts::table::{Scheme, Table, Value};
use jig_protocol_config::{self as config, Path, Problems, What};
use jig_protocol_worker_link as link;
use jig_test_worker as root;
use skein_channel::{Lower, LowerEvent};
use skein_io::{self as io, kernel::Fd};
use skein_lib::{Env, List, Map, Queue, Time, Token, Wall, stream};
use smith_host_domain as smith;
use smith_host_protocol as host;

/// Bounds the shared kernel hand-offs, which stay capped: twice the old
/// small-profile output room of 1,053 entries.
const KERNEL_HANDOFF_ENTRIES: u32 = 2_106;

/// Complete the worker's back-and-forth hand-offs across payload translation
/// before the next entrance (hosts.md, 6.8; service.md, 6).
const PAYLOAD_HAND_OFF_ROUNDS: u32 = 4;

/// Shell-prepared directories, one descriptor for each worker in document order.
#[derive(Debug)]
pub struct WorkerRuntime {
    pub roots: List<Fd>,
}
#[derive(Debug)]
pub struct Worker {
    pub(super) root: root::Domain,
    pub(super) root_steps: u64,
    admission_waits: u64,
    env: Env<root::Limits>,
    agent: Agent,
    sources: crate::program_sources::Component,
    source_events: Queue<jig_domain_host::Event>,
    link: link::Component,
    link_out: Queue<link::Request>,
    out: Queue<root::Request>,
    events: Queue<root::Event>,
    process_events: Queue<host::ProcessEvent>,
    processes: Map<Token, host::Process>,
    owners: Map<Token, Token>,
    entities: Map<Token, Token>,
    retired: List<Token>,
    values: Table,
    application: Table,
    pub(super) io: Queue<io::Request>,
    address: io::Addr,
    socket: Option<Token>,
    prefix: u64,
    next: u64,
    program: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    directory: Option<Fd>,
    root_close: Option<Token>,
    pub(super) submitted: Queue<io::kernel::Submit>,
    pub(super) started: bool,
    closed: bool,
    stopping: bool,
    pub(super) aborted: Option<()>,
}
fn problem(index: u32, member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"workers").child(config::Segment::Index(index)).member(member), what);
    problems
}
impl Worker {
    pub(super) fn new(
        config: config::Worker,
        agent: &Agent,
        files: &Files,
        directory: Fd,
        index: u32,
        seed: u64,
    ) -> Result<Self, Problems> {
        let config::Address::Addr(address) = config.engine else { return Err(problem(index, b"engine", What::Form)) };
        let secret = files.contents(&config.secret).ok_or(problem(index, b"secret", What::Unreadable))?;
        if secret.len() > usize::try_from(config.limits.link.secret).expect("secret bound") {
            return Err(problem(index, b"secret", What::Bound));
        }
        let bodies = &agent.limits.bodies;
        if config.limits.token > bodies.credential_value_bytes {
            return Err(problem(index, b"limits", What::Bound));
        }
        let mut limits = config.limits.hub;
        limits.slots = config.slots;
        limits.source_interval = config.check_interval;
        let root_limits = root::Limits {
            watermark: root::WATERMARK,
            hub: limits,
            agent: agent.host_limits,
            bounds: agent.bounds.clone(),
            stop_bound: config.stop_bound,
            workstreams: Box::default(),
        };
        if !root::valid(&root_limits) {
            return Err(problem(index, b"limits", What::Bound));
        }
        host::Process::new(Token::new(0), &agent.limits, agent.host_limits.detail_bytes, agent.endpoints.clone())
            .or(Err(problem(index, b"agent", What::Bound)))?;
        let link_limits = crate::limits::worker_link(&config);
        let link = link::Component::new(
            &link_limits,
            link::Credential { name: config.name, secret: config::secret(secret.into()), payload: 1 },
            seed,
        )
        .or(Err(problem(index, b"limits", What::Bound)))?;
        let slots = config.slots;
        let prefix = worker_prefix(u64::from(index).checked_add(1).ok_or(problem(index, b"slots", What::Bound))?)
            .ok_or(problem(index, b"slots", What::Bound))?;
        Ok(Self {
            root: root::Domain::new(&root_limits),
            root_steps: 0,
            admission_waits: 0,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: root_limits },
            agent: Agent {
                endpoints: agent.endpoints.clone(),
                limits: agent.limits,
                bounds: agent.bounds.clone(),
                host_limits: agent.host_limits,
            },
            sources: crate::program_sources::Component::new(limits.detail_bytes),
            source_events: Queue::with_capacity(2),
            link,
            link_out: Queue::with_capacity(link::MAX_OUT_DOWN),
            out: Queue::uncapped(),
            events: Queue::uncapped(),
            process_events: Queue::uncapped(),
            processes: Map::with_capacity(slots),
            owners: Map::with_capacity(slots),
            entities: Map::with_capacity(slots.checked_mul(4).expect("process entities")),
            retired: List::with_capacity(slots),
            values: Table::new(
                config.limits.core.accounts.accounts.checked_add(1).expect("account slots"),
                config.limits.token,
            ),
            application: Table::new(
                config.limits.core.accounts.credentials.checked_add(1).expect("credential slots"),
                config.limits.token,
            ),
            io: Queue::uncapped(),
            address,
            socket: None,
            prefix,
            next: 2,
            program: config.agent.program,
            arguments: config.agent.arguments,
            directory: Some(directory),
            root_close: None,
            submitted: Queue::with_capacity(1),
            started: false,
            closed: false,
            stopping: false,
            aborted: None,
        })
    }
    #[cfg(test)]
    pub(super) fn granted(&mut self, account: u32, generation: u64, value: Value, expires: Time) {
        let value = match value.scheme {
            Scheme::Bearer => link::GrantValue::Credential { scheme: link::Scheme::Bearer, token: value.token },
            Scheme::Key => link::GrantValue::Credential { scheme: link::Scheme::Key, token: value.token },
        };
        self.link_out.push(link::Request::Value { account, generation, expires, value });
        self.drain_link(self.env.now);
    }
    pub(super) fn configure_program_sources(
        &mut self,
        resources: crate::program_sources::Resources,
    ) -> Result<(), crate::program_sources::Resources> {
        self.sources.configure(resources)
    }
    pub fn stop(&mut self) {
        if !self.stopping {
            self.stopping = true;
            self.events.push(root::Event::Shutdown);
        }
    }
    /// End immediately, dropping domain work and aborting every owned IO entity.
    pub fn abort(&mut self) {
        if self.aborted.is_some() {
            return;
        }
        self.aborted = Some(());
        self.stopping = true;
        for _ in 0..self.events.len() {
            let _event = self.events.pop();
        }
        for _ in 0..self.out.len() {
            let _request = self.out.pop();
        }
        self.link_down(self.env.now, link::Event::Abort);
        if let Some(at) = self.sources.next_deadline() {
            self.sources.fire(at, &mut self.source_events, &mut self.io);
        }
        for (entity, _) in &self.entities {
            self.io.push(io::Request::Abort { entity: *entity });
        }
    }
    pub(super) fn accepts(&self, event: &io::Event) -> bool {
        match crate::workers::owner(event) {
            Some(owner) => {
                self.sources.owns(owner)
                    || owner == Token::new(self.prefix)
                    || self.owners.get(&owner).is_some()
                    || self.entities.get(&owner).is_some()
            }
            None => false,
        }
    }
    pub(super) fn queue_room(&self) -> u32 {
        self.link_out.capacity().max(KERNEL_HANDOFF_ENTRIES)
    }
    pub(super) fn room(&self) -> bool {
        self.backlog() < u64::from(self.env.limits.watermark)
    }
    #[must_use]
    pub fn backlog(&self) -> u64 {
        // `events` is the waiting room; counting it would let waiting records hold admission shut.
        self.root
            .backlog()
            .saturating_add(u64::from(self.out.len()))
            .saturating_add(u64::from(self.process_events.len()))
            .saturating_add(u64::from(self.io.len()))
    }
    /// Tune this reference worker's pacing without a document key.
    pub fn set_watermark(&mut self, watermark: u32) {
        self.env.limits.watermark = watermark;
    }
    /// Offers holding engine records at admission, for the reference service's worlds.
    #[must_use]
    pub fn admission_waits(&self) -> u64 {
        self.admission_waits
    }
    pub(super) fn up(&mut self, now: Time, mut event: io::Event) {
        let owner = crate::workers::owner(&event).expect("routed owner");
        if self.sources.owns(owner) {
            self.sources.from_io(event, &mut self.source_events, &mut self.io);
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
        for _ in 0..self.link_out.capacity() {
            if self.link_out.is_empty() {
                break;
            }
            match self.link_out.pop().expect("link output") {
                link::Request::Ready => self.events.push(root::Event::Ready),
                link::Request::Lost => self.events.push(root::Event::Lost),
                link::Request::GracePassed => self.events.push(root::Event::GracePassed),
                link::Request::Resend { run, attempt, turn } => {
                    self.events.push(root::Event::Resend { run, attempt, turn });
                }
                link::Request::RelayCancelled { run, attempt, call } => {
                    self.events.push(root::Event::RelayCancelled { run, attempt, call });
                }
                link::Request::Down { down, parts } => {
                    drop(parts);
                    self.events.push(root::Event::Link(down));
                }
                link::Request::Value { account, generation, expires, value } => {
                    let (table, value) = match value {
                        link::GrantValue::Credential { scheme: link::Scheme::Bearer, token } => {
                            (&mut self.values, Value { token, scheme: Scheme::Bearer })
                        }
                        link::GrantValue::Credential { scheme: link::Scheme::Key, token } => {
                            (&mut self.values, Value { token, scheme: Scheme::Key })
                        }
                        link::GrantValue::Application { secret } => {
                            (&mut self.application, Value { token: secret, scheme: Scheme::Bearer })
                        }
                    };
                    table.granted(account, generation, value, expires).expect("wire values fit tables");
                }
                link::Request::Dial => {
                    if self.stopping {
                        self.link.down(now, link::Event::Unreachable, &mut self.link_out);
                    } else {
                        self.io.push(io::Request::Connect { owner: Token::new(self.prefix), addr: self.address });
                    }
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
                link::Request::Fact { .. } => {}
            }
        }
        self.values.expire(now);
        self.application.expire(now);
    }
    pub(super) fn step(&mut self, event: root::Event) {
        self.root_steps = self.root_steps.wrapping_add(1);
        root::step(&mut self.root, &self.env, event, &mut self.out);
        for _ in 0..PAYLOAD_HAND_OFF_ROUNDS {
            for _ in 0..self.out.len() {
                let request = self.out.pop().expect("worker output count");
                match request {
                    root::Request::Source { owner, name, operation, deadline } => {
                        let io_owner = Token::new(self.prefix.checked_add(self.next).expect("source owner range"));
                        self.next = self.next.checked_add(1).expect("source owner sequence");
                        assert!(self.next < (1_u64 << 48_u32), "worker owner range");
                        self.sources.start(
                            owner,
                            io_owner,
                            name,
                            operation,
                            self.directory.expect("worker root"),
                            deadline,
                            &mut self.source_events,
                            &mut self.io,
                        );
                    }
                    root::Request::SourceFault => self.link_down(self.env.now, link::Event::SourceFault),
                    root::Request::Hello(hello) => self.link_down(self.env.now, link::Event::Hello { hello }),
                    root::Request::Link(up) => {
                        self.link_down(self.env.now, link::Event::Up { up, parts: Box::default() });
                    }
                    root::Request::CancelRelay { run, attempt, call } => {
                        self.link_down(self.env.now, link::Event::CancelRelay { run, attempt, call });
                    }
                    root::Request::Stop => self.link_down(self.env.now, link::Event::Stop),
                    root::Request::Process(request) => self.process(*request),
                    root::Request::Decode { agent, run, start } => {
                        let start = worker_route::decode(run, *start, &self.agent);
                        self.root_steps = self.root_steps.wrapping_add(1);
                        root::step(&mut self.root, &self.env, root::Event::Decoded { agent, start }, &mut self.out);
                    }
                    root::Request::Encode { owner, value } => {
                        let encoded = worker_route::encode(value, &self.agent.limits, &self.agent.endpoints);
                        self.root_steps = self.root_steps.wrapping_add(1);
                        root::step(&mut self.root, &self.env, root::Event::Encoded { owner, encoded }, &mut self.out);
                    }
                    root::Request::Fact { owner, body } => {
                        let activity = worker_route::activity(&body, &self.agent.limits);
                        self.root_steps = self.root_steps.wrapping_add(1);
                        root::step(&mut self.root, &self.env, root::Event::Activity { owner, activity }, &mut self.out);
                    }
                }
            }
        }
        assert!(self.out.is_empty(), "each payload hand-out is answered in this iteration");
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
                environment: Box::default(),
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
            smith::Down::Start { start, .. } => start_values(&self.values, &start.grants, self.env.now),
            smith::Down::Grant { grant } => grant_value(&self.values, grant, self.env.now),
            smith::Down::Message { .. }
            | smith::Down::Answer { .. }
            | smith::Down::Acknowledge { .. }
            | smith::Down::Cancel => Some(host::Values { paths: Box::default(), credentials: Box::default() }),
        };
        let Some(values) = values else {
            self.events.push(root::Event::Process(Box::new(smith::process::Event::Unsent { owner })));
            return;
        };
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
                smith_host_domain::Message { name, label, text, finish_requested },
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
    pub(super) fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        if !self.started {
            self.started = true;
        }
        self.sources.fire(now, &mut self.source_events, &mut self.io);
        for _ in 0..self.source_events.len() {
            let event = self.source_events.pop().expect("source terminal");
            if self.aborted.is_none() {
                self.events.push(root::Event::Source(Box::new(event)));
            }
        }
        self.link.fire(now, &mut self.link_out);
        self.drain_link(now);
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
        for _ in 0..self.events.len() {
            if !self.room() {
                for event in &self.events {
                    if let root::Event::Link(_) = event {
                        self.admission_waits = self.admission_waits.saturating_add(1);
                        break;
                    }
                }
                break;
            }
            let Some(event) = self.events.pop() else { break };
            self.step(event);
        }
        if self.aborted.is_none()
            && (self.root.work_pending()
                || match self.root.next_deadline() {
                    Some(deadline) => deadline <= now,
                    None => false,
                })
        {
            self.step(root::Event::Timer);
        }
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
        for agent in &agents {
            if let Some(process) = self.processes.get_mut(agent) {
                while let Some(request) = process.next_channel_down() {
                    process.channel_down(request, &mut self.io);
                }
            }
        }
        self.retired.clear();
        self.root.reclaim();
    }
    pub(super) fn next_deadline(&self) -> Option<Time> {
        if !self.started || self.aborted.is_some() {
            return None;
        }
        let mut next = [
            self.sources.next_deadline(),
            self.root.next_deadline(),
            self.link.next_deadline(),
            self.values.next_expiry(),
            self.application.next_expiry(),
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
    pub(super) fn work_pending(&self, now: Time) -> bool {
        !self.started
            || !self.io.is_empty()
            || !self.events.is_empty()
            || !self.process_events.is_empty()
            || !self.source_events.is_empty()
            || !self.submitted.is_empty()
            || (self.aborted.is_none() && self.root.work_pending())
            || match self.next_deadline() {
                Some(at) => at <= now,
                None => false,
            }
    }
    pub(super) fn closed(&self) -> bool {
        self.closed && self.sources.settled() && self.processes.is_empty() && self.io.is_empty()
    }
    pub(super) fn is_empty(&self) -> bool {
        self.closed() && self.directory.is_none() && self.submitted.is_empty()
    }
    pub(super) fn close_root(&mut self) {
        if self.closed()
            && self.root_close.is_none()
            && let Some(fd) = self.directory
        {
            self.root_close = Some(Token::new(self.prefix.checked_add(1).expect("root close")));
            self.submitted.push(io::kernel::Submit {
                op: Token::new(self.prefix.checked_add(1).expect("root close")),
                kind: io::kernel::Op::Close { fd },
            });
        }
    }
    pub(super) fn root_complete(&mut self, complete: &io::kernel::Complete) -> bool {
        if complete.op == Token::new(self.prefix.checked_add(1).expect("root close")) {
            assert!(self.root_close.is_some(), "the worker closes its own descriptor");
            self.directory = None;
            self.root_close = None;
            true
        } else {
            false
        }
    }
}
fn worker_prefix(index: u64) -> Option<u64> {
    if index >= (1_u64 << 12_u32) {
        return None;
    }
    (3_u64 << 60).checked_add(index.checked_shl(48)?)
}
fn grant_value(table: &Table, grant: &smith::Grant, now: Time) -> Option<host::Values> {
    let value = account(table, grant, now)?;
    Some(host::Values { paths: Box::default(), credentials: Box::new([value]) })
}
fn account(table: &Table, grant: &smith::Grant, now: Time) -> Option<smith_protocol_channel::GrantValue> {
    match grant.form {
        smith::Form::Program(name) => return Some(smith_protocol_channel::GrantValue::Program(name.as_bytes().into())),
        smith::Form::Credential(_) => {}
        smith::Form::Withdrawn => return None,
    }
    let value = table.value(grant.account, grant.generation, now)?;
    let token = value.token.expose().into();
    Some(match &value.scheme {
        Scheme::Bearer => smith_protocol_channel::GrantValue::Bearer(token),
        Scheme::Key => smith_protocol_channel::GrantValue::Key(token),
    })
}
fn start_values(table: &Table, grants: &[smith::Grant], now: Time) -> Option<host::Values> {
    let mut values = List::with_capacity(u32::try_from(grants.len()).ok()?);
    for grant in grants {
        values.push(account(table, grant, now)?).ok()?;
    }
    Some(host::Values { paths: Box::default(), credentials: values.into_boxed() })
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

#[cfg(test)]
mod program_grant_tests {
    use super::*;
    fn program_grant() -> smith::Grant {
        smith::Grant {
            form: smith::Form::Program(smith::SourceName::new(b"personal").expect("source name")),
            account: 7,
            generation: 2,
            valid: skein_lib::Duration::from_secs(60),
        }
    }
    #[test]
    fn a_program_grant_reaches_smith_as_a_program_value() {
        let table = Table::new(0, 1);
        let values = start_values(&table, &[program_grant()], Time::ZERO).expect("program values");
        assert_eq!(
            values.credentials.as_ref(),
            &[smith_protocol_channel::GrantValue::Program(b"personal".as_slice().into())]
        );
    }
    #[test]
    fn a_program_grant_needs_no_table_entry() {
        let table = Table::new(0, 1);
        assert_eq!(
            account(&table, &program_grant(), Time::ZERO),
            Some(smith_protocol_channel::GrantValue::Program(b"personal".as_slice().into()))
        );
    }
}

impl crate::Service {
    /// Access a reference worker to tune its pacing and observe admission in worlds.
    pub fn worker(&mut self, index: u32) -> Option<&mut Worker> {
        self.workers.get_mut(index)
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    use crate::tests::fixtures;

    /// Two waiting engine records must not themselves keep admission shut.
    const WAITING_RECORDS: u32 = 2;
    /// One request may bring admitted work past the pacing mark.
    const MARK: u32 = 2;
    /// These unsubmitted requests share a sink that the test drains by hand.
    const IO_ENTITY: Token = Token::new(71);
    /// The real initial dial fills a one-entry watermark.
    const DIAL_MARK: u32 = 1;

    fn started_service() -> crate::Service {
        let mut service = fixtures::worker_service();
        service.worker(0).expect("worker").iterate(Time::ZERO, Wall::EPOCH);
        service
    }

    use jig_domain_host::{LinkDown, SourceName};
    use jig_protocol_worker_wire as wire;
    use skein_fake_channel::ScriptedPeer;
    use skein_lib::{Duration, Intake, Reader};

    const RUN: Token = Token::new(7);
    const ATTEMPT: Token = Token::new(3);
    const OWNER: Token = Token::new(0);
    const OTHER_RUN: Token = Token::new(8);
    const SOCKET: Token = Token::new(72);
    // Three retained turns distinguish the prefix that passes from the stopped suffix.
    const TURNS: u32 = 3;
    // The check interval precedes the admitted spawn's one-second watchdog.
    const CHECK_INTERVAL: Duration = Duration::from_millis(250);
    const WATCHDOG: Duration = Duration::from_secs(1);
    // Each channel frame is at most the negotiated skip bound in this fixture.
    const WIRE_BYTES: u32 = 1_048_576;
    // Enough rounds for the opening, three turn frames and their output rights.
    const WIRE_ROUNDS: u32 = 128;

    struct Peer {
        peer: ScriptedPeer,
        intake: Intake,
        read: stream::Read,
    }
    impl Peer {
        fn new(layer: &mut Worker, input: &[u8]) -> Self {
            let start = skein_lib::bytes::find(input, b"[{\n").expect("worker member") + 1;
            let end = skein_lib::bytes::find(input, b"],\"application\"").expect("worker end");
            let document = config::document(
                &input[start..end],
                &config::DocumentLimits { bytes: WIRE_BYTES, tokens: WIRE_BYTES, depth: 32 },
            )
            .expect("worker document");
            let worker = config::worker(&document, 0, &config::Directories::working()).expect("worker config");
            let limits = crate::limits::worker_link(&worker);
            let codec = wire::sizes::limits(&limits.sizes).expect("codec");
            let mut channel_limits = link::channel_limits(&limits).expect("channel limits");
            let cap = wire::sizes::engine_cap(&limits.sizes).expect("engine cap");
            channel_limits.output_bytes = cap.bytes;
            channel_limits.output_frames = cap.frames;
            let mut peer = Self {
                peer: ScriptedPeer::new(
                    wire::schema(&codec).expect("schema"),
                    skein_channel::Role::Responder,
                    channel_limits,
                    wire::VERSION,
                    Box::default(),
                )
                .expect("engine peer"),
                intake: Intake::with_capacity(WIRE_BYTES),
                read: stream::Read::Nothing,
            };
            layer.socket = Some(SOCKET);
            layer.link_down(Time::ZERO, link::Event::Dialled { connection: Token::new(layer.prefix) });
            peer.pump(layer);
            layer.iterate(Time::ZERO, Wall::EPOCH);
            peer.pump(layer);
            peer
        }
        fn pump(&mut self, layer: &mut Worker) {
            for _ in 0..WIRE_ROUNDS {
                let mut progressed = false;
                let mut waiting = Queue::uncapped();
                for _ in 0..layer.io.len() {
                    match layer.io.pop().expect("io count") {
                        io::Request::Output { stream, down } if stream == SOCKET => {
                            progressed = true;
                            match down {
                                stream::OutputDown::Room { right, .. } => layer.link_down(
                                    layer.env.now,
                                    link::Event::Lower {
                                        connection: Token::new(layer.prefix),
                                        event: LowerEvent::Write(stream::OutputUp::Settled {
                                            right,
                                            outcome: stream::OutputOutcome::Granted,
                                        }),
                                    },
                                ),
                                stream::OutputDown::Send { bytes, .. } => self.peer.feed(&bytes).expect("wire frame"),
                                stream::OutputDown::Cancel { .. } | stream::OutputDown::Release { .. } => {}
                            }
                        }
                        io::Request::Stream { stream, down: stream::Down::Demand { read, .. } } if stream == SOCKET => {
                            self.read = read;
                            progressed = true;
                        }
                        request => waiting.push(request),
                    }
                }
                while let Some(request) = layer.io.pop() {
                    waiting.push(request);
                }
                layer.io = waiting;
                while let Some(bytes) = self.peer.pop_output() {
                    self.intake.append(&bytes).expect("fixture frame room");
                    progressed = true;
                }
                if let Some(bytes) = self.intake.meet(self.read) {
                    self.read = stream::Read::Nothing;
                    layer.link_down(
                        layer.env.now,
                        link::Event::Lower {
                            connection: Token::new(layer.prefix),
                            event: LowerEvent::Read(stream::Up::Bytes(bytes)),
                        },
                    );
                    progressed = true;
                }
                if !progressed {
                    return;
                }
            }
            panic!("wire fixture settles");
        }
        fn turns(&self) -> Vec<u32> {
            let mut turns = Vec::new();
            for frame in self.peer.observed() {
                if frame.kind == wire::Kind::Turn.number() {
                    turns.push(
                        wire::Turn::decode(&wire::CEILINGS, &mut Reader::new(&frame.body)).expect("turn").number(),
                    );
                }
            }
            turns
        }
    }

    // A scripted process face supplies domain records without operating pipes.
    // Payload hand-outs still return through the service's concrete codecs.
    fn scripted(layer: &mut Worker, event: root::Event) {
        let mut out = Queue::uncapped();
        root::step(&mut layer.root, &layer.env, event, &mut out);
        for _ in 0..PAYLOAD_HAND_OFF_ROUNDS {
            for _ in 0..out.len() {
                match out.pop().expect("scripted output") {
                    root::Request::Decode { agent, run, start } => {
                        let start = worker_route::decode(run, *start, &layer.agent);
                        root::step(&mut layer.root, &layer.env, root::Event::Decoded { agent, start }, &mut out);
                    }
                    root::Request::Encode { owner, value } => {
                        let encoded = worker_route::encode(value, &layer.agent.limits, &layer.agent.endpoints);
                        root::step(&mut layer.root, &layer.env, root::Event::Encoded { owner, encoded }, &mut out);
                    }
                    root::Request::Link(up) => {
                        layer.link_down(layer.env.now, link::Event::Up { up, parts: Box::default() })
                    }
                    root::Request::Process(_) => {}
                    other => panic!("unexpected scripted hand-out: {other:?}"),
                }
            }
        }
        assert!(out.is_empty(), "scripted hand-outs complete");
    }

    fn retained_turns() -> (crate::Service, Peer) {
        let document = fixtures::with(fixtures::WORKER_DOCUMENT, b"\"slots\":1", b"\"slots\":2");
        let document = fixtures::with(&document, b"\"hub.turns\":1", b"\"hub.turns\":3");
        let document = fixtures::with(&document, b"\"hub.slots\":1", b"\"hub.slots\":2");
        let mut agent = fixtures::agent();
        agent.host_limits.agents = 2;
        let mut service = crate::startup(
            &document,
            &fixtures::worker_files(),
            Some(&agent),
            &fixtures::seeds(),
            None,
            Some(fixtures::worker_runtime()),
        )
        .expect("worker service");
        let layer = service.worker(0).expect("worker");
        layer.iterate(Time::ZERO, Wall::EPOCH);
        let mut peer = Peer::new(layer, &document);
        scripted(layer, root::Event::Link(fixtures::worker_assignment()));
        scripted(
            layer,
            root::Event::Process(Box::new(smith::process::Event::Spawned { owner: OWNER, process: OWNER })),
        );
        scripted(layer, root::Event::Process(Box::new(smith::process::Event::Sent { owner: OWNER })));
        scripted(
            layer,
            root::Event::Process(Box::new(smith::process::Event::Received {
                owner: OWNER,
                message: smith::Up::Admitted,
            })),
        );
        for number in 1..=TURNS {
            let mut turn = fixtures::concrete_turn();
            turn.number = number;
            turn.spent = u64::from(number) * 2;
            scripted(
                layer,
                root::Event::Process(Box::new(smith::process::Event::Received {
                    owner: OWNER,
                    message: smith::Up::Turn { turn },
                })),
            );
            scripted(layer, root::Event::Process(Box::new(smith::process::Event::Sent { owner: OWNER })));
        }
        peer.pump(layer);
        while layer.io.pop().is_some() {}
        (service, peer)
    }
    fn pressure(layer: &mut Worker) {
        layer.set_watermark(MARK);
        for _ in 0..MARK {
            layer.io.push(io::Request::Close { entity: IO_ENTITY });
        }
    }
    fn resend(turn: u32) -> root::Event {
        root::Event::Resend { run: RUN, attempt: ATTEMPT, turn }
    }
    fn inventory(name: &[u8]) -> root::Event {
        root::Event::Link(LinkDown::Sources { sources: Box::new([SourceName::new(name).expect("source name")]) })
    }
    fn waiting(layer: &Worker) -> Vec<&root::Event> {
        layer.events.iter().collect()
    }

    #[test]
    fn admitted_records_pass_a_held_new_record_in_order() {
        let (mut service, mut peer) = retained_turns();
        let layer = service.worker(0).expect("worker");
        pressure(layer);
        layer.events.push(inventory(b"first"));
        layer.events.push(resend(1));
        layer.events.push(resend(2));
        layer.iterate(Time::ZERO, Wall::EPOCH);
        peer.pump(layer);
        assert_eq!(peer.turns(), [1, 2, 3, 1, 2], "both resends pass in wire order");
        assert_eq!(waiting(layer), [&inventory(b"first")]);
    }

    #[test]
    fn the_walk_stops_at_the_next_new_record() {
        let (mut service, mut peer) = retained_turns();
        let layer = service.worker(0).expect("worker");
        pressure(layer);
        layer.events.push(inventory(b"first"));
        layer.events.push(resend(1));
        layer.events.push(resend(2));
        layer.events.push(inventory(b"second"));
        layer.events.push(resend(3));
        layer.iterate(Time::ZERO, Wall::EPOCH);
        peer.pump(layer);
        assert_eq!(peer.turns(), [1, 2, 3, 1, 2], "the third resend cannot pass the second inventory");
        assert_eq!(waiting(layer), [&inventory(b"first"), &inventory(b"second"), &resend(3)]);
        while layer.io.pop().is_some() {}
        layer.iterate(Time::ZERO, Wall::EPOCH);
        peer.pump(layer);
        // Source command completions may queue output, so drain it before the next new inventory.
        while layer.io.pop().is_some() {}
        layer.iterate(Time::ZERO, Wall::EPOCH);
        peer.pump(layer);
        assert!(layer.events.is_empty());
        assert_eq!(peer.turns(), [1, 2, 3, 1, 2, 3]);
    }

    #[test]
    fn a_cancel_never_passes_its_held_assignment() {
        let (mut service, mut peer) = retained_turns();
        let layer = service.worker(0).expect("worker");
        let LinkDown::Assign { mut assignment } = fixtures::worker_assignment() else { unreachable!("assignment") };
        assignment.assignment.run = OTHER_RUN;
        let assign = root::Event::Link(LinkDown::Assign { assignment });
        pressure(layer);
        layer.events.push(assign);
        layer.events.push(resend(1));
        layer.events.push(root::Event::Link(LinkDown::Cancel { run: OTHER_RUN, attempt: ATTEMPT }));
        layer.events.push(resend(2));
        layer.iterate(Time::ZERO, Wall::EPOCH);
        peer.pump(layer);
        assert_eq!(peer.turns(), [1, 2, 3, 1], "the walk stops at the dependent cancel");
        assert!(matches!(
            waiting(layer).as_slice(),
            [
                root::Event::Link(LinkDown::Assign { .. }),
                root::Event::Link(LinkDown::Cancel { run: OTHER_RUN, attempt: ATTEMPT }),
                root::Event::Resend { turn: 2, .. }
            ]
        ));
        while layer.io.pop().is_some() {}
        // Admission of the assignment creates a spawn request. Drain that request before the cancel.
        layer.iterate(Time::ZERO, Wall::EPOCH);
        while layer.io.pop().is_some() {}
        layer.iterate(Time::ZERO, Wall::EPOCH);
        peer.pump(layer);
        assert_eq!(peer.turns(), [1, 2, 3, 1, 2]);
        let process = *layer.processes.iter().last().expect("second process").0;
        layer.step(root::Event::Process(Box::new(smith::process::Event::Unspawned {
            owner: process,
            detail: Box::default(),
        })));
        peer.pump(layer);
        let mut answers = Vec::new();
        for frame in peer.peer.observed() {
            if frame.kind == wire::Kind::Answer.number() {
                answers.push(wire::Answer::decode(&wire::CEILINGS, &mut Reader::new(&frame.body)).expect("answer"));
            }
        }
        assert_eq!(answers.len(), 1);
        assert!(matches!(answers[0].ending(), wire::Ending::Failed(failure)
            if matches!(failure.failure(), wire::Failure::Cancelled(_))));
        layer.step(root::Event::Link(LinkDown::Acknowledge { run: OTHER_RUN, attempt: ATTEMPT }));
        layer.root.reclaim();
        assert_eq!(layer.root.hosted(), 1, "acknowledgement releases only the cancelled run");
        assert!(
            !layer.io.iter().any(|request| matches!(request, io::Request::Spawn { .. })),
            "no start after cancellation"
        );
    }

    #[test]
    fn due_source_checks_wait_at_the_watermark() {
        let document =
            fixtures::with(fixtures::WORKER_DOCUMENT, b"\"check_interval_ms\":60000", b"\"check_interval_ms\":250");
        let mut service = crate::startup(
            &document,
            &fixtures::worker_files(),
            Some(&fixtures::agent()),
            &fixtures::seeds(),
            None,
            Some(fixtures::worker_runtime()),
        )
        .expect("worker service");
        let layer = service.worker(0).expect("worker");
        layer.iterate(Time::ZERO, Wall::EPOCH);
        layer.step(inventory(b"personal"));
        layer.iterate(Time::ZERO, Wall::EPOCH);
        layer.step(root::Event::Link(fixtures::worker_assignment()));
        while layer.io.pop().is_some() {}
        pressure(layer);
        let due = Time::ZERO.saturating_add(CHECK_INTERVAL);
        layer.iterate(due, Wall::EPOCH);
        assert_eq!(layer.root.next_deadline(), Some(due), "a held check stays due");
        assert!(layer.source_events.is_empty(), "no check started at the watermark");
        // Spawn timeout advances admitted work while the periodic check is still held.
        layer.iterate(Time::ZERO.saturating_add(WATCHDOG), Wall::EPOCH);
        assert_eq!(layer.root.hosted(), 1, "the terminal remains retained until acknowledged");
        assert_eq!(layer.root.next_deadline(), Some(due));
        while layer.io.pop().is_some() {}
        layer.iterate(Time::ZERO.saturating_add(WATCHDOG), Wall::EPOCH);
        assert_eq!(layer.source_events.len(), 1, "exactly one due check starts below the mark");
        layer.iterate(Time::ZERO.saturating_add(WATCHDOG), Wall::EPOCH);
        assert_eq!(
            layer.root.next_deadline(),
            Some(Time::ZERO.saturating_add(WATCHDOG).saturating_add(CHECK_INTERVAL))
        );
    }

    #[test]
    fn the_engines_records_wait_at_the_watermark_and_completions_drain_the_backlog() {
        for requests in [MARK, MARK + 1] {
            let mut service = started_service();
            let layer = service.worker(0).expect("worker");
            while layer.io.pop().is_some() {}
            layer.set_watermark(MARK);
            for _ in 0..requests {
                layer.io.push(io::Request::Close { entity: IO_ENTITY });
            }
            for _ in 0..WAITING_RECORDS {
                layer.events.push(root::Event::Link(LinkDown::Sources { sources: Box::default() }));
            }
            let before = layer.root_steps;
            layer.iterate(Time::ZERO, Wall::EPOCH);
            assert_eq!(layer.root_steps, before, "records wait at and past the mark");
            assert_eq!(layer.events.len(), WAITING_RECORDS);
            // The service's io dispatch drains admitted requests independently of admission.
            // No new record or completion is needed to reopen this FIFO.
            while layer.io.len() >= MARK {
                let _request = layer.io.pop();
            }
            layer.iterate(Time::ZERO, Wall::EPOCH);
            assert_eq!(layer.root_steps, before + u64::from(WAITING_RECORDS));
            assert!(layer.events.is_empty(), "waiting records cannot hold admission shut");
        }
    }

    #[test]
    fn the_service_drains_worker_io_while_admission_waits() {
        let mut service = started_service();
        let layer = service.worker(0).expect("worker");
        layer.set_watermark(DIAL_MARK);
        layer.events.push(root::Event::Link(LinkDown::Sources { sources: Box::default() }));
        let before = service.root_steps();
        crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
        assert_eq!(service.root_steps(), before);
        assert!(service.worker(0).expect("worker").io.is_empty(), "the io sink progresses despite admission");
        // No completion or new engine record arrives: draining alone reopens admission.
        crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
        assert!(service.root_steps() > before);
        assert!(service.worker(0).expect("worker").events.is_empty());
    }
}
