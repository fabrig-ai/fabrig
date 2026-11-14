//! One mmdebstrap run, its pipe readers, deadlines and scratch reclamation (hearth/docs/design/domain/render/spec.md, 2.2).
use crate::{Asked, Roots, Told, builder::vocabulary::Event, limits::Limits, render, translate, walk::Walker};
use crate::{Current, indices::Indices, resource::Resource};
use alloc::boxed::Box;
use hearth_protocol_mmdebstrap::output::{self, Reader, Seen};
use hearth_protocol_mmdebstrap::{tail, vector};
use render::errors::{Command, Ended, Limit, Tool};
use render::vocabulary::{Class, Event as RenderEvent, Failure, Request};
use skein_io::{
    self as io, file,
    kernel::{Exit, Pipe, Spawn, Way},
};
use skein_lib::{Env, Queue, Time, Token, Writer, bytes, stream::Up};
pub(super) const OWNER: Token = Token::new(21);
#[derive(Debug)]
struct PipeReader {
    held: Resource,
    reader: Reader,
    ended: bool,
}
impl PipeReader {
    fn new(limits: &hearth_protocol_mmdebstrap::limits::Limits) -> Self {
        Self { held: Resource::Awaited, reader: Reader::new(limits), ended: false }
    }
}
#[derive(Debug)]
pub(super) struct Resolve {
    request: Token,
    name: Box<[u8]>,
    invocation: vector::Invocation,
    lines: Box<[vector::Line]>,
    architecture: vector::Name,
    file_read: u32,
    stage: Stage,
    output: PipeReader,
    error: PipeReader,
    process: Resource,
    exit: Option<Exit>,
    pending: Queue<Asked>,
    deadline: Time,
    idle: Time,
    started: Time,
    fetched: u64,
    count: u32,
    scratch: Scratch,
}
impl Resolve {
    pub fn new(
        request: Request,
        roots: Roots,
        store: &[u8],
        file_read: u32,
        proxy: Option<&vector::Proxy>,
        env: &Env<Limits>,
        below: &mut Queue<Asked>,
    ) -> Self {
        let Request::Resolve { request, key, sources, names, variant, apt, deadline } = request else {
            unreachable!("resolve request")
        };
        let mut name = Writer::new(64);
        for byte in key.0 {
            crate::cache::hex_pair(&mut name, byte);
        }
        let name = name.finish();
        let mut scratch = Writer::new(store.len().saturating_add(73));
        for part in [store, b"/staging/", &name] {
            scratch.put(part).expect("scratch size");
        }
        let simulate = translate::resolution::simulate(&sources, &names, variant, &apt, &env.limits.mmdebstrap)
            .expect("validated render inputs");
        let lines = vector::lines(&simulate.sources).expect("validated sources");
        let invocation = vector::simulate(&simulate, &scratch.finish(), proxy, &env.limits.mmdebstrap)
            .expect("bounded scratch and request");
        let root = roots.staging.token;
        let path = name.clone();
        below.push(Asked::File(file::Request::MakeDirectory { owner: OWNER, root, path, mode: 0o700 }));
        Self {
            request,
            name,
            invocation,
            lines,
            architecture: simulate.sources.architecture,
            file_read,
            stage: Stage::Running,
            output: PipeReader::new(&env.limits.mmdebstrap),
            error: PipeReader::new(&env.limits.mmdebstrap),
            process: Resource::Awaited,
            exit: None,
            pending: Queue::uncapped(),
            deadline,
            idle: env.now.saturating_add(env.limits.mmdebstrap.idle),
            started: env.now,
            fetched: 0,
            count: 0,
            scratch: Scratch::Absent,
        }
    }
    pub fn next_deadline(&self) -> Option<Time> {
        if self.stage.ending() { None } else { Some(self.deadline.min(self.idle)) }
    }
    pub fn fire(&mut self, env: &Env<Limits>) {
        if match self.next_deadline() {
            Some(at) => at <= env.now,
            None => false,
        } {
            self.fail(Ended::Timeout, None);
        }
    }
    fn fail(&mut self, ended: Ended, cause: Option<render::resolution::Cause>) {
        if !self.stage.ending() {
            self.end(self.failure(ended, cause));
        }
    }
    fn failure(&self, ended: Ended, cause: Option<render::resolution::Cause>) -> RenderEvent {
        let failure = tail::failure(&self.error.reader, &self.output.reader);
        RenderEvent::Failed {
            request: self.request,
            class: match &cause {
                Some(cause) => cause.class(),
                None => Class::Operational,
            },
            at: None,
            failure: Failure::Run { tool: Tool { command: Command::Mmdebstrap, ended, tail: failure.tail }, cause },
        }
    }
    fn end(&mut self, terminal: RenderEvent) {
        self.stage = match core::mem::replace(&mut self.stage, Stage::Spent) {
            Stage::Running => Stage::Ending(terminal),
            Stage::Indices(indices) => Stage::Stopping { indices, terminal },
            stage @ (Stage::Stopping { .. } | Stage::Ending(_) | Stage::Spent) => stage,
        };
    }
    pub fn up(
        &mut self,
        event: Told,
        roots: Roots,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Asked>,
    ) {
        match event {
            Told::File(event) => match &mut self.stage {
                Stage::Indices(indices) | Stage::Stopping { indices, .. } => {
                    self.idle = env.now.saturating_add(env.limits.mmdebstrap.idle);
                    indices.up(event, env);
                }
                Stage::Running | Stage::Ending(_) | Stage::Spent => self.file_up(event, roots, env, below),
            },
            Told::Io(event) => match event {
                io::Event::Spawned { child, pipes, .. } => {
                    self.process = Resource::Open(child);
                    let mut pipes = pipes.into_iter();
                    let env = Env { now: env.now, wall: env.wall, limits: env.limits.mmdebstrap };
                    for pipe in [&mut self.output, &mut self.error] {
                        let stream = pipes.next().expect("resolve pipe");
                        pipe.held = Resource::Open(stream);
                        let mut requests = Queue::uncapped();
                        output::open(&mut pipe.reader, &env, &mut requests);
                        if let Some(down) = requests.pop() {
                            self.pending.push(Asked::Io(io::Request::Stream { stream, down }));
                        }
                    }
                }
                io::Event::Stream { owner, up } => self.stream(owner, up, env, above),
                io::Event::Exited { exit, .. } => self.exit = Some(exit),
                io::Event::Failed { .. } => self.fail(Ended::Start, None),
                io::Event::Closed { owner } => {
                    if owner == OWNER {
                        self.process = Resource::Closed;
                    }
                    for pipe in [&mut self.output, &mut self.error] {
                        if pipe.held.token() == Some(owner) {
                            pipe.held = Resource::Closed;
                        }
                    }
                }
                io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Output { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. } => unreachable!("resolve child terminal"),
            },
        }
    }
    fn file_up(&mut self, event: file::Event, roots: Roots, env: &Env<Limits>, below: &mut Queue<Asked>) {
        match &mut self.scratch {
            Scratch::Walking(walker) => {
                if walker.up(event, env.limits.walk, below).is_some() {
                    self.scratch = Scratch::Removing;
                    let root = roots.staging.fd;
                    let name = self.name.clone();
                    below.push(Asked::File(file::Request::Remove { owner: OWNER, root, name, directory: true }));
                }
                return;
            }
            Scratch::Absent | Scratch::Made | Scratch::Removing | Scratch::Cleaned => {}
        }
        match event {
            file::Event::Made { .. } => {
                self.scratch = Scratch::Made;
                let mut name = Writer::new(77);
                name.put(&self.name).expect("key");
                name.put(b"/sources.list").expect("suffix");
                below.push(Asked::File(file::Request::Store {
                    owner: OWNER,
                    root: roots.staging.token,
                    path: name.finish(),
                    no_follow: true,
                    bytes: self.invocation.sources_list.clone(),
                    expected: file::Expect::Absent,
                }));
            }
            file::Event::Stored { .. } => below.push(Asked::Io(io::Request::Spawn {
                owner: OWNER,
                spawn: Spawn {
                    program: bytes::copy_of(vector::PROGRAM),
                    args: self.invocation.args.clone(),
                    env: self.invocation.env.clone(),
                    root: roots.store.fd,
                    dir: Box::new([]),
                    pipes: Box::new([
                        Pipe { child: 1, way: Way::Out, parent: None },
                        Pipe { child: 2, way: Way::Out, parent: None },
                    ]),
                },
            })),
            file::Event::Removed { .. } => self.scratch = Scratch::Cleaned,
            file::Event::Refused { .. }
            | file::Event::Conflict { .. }
            | file::Event::Failed { .. }
            | file::Event::Cancelled { .. } => {
                if self.stage.ending() {
                    self.scratch = Scratch::Cleaned;
                } else {
                    self.fail(Ended::Start, None);
                }
            }
            file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Opened { .. }
            | file::Event::Loaded { .. }
            | file::Event::Scanned { .. }
            | file::Event::TooLarge { .. }
            | file::Event::Stated { .. }
            | file::Event::Written { .. }
            | file::Event::Read { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Closed { .. }
            | file::Event::Linked { .. }
            | file::Event::Renamed { .. }
            | file::Event::Listed { .. } => unreachable!("resolve scratch terminal"),
        }
    }
    fn stream(&mut self, owner: Token, up: Up, env: &Env<Limits>, above: &mut Queue<Event>) {
        if self.stage.ending() {
            return;
        }
        if let Up::Bytes(_) = &up {
            self.idle = env.now.saturating_add(env.limits.mmdebstrap.idle);
        }
        let pipe = if self.output.held.token() == Some(owner) { &mut self.output } else { &mut self.error };
        assert_eq!(pipe.held.token(), Some(owner), "current resolve pipe");
        let mut seen = Queue::uncapped();
        let mut requests = Queue::uncapped();
        let reader_env = Env { now: env.now, wall: env.wall, limits: env.limits.mmdebstrap };
        output::up(&mut pipe.reader, &reader_env, up, &mut seen, &mut requests);
        if let Some(down) = requests.pop() {
            self.pending.push(Asked::Io(io::Request::Stream { stream: owner, down }));
        }
        if let Some(event) = seen.pop() {
            match event {
                Seen::Inst(inst) => {
                    self.count = self.count.saturating_add(1);
                    if self.count > env.limits.mmdebstrap.insts {
                        self.too_many(env);
                    } else {
                        above.push(Event::Render(RenderEvent::Inst {
                            request: self.request,
                            inst: translate::resolution::inst(inst),
                        }));
                    }
                }
                Seen::Fetched(fetched) => self.fetched = self.fetched.saturating_add(fetched),
                Seen::Ended => pipe.ended = true,
                Seen::Failed(output::OutputError::TooMany) => self.too_many(env),
                Seen::Failed(output::OutputError::Inst | output::OutputError::Stream(_)) => {
                    self.fail(Ended::Output, None);
                }
            }
        }
    }
    fn too_many(&mut self, env: &Env<Limits>) {
        self.end(RenderEvent::Failed {
            request: self.request,
            class: Class::Configuration,
            at: None,
            failure: Failure::TooLarge { limit: Limit::Closure, max: u64::from(env.limits.mmdebstrap.insts) },
        });
    }
    pub fn index(&self) -> Option<(Token, Box<[u8]>)> {
        match &self.stage {
            Stage::Indices(indices) | Stage::Stopping { indices, .. } => indices.arrival(),
            Stage::Running | Stage::Ending(_) | Stage::Spent => None,
        }
    }
    pub fn index_settled(&mut self) {
        match &mut self.stage {
            Stage::Indices(indices) | Stage::Stopping { indices, .. } => indices.settled(),
            Stage::Running | Stage::Ending(_) | Stage::Spent => {}
        }
    }
    pub fn ready(&self) -> bool {
        let ready = match &self.stage {
            Stage::Indices(indices) | Stage::Stopping { indices, .. } => return indices.ready(),
            Stage::Running => self.exit.is_some() && self.output.ended && self.error.ended,
            Stage::Ending(_) => {
                self.process.is_open()
                    || self.output.held.is_open()
                    || self.error.held.is_open()
                    || (self.process.token().is_none()
                        && self.output.held.token().is_none()
                        && self.error.held.token().is_none()
                        && match self.scratch {
                            Scratch::Walking(_) | Scratch::Removing => false,
                            Scratch::Absent | Scratch::Made | Scratch::Cleaned => true,
                        })
            }
            Stage::Spent => false,
        };
        !self.pending.is_empty() || ready
    }
    pub fn resume(
        &mut self,
        roots: Roots,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Asked>,
    ) -> bool {
        if let Some(request) = self.pending.pop() {
            below.push(request);
            return false;
        }
        self.stage = match core::mem::replace(&mut self.stage, Stage::Spent) {
            Stage::Running => match self.exit {
                Some(exit) if self.output.ended && self.error.ended => match exit {
                    Exit::Code(0) => Stage::Indices(Indices::new(
                        self.request,
                        roots.staging.fd,
                        &self.name,
                        self.lines.clone(),
                        &self.architecture,
                        self.file_read,
                        env,
                    )),
                    Exit::Code(code) => {
                        let failure = tail::failure(&self.error.reader, &self.output.reader);
                        let cause = translate::resolution::cause(failure.cause, &self.lines, &failure.tail);
                        Stage::Ending(self.failure(Ended::Exit(code), Some(cause)))
                    }
                    Exit::Signal(signal) => Stage::Ending(self.failure(Ended::Signal(signal), None)),
                },
                Some(Exit::Code(_) | Exit::Signal(_)) | None => Stage::Running,
            },
            Stage::Indices(mut indices) => match indices.resume(env, above, below) {
                Some(true) => Stage::Ending(RenderEvent::Resolved {
                    request: self.request,
                    lines: indices.lines(),
                    tools: render::resolution::Tools { apt: Box::new([]), mmdebstrap: Box::new([]) },
                    costs: render::resolution::Costs {
                        fetched: self.fetched,
                        took: env.now.saturating_since(self.started),
                    },
                }),
                Some(false) => Stage::Ending(self.failure(Ended::Output, None)),
                None => Stage::Indices(indices),
            },
            Stage::Stopping { mut indices, terminal } => {
                indices.stop();
                match indices.resume(env, above, below) {
                    Some(true) => unreachable!("stopped indices fail"),
                    Some(false) => Stage::Ending(terminal),
                    None => Stage::Stopping { indices, terminal },
                }
            }
            stage @ (Stage::Ending(_) | Stage::Spent) => stage,
        };
        match self.stage {
            Stage::Ending(_) => {}
            Stage::Running | Stage::Indices(_) | Stage::Stopping { .. } | Stage::Spent => return false,
        }
        for pipe in [&mut self.output, &mut self.error] {
            if let Some(entity) = pipe.held.request_close() {
                below.push(Asked::Io(io::Request::Close { entity }));
                return false;
            }
        }
        if let Some(entity) = self.process.request_close() {
            below.push(Asked::Io(io::Request::Close { entity }));
        }
        if self.process.token().is_some() || self.output.held.token().is_some() || self.error.held.token().is_some() {
            return false;
        }
        match self.scratch {
            Scratch::Made => {
                self.scratch =
                    Scratch::Walking(Walker::at(env.limits.walk, roots.staging.fd, &self.name, OWNER, below));
                return false;
            }
            Scratch::Walking(_) | Scratch::Removing => return false,
            Scratch::Absent | Scratch::Cleaned => {}
        }
        match core::mem::replace(&mut self.stage, Stage::Spent) {
            Stage::Ending(terminal) => above.push(Event::Render(terminal)),
            Stage::Running | Stage::Indices(_) | Stage::Stopping { .. } | Stage::Spent => unreachable!("ending stage"),
        }
        true
    }
}

#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "indices and their terminal stay inline without another allocation")]
enum Stage {
    Running,
    Indices(Indices),
    Stopping { indices: Indices, terminal: RenderEvent },
    Ending(RenderEvent),
    Spent,
}
impl Stage {
    fn ending(&self) -> bool {
        match self {
            Self::Running | Self::Indices(_) => false,
            Self::Stopping { .. } | Self::Ending(_) | Self::Spent => true,
        }
    }
}
#[derive(Debug)]
enum Scratch {
    Absent,
    Made,
    Walking(Walker),
    Removing,
    Cleaned,
}
impl crate::Protocol {
    pub(super) fn resolve(&mut self, request: Request, env: &Env<Limits>, below: &mut Queue<Asked>) {
        let resolve =
            Resolve::new(request, self.roots, &self.store_path, self.file_read, self.proxy.as_ref(), env, below);
        self.current = Current::Resolving(resolve);
    }
}
