//! Successive jobs through the component, IO and simulated child pipes
//! (smith/docs/design/protocol/agent.md, 2; testing-strategy.md, 2.4).
//! The shell fixture knows only its authored output and sleep; the component
//! owns yields, retained output, deadlines, cancellation and settlement.

use super::{Adapter, Application, Job, Terminal, World, copy_terminal, job_terminals};
use skein_fake_machine::How;
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Spawn, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_sim::{Config, Handle};
use skein_world::{Expectation, Expectations, Host, HostedProgram, Inherited, Referee};
use smith_protocol_machine::{FromDomain, ToDomain};
use std::rc::Rc;

impl World {
    /// Run successive job acts with authored shell output on simulated pipes.
    pub fn simulated_jobs(&mut self, requests: &[FromDomain], delays: &[Duration]) -> Vec<ToDomain> {
        let owner = Job::Request(requests.first().expect("one spawn").clone(), false).owner();
        let root = self.machine.borrow_mut().open(self.root_handle, b".", How::Directory).expect("fresh job root");
        let referee = Expectations::new(
            self.seed,
            vec![Terminal { owner, expected: job_terminals(requests), deadline: Time::from_nanos(4_000_000_000) }],
        );
        let mut config = Config::calm();
        config.wall = Wall::from_nanos(self.now.as_nanos());
        let mut world = skein_world::World::new(self.seed, config, referee, self.memory)
            .with_machine(Adapter { files: Rc::clone(&self.machine), spawns: Rc::clone(&self.spawns) });
        world.host(HostedProgram { program: b"/bin/sh".as_slice().into(), make: shell, instances: 1, operations: 1 });
        world.spawn_root(Handle::new(root.raw()), |root| {
            Process::Machine(Box::new(Application::new(
                self.seed,
                self.writable,
                self.now,
                root,
                Job::Sequence { requests: requests.to_vec(), delays: delays.to_vec() },
                self.limits,
                &self.environment,
            )))
        });
        let outcome = world.run();
        let application = machine(&outcome.procs);
        assert!(application.component.closed(), "machine closes after every child and pipe terminal");
        assert!(application.is_empty(), "root and IO settle after the sequence");
        let answers = application.terminals.iter().map(copy_terminal).collect();
        self.trace.extend(outcome.trace.iter().map(|entry| format!("{entry:?}")));
        self.now = self.now.saturating_add(Duration::from_nanos(outcome.end.as_nanos()));
        if let Some(peaks) = outcome.peaks() {
            self.peaks.extend(peaks);
        }
        drop(outcome);
        self.settle();
        answers
    }
}

enum Process {
    Machine(Box<Application>),
    Shell(Shell),
}

impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Machine(application) => application.as_ref(),
            Self::Shell(shell) => shell,
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Machine(application) => application.as_mut(),
            Self::Shell(shell) => shell,
        }
    }
}

impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }

    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
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

fn machine(procs: &[Process]) -> &Application {
    match &procs[0] {
        Process::Machine(application) => application,
        Process::Shell(_) => panic!("the root process is the machine"),
    }
}

impl Expectation<Process> for Terminal {
    fn check(&self, now: Time, procs: &[Process]) -> Result<bool, String> {
        <Self as Expectation<Application>>::check(self, now, std::slice::from_ref(machine(procs)))
    }

    fn deadline(&self) -> Time {
        self.deadline
    }
}

impl Referee<Process> for Expectations<Process, Terminal> {
    fn observe(&mut self, now: Time, procs: &[Process]) {
        let application = machine(procs);
        assert!(application.terminals.len() <= application.expected, "one terminal per answering act");
        Expectations::observe(self, now, procs);
    }

    fn next_deadline(&self) -> Option<Time> {
        Expectations::next_deadline(self)
    }

    fn overdue(&self, now: Time) -> Option<String> {
        Expectations::overdue(self, now)
    }

    fn passed(&self) -> bool {
        Expectations::passed(self)
    }
}

