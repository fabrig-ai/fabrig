//! One bounded command on one skein-io kernel, including termination.
use skein_io::{
    Event, Io, Limits, Request,
    kernel::{Complete, Exit, Fd, Pipe, Spawn, Submit, Way},
};
use skein_lib::{
    Env, Queue, Secret, Time, Token, Wall,
    stream::{Down, Read, Up},
};
use skein_shell::{Clock, Config, Host, Kernel};
use temper_forge_setup::{
    prepare::{self, Ran},
    spec::Spec,
};
/// Why a command did not run to completion.
#[derive(Debug)]
pub enum Error {
    Kernel(String),
    Spawn(skein_io::Error),
    Interrupted,
}
pub(crate) fn first_line(ran: &Ran) -> std::borrow::Cow<'_, str> {
    let line = ran.stdout.expose().split(|b| *b == b'\n').next().unwrap_or_default();
    String::from_utf8_lossy(line.strip_suffix(b"\r").unwrap_or(line))
}

const OWNER: Token = Token::new(1);
/// Reads both pipes through their end, retaining only the configured prefix.
pub fn run_one(limits: &Limits, spec: &Spec) -> Result<Ran, Error> {
    run(limits, spec, None)
}
/// One deadline shared by a box's source-list and source-add commands (protocol/setup/box.md, 1).
pub fn run_until(limits: &Limits, spec: &Spec, deadline: Time) -> Result<Ran, Error> {
    run(limits, spec, Some(deadline))
}
fn run(limits: &Limits, spec: &Spec, deadline: Option<Time>) -> Result<Ran, Error> {
    let signals = skein_shell::open_termination_signals().map_err(|n| Error::Kernel(format!("signals: errno {n}")))?;
    let operations = skein_io::operations(limits).expect("usable io limits");
    let mut kernel = match Kernel::open(Config { operations }) {
        Ok(kernel) => kernel,
        Err(why) => {
            skein_shell::close_startup_fd(signals);
            return Err(Error::Kernel(why.to_string()));
        }
    };
    let root = match skein_shell::open_root(std::path::Path::new("/")) {
        Ok(root) => root,
        Err(n) => {
            skein_shell::close_startup_fd(signals);
            return Err(Error::Kernel(format!("root: errno {n}")));
        }
    };
    capture_with(limits, spec, operations, root, signals, deadline, |host| {
        skein_shell::drive(&mut kernel, &Clock::new(), host);
        skein_shell::close_startup_fd(root);
    })
}

/// Captures one command over supplied kernel and clock turns without opening startup descriptors.
#[doc(hidden)]
pub fn run_with(
    turns: &mut impl skein_shell::Turns,
    limits: &Limits,
    spec: &Spec,
    root: Fd,
    signals: Fd,
) -> Result<Ran, Error> {
    let operations = skein_io::operations(limits).expect("usable io limits");
    capture_with(limits, spec, operations, root, signals, None, |host| {
        skein_shell::drive_with(turns, host);
    })
}

fn capture_with(
    limits: &Limits,
    spec: &Spec,
    operations: u32,
    root: Fd,
    signals: Fd,
    deadline: Option<Time>,
    drive: impl FnOnce(&mut Running),
) -> Result<Ran, Error> {
    let mut host = Running::new(limits, spec, operations, root, signals, deadline);
    drive(&mut host);
    host.answer()
}

impl Running {
    fn new(limits: &Limits, spec: &Spec, operations: u32, root: Fd, signals: Fd, deadline: Option<Time>) -> Self {
        let mut io = Io::new(limits);
        let signal = io.adopt_signals(signals).expect("room for the signal descriptor");
        let spawn = Spawn {
            reach: None,
            program: spec.program.clone(),
            args: spec.arguments.clone(),
            env: spec.environment.clone(),
            root,
            dir: spec.directory.strip_prefix(b"/").expect("absolute command directory").into(),
            pipes: Box::new([
                Pipe { child: 1, way: Way::Out, parent: None },
                Pipe { child: 2, way: Way::Out, parent: None },
            ]),
        };
        let bound = usize::try_from(spec.output).expect("output bound");
        Self {
            io,
            limits: *limits,
            operations,
            signal: Some(signal),
            spawn: Some(spawn),
            child: None,
            pipes: [None, None],
            bytes: [Vec::with_capacity(bound), Vec::with_capacity(bound)],
            bound,
            exit: None,
            failure: None,
            deadline,
            complete: Queue::with_capacity(operations),
            subs: Queue::with_capacity(operations),
            events: Queue::uncapped(),
        }
    }

