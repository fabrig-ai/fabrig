//! One lazy program conversation (protocol/programs.md, sections 3 and 5).
//! Keeps a source grant, private run files, status child, MCP listener and
//! program pipes; never reads login contents. `from_domain` starts on demand and
//! handles cancellation and yield requests. `from_io` and `fire` advance startup,
//! `settled` observes each finish verdict, and `close` settles every resource.

use alloc::boxed::Box;
use skein_io::{self as io, file, kernel};
use skein_lib::{Duration, Env, List, Map, Queue, Secret, Tail, Time, Token};
use smith_domain::{GrantName, llm};
use smith_protocol_llm::{self as protocol, FromDomain, ToDomain};

use crate::{Programs, mcp};
use startup::Phase as State;

mod builtins;
mod calls;
mod compact;
mod conversation;
mod current;
use current::Complete;
mod diagnostics;
mod files;
mod intake;
mod interrupt;
mod measures;
mod memory;
mod native;
mod session;
pub use memory::backend_valid;
mod permission;
mod policy;
mod process;
mod recovery;
mod redaction;
mod replies;
mod start;
mod startup;
mod stop;
mod stopping;
mod stream;
mod tail;
mod transport;
mod turns;

#[cfg(test)]
mod builtin_tests;
#[cfg(test)]
mod conversation_tests;
#[cfg(test)]
mod exit_tests;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod measure_tests;
#[cfg(test)]
mod open_tests;
#[cfg(test)]
mod records_tests;
#[cfg(test)]
mod reply_tests;
#[cfg(test)]
mod resume_tests;
#[cfg(test)]
mod session_tests;
#[cfg(test)]
mod stop_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod turns_tests;

const PIPE_BYTES: u32 = 4096;
const INPUT_RIGHT: Token = Token::new(1);
const INTERRUPT_RIGHT: Token = Token::new(2);

#[derive(Clone, Copy)]
enum Owner {
    Status,
    Program,
    Files,
    Native,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pipes {
    input: Option<Token>,
    output: Option<Token>,
    errors: Option<Token>,
}

impl Pipes {
    const CLOSED: Self = Self { input: None, output: None, errors: None };

    fn from_spawn(pipes: &[Token]) -> Self {
        assert_eq!(pipes.len(), 3, "program requests three pipes");
        Self { input: pipes.first().copied(), output: pipes.get(1).copied(), errors: pipes.get(2).copied() }
    }

    fn all(self) -> [Option<Token>; 3] {
        [self.input, self.output, self.errors]
    }

    fn closed(&mut self, owner: Token) {
        for pipe in [&mut self.input, &mut self.output, &mut self.errors] {
            if *pipe == Some(owner) {
                *pipe = None;
            }
        }
    }
}

/// One normalized start directory, supplied by the service before Complete (smith/docs/design/protocol/programs.md 5.4).
#[derive(Clone, Debug)]
pub struct Mount {
    pub root: Token,
    pub host: Box<[u8]>,
    pub writable: bool,
}

/// An adopted programs root or a workspace supplied by the owning service.
#[derive(Clone, Copy, Debug)]
pub struct Root {
    pub file: Token,
    pub descriptor: kernel::Fd,
}

/// Fixed run startup and receiving bounds supplied by the owning service (5.1).
pub struct Settings {
    /// Maximum original host path length, in bytes (domain/tools.md 4).
    pub path_bytes: u32,
    pub program: Programs,
    pub directory: Option<Root>,
    pub workspace: Option<Root>,
    pub relay: Box<[u8]>,
    pub environment: Box<[Box<[u8]>]>,
    pub mcp: mcp::Limits,
    pub adapter: protocol::Limits,
    pub receiving: protocol::Receiving,
    /// JSON records retained from one complete program stream document.
    pub stream_tokens: u32,
    pub prompt_bytes: u64,
    pub contract_bytes: u32,
    pub line_bytes: u32,
    pub input_bytes: u32,
    pub max_turns: u32,
    pub grace: Duration,
    pub startup: Duration,
}

impl Settings {
    fn ledger_slots(&self) -> Option<u32> {
        self.receiving.max_completion_blocks.checked_mul(5)
    }

