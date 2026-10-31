//! Process-group execution until skein supplies contained trees
//! (protocol/agent.md, sections 2 and 3; domain/tools.md, section 5).
//! Commands may yield; their ends are answered after both pipes and the child close.

use alloc::boxed::Box;

use skein_io::{self as io, file, kernel};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, List, Map, Queue, Time, Token, Writer};
use smith_domain::{run, tools};

use crate::boundary::{Below, Spawn, ToDomain};
use crate::limits::Limits;
use crate::listing::Listing;
use crate::search::Search;

mod directory;
mod jobs;

use directory::Directory;

pub(crate) use jobs::Advance;
use jobs::Response;

pub(crate) const OUTPUT_READ: u32 = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Purpose {
    Shell,
    Search,
    List,
    Check,
}

/// The outstanding process-group stop stage (protocol/agent.md, section 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stop {
    /// Running, or naturally exited while IO settles it.
    Running,
    /// Terminate sent; Kill at the next deadline.
    Terminating { next: Time, why: Why },
    /// Kill sent; IO Close at the next deadline.
    Killing { next: Time, why: Why },
    /// IO kills what remains, reaps the leader and closes the child.
    Closing { why: Why },
}

/// The first request that started this process's stop ladder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Why {
    /// The domain's absolute deadline passed.
    Deadline,
    /// The domain cancelled or aborted the operation.
    Cancel,
    /// A job act asked to stop its group.
    Stop,
}

/// Bounded first and last bytes of a process's combined output.
#[derive(Debug)]
pub(crate) struct Capture {
    head: List<u8>,
    tail: List<u8>,
    next: u32,
    total: u64,
}

impl Capture {
    pub(crate) fn new(head: u32, tail: u32) -> Capture {
        Capture { head: List::with_capacity(head), tail: List::with_capacity(tail), next: 0, total: 0 }
    }

    pub(crate) fn push(&mut self, byte: u8) {
        self.total = self.total.saturating_add(1);
        if self.head.room() > 0 {
            self.head.push(byte).expect("head has room");
        } else if self.tail.capacity() > 0 {
            if self.tail.room() > 0 {
                self.tail.push(byte).expect("tail has room");
            } else {
                *self.tail.get_mut(self.next).expect("ring slot exists") = byte;
                self.next = self
                    .next
                    .checked_add(1)
                    .expect("ring index fits")
                    .checked_rem(self.tail.capacity())
                    .expect("tail capacity positive");
            }
        }
    }

    pub(crate) fn finish(mut self) -> (Box<[u8]>, Box<[u8]>, u64) {
        self.take(self.head.capacity(), self.tail.capacity())
    }

    pub(crate) fn take(&mut self, head: u32, tail: u32) -> (Box<[u8]>, Box<[u8]>, u64) {
        let kept = self.head.len().checked_add(self.tail.len()).expect("kept output fits");
        let gap = self.total > u64::from(kept);
        let first = head.min(if gap { self.head.len() } else { kept });
        let last = tail.min(if gap { self.tail.len() } else { kept.checked_sub(first).expect("head within kept") });
        let mut leading = Writer::new(usize::try_from(first).expect("head fits"));
        let mut trailing = Writer::new(usize::try_from(last).expect("tail fits"));
        for at in 0..first {
            leading.put(&[self.kept(at)]).expect("head room");
        }
        for at in kept.checked_sub(last).expect("tail within kept")..kept {
            trailing.put(&[self.kept(at)]).expect("tail room");
        }
        let dropped = self
            .total
            .checked_sub(u64::from(first))
            .expect("head within total")
            .checked_sub(u64::from(last))
            .expect("tail within total");
        self.head.clear();
        self.tail.clear();
        self.next = 0;
        self.total = 0;
        (leading.finish(), trailing.finish(), dropped)
    }

    fn kept(&self, at: u32) -> u8 {
        if at < self.head.len() {
            return *self.head.get(at).expect("head byte");
        }
        let at = at.checked_sub(self.head.len()).expect("tail offset");
        let index = if self.tail.len() == self.tail.capacity() {
            self.next
                .checked_add(at)
                .expect("ring index fits")
                .checked_rem(self.tail.len())
                .expect("tail length positive")
        } else {
            at
        };
        *self.tail.get(index).expect("tail byte")
    }
}

/// The admitted operation owns exactly the parser and directory it needs.
#[derive(Debug)]
pub(crate) enum Work {
    Shell,
    Check,
    Search(Search),
    List { listing: Listing, directory: Directory },
}

