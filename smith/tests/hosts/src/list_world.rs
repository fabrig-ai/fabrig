//! A host list read beside the shipped list child and an independent fake provider
//! (smith/docs/design/protocol/host.md, 2; smith/docs/design/protocol/agent.md, 6.1).
//! Skein's shared world owns scheduling, child lifetime, replay and teardown.

use skein_fake_peers::llm::{Observation, Peer};
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Env, Queue, Secret, Time, Token, Wall};
use skein_shell::{Delegate, Host};
use skein_world::{HostedProgram, Inherited, Referee};
use smith_channel::{self as wire, CEILINGS};
use smith_host_domain::ListOutcome;
use smith_host_protocol::{ListLaunch, ListRead};

const OWNER: Token = Token::new(100);
const ROOT_CLOSE: Token = Token::new(u64::MAX);
const ENDPOINT: u32 = 17;
const DEADLINE: Duration = Duration::from_millis(50);
/// The fixture's credential, accepted by the independent provider only through the ask.
pub const CREDENTIAL: &[u8] = b"host-list-private-fixture";

/// Which agent the list host starts in this scenario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    /// The shipped list process.
    List,
    /// The shipped list process refuses its startup configuration.
    Startup,
    /// A child that remains alive without answering.
    Hang,
    /// A child that succeeds without any output.
    Empty,
    /// A child that succeeds with invalid wire output.
    Garbage,
    /// A child whose stdout is beyond the wire answer bound.
    Oversized,
    /// A child that writes a readable answer but exits with failure.
    Failed,
    /// A child that writes a readable answer but ends by a signal.
    Signalled,
    /// A child whose valid answer is followed by extra bytes.
    Trailing,
}

impl Agent {
    fn program(self) -> &'static [u8] {
        match self {
            Self::List | Self::Startup => b"list-agent",
            Self::Hang => b"hang-agent",
            Self::Empty => b"empty-agent",
            Self::Garbage => b"garbage-agent",
            Self::Oversized => b"oversized-agent",
            Self::Failed => b"failed-agent",
            Self::Signalled => b"signalled-agent",
            Self::Trailing => b"trailing-agent",
        }
    }
}

fn limits() -> io::Limits {
    io::Limits {
        sockets: 4,
        names: 0,
        intake: 4096,
        receive: 4096,
        output: 8192,
        close_timeout: Duration::from_millis(10),
        retry: Duration::from_millis(1),
        ..smith_agent_process_world::limits().io
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "the fixture tracks cancellation, answer and the root's close independently"
)]
struct Parent {
    io: io::Io,
    read: ListRead,
    env: Env<io::Limits>,
    cancel: Option<Time>,
    cancelled: bool,
    answers: Vec<ListOutcome>,
    root: kernel::Fd,
    root_closing: bool,
    root_closed: bool,
    done: bool,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
}

impl Parent {
    fn new(root: kernel::Fd, agent: Agent, cancel: Option<Duration>, endpoint: u32) -> Self {
        let mut requests = Queue::uncapped();
        let read = ListRead::new(
            OWNER,
            ListLaunch {
                program: agent.program().into(),
                arguments: Box::new([configuration(agent)]),
                environment: Box::new([b"HOME=/home".as_slice().into(), b"LIST_ENV=kept".as_slice().into()]),
                root,
                directory: b".".as_slice().into(),
            },
            endpoint,
            wire::CredentialValue::new(
                &CEILINGS,
                wire::CredentialValueParts { scheme: wire::Scheme::Key, bytes: Secret::new(CREDENTIAL.into()) },
            )
            .expect("fixture credential"),
            Time::ZERO.saturating_add(DEADLINE),
            &mut requests,
        )
        .expect("list binding");
        assert!(!format!("{read:?}").contains("private"), "ListRead Debug redacts the ask");
        assert!(!format!("{read:?}").contains(&format!("{CREDENTIAL:?}")), "ListRead Debug redacts credential bytes");
        let limits = limits();
        Self {
            io: io::Io::new(&limits),
            read,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            cancel: cancel.map(|after| Time::ZERO.saturating_add(after)),
            cancelled: false,
            answers: Vec::new(),
            root,
            root_closing: false,
            root_closed: false,
            done: false,
            events: Queue::uncapped(),
            requests,
            completions: Queue::uncapped(),
            submissions: Queue::uncapped(),
        }
    }
}

