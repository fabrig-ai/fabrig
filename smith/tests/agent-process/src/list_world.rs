//! Standalone list child over process pipes and the shared fake (smith/docs/design/protocol/agent.md, 6.1).
//! The parent captures bytes and exit, and the referee ends the peer after the child.

use skein_fake_peers::llm::{Observation, Peer};
use skein_io::kernel::{self, Done, Exit, Fd, Op, Pipe, Spawn, Way};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Delegate, Host};
use skein_world::{HostedProgram, Inherited, Referee};

/// Wire result, captured independently of the child.
pub struct Run {
    pub output: Vec<u8>,
    pub errors: Vec<u8>,
    pub exit: Exit,
    pub observations: Vec<Observation>,
    pub opens: usize,
    pub spawns: usize,
}

enum Proc {
    Driver(Box<Driver>),
    List(Box<smith_agent_shell::list::Process>),
    Peer(Box<Peer>),
}

impl Delegate for Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Driver(p) => p.as_ref(),
            Self::List(p) => p.as_ref(),
            Self::Peer(p) => p.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Driver(p) => p.as_mut(),
            Self::List(p) => p.as_mut(),
            Self::Peer(p) => p.as_mut(),
        }
    }
}

// Both the pipe parent and a startup refusal use this finite raw pipe driver.
struct Driver {
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
    next: u64,
    pending: u32,
    input: Option<(Fd, Box<[u8]>, u32)>,
    output: Option<Fd>,
    error: Option<Fd>,
    pidfd: Option<Fd>,
    status: Option<Exit>,
    own_exit: u8,
    bytes: Vec<u8>,
    errors: Vec<u8>,
}

impl Driver {
    fn new() -> Self {
        Self {
            completions: Queue::uncapped(),
            submissions: Queue::uncapped(),
            next: 0,
            pending: 0,
            input: None,
            output: None,
            error: None,
            pidfd: None,
            status: None,
            own_exit: 0,
            bytes: Vec::new(),
            errors: Vec::new(),
        }
    }
    fn submit(&mut self, kind: Op) {
        self.next += 1;
        self.pending += 1;
        self.submissions.push(kernel::Submit { op: Token::new(self.next), kind });
    }
    fn close(&mut self, fd: Fd) {
        self.submit(Op::Close { fd });
    }
    fn read(&mut self, fd: Fd) {
        self.submit(Op::PipeRead { fd, buf: vec![0; 8192].into() });
    }
    fn write(&mut self) {
        let Some((fd, bytes, from)) = self.input.take() else { return };
        if usize::try_from(from).expect("pipe offset") == bytes.len() {
            self.close(fd);
        } else {
            self.submit(Op::PipeWrite { fd, bytes, from });
        }
    }
    fn completed(&mut self, complete: kernel::Complete) {
        self.pending -= 1;
        match complete.kind {
            Op::Spawn { spawn } => {
                let Ok(Done::Spawned { pidfd, .. }) = complete.result else { panic!("list child spawned") };
                self.close(spawn.root);
                let (_, bytes, from) = self.input.take().expect("ask");
                self.input = Some((spawn.pipes[0].parent.expect("stdin"), bytes, from));
                self.write();
                self.output = spawn.pipes[1].parent;
                self.error = spawn.pipes[2].parent;
                self.read(self.output.expect("stdout"));
                self.read(self.error.expect("stderr"));
                self.pidfd = Some(pidfd);
                self.submit(Op::Wait { pidfd, reap: false });
            }
            Op::PipeWrite { fd, bytes, from } => {
                let Ok(Done::Count(count)) = complete.result else { panic!("ask write") };
                self.input = Some((fd, bytes, from.checked_add(count).expect("bounded write")));
                self.write();
            }
            Op::PipeRead { fd, buf } => {
                let Ok(Done::Count(count)) = complete.result else { panic!("answer read") };
                if count == 0 {
                    if self.output == Some(fd) {
                        self.output = None;
                    } else {
                        self.error = None;
                    }
                    self.close(fd);
                } else {
                    let bytes = &buf[..usize::try_from(count).expect("read count")];
                    if self.output == Some(fd) {
                        self.bytes.extend_from_slice(bytes);
                    } else {
                        self.errors.extend_from_slice(bytes);
                    }
                    self.read(fd);
                }
            }
            Op::Wait { pidfd, reap } => {
                let Ok(Done::Exit(exit)) = complete.result else { panic!("list exit") };
                self.status = Some(exit);
                if reap {
                    self.pidfd = None;
                    self.close(pidfd);
                } else {
                    self.submit(Op::Wait { pidfd, reap: true });
                }
            }
            Op::Close { .. } => assert!(complete.result.is_ok(), "pipe driver close"),
            other => panic!("unexpected pipe driver operation {other:?}"),
        }
    }
}

impl Host for Driver {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.completed(complete);
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
        self.pending == 0
            && self.input.is_none()
            && self.output.is_none()
            && self.error.is_none()
            && self.pidfd.is_none()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then_some(Exit::Code(self.own_exit))
    }
    fn operations(&self) -> u32 {
        16
    }
}