/// One child and its output pipes, retained until io proves them closed.
#[derive(Debug)]
pub(crate) struct Process {
    work: Work,
    pub(crate) deadline: Time,
    pub(crate) child: Option<Token>,
    pub(crate) pipes: [Option<Token>; 2],
    pipe_closed: [bool; 2],
    pub(crate) child_closed: bool,
    exit: Option<kernel::Exit>,
    failure: Option<io::Error>,
    stop: Stop,
    output: Capture,
    response: Response,
}

impl Process {
    pub(crate) fn new(work: Work, deadline: Time, head: u32, tail: u32) -> Process {
        Process {
            work,
            deadline,
            child: None,
            pipes: [None, None],
            pipe_closed: [false, false],
            child_closed: false,
            exit: None,
            failure: None,
            stop: Stop::Running,
            output: Capture::new(head, tail),
            response: Response::Final,
        }
    }

    pub(crate) fn purpose(&self) -> Purpose {
        match &self.work {
            Work::Shell => Purpose::Shell,
            Work::Check => Purpose::Check,
            Work::Search(_) => Purpose::Search,
            Work::List { .. } => Purpose::List,
        }
    }

    pub(crate) fn directory_event(&mut self, event: file::Event, below: &mut Queue<Below>) {
        let stopping = match self.stop {
            Stop::Running => false,
            Stop::Terminating { .. } | Stop::Killing { .. } | Stop::Closing { .. } => true,
        };
        match &mut self.work {
            Work::List { directory, .. } => {
                directory.event(event, stopping, below);
                if directory.unspawned() {
                    self.child_closed = true;
                }
            }
            Work::Shell | Work::Check | Work::Search(_) => unreachable!("only a listing opens a directory"),
        }
    }

    #[expect(clippy::boxed_local, reason = "io transfers the owned pipe list")]
    pub(crate) fn spawned(
        &mut self,
        child: Token,
        pipes: Box<[Token]>,
        routes: &mut Map<Token, Token>,
        owner: Token,
        below: &mut Queue<Below>,
    ) {
        assert!(pipes.len() == 2, "stdout and stderr pipes requested");
        self.child = Some(child);
        for (index, pipe) in pipes.iter().enumerate() {
            *self.pipes.get_mut(index).expect("two pipe slots") = Some(*pipe);
            assert_eq!(routes.insert(*pipe, owner), Ok(None), "reserved route for a new io pipe");
            below.push(Below::Process(io::Request::Stream {
                stream: *pipe,
                down: Down::Demand { read: Read::Arrived { max: OUTPUT_READ }, room: 0 },
            }));
        }
        match self.stop {
            Stop::Running => {}
            Stop::Terminating { .. } => {
                below.push(Below::Process(io::Request::Signal {
                    child,
                    to: kernel::Target::Group,
                    signal: kernel::Signal::Terminate,
                }));
            }
            Stop::Killing { .. } => {
                below.push(Below::Process(io::Request::Signal {
                    child,
                    to: kernel::Target::Group,
                    signal: kernel::Signal::Kill,
                }));
            }
            Stop::Closing { .. } => {
                below.push(Below::Process(io::Request::Close { entity: child }));
            }
        }
    }

    pub(crate) fn byte(&mut self, pipe: Token, byte: u8) {
        let index = usize::from(self.pipes.first().copied().expect("pipe slot") != Some(pipe));
        assert_eq!(self.pipes.get(index).copied().expect("pipe slot"), Some(pipe), "byte belongs to child");
        match &mut self.work {
            Work::Search(search) if index == 0 => search.push(byte),
            Work::List { listing, .. } if index == 0 => listing.push(byte),
            Work::Shell | Work::Check | Work::Search(_) | Work::List { .. } => self.output.push(byte),
        }
    }

    pub(crate) fn pipe_closed(&mut self, pipe: Token) {
        let index = usize::from(self.pipes.first().copied().expect("pipe slot") != Some(pipe));
        assert_eq!(*self.pipes.get(index).expect("pipe slot"), Some(pipe), "closed pipe belongs to child");
        *self.pipe_closed.get_mut(index).expect("pipe slot") = true;
    }

    pub(crate) fn failed(&mut self, error: io::Error) {
        if self.failure.is_none() {
            self.failure = Some(error);
        }
    }

    pub(crate) fn exited(&mut self, exit: kernel::Exit, below: &mut Queue<Below>) {
        self.exit = Some(exit);
        match self.stop {
            Stop::Running => {
                // A natural leader exit can leave descendants holding its pipes.
                // Close kills its group before waiting for those pipe terminals.
                if let Some(child) = self.child {
                    below.push(Below::Process(io::Request::Close { entity: child }));
                }
            }
            Stop::Terminating { .. } | Stop::Killing { .. } | Stop::Closing { .. } => {}
        }
    }

