//! A parent invoking the sign-in fake through pipes in a simulated world
//! (smith/docs/design/protocol/programs.md, section 8; smith/docs/design/testing.md, section 4).

pub mod print;
pub mod relay;

use std::collections::VecDeque;

use skein_fake_machine::{How, Item, Machine as Files, Opened};
use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Op, Pipe, Signal, Spawn, Submit, Target, Way};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Delegate, Host};
use skein_world::{HostedProgram, Inherited, Machine, Referee};
use smith_fake_claude::{Fake, Story};

#[derive(Clone)]
pub struct Invocation {
    args: Vec<String>,
    env: Vec<String>,
    input: Vec<u8>,
    terminate: bool,
    stream: bool,
    no_input: bool,
    signal: Option<(skein_io::kernel::ServiceSignal, String)>,
    control_after: Option<String>,
}

impl Invocation {
    pub fn auth(command: &str) -> Self {
        Self {
            args: vec!["auth".into(), command.into()],
            env: vec!["HOME=home".into(), "CLAUDE_CONFIG_DIR=config".into()],
            input: Vec::new(),
            terminate: false,
            stream: false,
            no_input: false,
            signal: None,
            control_after: None,
        }
    }

    pub fn input(mut self, bytes: &[u8]) -> Self {
        self.input = bytes.into();
        self
    }

    pub fn environment(mut self, environment: &[&str]) -> Self {
        self.env = environment.iter().map(|entry| (*entry).into()).collect();
        self
    }

    pub fn arguments(mut self, arguments: &[&str]) -> Self {
        self.args = arguments.iter().map(|argument| (*argument).into()).collect();
        self
    }