    fn answer(self) -> Result<Ran, Error> {
        if let Some(why) = self.failure {
            return Err(why);
        }
        let exit = match self.exit.expect("a completed command's exit") {
            Exit::Code(n) => prepare::Exit::Code(n),
            Exit::Signal(n) => prepare::Exit::Signal(u8::try_from(n).expect("Unix signal fits")),
        };
        let [stdout, stderr] = self.bytes;
        Ok(Ran { exit, stdout: Secret::new(stdout.into_boxed_slice()), stderr: Secret::new(stderr.into_boxed_slice()) })
    }
}

struct Running {
    io: Io,
    limits: Limits,
    operations: u32,
    signal: Option<Token>,
    spawn: Option<Spawn>,
    child: Option<Token>,
    pipes: [Option<Token>; 2],
    bytes: [Vec<u8>; 2],
    bound: usize,
    exit: Option<Exit>,
    failure: Option<Error>,
    deadline: Option<Time>,
    complete: Queue<Complete>,
    subs: Queue<Submit>,
    events: Queue<Event>,
}
impl Running {
    fn down(&mut self, env: &Env<Limits>, request: Request) {
        skein_io::down(&mut self.io, env, request, &mut self.subs);
    }
    fn demand(&mut self, env: &Env<Limits>, pipe: Token) {
        self.down(
            env,
            Request::Stream {
                stream: pipe,
                down: Down::Demand { read: Read::Arrived { max: self.limits.receive }, room: 0 },
            },
        );
    }
    fn events(&mut self, env: &Env<Limits>) {
        while let Some(event) = self.events.pop() {
            if let Event::Spawned { child, pipes, .. } = event {
                self.child = Some(child);
                for (i, pipe) in pipes.iter().enumerate() {
                    self.pipes[i] = Some(*pipe);
                    self.demand(env, *pipe);
                }
                if self.failure.is_some() {
                    self.down(env, Request::Close { entity: child });
                }
            } else if let Event::Stream { owner, up } = event {
                if let Some(i) = self.pipes.iter().position(|pipe| *pipe == Some(owner)) {
                    if let Up::Bytes(bytes) = up {
                        let keep = bytes.len().min(self.bound.saturating_sub(self.bytes[i].len()));
                        self.bytes[i].extend_from_slice(&bytes[..keep]);
                        self.demand(env, owner);
                    } else if matches!(up, Up::End | Up::Failed(_)) {
                        self.down(env, Request::Close { entity: owner });
                    }
                }
            } else if let Event::Exited { exit, .. } = event {
                self.exit = Some(exit);
            } else if let Event::Shutdown { .. } = event {
                self.failure = Some(Error::Interrupted);
                if let Some(child) = self.child {
                    self.down(env, Request::Close { entity: child });
                }
            } else if let Event::Failed { owner, error } = event {
                self.failure = Some(Error::Spawn(error));
                if owner != OWNER {
                    self.down(env, Request::Close { entity: owner });
                }
            } else if let Event::Closed { owner } = event {
                if owner == OWNER {
                    self.child = None;
                }
                if self.signal == Some(owner) {
                    self.signal = None;
                }
            }
        }
        if self.spawn.is_none()
            && self.child.is_none()
            && (self.exit.is_some() || self.failure.is_some())
            && let Some(signal) = self.signal.take()
        {
            self.down(env, Request::Close { entity: signal });
        }
    }
}
impl Host for Running {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits };
        while self.io.is_ready() {
            skein_io::resume(&mut self.io, &env, &mut self.events, &mut self.subs);
            self.events(&env);
        }
        while let Some(complete) = self.complete.pop() {
            skein_io::up(&mut self.io, &env, complete, &mut self.events, &mut self.subs);
            self.events(&env);
        }
        while self.io.is_due(now) {
            skein_io::fire(&mut self.io, &env, &mut self.events, &mut self.subs);
            self.events(&env);
        }
        if self.failure.is_none() && self.deadline.is_some_and(|at| now >= at) {
            self.deadline = None;
            self.failure = Some(Error::Kernel("source deadline expired".into()));
            if let Some(child) = self.child {
                self.down(&env, Request::Close { entity: child });
            }
        }
        if let Some(spawn) = self.spawn.take() {
            self.down(&env, Request::Spawn { owner: OWNER, spawn });
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.subs
    }
    fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || self.io.is_due(now)
            || !self.complete.is_empty()
            || self.failure.is_none() && self.deadline.is_some_and(|at| now >= at)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        if self.failure.is_none() { self.deadline } else { None }
    }
    fn is_empty(&self) -> bool {
        self.spawn.is_none() && self.io.is_empty()
    }
    fn operations(&self) -> u32 {
        self.operations
    }
}