// These are the real job stories' shell commands, with the same sleep inputs.
// This fixture interprets no other shell syntax and makes no job-policy decisions.
fn shell(spawn: &Spawn, inherited: &Inherited) -> Process {
    let (before, delay, after): (&[u8], Duration, &[u8]) = match spawn.args[1].as_ref() {
        b"printf one; sleep 0.08; printf two" => (b"one", Duration::from_millis(80), b"two"),
        b"printf abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz; sleep 0.08; printf end" => {
            (b"abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz", Duration::from_millis(80), b"end")
        }
        b"sleep 5" => (b"", Duration::from_secs(5), b""),
        b"sleep 0.06" => (b"", Duration::from_millis(60), b""),
        b"sleep 1" => (b"", Duration::from_secs(1), b""),
        other => panic!("unauthored shell fixture: {other:?}"),
    };
    let stdout = inherited.pipes.iter().find(|(child, _)| *child == 1).expect("stdout pipe").1;
    Process::Shell(Shell {
        stdout,
        fds: inherited.pipes.iter().map(|(_, fd)| *fd).chain(std::iter::once(inherited.signal)).collect(),
        before: Some(before.into()),
        delay: Some(delay),
        until: None,
        after: Some(after.into()),
        completions: Queue::with_capacity(1),
        submissions: Queue::with_capacity(1),
        waiting: false,
        closed: false,
        next: 1,
    })
}

struct Shell {
    stdout: Fd,
    fds: Vec<Fd>,
    before: Option<Box<[u8]>>,
    delay: Option<Duration>,
    until: Option<Time>,
    after: Option<Box<[u8]>>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    waiting: bool,
    closed: bool,
    next: u64,
}

impl Shell {
    fn submit(&mut self, kind: Op) {
        self.submissions.push(Submit { op: Token::new(self.next), kind });
        self.next = self.next.checked_add(1).expect("finite shell operations");
        self.waiting = true;
    }

    fn write(&mut self, bytes: Box<[u8]>) -> bool {
        if bytes.is_empty() {
            return false;
        }
        self.submit(Op::PipeWrite { fd: self.stdout, bytes, from: 0 });
        true
    }
}

impl Host for Shell {
    fn iterate(&mut self, now: Time, _wall: Wall) {
        if let Some(complete) = self.completions.pop() {
            self.waiting = false;
            match (complete.kind, complete.result) {
                (Op::PipeWrite { fd, bytes, from }, Ok(Done::Count(count))) => {
                    assert!(count > 0, "a pipe write makes progress");
                    let from = from.checked_add(count).expect("bounded fixture output");
                    if usize::try_from(from).expect("fixture length fits") < bytes.len() {
                        self.submit(Op::PipeWrite { fd, bytes, from });
                    }
                }
                (Op::Close { .. }, Ok(Done::Nothing)) => {}
                other => panic!("shell pipe operation failed: {other:?}"),
            }
        }
        if self.waiting || self.closed {
            return;
        }
        if let Some(bytes) = self.before.take()
            && self.write(bytes)
        {
            return;
        }
        if let Some(delay) = self.delay.take() {
            self.until = Some(now.saturating_add(delay));
        }
        if self.until.is_some_and(|at| now < at) {
            return;
        }
        self.until = None;
        if let Some(bytes) = self.after.take()
            && self.write(bytes)
        {
            return;
        }
        if let Some(fd) = self.fds.pop() {
            self.submit(Op::Close { fd });
        } else {
            self.closed = true;
        }
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }

    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty() || (!self.waiting && !self.closed && self.until.is_none_or(|at| now >= at))
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.until
    }

    fn is_empty(&self) -> bool {
        self.closed && self.completions.is_empty() && self.submissions.is_empty()
    }

    fn exit(&self) -> Option<Exit> {
        self.is_empty().then_some(Exit::Code(0))
    }

    fn operations(&self) -> u32 {
        1
    }
}