    pub fn terminate(mut self) -> Self {
        self.terminate = true;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct Record {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit: Option<Exit>,
}

#[derive(Debug)]
enum State {
    Spawn,
    Input,
    CloseInput,
    Stdout,
    CloseStdout,
    Stderr,
    CloseStderr,
    Wait,
    CloseChild,
    CloseRoot,
    Done,
}

const CONTROL_INTERRUPT: &[u8] =
    b"{\"type\":\"control_request\",\"request_id\":\"test-1\",\"request\":{\"subtype\":\"interrupt\"}}\n";

pub struct Parent {
    invocations: VecDeque<Invocation>,
    pub records: Vec<Record>,
    record: Record,
    state: State,
    root: Fd,
    pipes: Vec<Fd>,
    child: Option<Fd>,
    input_from: u32,
    waiting: bool,
    signal_waiting: bool,
    control_write: Option<Token>,
    control_ready: bool,
    signal_at: Option<Time>,
    pub signalled: bool,
    input_closed: bool,
    output_ended: bool,
    input_end: usize,
    signal_ready: bool,
    sequence: u64,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}

impl Parent {
    fn new(root: Fd, invocations: Vec<Invocation>) -> Self {
        Self {
            invocations: invocations.into(),
            records: Vec::new(),
            record: Record::default(),
            state: State::Spawn,
            root,
            pipes: Vec::new(),
            child: None,
            input_from: 0,
            waiting: false,
            signal_waiting: false,
            control_write: None,
            control_ready: false,
            signal_at: None,
            signalled: false,
            input_closed: false,
            output_ended: false,
            input_end: 0,
            signal_ready: false,
            sequence: 0,
            completions: Queue::with_capacity(8),
            submissions: Queue::with_capacity(8),
        }
    }

    fn issue(&mut self, kind: Op) {
        self.sequence += 1;
        self.submissions.push(Submit { op: Token::new(self.sequence), kind });
    }

    fn send_control(&mut self) {
        self.control_ready = false;
        self.invocations.front_mut().expect("invocation").control_after = None;
        self.issue(Op::PipeWrite { fd: self.pipes[0], bytes: CONTROL_INTERRUPT.into(), from: 0 });
        self.control_write = Some(Token::new(self.sequence));
    }

    fn complete(&mut self, complete: Complete, now: Time) {
        if self.control_write == Some(complete.op) {
            assert_eq!(
                complete.result,
                Ok(Done::Count(u32::try_from(CONTROL_INTERRUPT.len()).expect("short control line")))
            );
            self.control_write = None;
            return;
        }
        if matches!(complete.kind, Op::Signal { .. }) {
            assert_eq!(complete.result, Ok(Done::Nothing));
            self.signal_waiting = false;
            return;
        }
        self.waiting = false;
        let result = complete.result.expect("parent operation succeeds");
        match self.state {
            State::Spawn => {
                let Done::Spawned { pidfd, .. } = result else { panic!("child spawn") };
                let Op::Spawn { spawn } = complete.kind else { panic!("spawn operation") };
                self.child = Some(pidfd);
                self.pipes = (0..3)
                    .map(|number| {
                        spawn
                            .pipes
                            .iter()
                            .find(|pipe| pipe.child == number)
                            .map(|pipe| pipe.parent.expect("parent pipe"))
                            .unwrap_or(Fd::new(-1))
                    })
                    .collect();
                self.input_from = 0;
                self.input_end = 0;
                self.input_closed = self.invocations.front().expect("invocation").no_input;
                self.output_ended = false;
                self.signal_ready = false;
                self.signalled = false;
                if self.invocations.front().expect("invocation").terminate {
                    self.signal_at = Some(now.saturating_add(Duration::from_millis(20)));
                }
                self.state =
                    if self.invocations.front().expect("invocation").stream { State::Stdout } else { State::Input };
            }
            State::Input => {
                let Done::Count(count) = result else { panic!("input write") };
                self.input_from += count;
                if self.invocations.front().expect("invocation").stream && self.input_from as usize == self.input_end {
                    self.state = State::Stdout;
                }
            }
            State::CloseInput => {
                self.input_closed = true;
                self.state = if self.output_ended { State::CloseStdout } else { State::Stdout };
            }
            State::Stdout | State::Stderr => {
                let Done::Count(count) = result else { panic!("output read") };
                let Op::PipeRead { buf, .. } = complete.kind else { panic!("pipe read") };
                match self.state {
                    State::Stdout => {
                        self.record.stdout.extend_from_slice(&buf[..count as usize]);
                        if count == 0 {
                            self.output_ended = true;
                            if self.invocations.front().expect("invocation").stream && !self.input_closed {
                                self.state = State::CloseInput;
                                self.issue(Op::Close { fd: self.pipes[0] });
                                self.waiting = true;
                            } else {
                                self.state = State::CloseStdout;
                            }
                        } else if self.record.stdout.last() == Some(&b'\n') {
                            let line =
                                self.record.stdout.split(|byte| *byte == b'\n').rev().nth(1).expect("output line");
                            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(line) {
                                let kind =
                                    value["subtype"].as_str().unwrap_or_else(|| value["type"].as_str().unwrap_or(""));
                                if !self.signalled
                                    && self
                                        .invocations
                                        .front()
                                        .expect("invocation")
                                        .signal
                                        .as_ref()
                                        .is_some_and(|(_, after)| after == kind)
                                {
                                    self.signal_ready = true;
                                }
                                if self.invocations.front().expect("invocation").control_after.as_deref() == Some(kind)
                                {
                                    self.send_control();
                                }
                                if self.invocations.front().expect("invocation").stream
                                    && !self.input_closed
                                    && (kind == "init" || value["type"] == "result")
                                {
                                    self.state = State::Input;
                                }
                            }
                        }
                    }
                    State::Stderr => {
                        self.record.stderr.extend_from_slice(&buf[..count as usize]);
                        if count == 0 {
                            self.state = State::CloseStderr;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            State::CloseStdout => self.state = State::Stderr,
            State::CloseStderr => self.state = State::Wait,
            State::Wait => {
                let Done::Exit(exit) = result else { panic!("child exit") };
                self.record.exit = Some(exit);
                self.records.push(std::mem::take(&mut self.record));
                self.state = State::CloseChild;
            }
            State::CloseChild => {
                self.child = None;
                self.invocations.pop_front();
                self.state = State::Spawn;
            }
            State::CloseRoot => self.state = State::Done,
            State::Done => panic!("no operation after settlement"),
        }
    }

    fn advance(&mut self) {
        let kind = match self.state {
            State::Spawn => match self.invocations.front() {
                Some(invocation) => Op::Spawn {
                    spawn: Box::new(Spawn {
                        program: b"claude".as_slice().into(),
                        args: invocation.args.iter().map(|arg| arg.as_bytes().into()).collect(),
                        env: invocation.env.iter().map(|entry| entry.as_bytes().into()).collect(),
                        root: self.root,
                        dir: Box::new([]),
                        pipes: [
                            Pipe { child: 0, way: Way::In, parent: None },
                            Pipe { child: 1, way: Way::Out, parent: None },
                            Pipe { child: 2, way: Way::Out, parent: None },
                        ]
                        .into_iter()
                        .filter(|pipe| !invocation.no_input || pipe.child != 0)
                        .collect(),
                    }),
                },
                None => {
                    self.state = State::CloseRoot;
                    Op::Close { fd: self.root }
                }
            },
            State::Input => {
                let invocation = self.invocations.front().expect("invocation");
                if invocation.stream
                    && self.input_from as usize == self.input_end
                    && self.input_end < invocation.input.len()
                {
                    self.input_end +=
                        invocation.input[self.input_end..].iter().position(|byte| *byte == b'\n').expect("user line")
                            + 1;
                }
                let bytes = &self.invocations.front().expect("invocation").input;
                if self.input_from as usize == bytes.len() {
                    self.state = State::CloseInput;
                    Op::Close { fd: self.pipes[0] }
                } else {
                    if invocation.stream && self.input_from as usize == self.input_end {
                        self.state = State::Stdout;
                        return;
                    }
                    let end = if invocation.stream { self.input_end } else { bytes.len() };
                    Op::PipeWrite { fd: self.pipes[0], bytes: bytes[..end].into(), from: self.input_from }
                }
            }
            State::Stdout => Op::PipeRead {
                fd: self.pipes[1],
                buf: vec![0; if self.invocations.front().expect("invocation").stream { 1 } else { 1024 }]
                    .into_boxed_slice(),
            },
            State::Stderr => Op::PipeRead { fd: self.pipes[2], buf: vec![0; 1024].into_boxed_slice() },
            State::CloseStdout => Op::Close { fd: self.pipes[1] },
            State::CloseStderr => Op::Close { fd: self.pipes[2] },
            State::Wait => Op::Wait { pidfd: self.child.expect("child"), reap: false },
            State::CloseChild => Op::Close { fd: self.child.expect("child") },
            State::CloseInput | State::CloseRoot | State::Done => return,
        };
        self.issue(kind);
        self.waiting = true;
    }
}

impl Host for Parent {
    fn iterate(&mut self, now: Time, _wall: Wall) {
        if self.control_ready {
            self.send_control();
        }
        while let Some(complete) = self.completions.pop() {
            self.complete(complete, now);
        }
        if self.signal_at.is_some_and(|at| now >= at) {
            assert!(
                self.record.stdout.is_empty() || self.signalled,
                "a timed termination follows silence or the scripted interrupt"
            );
            assert!(self.record.exit.is_none(), "silent status has not exited before its signal");
            self.signal_at = None;
            self.signalled = true;
            self.signal_waiting = true;
            self.issue(Op::Signal {
                pidfd: self.child.expect("silent child"),
                signal: Signal::Terminate,
                to: Target::Child,
            });
        }
        if !self.waiting && !self.signal_waiting && (!self.signal_ready || self.signalled) {
            self.advance();
        }
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || self.control_ready
            || (!self.waiting && !self.signal_waiting && !self.is_empty())
            || self.signal_at.is_some_and(|at| now >= at)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.signal_at
    }
    fn is_empty(&self) -> bool {
        matches!(self.state, State::Done) && self.completions.is_empty() && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        8
    }
}

pub enum Process {
    Parent(Box<Parent>),
    Fake(Box<Fake>),
    Server(Box<smith_fake_claude::Server>),
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Process::Parent(host) => host.as_ref(),
            Process::Fake(host) => host.as_ref(),
            Process::Server(host) => host.as_ref(),
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Process::Parent(host) => host.as_mut(),
            Process::Fake(host) => host.as_mut(),
            Process::Server(host) => host.as_mut(),
        }
    }
}

pub struct Judge {
    last_word: skein_world::LastWord,
    controls: skein_world::Controls,
}

impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let server_called = match procs.get(2) {
            Some(Process::Server(server)) => !server.calls().is_empty(),
            _ => false,
        };
        let server_present = procs.len() > 2;
        if let Process::Parent(parent) = &mut procs[0] {
            if server_called
                && parent.invocations.front().and_then(|invocation| invocation.control_after.as_deref()) == Some("call")
            {
                parent.control_ready = true;
            }
            let after = parent
                .invocations
                .front()
                .and_then(|invocation| invocation.signal.as_ref())
                .map(|(_, after)| after.as_str());
            if (after == Some("server") && server_present) || (after == Some("call") && server_called) {
                parent.signal_ready = true;
            }
        }
        if let Process::Parent(parent) = &mut procs[0]
            && parent.signal_ready
            && !parent.signalled
        {
            let signal = parent.invocations.front().expect("invocation").signal.as_ref().expect("signal").0;
            self.controls.signal(1, signal);
            parent.signalled = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        let Process::Parent(parent) = &procs[0] else { panic!("parent first") };
        if let (Some(Process::Fake(fake)), Some(Process::Server(server))) = (procs.get(1), procs.get(2))
            && fake.exit().is_some()
        {
            assert_eq!(server.exit(), Some(Exit::Code(0)), "server settles before fake exits");
        }
        if parent.signal_at.is_some() {
            for child in procs.iter().skip(1) {
                assert!(child.exit().is_none(), "silent child waits for termination");
            }
        }
    }
    fn passed(&self) -> bool {
        true
    }
    fn last_word(&self) -> skein_world::LastWord {
        self.last_word
    }
}

pub struct MachineFiles {
    files: Files,
    base: Opened,
}

impl Machine for MachineFiles {
    fn open_root(&mut self, path: &[u8]) -> Result<skein_sim::Handle, Error> {
        self.files
            .open(self.base, path, How::Directory)
            .map(|opened| skein_sim::Handle::new(opened.raw()))
            .map_err(skein_fake_machine::kernel_error)
    }
    fn close_root(&mut self, root: skein_sim::Handle) {
        self.files.close(Opened::new(root.raw()));
    }
    fn open_append(&mut self, root: skein_sim::Handle, path: &[u8], mode: u32) -> Result<skein_sim::Handle, Error> {
        self.files
            .open(Opened::new(root.raw()), path, How::Append { mode })
            .map(|opened| skein_sim::Handle::new(opened.raw()))
            .map_err(skein_fake_machine::kernel_error)
    }
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        skein_fake_machine::step(&mut self.files, call, answers);
    }
}

fn make_server(spawn: &Spawn, inherited: &Inherited) -> Process {
    Process::Server(Box::new((smith_fake_claude::served(b"server").make)(spawn, inherited)))
}

fn make_fake(spawn: &Spawn, inherited: &Inherited) -> Process {
    Process::Fake(Box::new((smith_fake_claude::hosted(b"claude").make)(spawn, inherited)))
}

pub struct Run {
    pub peaks: Option<Vec<u64>>,
    pub records: Vec<Record>,
    pub login: Option<Vec<u8>>,
    pub config_exists: bool,
    pub observations: Option<Vec<u8>>,
    pub signalled: bool,
    pub files: Vec<File>,
    pub calls: Vec<String>,
    pub server_exit: Option<Exit>,
}

pub struct File {
    pub path: Vec<u8>,
    pub bytes: Option<Vec<u8>>,
}

impl Run {
    pub fn file(&self, path: &[u8]) -> Option<&[u8]> {
        self.files.iter().find(|file| file.path == path).and_then(|file| file.bytes.as_deref())
    }
}

#[derive(Default)]
pub struct Options<'a> {
    pub story: Option<Story>,
    pub login: Option<&'a [u8]>,
    pub extra: Vec<Item>,
    pub paths: &'a [&'a [u8]],
    pub without_directory: bool,
}