impl Host for Parent {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        for _ in 0..self.completions.len() {
            let complete = self.completions.pop().expect("completion at stage entry");
            if complete.op == ROOT_CLOSE {
                assert!(complete.result.is_ok(), "parent root closes");
                self.root_closed = true;
            } else {
                self.completions.push(complete);
            }
        }
        io::stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        while let Some(event) = self.events.pop() {
            assert!(self.read.owns(event.owner().expect("list event owner")), "route only owned list events");
            self.read.from_io(event, &mut self.requests);
            if let Some(answer) = self.read.answer() {
                self.io.reclaim();
                assert!(self.io.is_empty(), "the answer waits for the child and every pipe to settle");
                assert!(self.read.answer().is_none(), "one answer right");
                self.answers.push(answer);
                self.done = true;
            }
        }
        if self.cancel.is_some_and(|at| at <= now) {
            self.cancel = None;
            self.cancelled = true;
            self.read.cancel(&mut self.requests);
        }
        self.read.fire(now, &mut self.requests);
        assert!(self.read.answer().is_none(), "a deadline starts closure before answering");
        self.io.reclaim();
        if self.cancelled && self.io.is_empty() && self.requests.is_empty() {
            self.done = true;
        }
        if self.done && !self.root_closing {
            self.root_closing = true;
            self.submissions.push(kernel::Submit { op: ROOT_CLOSE, kind: kernel::Op::Close { fd: self.root } });
        }
        io::stage::down(&mut self.io, &self.env, &mut self.requests, &mut self.submissions);
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || self.io.is_due(now)
            || !self.completions.is_empty()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || self.next_policy_deadline().is_some_and(|at| at <= now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [self.read.next_deadline(), self.cancel].into_iter().flatten().min()
    }
    fn is_empty(&self) -> bool {
        self.done
            && self.root_closed
            && self.io.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&limits()).expect("list host operations").checked_add(1).expect("root close")
    }
}

// Faulty children own raw inherited pipes, write at most one bounded fixture, and
// close every descriptor; the hanging child's descriptors end only when killed.
struct Broken {
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
    pending: u32,
    next: u64,
    hang: bool,
    exit: kernel::Exit,
    output: Option<(kernel::Fd, Box<[u8]>)>,
}

impl Broken {
    fn new(spawn: &kernel::Spawn, inherited: &Inherited, code: u8) -> Self {
        let mut child = Self {
            completions: Queue::uncapped(),
            submissions: Queue::uncapped(),
            pending: 0,
            next: 0,
            hang: spawn.program.as_ref() == b"hang-agent",
            exit: match spawn.program.as_ref() {
                b"failed-agent" => kernel::Exit::Code(1),
                b"signalled-agent" => kernel::Exit::Signal(15),
                _ => kernel::Exit::Code(code),
            },
            output: None,
        };
        if child.hang {
            return child;
        }
        let output = inherited.pipes.iter().find(|(number, _)| *number == 1).expect("stdout").1;
        let bytes: Box<[u8]> = match spawn.program.as_ref() {
            b"garbage-agent" => b"garbage".as_slice().into(),
            b"oversized-agent" => {
                vec![
                    0;
                    usize::try_from(wire::ListAnswer::encoded_bound(&CEILINGS).expect("answer bound"))
                        .expect("bound fits usize")
                        .checked_add(1)
                        .expect("one past bound")
                ]
                .into()
            }
            // A successful wire answer with no models.
            b"failed-agent" | b"signalled-agent" => Box::new([0, 0, 0, 0, 0]),
            // Listed with zero rows, followed by one trailing byte.
            b"trailing-agent" => Box::new([0, 0, 0, 0, 0, 0xff]),
            _ => Box::new([]),
        };
        if code == 0 {
            child.output = Some((output, bytes));
            let input = inherited.pipes.iter().find(|(number, _)| *number == 0).expect("stdin").1;
            let error = inherited.pipes.iter().find(|(number, _)| *number == 2).expect("stderr").1;
            child.submit(kernel::Op::Close { fd: error });
            child.submit(kernel::Op::PipeRead { fd: input, buf: vec![0; 4096].into() });
        } else {
            for (_, pipe) in &inherited.pipes {
                child.submit(kernel::Op::Close { fd: *pipe });
            }
        }
        child.submit(kernel::Op::Close { fd: inherited.signal });
        child
    }
    fn submit(&mut self, kind: kernel::Op) {
        self.pending = self.pending.checked_add(1).expect("few scripted operations");
        self.next = self.next.checked_add(1).expect("scripted operation identity");
        self.submissions.push(kernel::Submit { op: Token::new(self.next), kind });
    }
}

