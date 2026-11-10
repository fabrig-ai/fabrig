//! Job file and process operations (temper/docs/design/protocol/tooling.md, 2, 3 and 6).
mod files;
use super::Worker;
use alloc::boxed::Box;
use root::tooling::{Event, Exit, Request};
use skein_io as io;
use skein_lib::{List, Map, Queue, Time, Token, Writer, stream};
use temper_worker_domain as root;
use temper_worker_protocol_git::ring::ByteRing;

#[derive(Debug)]
pub(super) struct Jobs {
    processes: Map<Token, Process>,
    files: Map<Token, files::File>,
}
#[derive(Debug)]
struct Process {
    owner: Token,
    deadline: Option<Time>,
    child: Option<Token>,
    pipes: [Option<Token>; 2],
    ended: [bool; 2],
    exit: Option<Exit>,
    told: bool,
    killed: bool,
    aborted: bool,
    tail: ByteRing,
    setup: Box<[u8]>,
}
impl Jobs {
    pub(super) fn new(slots: u32) -> Self {
        Self { processes: Map::with_capacity(slots), files: Map::with_capacity(slots) }
    }
    pub(super) fn accepts(&self, owner: Token) -> bool {
        for (token, process) in &self.processes {
            if *token == owner || process.child == Some(owner) || process.pipes.contains(&Some(owner)) {
                return true;
            }
        }
        false
    }
    pub(super) fn empty(&self) -> bool {
        self.processes.is_empty() && self.files.is_empty()
    }
    pub(super) fn deadline(&self) -> Option<Time> {
        let mut next: Option<Time> = None;
        for (_, process) in &self.processes {
            if let Some(at) = process.deadline {
                next = Some(match next {
                    Some(next) => next.min(at),
                    None => at,
                });
            }
        }
        next
    }
}
impl Worker {
    fn job_token(&mut self) -> Token {
        let token = Token::new(self.prefix.checked_add(self.next).expect("worker owner range"));
        self.next = self.next.checked_add(1).expect("worker owner sequence");
        assert!(self.next < (1_u64 << 48_u32), "worker owner range");
        token
    }
    pub(crate) fn job_request(&mut self, owner: Token, op: Request) {
        match op {
            Request::Write { path, bytes, .. } => {
                let bounds = &self.env.limits.tooling;
                if self.jobs.files.len() == self.jobs.files.capacity()
                    || bytes.len() > usize::try_from(bounds.file_bytes).expect("file bound")
                    || !relative(&path, bounds.path_bytes)
                {
                    above(&mut self.events, owner, Event::Written { owner, written: false });
                    return;
                }
                let (file, op) = files::File::write(owner, self.directory.expect("worker root"), path, bytes);
                self.job_file(file, op);
            }
            Request::Read { path, most, .. } => {
                let bounds = &self.env.limits.tooling;
                if self.jobs.files.len() == self.jobs.files.capacity() || !relative(&path, bounds.path_bytes) {
                    above(&mut self.events, owner, Event::Read { owner, bytes: None });
                    return;
                }
                let (file, op) =
                    files::File::read(owner, self.directory.expect("worker root"), path, most.min(bounds.file_bytes));
                self.job_file(file, op);
            }
            Request::Spawn { program, args, env, dir, deadline, .. } => {
                let bound = self.env.limits.tooling.path_bytes;
                if !relative(&program, bound) || !relative(&dir, bound) || args.first() != Some(&program) {
                    above(
                        &mut self.events,
                        owner,
                        Event::Exited {
                            owner,
                            exit: Exit::Signal,
                            tail: b"spawn: invalid command, directory or argv".as_slice().into(),
                        },
                    );
                    above(&mut self.events, owner, Event::Gone { owner });
                    return;
                }
                let mut arguments = List::with_capacity(u32::try_from(args.len()).expect("bounded job arguments"));
                for (index, argument) in args.into_iter().enumerate() {
                    if index == 0 {
                        continue;
                    }
                    // The capability's request path is relative to the worker root.
                    arguments
                        .push(if index == 2 { joined(&[&self.path, b"/", &argument]) } else { argument })
                        .expect("job arguments");
                }
                let mut environment = List::with_capacity(u32::try_from(env.len()).expect("bounded job environment"));
                for (name, value) in env {
                    let value = if name.as_ref() == b"HOME" || name.as_ref() == b"TEMPER_JOB" {
                        joined(&[&self.path, b"/", &value])
                    } else {
                        value
                    };
                    environment.push(joined(&[&name, b"=", &value])).expect("job environment");
                }
                let token = self.job_token();
                self.jobs
                    .processes
                    .insert(
                        token,
                        Process::new(owner, deadline, self.env.limits.tooling.tail_bytes, &self.path, &program, &dir),
                    )
                    .expect("root bounds jobs");
                self.io.push(io::Request::Spawn {
                    owner: token,
                    spawn: io::kernel::Spawn {
                        program: joined(&[&self.path, b"/", &program]),
                        args: arguments.into_boxed(),
                        env: environment.into_boxed(),
                        root: self.directory.expect("worker root"),
                        dir,
                        pipes: Box::new([
                            io::kernel::Pipe { child: 1, way: io::kernel::Way::Out, parent: None },
                            io::kernel::Pipe { child: 2, way: io::kernel::Way::Out, parent: None },
                        ]),
                    },
                });
            }
            Request::Kill { .. } => {
                let mut found = None;
                for (token, process) in &self.jobs.processes {
                    if process.owner == owner {
                        found = Some(*token);
                        break;
                    }
                }
                if let Some(token) = found {
                    self.jobs.processes.get_mut(&token).expect("owned job").kill(false, &mut self.io);
                }
            }
            Request::Started { .. } | Request::StartRefused { .. } | Request::Ended { .. } | Request::Gone { .. } => {}
        }
    }
    fn job_file(&mut self, file: files::File, kind: io::kernel::Op) {
        let op = self.job_token();
        self.jobs.files.insert(op, file).expect("root bounds job files");
        self.submitted.push(io::kernel::Submit { op, kind });
    }
    pub(super) fn job_complete(&mut self, complete: io::kernel::Complete) -> Result<(), io::kernel::Complete> {
        let Some(mut file) = self.jobs.files.remove(&complete.op) else {
            return Err(complete);
        };
        match file.complete(complete, self.aborted.is_some()) {
            files::Next::Run(op) => self.job_file(file, op),
            files::Next::End(event) => above(&mut self.events, file.owner(), event),
        }
        Ok(())
    }
    pub(super) fn job_up(&mut self, event: io::Event) {
        let owner = event.owner().expect("job io owner");
        let mut found = None;
        for (token, process) in &self.jobs.processes {
            if *token == owner || process.child == Some(owner) || process.pipes.contains(&Some(owner)) {
                found = Some(*token);
                break;
            }
        }
        let token = found.expect("owned job event");
        let process = self.jobs.processes.get_mut(&token).expect("job process");
        if process.up(token, event, &mut self.io, &mut self.events) {
            drop(self.jobs.processes.remove(&token));
        }
    }
    pub(super) fn jobs_fire(&mut self, now: Time, abort: bool) {
        let mut due = List::with_capacity(self.jobs.processes.len());
        for (token, process) in &self.jobs.processes {
            if abort
                || match process.deadline {
                    Some(at) => at <= now,
                    None => false,
                }
            {
                due.push(*token).expect("job count");
            }
        }
        for token in &due {
            let process = self.jobs.processes.get_mut(token).expect("owned job");
            if abort {
                process.abort(&mut self.io);
            } else {
                process.kill(true, &mut self.io);
            }
        }
    }
}
fn above(events: &mut Queue<root::Event>, owner: Token, event: Event) {
    events.push(root::Event::Job { owner, event });
}
fn relative(path: &[u8], bound: u32) -> bool {
    if path.is_empty()
        || path.starts_with(b"/")
        || path.contains(&0)
        || path.len() > usize::try_from(bound).expect("path bound")
    {
        return false;
    }
    let mut start = 0;
    for (index, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            if index == start || path.get(start..index) == Some(b"..".as_slice()) {
                return false;
            }
            start = index.checked_add(1).expect("bounded path");
        }
    }
    if path.get(start..) == Some(b"..".as_slice()) || start == path.len() {
        return false;
    }
    true
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("bounded job path");
    }
    let mut writer = Writer::new(size);
    for part in parts {
        writer.put(part).expect("measured bytes");
    }
    writer.finish()
}
impl Process {
    fn new(owner: Token, deadline: Time, tail_bytes: u32, root: &[u8], program: &[u8], dir: &[u8]) -> Self {
        Self {
            owner,
            deadline: Some(deadline),
            child: None,
            pipes: [None; 2],
            ended: [false; 2],
            exit: None,
            told: false,
            killed: false,
            aborted: false,
            tail: ByteRing::new(tail_bytes),
            setup: joined(&[b"spawn command ", root, b"/", program, b" in ", root, b"/", dir]),
        }
    }
    fn abort(&mut self, io: &mut Queue<io::Request>) {
        if self.aborted {
            return;
        }
        self.aborted = true;
        self.kill(false, io);
        self.close_aborted(io);
    }
    fn close_aborted(&self, io: &mut Queue<io::Request>) {
        if let Some(child) = self.child {
            io.push(io::Request::Abort { entity: child });
        }
        for pipe in self.pipes.iter().flatten() {
            io.push(io::Request::Abort { entity: *pipe });
        }
    }
    fn kill(&mut self, deadline: bool, io: &mut Queue<io::Request>) {
        if self.killed {
            return;
        }
        self.killed = true;
        self.deadline = None;
        if deadline {
            self.exit = Some(Exit::Deadline);
        }
        if let Some(child) = self.child {
            io.push(io::Request::Signal { child, signal: io::kernel::Signal::Kill, to: io::kernel::Target::Group });
        }
    }
    fn demand(pipe: Token, io: &mut Queue<io::Request>) {
        io.push(io::Request::Stream {
            stream: pipe,
            down: stream::Down::Demand { read: stream::Read::Arrived { max: 4096 }, room: 0 },
        });
    }
    fn up(
        &mut self,
        token: Token,
        event: io::Event,
        io: &mut Queue<io::Request>,
        events: &mut Queue<root::Event>,
    ) -> bool {
        match event {
            io::Event::Spawned { child, pipes, .. } => {
                assert_eq!(pipes.len(), 2, "job output pipes");
                self.child = Some(child);
                self.pipes = [pipes.first().copied(), pipes.get(1).copied()];
                above(events, self.owner, Event::Spawned { owner: self.owner });
                if self.killed {
                    io.push(io::Request::Signal {
                        child,
                        signal: io::kernel::Signal::Kill,
                        to: io::kernel::Target::Group,
                    });
                }
                if self.aborted {
                    self.close_aborted(io);
                } else {
                    for pipe in &pipes {
                        Self::demand(*pipe, io);
                    }
                }
            }
            io::Event::Exited { exit, .. } => {
                if self.exit.is_none() {
                    self.exit = Some(match exit {
                        io::kernel::Exit::Code(code) => Exit::Code(code),
                        io::kernel::Exit::Signal(_) => Exit::Signal,
                    });
                }
            }
            io::Event::Stream { owner, up } => {
                let index = usize::from(self.pipes.first() != Some(&Some(owner)));
                match up {
                    stream::Up::Bytes(bytes) => {
                        if index == 1 {
                            self.tail.write(&bytes);
                        }
                        if !self.aborted {
                            Self::demand(owner, io);
                        }
                    }
                    stream::Up::End | stream::Up::Failed(_) => {
                        *self.ended.get_mut(index).expect("output pipe") = true;
                        io.push(io::Request::Close { entity: owner });
                    }
                    stream::Up::Room => {}
                }
            }
            io::Event::Closed { owner } if owner == token || self.child == Some(owner) => {
                self.tell(events);
                above(events, self.owner, Event::Gone { owner: self.owner });
                return true;
            }
            io::Event::Failed { error, .. } => {
                self.tail.write(&self.setup);
                self.tail.write(b" failed: ");
                self.tail.write(match error {
                    io::Error::Refused => b"Refused",
                    io::Error::Unreachable => b"Unreachable",
                    io::Error::TimedOut => b"TimedOut",
                    io::Error::Reset => b"Reset",
                    io::Error::Busy => b"Busy",
                    io::Error::Other => b"Other",
                });
                if self.exit.is_none() {
                    self.exit = Some(Exit::Signal);
                }
                self.deadline = None;
            }
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Output { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. }
            | io::Event::Resized
            | io::Event::Shutdown { .. }
            | io::Event::Closed { .. } => {}
        }
        if self.exit.is_some() && self.ended == [true; 2] && !self.told {
            self.tell(events);
            if let Some(child) = self.child {
                io.push(io::Request::Close { entity: child });
            }
        }
        false
    }
    fn tell(&mut self, events: &mut Queue<root::Event>) {
        if self.told {
            return;
        }
        self.told = true;
        self.deadline = None;
        above(
            events,
            self.owner,
            Event::Exited {
                owner: self.owner,
                exit: self.exit.take().unwrap_or(Exit::Signal),
                tail: core::mem::replace(&mut self.tail, ByteRing::new(0)).tail(),
            },
        );
    }
}
