//! Public reading through the native service and simulated TCP sockets
//! (jig/docs/design/protocol/client/default-http.md, 5 and jig/docs/design/protocol/client/public-reads.md).
use crate::{
    fixtures,
    native::{self, NativeClient, Received, Script},
};
use jig_client::contract as c;
use skein_http::client as http;
use skein_io::{
    self as io,
    kernel::{Complete, Submit},
};
use skein_lib::{
    Duration, Env, Queue, Secret, Time, Token, Wall,
    stream::{Down, Read, Up},
};
use skein_shell::{Delegate, Host};
use skein_world::{Memory, Referee};

#[path = "../../../testing/jig-client-protocol-kit/src/native/config.rs"]
mod config;

pub const TITLE: &[u8] = b"Visible HTTP goal";

struct Service(jig_test_service::Service);
impl Host for Service {
    fn iterate(&mut self, now: Time, wall: Wall) {
        jig_test_service::iterate(&mut self.0, now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.0.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.0.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.0.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.0.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.0.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn operations(&self) -> u32 {
        self.0.operations()
    }
}

/// Observed HTTP bytes, decoded only after their native socket completed.
#[derive(Debug)]
pub struct Reply {
    pub status: u16,
    pub headers: Box<[skein_http::Header]>,
    pub body: Vec<u8>,
}
struct Probe {
    io: io::Io,
    env: Env<io::Limits>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    told: Queue<io::Event>,
    requests: Queue<io::Request>,
    client: http::Client,
    above: Queue<http::Event>,
    below: Queue<Down>,
    call: Option<jig_client_protocol_http::Call>,
    socket: Option<Token>,
    current: Reply,
    replies: Vec<Reply>,
    finished: bool,
    wake: bool,
}
impl Probe {
    fn new() -> Self {
        let limits = native::settings().io;
        Self {
            io: io::Io::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            complete: Queue::uncapped(),
            submit: Queue::uncapped(),
            told: Queue::uncapped(),
            requests: Queue::uncapped(),
            client: http::Client::new(&fixtures::connections_limits().http),
            above: Queue::uncapped(),
            below: Queue::uncapped(),
            call: None,
            socket: None,
            current: Reply { status: 0, headers: Box::new([]), body: Vec::new() },
            replies: Vec::new(),
            finished: false,
            wake: false,
        }
    }
    fn start(&mut self, project: u64) {
        assert!(self.socket.is_none() && self.call.is_none(), "one public exchange at a time");
        self.client = http::Client::new(&fixtures::connections_limits().http);
        let path = jig_client_protocol_http::PublicPath {
            owner: 0,
            version: 1,
            kind: 2,
            ids: jig_client_kit::test::list([project]),
        };
        let call =
            jig_client_protocol_http::public(&jig_client_protocol_http::Mapping::new(b"/jig").expect("prefix"), &path);
        assert!(call.headers.is_empty() && call.body.is_empty(), "credential-less public GET");
        self.call = Some(call);
        self.requests.push(io::Request::Connect { owner: Token::new(1), addr: native::settings().engine });
    }
    fn http_env(&self) -> Env<http::Limits> {
        self.env.with(fixtures::connections_limits().http)
    }
    fn http(&mut self, request: http::Request) {
        let env = self.http_env();
        http::down(&mut self.client, &env, request, &mut self.above, &mut self.below);
    }
    fn read(&mut self) {
        self.http(http::Request::Body(Down::Demand {
            read: Read::Arrived { max: self.http_env().limits.read },
            room: 0,
        }));
    }
}
impl Host for Probe {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.wake = false;
        self.env.now = now;
        self.env.wall = wall;
        while self.io.is_ready() {
            io::resume(&mut self.io, &self.env, &mut self.told, &mut self.submit);
        }
        while let Some(complete) = self.complete.pop() {
            io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.submit);
        }
        while self.io.is_due(now) {
            io::fire(&mut self.io, &self.env, &mut self.told, &mut self.submit);
        }
        while let Some(event) = self.told.pop() {
            match event {
                io::Event::Connecting { socket, .. } => self.socket = Some(socket),
                io::Event::Connected { .. } => {
                    let call = self.call.take().expect("queued public GET");
                    let mut headers = vec![skein_http::Header {
                        name: b"Host".as_slice().into(),
                        value: b"127.0.0.1:7100".as_slice().into(),
                    }];
                    headers.extend(call.headers.into_boxed());
                    self.http(http::Request::Call(http::Call {
                        method: call.method,
                        target: call.path,
                        headers: headers.into_boxed_slice(),
                        body: http::Body::None,
                        close: true,
                    }));
                }
                io::Event::Stream { up, .. } => {
                    let env = self.http_env();
                    http::up(&mut self.client, &env, up, &mut self.above, &mut self.below);
                }
                io::Event::Closed { .. } => {
                    self.socket = None;
                    self.wake = true;
                }
                io::Event::Failed { error, .. } => panic!("public probe io: {error:?}"),
                io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Output { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. } => panic!("unrelated probe io"),
            }
        }
        while let Some(event) = self.above.pop() {
            match event {
                http::Event::Response(response) => {
                    self.current.status = response.status;
                    self.current.headers = response.headers;
                    self.read();
                }
                http::Event::Body(Up::Bytes(bytes)) => {
                    self.current.body.extend_from_slice(&bytes);
                    self.read();
                }
                http::Event::Done(_) => {
                    eprintln!(
                        "public HTTP response {} at {:?}, {} bytes",
                        self.current.status,
                        self.env.now,
                        self.current.body.len()
                    );
                    self.replies.push(std::mem::replace(
                        &mut self.current,
                        Reply { status: 0, headers: Box::new([]), body: Vec::new() },
                    ));
                    self.http(http::Request::Close);
                    self.requests.push(io::Request::Abort { entity: self.socket.expect("connected probe") });
                }
                http::Event::Body(Up::End) | http::Event::Closed => {}
                http::Event::Failed(error) => panic!("public probe HTTP: {error:?}"),
                http::Event::Upload(_) | http::Event::Body(Up::Room | Up::Failed(_)) => {
                    panic!("unexpected probe stream")
                }
            }
        }
        while let Some(down) = self.below.pop() {
            self.requests.push(io::Request::Stream { stream: self.socket.expect("connected probe"), down });
        }
        while self.io.takes() {
            let Some(request) = self.requests.pop() else { break };
            io::down(&mut self.io, &self.env, request, &mut self.submit);
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, now: Time) -> bool {
        self.wake
            || self.io.is_ready()
            || self.io.is_due(now)
            || !self.complete.is_empty()
            || !self.requests.is_empty()
            || !self.told.is_empty()
            || !self.above.is_empty()
            || !self.below.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.finished && self.io.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("probe io sizes")
    }
}

enum Process {
    Service(Box<Service>),
    Client(Box<NativeClient>),
    Probe(Box<Probe>),
}
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Service(p) => p.as_ref(),
            Self::Client(p) => p.as_ref(),
            Self::Probe(p) => p.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Service(p) => p.as_mut(),
            Self::Client(p) => p.as_mut(),
            Self::Probe(p) => p.as_mut(),
        }
    }
}
struct Judge {
    finding: Option<String>,
    phase: u8,
    project: Option<u64>,
    period: bool,
}
fn send(client: &mut NativeClient, number: u64, ask: c::Ask) {
    client.ask_account_sign_in(jig_client::Key { asker: jig_client::Asker::JIG, number }, ask);
}
fn answered(client: &NativeClient, expected: &c::DocumentOutcome) -> bool {
    client.received.iter().any(|r| matches!(r, Received::Answer { answer: c::Answer::Done(c::Outcome::Document(outcome)), .. } if outcome == expected))
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.finding.is_none() {
            self.finding = self.check_act(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Process]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (self.phase != 8).then_some(Time::ZERO.saturating_add(Duration::from_secs(5)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.phase != 8 && now >= Time::ZERO.saturating_add(Duration::from_secs(5)))
            .then(|| format!("public HTTP phase {}", self.phase))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.phase == 8
    }
}
struct Machine(skein_fake_machine::Machine);
impl skein_world::Machine for Machine {
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        skein_fake_machine::step(&mut self.0, call, answers);
    }
}
fn document() -> Vec<u8> {
    let limits = fixtures::public_service_limits();
    let figures = limits.clients.public;
    let index = limits.root.core.public;
    let extra = format!(
        "\"clients_http.public_calls\":2,\"clients.public.reads\":{},\"clients.public.rate\":{},\"clients.public.rate_burst\":{},\"clients.public.bytes\":{},\"clients.public.bytes_burst\":{},\"clients.public.answer_bytes\":{},\"clients.public.max_age\":{},\"core.public.goals\":{},\"core.public.ended\":{},\"core.public.lines\":{},\"core.public.depth\":{},",
        figures.reads,
        figures.rate,
        figures.rate_burst,
        figures.bytes,
        figures.bytes_burst,
        figures.answer_bytes,
        figures.max_age,
        index.goals,
        index.ended,
        index.lines,
        index.depth
    );
    let mut document = config::document();
    let start = skein_lib::bytes::find(&document, b"\"tasks.result_bytes\"").expect("native overrides");
    document.splice(start..start, extra.bytes());
    document
}
fn files() -> jig_test_service::Files {
    jig_test_service::Files {
        entries: jig_client_kit::test::list([
            (b"charters/chat.txt".as_slice().into(), Secret::new(b"hidden HTTP charter".as_slice().into())),
            (b"app.secret".as_slice().into(), Secret::new(b"application-token".as_slice().into())),
            (
                b"secrets/worker-a.digest".as_slice().into(),
                Secret::new(b"YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWE".as_slice().into()),
            ),
            (
                b"secrets/worker-b.digest".as_slice().into(),
                Secret::new(b"YmJiYmJiYmJiYmJiYmJiYmJiYmJiYmJiYmJiYmJiYmI".as_slice().into()),
            ),
        ]),
    }
}

