//! A host driving source operations beside the hosted program fake
//! (protocol/host.md, section 2; protocol/programs.md, section 8).
//! The shared world owns scheduling, teardown, memory and replay mechanics.

use skein_fake_machine::{How, Item, Machine as Files, Opened};
use skein_io::{self as io, file, file_layer, kernel};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_shell::{Delegate, Host};
use skein_world::{HostedProgram, Inherited, Machine, Referee};
use smith_fake_claude::{Fake, Story};
use smith_host_domain::SourceName;
use smith_host_protocol::{ProgramSource, SourceAnswer, SourceAsk, SourceInspection};
use smith_protocol_program::Programs;

const OWNER: Token = Token::new(100);
const ROOT_CLOSE: Token = Token::new(u64::MAX);
const DEADLINE: Duration = Duration::from_millis(50);

fn file_limits() -> file_layer::Limits {
    file_layer::Limits { files: 1, read: 1, entries: 1, whole: 1, timeout: DEADLINE }
}

fn limits() -> io::Limits {
    io::Limits {
        sockets: 3,
        names: 0,
        refusals: 1,
        intake: 4096,
        receive: 4096,
        output: 1,
        sends: 1,
        accepts: 1,
        backlog: 1,
        close_timeout: Duration::from_millis(10),
        retry: Duration::from_millis(1),
    }
}

struct Parent {
    io: io::Io,
    env: Env<io::Limits>,
    source: Option<ProgramSource>,
    inspection: Option<SourceInspection>,
    files: file_layer::FileIo,
    file_requests: Queue<file::Request>,
    file_events: Queue<file::Event>,
    asks: Queue<SourceAsk>,
    base: Vec<Box<[u8]>>,
    program: Box<[u8]>,
    answers: Vec<SourceAnswer>,
    root: kernel::Fd,
    root_closing: bool,
    launches: u32,
    deadline: Time,
    root_closed: bool,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
}

impl Parent {
    fn new(root: kernel::Fd, asks: &[SourceAsk], base: &[Box<[u8]>], missing: bool) -> Self {
        let limits = limits();
        let mut pending = Queue::with_capacity(u32::try_from(asks.len()).expect("short scenario"));
        for ask in asks {
            pending.push(*ask);
        }
        Self {
            io: io::Io::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            source: None,
            inspection: None,
            files: file_layer::FileIo::with_limits(&file_limits()),
            file_requests: Queue::uncapped(),
            file_events: Queue::uncapped(),
            asks: pending,
            base: base.to_vec(),
            program: if missing { b"missing".as_slice().into() } else { b"claude".as_slice().into() },
            answers: Vec::new(),
            root,
            root_closing: false,
            launches: 0,
            deadline: Time::ZERO,
            root_closed: false,
            events: Queue::with_capacity(32),
            requests: Queue::with_capacity(32),
            completions: Queue::with_capacity(32),
            submissions: Queue::with_capacity(64),
        }
    }
    fn launch(&mut self, ask: SourceAsk) {
        self.launches = self.launches.checked_add(1).expect("bounded source asks");
        self.source = Some(
            ProgramSource::new(
                OWNER,
                ask,
                &Programs { directory: b"/programs".as_slice().into(), claude: self.program.clone() },
                SourceName::new(b"alice").expect("source name"),
                &self.base,
                self.root,
                b".".as_slice().into(),
                self.deadline,
                32,
                &mut self.requests,
            )
            .expect("source binding"),
        );
    }
    fn inspect(&mut self, now: Time) {
        if let Some(inspection) = &mut self.inspection {
            inspection.fire(now);
        }
        while let Some(event) = self.file_events.pop() {
            self.inspection.as_mut().expect("inspection before files").from_file(event, &mut self.file_requests);
        }
        if let Some(inspection) = &mut self.inspection
            && let Some(answer) = inspection.answer()
        {
            self.inspection = None;
            match answer {
                Ok(()) => self.launch(SourceAsk::Check),
                Err(check) => self.answers.push(SourceAnswer::Check(check)),
            }
        }
        let env = self.env.with(file_limits());
        file_layer::step::expire(&mut self.files, &env, &mut self.submissions);
        if self.files.takes()
            && let Some(request) = self.file_requests.pop()
        {
            file_layer::step::down(&mut self.files, &env, request, &mut self.file_events, &mut self.submissions);
        }
    }
    fn issue(&mut self, now: Time) {
        if self.source.is_none() && self.inspection.is_none() && !self.root_closing {
            if let Some(ask) = self.asks.pop() {
                self.deadline = now.saturating_add(DEADLINE);
                match ask {
                    SourceAsk::Check => {
                        self.inspection = Some(SourceInspection::new(
                            OWNER,
                            self.root,
                            b"programs".as_slice().into(),
                            SourceName::new(b"alice").expect("source name"),
                            1000,
                            self.deadline,
                            &mut self.file_requests,
                        ));
                    }
                    SourceAsk::SignOut => self.launch(ask),
                }
            } else {
                self.root_closing = true;
                self.submissions.push(kernel::Submit { op: ROOT_CLOSE, kind: kernel::Op::Close { fd: self.root } });
            }
        }
    }
}