    /// Retained program JSON tokens: `N = line_bytes` (smith/docs/design/protocol/programs.md, section 5.5).
    /// Each reader record consumes at least one distinct encoded byte; both
    /// opening and closing container brackets are retained records. With bytes
    /// alone as the input this is tight: nested arrays around an empty array
    /// (even lengths) or a one-digit number (odd lengths) attain one record per
    /// byte. Flat arrays of one-digit numbers are less dense than containers.
    /// The reader's independent depth check still applies. Covering the whole
    /// admitted line also covers ignored fields, without envelope assumptions.
    #[must_use]
    pub const fn stream_tokens(line_bytes: u32) -> u32 {
        line_bytes
    }
}

/// A host's program grant, installed or renewed before its lapse (section 3).
#[derive(Debug)]
pub struct Grant {
    pub name: GrantName,
    pub source: Box<[u8]>,
    pub lapses: Time,
}

/// A grant does not renew this account and source, or has an invalid name.
#[derive(Debug)]
pub struct Refused;

/// One lower effect, routed by the service to native I/O or its file driver.
pub enum Below {
    /// A process, pipe or MCP operation, ending through `BelowEvent::Io`.
    Io(io::Request),
    /// A file operation, ending through `BelowEvent::File`.
    File { request: file::Request, deadline: Time },
}

/// A lower terminal supplied by the service to this backend (sections 3 and 5).
pub enum BelowEvent {
    /// A child, pipe or listener event belonging to this backend.
    Io(io::Event),
    /// One terminal for a private file operation or native version snapshot.
    File { event: file::Event, descriptor: Option<kernel::Fd> },
}

/// The run's program adapter, driven by the service (smith/docs/design/protocol/programs.md, sections 3, 5).
pub struct Backend {
    mounts: Box<[Mount]>,
    first: Token,
    settings: Settings,
    names: start::Names,
    grant: Option<Grant>,
    startup: startup::Startup,
    stopping: stopping::Stopping,
    current: current::Current,
    pending_failure: Option<(llm::Failure, Box<[u8]>)>,
    files: files::Files,
    native: Option<native::Snapshot>,
    native_writes: Map<Box<[u8]>, smith_domain::tools::Place>,
    process: process::Process,
    outgoing: Option<Box<[u8]>>,
    written: u32,
    interrupt: interrupt::Interrupt,
    compactions: u32,
    /// One-based index of the next message whose calls may be permitted.
    permitted_message: u64,
    run_time: Duration,
    line: List<u8>,
    stdout_demanding: bool,
    intake: List<u8>,
    intake_at: u32,
    intake_due: Option<Time>,
    measures: Queue<protocol::Measure>,
    prompt_pending: Option<List<smith_events::Block>>,
    prompt_due: bool,
    delta_carry: List<redaction::Carry>,
    calls: List<mcp::Up>,
    ledger: List<calls::Call>,
    turns: turns::Turns,
    stderr: Tail,
    stderr_wrapped: bool,
    io: Queue<io::Request>,
}

impl core::fmt::Debug for Backend {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Backend")
            .field("state", &self.startup.phase())
            .field("owner", &self.current.owner())
            .finish_non_exhaustive()
    }
}

impl Backend {
    /// Retain settings and names; emit nothing until the first Complete (section 3).
    #[must_use]
    pub fn new(first: Token, settings: Settings, entropy: &Secret) -> Self {
        let failure_bytes = settings.receiving.max_failure_bytes;
        let line = List::with_capacity(settings.line_bytes);
        let calls = List::with_capacity(u32::from(settings.mcp.calls));
        let turns = turns::Turns::new(&settings.receiving, settings.adapter.client.answer);
        let ledger = List::with_capacity(settings.ledger_slots().expect("call ledger"));
        let delta_carry = redaction::carries(&settings);
        let measure_capacity = measures::capacity(&settings).expect("measure capacity");
        Self {
            mounts: Box::default(),
            first,
            settings,
            names: start::Names::new(entropy),
            grant: None,
            startup: startup::Startup::new(),
            stopping: stopping::Stopping::new(),
            current: current::Current::new(),
            pending_failure: None,
            files: files::Files::new(),
            native: None,
            native_writes: Map::with_capacity(ledger.capacity()),
            process: process::Process::new(),
            outgoing: None,
            written: 0,
            interrupt: interrupt::Interrupt::new(),
            compactions: 0,
            permitted_message: 1,
            run_time: Duration::ZERO,
            line,
            stdout_demanding: false,
            intake: List::with_capacity(PIPE_BYTES),
            intake_at: 0,
            intake_due: None,
            measures: Queue::with_capacity(measure_capacity),
            prompt_pending: None,
            prompt_due: false,
            delta_carry,
            calls,
            ledger,
            turns,
            stderr: Tail::with_capacity(failure_bytes),
            stderr_wrapped: false,
            io: Queue::with_capacity(32),
        }
    }