#[must_use]
pub fn run(seed: u64) -> (u64, Vec<Reply>) {
    let mut machine = skein_fake_machine::Machine::for_user(1000);
    let engine_root = machine.lay(&[]);
    let client_root = machine.lay(&[]);
    let mut world = skein_world::World::new(
        seed,
        skein_sim::Config::calm(),
        Judge { finding: None, phase: 0, project: None, period: false },
        Memory::Unchecked,
    )
    .with_machine(Machine(machine));
    world.spawn_root(skein_sim::Handle::new(engine_root.raw()), |root| {
        let seeds = jig_test_service::Seeds {
            deployment: [7; 16],
            installation: [9; 16],
            seed,
            crypto: skein_world::crypto_seed(seed),
        };
        let mut service = jig_test_service::startup(&document(), &files(), None, &seeds, None, None)
            .expect("public service configuration");
        service.adopt_store(root, 1000).expect("store directory");
        Process::Service(Box::new(Service(service)))
    });
    world.spawn_root(skein_sim::Handle::new(client_root.raw()), |root| {
        let mut client = NativeClient::new(native::settings(), root, 1000, seed, Script::account_sign_in());
        client.hold_for_account_sign_in();
        Process::Client(Box::new(client))
    });
    world.spawn(|| Process::Probe(Box::new(Probe::new())));
    let mut outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty), "public service drained seed {seed}");
    let mut procs = outcome.procs.iter_mut();
    let Process::Client(client) = procs.nth(1).expect("signed-in client") else { panic!("client") };
    let project = client
        .received
        .iter()
        .find_map(|r| {
            if let Received::Delivery(c::Delivery::Me(me)) = r {
                me.projects.iter().find_map(|p| p.public)
            } else {
                None
            }
        })
        .expect("public identifier in me");
    let Process::Probe(probe) = procs.next().expect("public probe") else { panic!("probe") };
    (project, std::mem::take(&mut probe.replies))
}