impl Host for Parent {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        let file_env = self.env.with(file_limits());
        for _ in 0..self.completions.len() {
            let complete = self.completions.pop().expect("completion present at stage entry");
            if complete.op == ROOT_CLOSE {
                assert!(complete.result.is_ok(), "parent root closes");
                self.root_closed = true;
            } else if file_layer::owns(complete.op) {
                file_layer::step::up(
                    &mut self.files,
                    &file_env,
                    complete,
                    &mut self.file_events,
                    &mut self.submissions,
                );
            } else {
                self.completions.push(complete);
            }
        }
        io::stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        while let Some(event) = self.events.pop() {
            let source = self.source.as_mut().expect("source issued before I/O");
            let owner = match &event {
                io::Event::Spawned { owner, .. }
                | io::Event::Exited { owner, .. }
                | io::Event::Failed { owner, .. }
                | io::Event::Closed { owner }
                | io::Event::Stream { owner, .. } => *owner,
                io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Output { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. } => unreachable!("source events only"),
            };
            assert!(source.owns(owner), "route only owned source events");
            source.from_io(event, &mut self.requests);
        }
        if let Some(source) = &mut self.source {
            source.fire(now, &mut self.requests);
            if let Some(answer) = source.answer() {
                assert!(source.answer().is_none(), "one answer right");
                self.answers.push(answer);
                self.source = None;
            }
        }
        self.inspect(now);
        self.io.reclaim();
        self.issue(now);
        while let Some(request) = self.requests.pop() {
            io::down(&mut self.io, &self.env, request, &mut self.submissions);
        }
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
            || (self.source.is_none() && self.inspection.is_none() && !self.root_closing)
            || !self.file_events.is_empty()
            || (!self.file_requests.is_empty() && self.files.takes())
            || self.files.is_due(now)
            || self.inspection.as_ref().and_then(SourceInspection::next_deadline).is_some_and(|at| at <= now)
            || self.source.as_ref().and_then(ProgramSource::next_deadline).is_some_and(|at| at <= now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.files.next_deadline()].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [
            self.source.as_ref().and_then(ProgramSource::next_deadline),
            self.inspection.as_ref().and_then(SourceInspection::next_deadline),
        ]
        .into_iter()
        .flatten()
        .min()
    }
    fn is_empty(&self) -> bool {
        self.root_closed
            && self.io.is_empty()
            && self.inspection.is_none()
            && self.files.takes()
            && self.files.open_files() == 0
            && self.file_events.is_empty()
            && self.file_requests.is_empty()
            && self.source.is_none()
            && self.asks.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("parent operations").checked_add(3).expect("root close and inspection")
    }
}