    /// Adopt the shell's programs directory; its close belongs to this backend.
    pub fn directory(&mut self, root: Root) {
        assert_eq!(self.startup.phase(), State::Idle, "programs root is supplied before startup");
        self.settings.directory = Some(root);
    }

    /// Supply the first admitted workspace directory, borrowed until backend close.
    pub fn workspace(&mut self, root: Root) {
        assert_eq!(self.startup.phase(), State::Idle, "workspace is supplied before startup");
        self.settings.workspace = Some(root);
    }

    /// Supply all admitted host directories, with the working directory first (5.4).
    pub fn mounts(&mut self, mounts: Box<[Mount]>) {
        assert_eq!(self.startup.phase(), State::Idle, "mounts precede startup");
        self.mounts = mounts;
    }

    /// Install the source or renew the same account's generation and lapse (section 3).
    pub fn grant(&mut self, grant: Grant) -> Result<(), Refused> {
        crate::configuration_directory(&self.settings.program, &grant.source).or(Err(Refused))?;
        match &self.grant {
            Some(held)
                if held.name.account != grant.name.account
                    || held.source != grant.source
                    || grant.name.generation < held.name.generation =>
            {
                return Err(Refused);
            }
            Some(_) | None => {}
        }
        self.grant = Some(grant);
        Ok(())
    }

    /// Whether the service should route this listener, child, pipe or file owner here.
    #[must_use]
    pub fn owns(&self, owner: Token) -> bool {
        owner == self.token(Owner::Status)
            || owner == self.token(Owner::Program)
            || owner == self.token(Owner::Files)
            || owner == self.token(Owner::Native)
            || self.process.pipes().all().contains(&Some(owner))
            || self.startup.owns(owner)
    }

    /// Set the admitted charter's turn guard before its first completion (5.7).
    pub fn turn_limit(&mut self, turns: u32) {
        self.settings.max_turns = turns;
    }

    /// Supply the admitted run time for the program's MCP timeout (smith/docs/design/protocol/programs.md 5.7).
    pub fn run_time(&mut self, time: Duration) {
        self.run_time = time;
    }

    /// Whether a cancellation belongs to the outstanding program completion.
    #[must_use]
    pub fn pending(&self, owner: Token) -> bool {
        self.current.owner() == Some(owner)
    }

    /// Whether a completion grant belongs to the installed program account.
    #[must_use]
    pub fn holds(&self, name: GrantName) -> bool {
        match &self.grant {
            Some(grant) => grant.name.account == name.account,
            None => false,
        }
    }

    /// Begin startup, cancel, or yield the outstanding completion (sections 3, 5.7, 5.9).
    pub fn from_domain(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        request: FromDomain,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        match request {
            request @ FromDomain::Complete { owner, .. } => {
                let request = Complete::from_request(request);
                if self.stopping.intent() == stopping::Intent::YieldRequested {
                    self.stopping.intend(stopping::Intent::Ordinary);
                }
                self.begin_prompt(request.observation.is_some());
                if let Some((failure, detail)) = self.pending_failure.take() {
                    self.terminal_measure(env.now, owner);
                    up.push(ToDomain::Failed {
                        owner,
                        failure,
                        evidence: llm::Evidence::Unknown,
                        detail,
                        report: llm::Report::none(),
                    });
                    return;
                }
                match self.startup.phase() {
                    State::Running => self.window_request(env, request, up, below),
                    State::Idle | State::Checking | State::Starting | State::Ending | State::Ended => {
                        self.start(env, request, up, below);
                    }
                }
            }
            FromDomain::Close { .. } => return,
            FromDomain::Yield { owner } => {
                if self.pending(owner) {
                    if self.stopping.intent() == stopping::Intent::Ordinary {
                        self.stopping.intend(stopping::Intent::YieldRequested);
                    }
                    self.yield_for_wind_down(env, up, below);
                }
                return;
            }
            FromDomain::Cancel { owner } => {
                if self.current.owner() == Some(owner) {
                    self.close(env, up, below);
                }
            }
        }
        self.consume_stdout(env, up, below);
        self.flush(below);
    }