pub fn run(story: Option<Story>, login: Option<&[u8]>, invocations: Vec<Invocation>) -> Run {
    run_with(invocations, Options { story, login, ..Options::default() })
}

pub fn run_without_directory(invocation: Invocation) -> Run {
    run_with(vec![invocation], Options { without_directory: true, ..Options::default() })
}

pub fn run_with(invocations: Vec<Invocation>, options: Options<'_>) -> Run {
    let Options { story, login, extra, paths, without_directory } = options;
    let config_present = !without_directory;
    let mut items = vec![Item::directory(b"home")];
    if config_present {
        items.push(Item::directory(b"config"));
    }
    if let Some(story) = story {
        items.push(Item::file(b"config/fake-claude.story", &story.encode()));
    }
    if let Some(login) = login {
        items.push(Item::file(b"config/fake-claude.login", login));
    }
    items.extend(extra);
    let mut files = Files::new();
    let base = files.lay(&items);
    let mut machine = MachineFiles { files, base };
    let root = machine.open_root(b".").expect("parent root");
    let last_word = if invocations.iter().any(|invocation| invocation.terminate) {
        skein_world::LastWord::After(Time::ZERO.saturating_add(Duration::from_millis(20)))
    } else {
        skein_world::LastWord::LastExpectation
    };
    let mut world =
        skein_world::World::new_controlled(7, skein_sim::Config::calm(), skein_world::Memory::Checked, |controls| {
            Judge { last_word, controls }
        })
        .with_machine(machine);
    let server = smith_fake_claude::served(b"server");
    world.host(HostedProgram {
        program: server.program,
        make: make_server,
        instances: server.instances,
        operations: server.operations,
    });
    let factory = smith_fake_claude::hosted(b"claude");
    let program = HostedProgram {
        program: factory.program,
        make: make_fake,
        instances: factory.instances,
        operations: factory.operations,
    };
    if config_present {
        world.host_startup(
            program,
            |spawn| {
                smith_fake_claude::config_root(spawn).into_iter().chain(smith_fake_claude::mcp_root(spawn)).collect()
            },
            |spawn| smith_fake_claude::observation_append(spawn).into_iter().collect(),
        );
    } else {
        // The binary admits ENOENT at startup with no configuration or append
        // descriptor; the command still receives CLAUDE_CONFIG_DIR=config.
        world.host(program);
    }
    world.spawn_root(root, |fd| Process::Parent(Box::new(Parent::new(fd, invocations.clone()))));
    let mut outcome = world.run();
    let Process::Parent(parent) = &outcome.procs[0] else { panic!("parent process") };
    let records = parent.records.clone();
    let signalled = parent.signalled;
    let (calls, server_exit) = match outcome.procs.get(2) {
        Some(Process::Server(server)) => (server.calls().to_vec(), server.exit()),
        _ => (Vec::new(), None),
    };
    let mut read = |path: &[u8]| match outcome.machine.files.open(base, path, How::Read) {
        Ok(file) => {
            let bytes = outcome.machine.files.read(file, 0, 65_536).expect("fixture file");
            outcome.machine.files.close(file);
            Some(bytes)
        }
        Err(skein_fake_machine::Refusal::NotFound) => None,
        Err(error) => panic!("fixture file: {error:?}"),
    };
    let login = read(b"config/fake-claude.login");
    let observations = read(b"config/fake-claude.observations");
    let files = paths.iter().map(|path| File { path: path.to_vec(), bytes: read(path) }).collect();
    let config_exists = match outcome.machine.files.open(base, b"config", How::PathNoFollow) {
        Ok(config) => {
            outcome.machine.files.close(config);
            true
        }
        Err(skein_fake_machine::Refusal::NotFound) => false,
        Err(error) => panic!("configuration directory: {error:?}"),
    };
    outcome.machine.files.close(base);
    assert_eq!(outcome.machine.files.open_handles(), 0, "all invocation descriptors settle");
    Run { peaks: outcome.peaks(), records, login, config_exists, observations, signalled, calls, server_exit, files }
}
