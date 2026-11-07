//! A short program child (protocol/programs.md, sections 3 and 4.1).
//! Keeps the child, optional input and bounded output captures until closure;
//! knows neither source credentials nor host vocabulary. `from_io` and
//! `fire` settle the child before `ended` exposes its borrowed record.

use alloc::boxed::Box;
use skein_io::{self as io, kernel};
use skein_lib::{List, Queue, Secret, Time, Token, stream};

use crate::STATUS_BYTES;

const READ_BYTES: u32 = 4096;

/// A caller's executable, arguments, whole environment and working directory.
#[derive(Clone)]
pub struct Launch {
    pub program: Box<[u8]>,
    pub arguments: Box<[Box<[u8]>]>,
    pub environment: Box<[Box<[u8]>]>,
    pub root: kernel::Fd,
    pub directory: Box<[u8]>,
}

impl core::fmt::Debug for Launch {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.debug_struct("Launch").field("environment_entries", &self.environment.len()).finish_non_exhaustive()
    }
}

/// A settled program command returned to its host or agent reader.
#[derive(Debug, PartialEq, Eq)]
pub enum Ended<'a> {
    /// The program could not start; all spawn resources have settled.
    Unspawned { detail: &'a [u8] },
    /// The host's deadline killed the child; its resources have settled.
    Deadline { detail: &'a [u8] },
    /// The child exited or was signalled; stdout is absent past the capture bound.
    Exited { exit: kernel::Exit, stdout: Option<&'a [u8]>, detail: &'a [u8] },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Active,
    ChildClosed,
    Done,
}

/// One program command driven by its caller through I/O events and a deadline.
pub struct Command {
    io_owner: Token,
    child: Option<Token>,
    pipes: [Option<Token>; 2],
    input_pipe: Option<Token>,
    input: Option<Secret>,
    stdin: bool,
    output_bytes: u32,
    deadline: Time,
    state: State,
    timed_out: bool,
    exit: Option<kernel::Exit>,
    stdout: List<u8>,
    stderr: List<u8>,
    tail_next: u32,
}

impl Command {
    /// Spawn with no stdin pipe, bounded stdout and a stderr tail (smith/docs/design/protocol/programs.md, 4.2).
    pub fn new(
        io_owner: Token,
        launch: Launch,
        deadline: Time,
        detail_bytes: u32,
        below: &mut Queue<io::Request>,
    ) -> Self {
        Self::spawn(io_owner, launch, None, STATUS_BYTES, deadline, detail_bytes, below)
    }

    /// Spawn with one secret ask on stdin and a caller-bounded stdout capture (smith/docs/design/protocol/hosts.md, 2).
    pub fn with_input(
        io_owner: Token,
        launch: Launch,
        input: Secret,
        output_bytes: u32,
        deadline: Time,
        detail_bytes: u32,
        below: &mut Queue<io::Request>,
    ) -> Self {
        Self::spawn(io_owner, launch, Some(input), output_bytes, deadline, detail_bytes, below)
    }

    fn spawn(
        io_owner: Token,
        launch: Launch,
        input: Option<Secret>,
        output_bytes: u32,
        deadline: Time,
        detail_bytes: u32,
        below: &mut Queue<io::Request>,
    ) -> Self {
        let mut pipes = List::with_capacity(3);
        pipes.push(kernel::Pipe { child: 1, way: kernel::Way::Out, parent: None }).expect("stdout slot");
        pipes.push(kernel::Pipe { child: 2, way: kernel::Way::Out, parent: None }).expect("stderr slot");
        if input.is_some() {
            pipes.push(kernel::Pipe { child: 0, way: kernel::Way::In, parent: None }).expect("stdin slot");
        }
        below.push(io::Request::Spawn {
            owner: io_owner,
            spawn: kernel::Spawn {
                program: launch.program,
                args: launch.arguments,
                env: launch.environment,
                root: launch.root,
                dir: launch.directory,
                pipes: pipes.into_boxed(),
            },
        });
        Self {
            io_owner,
            child: None,
            pipes: [None, None],
            input_pipe: None,
            stdin: input.is_some(),
            input,
            output_bytes,
            deadline,
            state: State::Active,
            timed_out: false,
            exit: None,
            stdout: List::with_capacity(output_bytes.checked_add(1).expect("output sentinel")),
            stderr: List::with_capacity(detail_bytes),
            tail_next: 0,
        }
    }

    /// Whether the caller should route this owner to the command.
    #[must_use]
    pub fn owns(&self, owner: Token) -> bool {
        self.io_owner == owner || self.pipes.contains(&Some(owner)) || self.input_pipe == Some(owner)
    }

    /// Consume one child or output-pipe event, retaining closure evidence.
    pub fn from_io(&mut self, event: io::Event, below: &mut Queue<io::Request>) {
        match event {
            io::Event::Spawned { child, pipes, .. } => {
                let expected = if self.stdin { 3 } else { 2 };
                assert_eq!(pipes.len(), expected, "command asks for output pipes and optional stdin");
                self.child = Some(child);
                for (slot, pipe) in self.pipes.iter_mut().zip(pipes.iter()) {
                    *slot = Some(*pipe);
                    Self::demand(*pipe, below);
                }
                self.input_pipe = pipes.get(2).copied();
                if self.timed_out {
                    self.input = None;
                    self.stop(below);
                } else if let Some(pipe) = self.input_pipe {
                    let input = self.input.as_ref().expect("stdin ask");
                    if input.is_empty() {
                        self.input = None;
                        below.push(io::Request::Close { entity: pipe });
                    } else {
                        below.push(io::Request::Stream {
                            stream: pipe,
                            down: stream::Down::Demand {
                                read: stream::Read::Nothing,
                                room: u32::try_from(input.len()).expect("bounded ask"),
                            },
                        });
                    }
                }
            }
            io::Event::Stream { owner, up } => match up {
                stream::Up::Bytes(bytes) => {
                    if self.pipes.first().copied().expect("stdout slot") == Some(owner) {
                        for byte in bytes {
                            if self.stdout.room() > 0 {
                                self.stdout.push(byte).expect("stdout room");
                            }
                        }
                    } else {
                        self.keep_tail(&bytes);
                    }
                    if !self.timed_out {
                        Self::demand(owner, below);
                    }
                }
                stream::Up::End => below.push(io::Request::Close { entity: owner }),
                stream::Up::Failed(_) => {
                    // A broken capture cannot be read as a complete status.
                    for _ in 0..self.stdout.room() {
                        self.stdout.push(0).expect("stdout sentinel room");
                    }
                    below.push(io::Request::Abort { entity: owner });
                }
                stream::Up::Room => {
                    assert_eq!(self.input_pipe, Some(owner), "only stdin asks for write room");
                    if let Some(input) = self.input.take() {
                        below.push(io::Request::Stream {
                            stream: owner,
                            down: stream::Down::Send(input.into_exposed()),
                        });
                        below.push(io::Request::Close { entity: owner });
                    }
                }
            },
            io::Event::Exited { exit, .. } => {
                self.exit = Some(exit);
                if let Some(child) = self.child {
                    below.push(io::Request::Close { entity: child });
                }
            }
            io::Event::Failed { .. } => {
                self.stop(below);
                if let Some(child) = self.child {
                    below.push(io::Request::Close { entity: child });
                }
            }
            io::Event::Closed { owner } => {
                if owner == self.io_owner {
                    self.state = State::ChildClosed;
                } else {
                    if self.input_pipe == Some(owner) {
                        self.input_pipe = None;
                        self.input = None;
                    }
                    for pipe in &mut self.pipes {
                        if *pipe == Some(owner) {
                            *pipe = None;
                        }
                    }
                }
            }
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Output { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. }
            | io::Event::Usage { .. } => unreachable!("only child and output events route to a command"),
        }
        self.settle();
    }

    /// Kill at the deadline, waiting for child and pipe closures before ending.
    pub fn fire(&mut self, now: Time, below: &mut Queue<io::Request>) {
        if self.state != State::Done && !self.timed_out && now >= self.deadline {
            self.cancel(below);
        }
    }

    /// Cancel once, retaining the child and pipes until their close reports.
    pub fn cancel(&mut self, below: &mut Queue<io::Request>) {
        if self.state != State::Done && !self.timed_out {
            self.timed_out = true;
            self.input = None;
            self.stop(below);
        }
    }

    /// The host deadline while the command is active and has not timed out.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.state == State::Done || self.timed_out { None } else { Some(self.deadline) }
    }

    /// The ending once the child and both output pipes have settled.
    #[must_use]
    pub fn ended(&self) -> Option<Ended<'_>> {
        if self.state != State::Done {
            return None;
        }
        let detail = self.stderr.as_slice();
        if self.timed_out {
            return Some(Ended::Deadline { detail });
        }
        match self.exit {
            Some(exit) => Some(Ended::Exited {
                exit,
                stdout: if self.stdout.len() > self.output_bytes { None } else { Some(self.stdout.as_slice()) },
                detail,
            }),
            None => Some(Ended::Unspawned { detail }),
        }
    }

    fn demand(pipe: Token, below: &mut Queue<io::Request>) {
        below.push(io::Request::Stream {
            stream: pipe,
            down: stream::Down::Demand { read: stream::Read::Arrived { max: READ_BYTES }, room: 0 },
        });
    }

    fn stop(&self, below: &mut Queue<io::Request>) {
        if let Some(child) = self.child {
            below.push(io::Request::Signal { child, to: kernel::Target::Child, signal: kernel::Signal::Kill });
        }
        for pipe in self.pipes.into_iter().chain([self.input_pipe]).flatten() {
            below.push(io::Request::Abort { entity: pipe });
        }
    }

    fn keep_tail(&mut self, bytes: &[u8]) {
        for byte in bytes {
            if self.stderr.room() > 0 {
                self.stderr.push(*byte).expect("stderr room");
            } else if self.stderr.capacity() > 0 {
                *self.stderr.get_mut(self.tail_next).expect("tail slot") = *byte;
                self.tail_next = self
                    .tail_next
                    .checked_add(1)
                    .expect("tail index")
                    .checked_rem(self.stderr.capacity())
                    .expect("nonempty tail");
            }
        }
    }

    fn settle(&mut self) {
        if self.state != State::ChildClosed || self.pipes != [None, None] || self.input_pipe.is_some() {
            return;
        }
        if self.tail_next > 0 {
            let mut tail = List::with_capacity(self.stderr.len());
            for index in 0..self.stderr.len() {
                let at = self
                    .tail_next
                    .checked_add(index)
                    .expect("tail offset")
                    .checked_rem(self.stderr.len())
                    .expect("nonempty tail");
                tail.push(*self.stderr.get(at).expect("tail slot")).expect("tail room");
            }
            self.stderr = tail;
            self.tail_next = 0;
        }
        self.input = None;
        self.state = State::Done;
    }
}