impl Judge {
    fn check_act(&mut self, _now: Time, procs: &mut [Process]) -> Result<(), String> {
        let [Process::Service(service), Process::Client(client), Process::Probe(probe)] = procs else {
            return Err(String::from("service, signed-in client and public probe"));
        };
        if self.phase == 0
            && let Some(link) = service.0.links().pop()
        {
            client.sign_in(&link).map_err(|why| format!("printed service link: {why:?}"))?;
            self.phase = 1;
        }
        if !self.period && client.signed_in > 0 {
            service.0.engine().ok_or("engine")?.parties_in().push(jig_domain_core::Event::Period {
                project: 1,
                period: 1,
                budget: 100,
                next: None,
            });
            self.period = true;
        }
        if self.phase == 1 && client.account_sign_in_ready() {
            client.idle_fixture_view();
            let mut task = fixtures::create_tasks().get(0).ok_or("task fixture")?.clone();
            task.tracked = Some(17);
            task.spec.words = c::Text::new(b"Visible HTTP goal\nhidden HTTP spec".as_slice().into(), 128)
                .map_err(|why| format!("spec: {why:?}"))?;
            send(client, 1, c::Ask::Create { project: 1, tasks: jig_client_kit::test::list([task]) });
            self.phase = 2;
        }
        if self.phase == 2 && client.chats_made == 1 {
            send(client, 2, c::Ask::SetVisibility { project: 1, public: true });
            self.phase = 3;
        }
        if self.phase == 3 && answered(client, &c::DocumentOutcome::VisibilitySet { project: 1, public: true }) {
            let project = client.received.iter().rev().find_map(|r| {
                if let Received::Delivery(c::Delivery::Me(me)) = r {
                    me.projects.iter().find(|p| p.project == 1).and_then(|p| p.public)
                } else {
                    None
                }
            });
            if let Some(project) = project {
                self.project = Some(project);
                probe.start(project);
                self.phase = 4;
            }
        }
        if self.phase == 4 && probe.replies.len() == 1 && probe.socket.is_none() {
            send(client, 3, c::Ask::SetVisibility { project: 1, public: false });
            self.phase = 5;
        }
        if self.phase == 5 && answered(client, &c::DocumentOutcome::VisibilitySet { project: 1, public: false }) {
            probe.start(self.project.ok_or("me's public identifier")?);
            self.phase = 6;
        }
        if self.phase == 6 && probe.replies.len() == 2 && probe.socket.is_none() {
            probe.finished = true;
            client.finish_account_sign_in();
            self.phase = 7;
        }
        if self.phase == 7 && client.is_empty() {
            service.0.shutdown();
            self.phase = 8;
        }

        Ok(())
    }
}