impl Host for Broken {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.pending = self.pending.checked_sub(1).expect("scripted operation pending");
            match complete.kind {
                kernel::Op::PipeRead { fd, buf } => {
                    let Ok(kernel::Done::Count(count)) = complete.result else { unreachable!("scripted stdin read") };
                    if count == 0 {
                        self.submit(kernel::Op::Close { fd });
                        let (output, bytes) = self.output.take().expect("scripted stdout");
                        if bytes.is_empty() {
                            self.submit(kernel::Op::Close { fd: output });
                        } else {
                            self.submit(kernel::Op::PipeWrite { fd: output, bytes, from: 0 });
                        }
                    } else {
                        self.submit(kernel::Op::PipeRead { fd, buf });
                    }
                }
                kernel::Op::PipeWrite { fd, bytes, from } => {
                    let Ok(kernel::Done::Count(count)) = complete.result else { unreachable!("scripted stdout write") };
                    let from = from.checked_add(count).expect("fixture output offset");
                    if usize::try_from(from).expect("offset fits usize") == bytes.len() {
                        self.submit(kernel::Op::Close { fd });
                    } else {
                        self.submit(kernel::Op::PipeWrite { fd, bytes, from });
                    }
                }
                kernel::Op::Close { .. } => assert!(complete.result.is_ok(), "scripted pipe close"),
                _ => unreachable!("only scripted writes and closes"),
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.completions.is_empty() || !self.submissions.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        !self.hang && self.pending == 0 && self.completions.is_empty() && self.submissions.is_empty()
    }
    fn exit(&self) -> Option<kernel::Exit> {
        self.is_empty().then_some(self.exit)
    }
    fn operations(&self) -> u32 {
        8
    }
}

enum Proc {
    Parent(Box<Parent>),
    List(Box<smith_agent_shell::list::Process>),
    Broken(Box<Broken>),
    Peer(Box<Peer>),
}
impl Delegate for Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Parent(p) => p.as_ref(),
            Self::List(p) => p.as_ref(),
            Self::Broken(p) => p.as_ref(),
            Self::Peer(p) => p.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Parent(p) => p.as_mut(),
            Self::List(p) => p.as_mut(),
            Self::Broken(p) => p.as_mut(),
            Self::Peer(p) => p.as_mut(),
        }
    }
}

fn make(spawn: &kernel::Spawn, inherited: &Inherited) -> Proc {
    assert_eq!(spawn.args.len(), 2, "configuration and --list alone");
    assert_eq!(spawn.args.first().expect("flag").as_ref(), b"--list", "list flag precedes the configuration");
    assert_eq!(
        spawn.env.as_ref(),
        &[Box::from(b"HOME=/home".as_slice()), Box::from(b"LIST_ENV=kept".as_slice())],
        "environment is passed whole"
    );
    for value in spawn.args.iter().chain(spawn.env.iter()) {
        assert!(skein_lib::bytes::find(value, CREDENTIAL).is_none(), "no credential in arguments or environment");
    }
    if spawn.program.as_ref() != b"list-agent" {
        return Proc::Broken(Box::new(Broken::new(spawn, inherited, 0)));
    }
    let pipe = |number| inherited.pipes.iter().find(|(child, _)| *child == number).expect("child pipe").1;
    match smith_agent_shell::config::decode(spawn.args.get(1).expect("configuration")).and_then(|decoded| {
        smith_agent_shell::list::Process::new(
            decoded,
            smith_agent_shell::list::Faces {
                stdin: pipe(0),
                stdout: pipe(1),
                stderr: Some(pipe(2)),
                signal: inherited.signal,
            },
        )
    }) {
        Ok(child) => Proc::List(Box::new(child)),
        Err(_) => Proc::Broken(Box::new(Broken::new(spawn, inherited, 1))),
    }
}