    pub(crate) fn ready(&self) -> bool {
        self.child_closed
            && (self.child.is_none()
                || (*self.pipe_closed.first().expect("pipe slot") && *self.pipe_closed.get(1).expect("pipe slot")))
    }

    pub(crate) fn stop(&mut self, now: Time, grace: Duration, why: Why, below: &mut Queue<Below>) {
        match self.stop {
            Stop::Running => {
                if self.exit.is_some() || self.child_closed {
                    return;
                }
                match &self.work {
                    Work::List { directory, .. } => directory.cancel(below),
                    Work::Shell | Work::Check | Work::Search(_) => {}
                }
                self.stop = Stop::Terminating { next: now.saturating_add(grace), why };
                if let Some(child) = self.child {
                    below.push(Below::Process(io::Request::Signal {
                        child,
                        to: kernel::Target::Group,
                        signal: kernel::Signal::Terminate,
                    }));
                }
            }
            Stop::Terminating { .. } | Stop::Killing { .. } | Stop::Closing { .. } => {}
        }
    }

    pub(crate) fn fire(&mut self, now: Time, grace: Duration, below: &mut Queue<Below>) {
        if self.child_closed {
            return;
        }
        match self.stop {
            Stop::Running => {
                if now >= self.deadline {
                    self.stop(now, grace, Why::Deadline, below);
                }
            }
            Stop::Terminating { next, why } => {
                if now >= next {
                    self.stop = Stop::Killing { next: now.saturating_add(grace), why };
                    if let Some(child) = self.child {
                        below.push(Below::Process(io::Request::Signal {
                            child,
                            to: kernel::Target::Group,
                            signal: kernel::Signal::Kill,
                        }));
                    }
                }
            }
            Stop::Killing { next, why } => {
                if now >= next {
                    self.stop = Stop::Closing { why };
                    if let Some(child) = self.child {
                        below.push(Below::Process(io::Request::Close { entity: child }));
                    }
                }
            }
            Stop::Closing { .. } => {}
        }
    }

    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let process = if self.child_closed {
            None
        } else {
            match self.stop {
                Stop::Running => {
                    if self.exit.is_none() {
                        Some(self.deadline)
                    } else {
                        None
                    }
                }
                Stop::Terminating { next, .. } | Stop::Killing { next, .. } => Some(next),
                Stop::Closing { .. } => None,
            }
        };
        match self.answer_deadline() {
            Some(answer) => match process {
                Some(process) => Some(process.min(answer)),
                None => Some(answer),
            },
            None => process,
        }
    }

    pub(crate) fn answer(self, owner: Token) -> ToDomain {
        let why = match self.stop {
            Stop::Running => None,
            Stop::Terminating { why, .. } | Stop::Killing { why, .. } | Stop::Closing { why } => Some(why),
        };
        let timed_out = why == Some(Why::Deadline);
        let cancelled = why == Some(Why::Cancel);
        let (head, tail, dropped) = self.output.finish();
        match self.work {
            Work::Shell => {
                let done = if cancelled {
                    tools::Done::Cancelled
                } else if self.failure.is_some() {
                    tools::Done::Failed { fault: tools::Fault::Other }
                } else {
                    let exit = tool_exit(self.exit, timed_out);
                    tools::Done::Exited { exit, head, tail, dropped }
                };
                ToDomain::Done { owner, done }
            }
            Work::Search(search) => {
                let done = if cancelled {
                    tools::Done::Cancelled
                } else if self.failure.is_some() {
                    tools::Done::Failed { fault: tools::Fault::Other }
                } else {
                    search.finish(self.exit, timed_out, head, tail, dropped)
                };
                ToDomain::Done { owner, done }
            }
            Work::List { listing, directory } => {
                let done = if cancelled {
                    tools::Done::Cancelled
                } else if timed_out {
                    tools::Done::TimedOut
                } else {
                    match directory.failure() {
                        Some(done) => done,
                        None => match self.failure {
                            Some(error) => listing_failure(error),
                            None => listing.finish(self.exit, timed_out),
                        },
                    }
                };
                ToDomain::Done { owner, done }
            }
            Work::Check => {
                if cancelled {
                    ToDomain::Aborted { owner }
                } else {
                    let output = tail;
                    let cut = dropped
                        .checked_add(u64::try_from(head.len()).expect("head length fits"))
                        .expect("output count fits");
                    let exit = run_exit(self.exit, timed_out, self.failure.is_some());
                    ToDomain::Checked { owner, ran: run::Ran { exit, output, cut } }
                }
            }
        }
    }
}