fn make(spawn: &Spawn, inherited: &Inherited) -> Proc {
    let pipe = |number| inherited.pipes.iter().find(|(child, _)| *child == number).expect("child pipe").1;
    let process = smith_agent_shell::config::decode(&spawn.args[0]).and_then(|decoded| {
        smith_agent_shell::list::Process::new(
            decoded,
            smith_agent_shell::list::Faces {
                stdin: pipe(0),
                stdout: pipe(1),
                stderr: Some(pipe(2)),
                signal: inherited.signal,
            },
        )
    });
    match process {
        Ok(process) => Proc::List(Box::new(process)),
        Err(why) => {
            let mut driver = Driver::new();
            driver.own_exit = 1;
            driver.close(pipe(0));
            driver.close(pipe(1));
            driver.close(inherited.signal);
            driver.input = Some((pipe(2), format!("smith-agent: {why}\n").into_bytes().into(), 0));
            driver.write();
            Proc::Driver(Box::new(driver))
        }
    }
}

struct Judge {
    finding: Option<String>,
    ended: bool,
}
impl Judge {
    fn act_checked(&mut self, _now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let done = matches!(procs.first().ok_or("parent")?, Proc::Driver(parent) if parent.status.is_some() && parent.is_empty());
        if done && !self.ended {
            self.ended = true;
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
        self.finding.is_none() && (self.ended)
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.ended).then_some(Time::ZERO.saturating_add(Duration::from_secs(30)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.next_deadline().filter(|deadline| now >= *deadline).map(|_| "list child did not settle".into())
    }
}

/// Host the shipped list process with one EOF-terminated ask and optional fake peer.
#[must_use]
pub fn run(configuration: &[u8], ask: &[u8], peer: Option<impl FnOnce() -> Peer>) -> Run {
    let mut files = skein_fake_machine::Machine::new();
    let root = files.lay(&[]);
    let parent_root = files.open(root, b".", skein_fake_machine::How::Directory).expect("parent root");
    let mut world = skein_world::World::new(
        7,
        skein_sim::Config::calm(),
        Judge { finding: None, ended: false },
        skein_world::Memory::Checked,
    )
    .with_machine(skein_fake_machine::Rooted { files, root });
    world.host(HostedProgram {
        program: b"smith-agent".as_slice().into(),
        make,
        instances: 1,
        operations: smith_agent_shell::list::operations(),
    });
    world.spawn_root(skein_sim::Handle::new(parent_root.raw()), |root| {
        let mut parent = Driver::new();
        parent.input = Some((Fd::new(0), ask.into(), 0));
        parent.submit(Op::Spawn {
            spawn: Box::new(Spawn {
                program: b"smith-agent".as_slice().into(),
                args: Box::new([configuration.into()]),
                env: Box::new([]),
                root,
                dir: Box::new([]),
                pipes: Box::new([
                    Pipe { child: 0, way: Way::In, parent: None },
                    Pipe { child: 1, way: Way::Out, parent: None },
                    Pipe { child: 2, way: Way::Out, parent: None },
                ]),
            }),
        });
        Proc::Driver(Box::new(parent))
    });
    if let Some(make_peer) = peer {
        world.spawn(|| Proc::Peer(Box::new(make_peer())));
    }
    let mut outcome = world.run();
    let Proc::Driver(parent) = &outcome.procs[0] else { panic!("parent") };
    let opens = outcome
        .trace
        .iter()
        .filter(|entry| matches!(&entry.event, skein_sim::Event::Submit { kind: skein_sim::Summary::Open { .. }, .. }))
        .count();
    let spawns = outcome
        .trace
        .iter()
        .filter(|entry| matches!(&entry.event, skein_sim::Event::Submit { kind: skein_sim::Summary::Spawn { .. }, .. }))
        .count();
    let run = Run {
        opens,
        spawns,
        output: parent.bytes.clone(),
        errors: parent.errors.clone(),
        exit: parent.status.expect("child exit"),
        observations: outcome
            .procs
            .iter()
            .filter_map(|proc| match proc {
                Proc::Peer(peer) => Some(peer.observations()),
                Proc::Driver(_) | Proc::List(_) => None,
            })
            .flatten()
            .map(copy_observation)
            .collect(),
    };
    outcome.machine.files.close(root);
    assert_eq!(outcome.machine.files.open_handles(), 0, "list world handles settle");
    run
}

fn copy_observation(value: &Observation) -> Observation {
    match value {
        Observation::List { connection, route, scheme, index } => {
            Observation::List { connection: *connection, route: *route, scheme: *scheme, index: *index }
        }
        Observation::Query { .. } => panic!("a list starts no completion"),
        Observation::Accepted { connection } => Observation::Accepted { connection: *connection },
        Observation::Closed { connection } => Observation::Closed { connection: *connection },
        Observation::Head { connection, headers } => {
            Observation::Head { connection: *connection, headers: headers.clone() }
        }
        Observation::Credential { connection, route, scheme, index } => {
            Observation::Credential { connection: *connection, route: *route, scheme: *scheme, index: *index }
        }
        Observation::Answered { .. } => panic!("a list starts no run"),
    }
}