struct Judge {
    finding: Option<String>,
    done: bool,
}
impl Judge {
    fn act_checked(&mut self, _now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let done = matches!(procs.first().ok_or("parent")?, Proc::Parent(parent) if parent.is_empty());
        if done && !self.done {
            self.done = true;
            for proc in procs {
                if let Proc::Peer(peer) = proc {
                    peer.shutdown();
                }
            }
        }
        Ok(())
    }
}

impl Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        Ok(())
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.done)
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.done).then_some(Time::ZERO.saturating_add(Duration::from_secs(1)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.next_deadline().filter(|at| now >= *at).map(|_| "host list settles".into())
    }
}

fn configuration(agent: Agent) -> Box<[u8]> {
    serde_json::to_vec(&serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":256,"read_window":256}},
        "endpoints":[{"name":"local","number":ENDPOINT,"dialect":2,"provider":"anthropic",
            "address":if agent == Agent::Startup { "192.0.2.1:443" } else { "127.0.0.1:443" },
            "transport":"plaintext","connect":10000,"handshake":10000,
            "limits":{"window":65536,"output":512,"reasoning_item":64,"head":60000,"idle":30000}}],
        "environment":[]
    }))
    .expect("fixture configuration")
    .into()
}

/// The host's settled answers and the number of credentialed list requests seen by its peer.
#[derive(Debug)]
pub struct Run {
    pub answers: Vec<ListOutcome>,
    pub requests: usize,
    pub cancelled: bool,
    pub trace: Vec<String>,
}

/// Drive one host list request over simulated pipes; no provider or real process is contacted.
#[must_use]
pub fn run(
    agent: Agent,
    cancel: Option<Duration>,
    status: Option<u16>,
    body: &[u8],
    unknown: bool,
    short: bool,
    peer: bool,
) -> Run {
    let mut files = skein_fake_machine::Machine::new();
    let root = files.lay(&[]);
    let parent_root = files.open(root, b".", skein_fake_machine::How::Directory).expect("parent root");
    let mut config = skein_sim::Config::calm();
    // Seven buffered bytes force both the ask and the answer through partial pipe writes.
    if short {
        config.buffer = 7;
    }
    let mut world =
        skein_world::World::new(7, config, Judge { finding: None, done: false }, skein_world::Memory::Checked)
            .with_machine(skein_fake_machine::Rooted { files, root });
    world.host(HostedProgram {
        program: agent.program().into(),
        make,
        instances: 1,
        operations: smith_agent_shell::list::operations().max(8),
    });
    world.spawn_root(skein_sim::Handle::new(parent_root.raw()), |root| {
        Proc::Parent(Box::new(Parent::new(
            root,
            agent,
            cancel,
            if unknown { ENDPOINT.checked_add(1).expect("unknown endpoint") } else { ENDPOINT },
        )))
    });
    if peer {
        world.spawn(|| {
            let mut peer = smith_agent_process_world::fake::peer_for(true);
            // The peer's independent fixture already accepts key "token" at index zero.
            peer.accept(skein_llm::Credential::key(CREDENTIAL.into())).expect("host credential");
            peer.set_list(Some(body.into()), status);
            Proc::Peer(Box::new(peer))
        });
    }
    let mut outcome = world.run();
    let Proc::Parent(parent) = outcome.procs.first().expect("parent") else { unreachable!("parent first") };
    let requests = outcome
        .procs
        .iter()
        .filter_map(|proc| match proc {
            Proc::Peer(peer) => Some(peer.observations()),
            _ => None,
        })
        .flatten()
        .filter(|observation| match observation {
            Observation::List { index, .. } => {
                assert_eq!(*index, Some(1), "stdin's credential reached the provider");
                true
            }
            Observation::Query { .. } | Observation::Answered { .. } => unreachable!("list read started a run"),
            _ => false,
        })
        .count();
    let result = Run {
        answers: parent.answers.clone(),
        requests,
        cancelled: parent.cancelled,
        trace: outcome.trace.iter().map(|entry| format!("{entry:?}")).collect(),
    };
    outcome.machine.files.close(root);
    assert_eq!(outcome.machine.files.open_handles(), 0, "all list file handles settle");
    result
}