    /// Consume one lower terminal and advance startup or resource settlement (5.1).
    pub fn from_io(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        event: BelowEvent,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        match event {
            BelowEvent::File { event, descriptor } => {
                if event.owner() == self.token(Owner::Native) {
                    self.native_event(event, up);
                } else {
                    self.file_event(env, event, descriptor, up, below);
                }
            }
            BelowEvent::Io(event) => self.io_event(env, event, up, below),
        }
        self.progress(env, up, below);
        self.flush(below);
    }

    /// Advance the check, startup deadline and close grace at the supplied clock.
    pub fn fire(&mut self, env: &Env<protocol::ComponentLimits>, up: &mut Queue<ToDomain>, below: &mut Queue<Below>) {
        self.startup.fire(env.now, &mut self.io);
        let drain_due = match self.process.next_deadline() {
            Some(deadline) => env.now >= deadline,
            None => false,
        };
        if drain_due {
            self.terminate(env.now);
        }
        let eof_due = match self.startup.eof_deadline() {
            Some(deadline) => env.now >= deadline,
            None => false,
        };
        if eof_due {
            self.fail_program(env, b"smith: the MCP server ended", up, below);
        }
        let stop_due = match self.stopping.next_deadline() {
            Some(deadline) => env.now >= deadline,
            None => false,
        };
        if stop_due {
            self.terminate(env.now);
        }
        let startup_due = match self.startup.startup_deadline() {
            Some(deadline) => env.now >= deadline,
            None => false,
        };
        if startup_due {
            self.fail_program(env, b"smith: the program did not initialize", up, below);
        }
        let completion_due = match self.current.next_deadline() {
            Some(until) => env.now >= until,
            None => false,
        };
        if self.completion_active() && completion_due {
            let phase = self.current.phase(env.now);
            self.fail(env, llm::Failure::TimedOut { phase }, b"smith: program completion deadline", up, below);
        }
        self.progress(env, up, below);
        self.flush(below);
    }

    fn terminate(&mut self, now: Time) {
        self.stopping.terminate(now, self.settings.grace, &mut self.process, &mut self.io);
    }