impl core::fmt::Debug for Command {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Command")
            .field("io_owner", &self.io_owner)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, Ended, Launch};
    use alloc::{boxed::Box, format};
    use skein_io::{self as io, kernel};
    use skein_lib::{Queue, Secret, Time, Token, stream};

    const OWNER: Token = Token::new(1);
    const CHILD: Token = Token::new(2);
    const OUT: Token = Token::new(3);
    const ERR: Token = Token::new(4);
    const INPUT: Token = Token::new(5);
    const SECRET: &[u8] = b"private-input";
    // An intentionally tiny stdout bound, and one byte beyond it.
    const OUTPUT_BYTES: u32 = 3;

    #[test]
    #[expect(clippy::disallowed_macros, reason = "the unit test renders Debug to check credential redaction")]
    fn an_input_command_releases_its_secret_and_settles_every_pipe() {
        for (bytes, output, readable) in
            [(SECRET, b"abc".as_slice(), true), (b"", b"abcd", false), (SECRET, SECRET, false)]
        {
            let mut below = Queue::uncapped();
            let mut command = Command::with_input(
                OWNER,
                Launch {
                    program: b"agent".as_slice().into(),
                    arguments: Box::new([]),
                    environment: Box::new([]),
                    root: kernel::Fd::new(3),
                    directory: b".".as_slice().into(),
                },
                Secret::new(bytes.into()),
                OUTPUT_BYTES,
                Time::from_nanos(100),
                32,
                &mut below,
            );
            let Some(io::Request::Spawn { spawn, .. }) = below.pop() else { panic!("spawn request") };
            assert_eq!(spawn.args.len(), 0, "no secret argument");
            assert_eq!(spawn.env.len(), 0, "no secret variable");
            assert_eq!(spawn.pipes.get(2), Some(&kernel::Pipe { child: 0, way: kernel::Way::In, parent: None }));
            command.from_io(
                io::Event::Spawned { owner: OWNER, child: CHILD, pipes: Box::new([OUT, ERR, INPUT]) },
                &mut below,
            );
            assert!(command.owns(INPUT), "stdin routes to its command");
            if !bytes.is_empty() {
                command.from_io(io::Event::Stream { owner: INPUT, up: stream::Up::Room }, &mut below);
            }
            let mut written = None;
            let mut finished = false;
            while let Some(request) = below.pop() {
                match request {
                    io::Request::Stream { stream: INPUT, down: stream::Down::Send(bytes) } => written = Some(bytes),
                    io::Request::Close { entity: INPUT } => finished = true,
                    io::Request::Listen { .. }
                    | io::Request::Connect { .. }
                    | io::Request::Bind { .. }
                    | io::Request::Reject { .. }
                    | io::Request::Stream { .. }
                    | io::Request::Output { .. }
                    | io::Request::Spawn { .. }
                    | io::Request::Signal { .. }
                    | io::Request::Usage { .. }
                    | io::Request::Window { .. }
                    | io::Request::Close { .. }
                    | io::Request::Abort { .. } => {}
                }
            }
            assert_eq!(written.as_deref(), if bytes.is_empty() { None } else { Some(bytes) });
            assert!(finished, "stdin ends after its ask");
            assert!(command.input.is_none(), "Secret is released when its bytes move to the write");
            command.from_io(io::Event::Stream { owner: ERR, up: stream::Up::Bytes(SECRET.into()) }, &mut below);
            command.from_io(io::Event::Stream { owner: OUT, up: stream::Up::Bytes(output.into()) }, &mut below);
            assert_eq!(
                format!("{command:?}"),
                "Command { io_owner: Token(1), state: Active, .. }",
                "captures stay out of Debug"
            );
            command.from_io(io::Event::Exited { owner: OWNER, exit: kernel::Exit::Code(0) }, &mut below);
            for owner in [OWNER, OUT, ERR] {
                command.from_io(io::Event::Closed { owner }, &mut below);
            }
            assert!(command.ended().is_none(), "stdin still owns its closure");
            command.from_io(io::Event::Closed { owner: INPUT }, &mut below);
            let Some(Ended::Exited { stdout, .. }) = command.ended() else { panic!("settled input command") };
            assert_eq!(stdout, if readable { Some(output) } else { None });
        }
    }
}