fn tool_exit(exit: Option<kernel::Exit>, timed_out: bool) -> tools::Exit {
    if timed_out {
        return tools::Exit::TimedOut;
    }
    match exit {
        Some(kernel::Exit::Code(code)) => tools::Exit::Code { code },
        Some(kernel::Exit::Signal(signal)) => tools::Exit::Signal { signal: u8::try_from(signal).unwrap_or(u8::MAX) },
        None => tools::Exit::Signal { signal: 0 },
    }
}

fn run_exit(exit: Option<kernel::Exit>, timed_out: bool, failed: bool) -> run::Exit {
    if timed_out {
        return run::Exit::TimedOut;
    }
    if failed {
        return run::Exit::Unstarted;
    }
    match exit {
        Some(kernel::Exit::Code(code)) => run::Exit::Code { code },
        Some(kernel::Exit::Signal(signal)) => run::Exit::Signalled { signal },
        None => run::Exit::Unstarted,
    }
}

fn pipes() -> Box<[kernel::Pipe]> {
    Box::new([
        kernel::Pipe { child: 1, way: kernel::Way::Out, parent: None },
        kernel::Pipe { child: 2, way: kernel::Way::Out, parent: None },
    ])
}

#[expect(clippy::too_many_arguments, reason = "owned request fields cross the machine boundary")]
pub(crate) fn shell(
    owner: Token,
    cwd: tools::Place,
    command: Box<[u8]>,
    environment: &crate::Environment,
    head: u32,
    tail: u32,
    deadline: Time,
    limits: &Limits,
) -> Option<(Process, Below)> {
    if head > limits.output_bytes
        || tail > limits.output_bytes
        || command.contains(&0)
        || command.len() > usize::try_from(limits.tool_payload).ok()?
    {
        return None;
    }
    let env = environment.for_process(Purpose::Shell);
    let args = Box::new([Box::from(&b"-c"[..]), command]);
    let dir = crate::files::io_path(cwd.path);
    let spawn = Spawn { program: Box::from(&b"/bin/sh"[..]), args, env, dir, pipes: pipes() };
    Some((Process::new(Work::Shell, deadline, head, tail), Below::Spawn { owner, root: cwd.root, spawn }))
}

#[expect(clippy::too_many_arguments, reason = "owned search request fields cross the machine boundary")]
pub(crate) fn search(
    owner: Token,
    at: tools::Place,
    pattern: Box<[u8]>,
    glob: Option<Box<[u8]>>,
    hits: u32,
    bytes: u32,
    deadline: Time,
    environment: &crate::Environment,
    limits: &Limits,
) -> Option<(Process, Below)> {
    if hits > limits.search_hits || bytes > limits.search_bytes || pattern.contains(&0) {
        return None;
    }
    let filter_bytes = match &glob {
        Some(filter) if filter.contains(&0) => return None,
        Some(filter) => filter.len(),
        None => 0,
    };
    if pattern.len().checked_add(filter_bytes)? > usize::try_from(limits.tool_payload).ok()? {
        return None;
    }
    let count = if glob.is_some() { 6 } else { 5 };
    let mut args = List::with_capacity(count);
    args.push(Box::from(&b"--no-config"[..])).ok()?;
    args.push(Box::from(&b"--json"[..])).ok()?;
    let mut expression = Writer::new(b"--regexp=".len().checked_add(pattern.len())?);
    expression.put(b"--regexp=").ok()?;
    expression.put(&pattern).ok()?;
    args.push(expression.finish()).ok()?;
    if let Some(glob) = glob {
        let mut filter = Writer::new(b"--glob=".len().checked_add(glob.len())?);
        filter.put(b"--glob=").ok()?;
        filter.put(&glob).ok()?;
        args.push(filter.finish()).ok()?;
    }
    args.push(Box::from(&b"--"[..])).ok()?;
    args.push(crate::files::io_path(at.path)).ok()?;
    let spawn = Spawn {
        program: Box::from(&b"/usr/bin/rg"[..]),
        args: args.into_boxed(),
        env: environment.for_process(Purpose::Search),
        dir: crate::files::io_path(Box::new([])),
        pipes: pipes(),
    };
    let parser = Search::new(hits, bytes, limits.search_line_bytes);
    Some((Process::new(Work::Search(parser), deadline, bytes, bytes), Below::Spawn { owner, root: at.root, spawn }))
}