    /// Earliest check, server authentication, startup or close deadline.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.closed() {
            return None;
        }
        let mut earliest = self.stopping.next_deadline();
        if self.completion_active()
            && let Some(at) = self.current.next_deadline()
        {
            earliest = Some(match earliest {
                Some(held) => held.min(at),
                None => at,
            });
        }
        if let Some(at) = self.intake_due {
            earliest = Some(match earliest {
                Some(held) => held.min(at),
                None => at,
            });
        }
        for at in [
            self.startup.eof_deadline(),
            self.startup.startup_deadline(),
            self.process.next_deadline(),
            match self.startup.check() {
                Some(check) => check.next_deadline(),
                None => None,
            },
            match self.startup.server() {
                Some(server) => server.next_deadline(),
                None => None,
            },
        ]
        .into_iter()
        .flatten()
        {
            earliest = Some(match earliest {
                Some(held) => held.min(at),
                None => at,
            });
        }
        earliest
    }

    /// Reply to finish from its own settled domain result, keyed by provider ID (5.5).
    /// Other calls keep their ordinary Complete path; unjudged calls fail at close.
    pub fn settled(&mut self, messages: &[smith_domain::session::llm::Message], below: &mut Queue<Below>) {
        use smith_domain::session::llm::{Block, Returned};
        for message in messages {
            for block in &message.content {
                match block {
                    Block::ToolResult { id, result } => {
                        for index in 0..self.ledger.len() {
                            let call = self.ledger.get_mut(index).expect("ledger cell");
                            if !call.builtin
                                && call.id == *id
                                && call.name.as_ref() == b"finish"
                                && !call.answered
                                && call.answer.is_none()
                            {
                                call.answer = Some(match result {
                                    Returned::Text { text, error, .. }
                                        if text.len()
                                            <= usize::try_from(self.settings.adapter.rendered_result)
                                                .expect("result ceiling") =>
                                    {
                                        if *error {
                                            mcp::Answer::Error(text.clone())
                                        } else {
                                            mcp::Answer::Text(text.clone())
                                        }
                                    }
                                    Returned::Text { .. }
                                    | Returned::Owned { .. }
                                    | Returned::Invalid { .. }
                                    | Returned::NotRun { .. }
                                    | Returned::Withdrawn => {
                                        mcp::Answer::Error(b"smith: the program turn was cancelled".as_slice().into())
                                    }
                                });
                            }
                        }
                    }
                    Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolCall { .. } => {}
                }
            }
        }
        self.answer_calls();
        self.flush(below);
    }

    /// End the conversation, closing input before terminating at its grace (5.7).
    pub fn close(&mut self, env: &Env<protocol::ComponentLimits>, up: &mut Queue<ToDomain>, below: &mut Queue<Below>) {
        self.close_turn(env, self.parking_wait(), up, below);
    }

    fn close_turn(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        parking: bool,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        match self.startup.phase() {
            State::Ending | State::Ended => return,
            State::Idle | State::Checking | State::Starting | State::Running => {}
        }
        let active = self.active_turn();
        self.startup.close(&mut self.io);
        if let Some(owner) = self.current.owner() {
            self.terminal_measure(env.now, owner);
            self.current.settle(current::Terminal::Cancelled, up);
        }
        self.current.clear(up);
        self.outgoing = None;
        self.stop(env, active, parking, up, below);
        if parking && self.stopping.action() == Some(stopping::Action::Finish) {
            self.stopping.begin(stopping::Action::Park, self.stopping.next_deadline());
        }
        self.read_stdout();
        if self.process.child().is_none() && !self.process.spawning() {
            self.startup.close_server(&mut self.io);
        }
        self.progress(env, up, below);
        self.flush(below);
    }

    /// Every owned resource, including the programs root, has reported closed.
    #[must_use]
    pub fn closed(&self) -> bool {
        self.startup.phase() == State::Ended
    }

    fn token(&self, owner: Owner) -> Token {
        let offset = match owner {
            Owner::Status => 3,
            Owner::Program => 4,
            Owner::Files => 5,
            Owner::Native => 6,
        };
        Token::new(self.first.raw().checked_add(offset).expect("backend token range"))
    }

    fn flush(&mut self, below: &mut Queue<Below>) {
        self.load_native(below);
        for _ in 0..self.io.capacity() {
            match self.io.pop() {
                Some(request) => below.push(Below::Io(request)),
                None => break,
            }
        }
    }

    fn fail_program(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        detail: &[u8],
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        self.fail(env, llm::Failure::Program { exit: self.process.exit() }, detail, up, below);
    }

    fn fail(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        failure: llm::Failure,
        detail: &[u8],
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        let detail = self.private_diagnostic(detail);
        let restart = self.current.owner().is_some() && recovery::restartable(failure);
        if let Some(owner) = self.current.owner() {
            self.terminal_measure(env.now, owner);
            self.current.settle(
                current::Terminal::Failed {
                    failure,
                    evidence: llm::Evidence::Unsent,
                    detail,
                    report: llm::Report::none(),
                },
                up,
            );
        } else if self.pending_failure.is_none() {
            self.pending_failure = Some((failure, detail));
        }
        if restart {
            let active = self.active_turn();
            self.startup.switch(&mut self.io);
            self.outgoing = None;
            self.stop(env, active, false, up, below);
            self.current.clear(up);
        } else {
            self.close_turn(env, false, up, below);
        }
    }
}

// The channel's program detail is text. Preserve valid bytes and replace only
// malformed sequences, keeping the same byte ceiling even at a split UTF-8 tail.
#[expect(
    clippy::disallowed_methods,
    reason = "protocol/programs.md, section 5.6: the protocol validates stderr at its text boundary"
)]
fn diagnostic(bytes: &[u8], maximum: u32) -> Box<[u8]> {
    let length = bytes.len().min(usize::try_from(maximum).expect("failure cap"));
    let mut output = List::with_capacity(u32::try_from(length).expect("failure cap"));
    let mut from = 0_usize;
    for _ in 0..=length {
        let rest = bytes.get(from..length).expect("diagnostic cursor");
        match core::str::from_utf8(rest) {
            Ok(_) => {
                for byte in rest {
                    output.push(*byte).expect("diagnostic room");
                }
                break;
            }
            Err(error) => {
                let valid = error.valid_up_to();
                for byte in rest.get(..valid).expect("valid UTF-8 prefix") {
                    output.push(*byte).expect("diagnostic room");
                }
                from = from.checked_add(valid).expect("bounded diagnostic cursor");
                match error.error_len() {
                    Some(count) => {
                        output.push(b'?').expect("one byte replaces at least one malformed byte");
                        from = from.checked_add(count).expect("bounded diagnostic cursor");
                    }
                    None => break,
                }
            }
        }
    }
    output.into_boxed()
}