enum Process {
    Parent(Box<Parent>),
    Fake(Box<Fake>),
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Parent(parent) => parent.as_ref(),
            Self::Fake(fake) => fake.as_ref(),
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Parent(parent) => parent.as_mut(),
            Self::Fake(fake) => fake.as_mut(),
        }
    }
}

struct MachineFiles {
    files: Files,
    base: Opened,
}

impl Machine for MachineFiles {
    fn open_root(&mut self, path: &[u8]) -> Result<skein_sim::Handle, kernel::Error> {
        // The planted root is the simulated machine's absolute namespace.
        let path = path.strip_prefix(b"/").unwrap_or(path);
        self.files
            .open(self.base, path, How::Directory)
            .map(|opened| skein_sim::Handle::new(opened.raw()))
            .map_err(skein_fake_machine::kernel_error)
    }
    fn close_root(&mut self, root: skein_sim::Handle) {
        self.files.close(Opened::new(root.raw()));
    }
    fn open_append(
        &mut self,
        root: skein_sim::Handle,
        path: &[u8],
        mode: u32,
    ) -> Result<skein_sim::Handle, kernel::Error> {
        self.files
            .open(Opened::new(root.raw()), path, How::Append { mode })
            .map(|opened| skein_sim::Handle::new(opened.raw()))
            .map_err(skein_fake_machine::kernel_error)
    }
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        if matches!(&call.ask, skein_sim::Ask::Spawn { program, .. } if program.as_ref() == b"missing") {
            answers.push(skein_sim::Answer { ticket: call.ticket, result: Err(kernel::Error::NotFound) });
        } else {
            skein_fake_machine::step(&mut self.files, call, answers);
        }
    }
}

struct Judge {
    expected: usize,
    met: bool,
}

impl Referee<Process> for Judge {
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        let Process::Parent(parent) = procs.first().expect("parent") else { unreachable!("parent first") };
        assert!(parent.answers.len() <= self.expected, "one answer per ask");
        self.met = parent.answers.len() == self.expected;
    }
    fn overdue(&self, now: Time) -> Option<String> {
        if now > Time::ZERO.saturating_add(Duration::from_secs(1)) { Some("source settles".into()) } else { None }
    }
    fn passed(&self) -> bool {
        self.met
    }
    fn last_word(&self) -> skein_world::LastWord {
        skein_world::LastWord::LastExpectation
    }
}

fn make_fake(spawn: &kernel::Spawn, inherited: &Inherited) -> Process {
    assert!(inherited.pipes.iter().all(|(number, _)| *number != 0), "source commands have no stdin pipe");
    assert_eq!(
        spawn.env.iter().filter(|entry| entry.starts_with(b"ANTHROPIC_")).map(AsRef::as_ref).collect::<Vec<_>>(),
        [b"ANTHROPIC_CONFIG_DIR=/programs/alice/anthropic".as_slice()],
        "source commands use only the source's Anthropic profiles"
    );
    Process::Fake(Box::new((smith_fake_claude::hosted(b"claude").make)(spawn, inherited)))
}

/// Settled host answers, fake observations and remaining login file.
#[derive(Debug)]
pub struct Run {
    pub answers: Vec<SourceAnswer>,
    pub observations: Vec<smith_fake_claude::Observation>,
    pub login: Option<Vec<u8>>,
    pub trace: Vec<String>,
    /// Modes after the check, with absent levels left as None.
    pub directories: [Option<u32>; 2],
    /// Number of commands launched by the parent.
    pub launches: u32,
}