pub(crate) fn check(
    owner: Token,
    program: run::Place,
    environment: &crate::Environment,
    tail: u32,
    deadline: Time,
    limits: &Limits,
) -> Option<(Process, Below)> {
    if tail > limits.output_bytes || program.path.is_empty() || program.path.contains(&0) {
        return None;
    }
    let env = environment.for_process(Purpose::Check);
    let spawn = Spawn {
        program: program.path,
        args: Box::new([]),
        env,
        dir: crate::files::io_path(Box::new([])),
        pipes: pipes(),
    };
    Some((Process::new(Work::Check, deadline, 0, tail), Below::Spawn { owner, root: program.root, spawn }))
}

pub(crate) fn stream(process: &mut Process, pipe: Token, up: Up, below: &mut Queue<Below>) {
    match up {
        Up::Bytes(bytes) => {
            for byte in bytes {
                process.byte(pipe, byte);
            }
            below.push(Below::Process(io::Request::Stream {
                stream: pipe,
                down: Down::Demand { read: Read::Arrived { max: OUTPUT_READ }, room: 0 },
            }));
        }
        Up::End => below.push(Below::Process(io::Request::Close { entity: pipe })),
        Up::Failed(_) => {
            process.failed(io::Error::Other);
            below.push(Below::Process(io::Request::Abort { entity: pipe }));
        }
        Up::Room => unreachable!("read-only output pipes grant no write room"),
    }
}

/// Start a selected-file listing; rg owns ignore and glob rules, the parser owns the tree.
#[expect(clippy::too_many_arguments, reason = "owned listing request fields cross the machine boundary")]
pub(crate) fn list(
    owner: Token,
    at: tools::Place,
    depth: tools::Depth,
    glob: Option<Box<[u8]>>,
    maximum: u32,
    bytes: u64,
    deadline: Time,
    environment: &crate::Environment,
    limits: &Limits,
) -> Option<(Process, Below)> {
    if maximum > limits.entries || bytes > limits.entry_bytes {
        return None;
    }
    match depth {
        tools::Depth::Levels(0) => return None,
        tools::Depth::Levels(_) | tools::Depth::Whole => {}
    }
    match &glob {
        Some(filter) if filter.contains(&0) || filter.len() > usize::try_from(limits.tool_payload).ok()? => {
            return None;
        }
        Some(_) | None => {}
    }
    let mut args = List::with_capacity(9);
    for word in
        [b"--no-config".as_slice(), b"--files", b"--null", b"--sort=path", b"--glob=!**/.git/**", b"--glob=!**/.git"]
    {
        args.push(Box::from(word)).ok()?;
    }
    if let Some(glob) = glob {
        let mut filter = Writer::new(b"--glob=".len().checked_add(glob.len())?);
        filter.put(b"--glob=").ok()?;
        filter.put(&glob).ok()?;
        args.push(filter.finish()).ok()?;
    }
    args.push(Box::from(b"--".as_slice())).ok()?;
    args.push(Box::from(b".".as_slice())).ok()?;
    let spawn = Spawn {
        program: Box::from(b"/usr/bin/rg".as_slice()),
        args: args.into_boxed(),
        env: environment.for_process(Purpose::List),
        dir: crate::files::io_path(at.path),
        pipes: pipes(),
    };
    let listing = Listing::new(maximum, bytes, depth, limits.search_line_bytes);
    let (directory, request) = Directory::open(owner, at.root, spawn, deadline);
    let process = Process::new(Work::List { listing, directory }, deadline, 0, 0);
    Some((process, request))
}

fn listing_failure(error: io::Error) -> tools::Done {
    match error {
        io::Error::TimedOut => tools::Done::TimedOut,
        io::Error::Refused | io::Error::Unreachable | io::Error::Reset | io::Error::Busy | io::Error::Other => {
            tools::Done::Failed { fault: tools::Fault::Other }
        }
    }
}

#[cfg(test)]
mod signal_tests {
    use super::{Process, ToDomain, Work, kernel};
    use skein_lib::{Time, Token};

    #[test]
    fn a_check_ended_by_a_signal_keeps_its_number() {
        for signal in [1, 15, 27, u32::MAX] {
            let mut process = Process::new(Work::Check, Time::ZERO, 0, 0);
            process.exited(kernel::Exit::Signal(signal), &mut skein_lib::Queue::uncapped());
            let ToDomain::Checked { owner, ran } = process.answer(Token::new(7)) else {
                panic!("a check has a checked terminal");
            };
            assert_eq!(owner, Token::new(7));
            assert_eq!(ran.exit, smith_domain::run::Exit::Signalled { signal });
        }
    }
}