/// Run source asks with chosen directory modes or absent levels on a fake machine.
#[must_use]
pub fn run(
    seed: u64,
    story: Option<Story>,
    login: Option<&[u8]>,
    asks: &[SourceAsk],
    missing: bool,
    extra_environment: &[Box<[u8]>],
    directories: [Option<u32>; 2],
) -> Run {
    // Peak across every program_source test: source checks and sign-out with the hosted fake.
    // The signed-in-otherwise check's hosted fake is largest.
    // Measured peak: 203_129 bytes on 2026-10-29; a quarter of headroom, rounded up.
    const SOURCE_PARENT_CEILING: u64 = 260_000;

    let mut machine = fixture(story, login, directories);
    let root = machine.open_root(b".").expect("parent root");
    let mut environment = vec![Box::from(b"HOME=/home".as_slice())];
    environment.extend_from_slice(extra_environment);
    let mut world = skein_world::World::new(
        seed,
        skein_sim::Config::calm(),
        Judge { expected: asks.len(), met: false },
        skein_world::Memory::Within(SOURCE_PARENT_CEILING),
    )
    .with_machine(machine);
    let factory = smith_fake_claude::hosted(b"claude");
    world.host_startup(
        HostedProgram {
            program: factory.program,
            make: make_fake,
            instances: factory.instances,
            operations: factory.operations,
        },
        |spawn| smith_fake_claude::config_root(spawn).into_iter().chain(smith_fake_claude::mcp_root(spawn)).collect(),
        |spawn| smith_fake_claude::observation_append(spawn).into_iter().collect(),
    );
    world.spawn_root(root, |fd| Process::Parent(Box::new(Parent::new(fd, asks, &environment, missing))));
    collect(world.run())
}

fn fixture(story: Option<Story>, login: Option<&[u8]>, directories: [Option<u32>; 2]) -> MachineFiles {
    let mut items = vec![Item::directory(b"home")];
    for (path, mode) in [b"programs".as_slice(), b"programs/alice".as_slice()].into_iter().zip(directories) {
        if let Some(mode) = mode {
            let mut item = Item::directory(path);
            item.mode = mode;
            items.push(item);
        }
    }
    if let Some(story) = story {
        items.push(Item::file(b"programs/alice/fake-claude.story", &story.encode()));
    }
    if let Some(login) = login {
        items.push(Item::file(b"programs/alice/fake-claude.login", login));
    }
    let mut files = Files::new();
    let base = files.lay(&items);
    MachineFiles { files, base }
}

fn collect(mut outcome: skein_world::Outcome<Process, MachineFiles>) -> Run {
    let base = outcome.machine.base;
    let Process::Parent(parent) = outcome.procs.first_mut().expect("parent") else { unreachable!("parent first") };
    let answers = parent.answers.clone();
    let launches = parent.launches;
    let mut read = |path: &[u8]| match outcome.machine.files.open(base, path, How::Read) {
        Ok(file) => {
            let bytes = outcome.machine.files.read(file, 0, 65_536).expect("fixture file");
            outcome.machine.files.close(file);
            Some(bytes)
        }
        Err(skein_fake_machine::Refusal::NotFound) => None,
        Err(error) => unreachable!("fixture file: {error:?}"),
    };
    let login = read(b"programs/alice/fake-claude.login");
    let observations = read(b"programs/alice/fake-claude.observations")
        .map_or_else(Vec::new, |bytes| smith_fake_claude::observations(&bytes));
    let directories = [b"programs".as_slice(), b"programs/alice".as_slice()].map(|path| {
        match outcome.machine.files.open(base, path, How::Directory) {
            Ok(opened) => {
                let mode = outcome.machine.files.stat(opened).mode;
                outcome.machine.files.close(opened);
                Some(mode)
            }
            Err(skein_fake_machine::Refusal::NotFound) => None,
            Err(error) => unreachable!("fixture directory: {error:?}"),
        }
    });
    outcome.machine.files.close(base);
    assert_eq!(outcome.machine.files.open_handles(), 0, "all source descriptors settle");
    Run {
        answers,
        observations,
        login,
        directories,
        launches,
        trace: outcome.trace.iter().map(|entry| format!("{entry:?}")).collect(),
    }
}
