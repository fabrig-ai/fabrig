//! Bounded startup and staged routing for one agent process.
//! The framed service retains a root domain, protocol components and Skein IO.
//! The host effects face retains lower components without a root or channel.
//! Both keep a file driver and queues for their requests and terminals. It never
//! chooses a domain outcome or interprets a bearer token.
//! Contract: smith/docs/design/protocol/agent.md, sections 4–6; smith/docs/design/protocol/README.md, section 6.

use alloc::boxed::Box;

mod admission;
mod down;
mod effects;
mod face;
pub(crate) mod link;
mod opening;
mod readiness;
mod request;
mod roots;
mod routing;
mod stages;
mod up;

#[cfg(test)]
#[expect(clippy::wildcard_enum_match_arm, reason = "test assertions describe unexpected events")]
pub(crate) mod tests;

pub use effects::Effects;

use readiness::{
    channel_event_ready, channel_room, domain_entrance_ready, domain_request_credit, domain_requests_dispatched,
    llm_events_ready, pending_message_ready, sinks_take, submissions_ready,
};
use routing::component_owner;

mod dump;
mod program;
pub use dump::{Dump, Dumped, Model, Started};

pub(crate) use program::bound;
pub use program::{AdoptError, ProgramBackend, program_operations, program_valid};
pub mod teardown;

use teardown::Teardown;

use skein_channel::{Lower as ChannelLower, StreamMode};
use skein_io::{self as io, file, file_layer, kernel};
use skein_lib::{Duration, Env, Map, Queue, Slab, Time, Token, Wall};
use smith_domain::{self as domain, run};
use smith_protocol_channel as channel;
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;

#[derive(Debug)]
struct PendingMessage {
    finish_requested: bool,
    name: Token,
    label: Box<[u8]>,
    text: Box<[u8]>,
}

// Route names also cover component terminals that do not name native IO operations.
pub(crate) const ROUTE_HEADROOM: u32 = 64;

const COMPONENT_TAG: u8 = 8;

/// Fixed limits for one agent process and every stage it owns.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub domain: domain::Limits,
    pub channel: channel::Limits,
    pub llm: llm::ComponentLimits,
    pub machine: machine::Limits,
    pub io: io::Limits,
    pub file_slots: u32,
    pub file_read: u32,
    pub file_entries: u32,
    pub file_bytes: u32,
    pub file_timeout: Duration,
    /// Longest hold before projecting live bytes (smith/docs/design/protocol/limits.md 2.3).
    pub live_interval: Duration,
    /// Backlog watermark and maximum entries drained per iteration at each stage.
    pub queue: u32,
    /// Concurrent IO owner and kernel submission routes.
    pub routes: u32,
    /// Declared process memory in bytes, carried without a startup fit check.
    pub memory: u64,
    /// Event ownership, absent for a host that has no process stream.
    pub events: Option<smith_protocol_events::EventLimits>,
}

/// All startup inputs except the inherited descriptors, which the shell
/// adopts after constructing the service (smith/docs/design/protocol/agent.md, section 4).
#[derive(Debug)]
pub struct Config {
    pub limits: Limits,
    pub domain: domain::Config,
    pub channel_endpoints: smith_protocol_translate::Endpoints,
    pub llm_endpoints: llm::Endpoints,
    pub environment: machine::Environment,
    pub stream_mode: StreamMode,
    pub program: Option<ProgramBackend>,
}

/// Why startup cannot build a bounded agent service.
#[derive(Debug)]
pub enum ConfigError {
    /// IO cannot run under the requested capacities.
    Io,
    /// A stage cannot reserve its maximum output.
    Queue,
    /// A part cannot be built under the requested limits.
    Limits,
    /// The host channel cannot open under these limits or endpoints.
    Channel(channel::Error),
    /// The LLM pool cannot retain these endpoints or limits.
    Llm(llm::ComponentError),
}

/// The first failure that prevented an answer from crossing the channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The channel ended before the answer.
    ChannelEnded,
    /// The channel's output side failed.
    WriteFailed,
    /// A host grant could not be installed below the domain.
    Credential,
    /// A required host record could not be encoded or queued.
    Send,
}

/// A lower owner belongs to exactly one protocol component.
#[derive(Clone, Copy, Debug)]
enum Owner {
    Llm(Token),
    Machine(Token),
    Program(Token),
}

/// The running agent and every bounded queue between its layers.
#[derive(Debug)]
pub struct Service {
    pub(crate) stages: Stages,
    frame: Frame,
}

/// The framed agent's concrete domain, host protocol and native IO.
#[derive(Debug)]
struct Frame {
    domain: domain::Domain,
    channel: channel::Component,
    io: io::Io,
    program_diagnostics: Queue<(Token, run::Fault)>,
}

/// Lower construction and the inputs retained for framed composition.
#[derive(Debug)]
struct Parts {
    stages: Stages,
    domain: domain::Config,
    channel_endpoints: smith_protocol_translate::Endpoints,
    stream_mode: StreamMode,
}

/// Shared lower components, routing queues and their ordered close state.
#[derive(Debug)]
pub(crate) struct Stages {
    dumps: Option<dump::Kept>,
    limits: Limits,
    events: Option<smith_protocol_events::events::Producer>,
    teardown: Teardown,
    product_exit: Option<u8>,
    usage_unread: bool,
    live: crate::live::Live,
    llm: llm::Component,
    machine: machine::Component,
    program: Option<Box<program::Program>>,
    files: file_layer::FileIo,
    domain_env: Env<domain::Limits>,
    llm_env: Env<llm::ComponentLimits>,
    machine_env: Env<machine::Limits>,
    io_env: Env<io::Limits>,
    channel_events: Queue<channel::Heard>,
    pending_messages: Queue<PendingMessage>,
    channel_below: Queue<ChannelLower>,
    llm_events: Queue<llm::ToDomain>,
    llm_below: Queue<io::Request>,
    machine_events: Queue<machine::ToDomain>,
    machine_below: Queue<machine::Below>,
    domain_events: Queue<domain::Event>,
    lower_events: Queue<domain::Below>,
    domain_requests: Queue<domain::Request>,
    io_events: Queue<io::Event>,
    io_requests: Queue<io::Request>,
    file_events: Queue<file::Event>,
    io_submissions: Queue<kernel::Submit>,
    file_submissions: Queue<kernel::Submit>,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    io_completions: Queue<kernel::Complete>,
    owners: Slab<Owner>,
    in_flight: u32,
    signal_reads: u32,
    // Kernel submissions counted from 1 also seed the file layer.
    submitted: u64,
    next_send: u64,
    input: Option<Token>,
    output: Option<Token>,
    signals: Option<Token>,
    admission: admission::Admission,
    run_started: Time,
    pub(crate) line: crate::terminal::Last,
    opening: opening::Opening,
    pipe_routes: Map<Token, Token>,
    roots: roots::Roots,
}

const START_REPLY: Token = Token::new(1);
const ROOT_CLOSE_OWNER: Token = Token::new(u64::MAX);

const USAGE_OWNER: Token = Token::new(u64::MAX - 1);

/// Whether the owned lower parts can be built (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn effects_valid(limits: &Limits, endpoints: &llm::Endpoints) -> bool {
    // FileIo requires four positive capacities and a positive timeout.
    limits.file_slots > 0
        && limits.file_read > 0
        && limits.file_entries > 0
        && limits.file_bytes > 0
        && limits.file_timeout.as_nanos() > 0
        && limits.machine.processes.checked_mul(2).is_some()
        && llm::component_valid(&limits.llm, endpoints)
        && machine::valid(&limits.machine)
        && smith_protocol_events::events::valid(&crate::projection::sources(limits))
}

/// Whether all service parts and their IO composition fit (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits, endpoints: &llm::Endpoints) -> bool {
    effects_valid(limits, endpoints)
        && domain::valid(&limits.domain)
        && channel::valid(&limits.channel)
        && io_valid(limits)
}

/// Whether native IO tables and the shared socket composition fit (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn io_valid(limits: &Limits) -> bool {
    let Some(other) = limits.io.sockets.checked_sub(limits.llm.connection.connections) else { return false };
    let writers = match limits.events {
        Some(events) => match events.sinks.destinations.checked_add(1) {
            Some(writers) => writers,
            None => return false,
        },
        None => 1,
    };
    let child = limits.llm.connection.io;
    io::valid(&limits.io)
        && other >= writers
        && child.intake == limits.io.intake
        && child.receive == limits.io.receive
        && child.output == limits.io.output
        && child.sends == limits.io.sends
}

impl Service {
    /// Prepare every child and queue before a channel byte is read.
    pub fn new(config: Config, seed: u64) -> Result<Service, ConfigError> {
        let limits = config.limits;
        if !limits.io.is_usable() || limits.io.sockets < 3 || limits.file_slots < limits.machine.roots {
            return Err(ConfigError::Io);
        }
        let lower_routes = match io::operations(&limits.io) {
            Some(operations) => operations.checked_add(2).ok_or(ConfigError::Queue)?,
            None => return Err(ConfigError::Queue),
        };
        if limits.routes < lower_routes {
            return Err(ConfigError::Queue);
        }
        if !valid(&limits, &config.llm_endpoints) || !program_valid(&config) {
            return Err(ConfigError::Limits);
        }
        let Parts { stages, domain, channel_endpoints, stream_mode } = Stages::parts(config)?;
        let limits = stages.limits;
        let channel = match channel::Component::new(&limits.channel, stream_mode, channel_endpoints) {
            Ok(channel) => channel,
            Err(error) => return Err(ConfigError::Channel(error)),
        };
        let frame = Frame {
            domain: domain::Domain::new(&limits.domain, domain, seed),
            channel,
            io: io::Io::new(&limits.io),
            program_diagnostics: Queue::with_capacity(limits.queue),
        };
        Ok(Service { stages, frame })
    }

    /// Adopt the shell's opened append descriptor under its declared write deadline.
    pub fn adopt_destination(
        &mut self,
        fd: kernel::Fd,
        destination: smith_protocol_events::sinks::Destination,
        delivery: smith_protocol_events::sinks::Delivery,
        append: bool,
    ) -> Result<(), kernel::Fd> {
        let Some(events) = &mut self.stages.events else { return Err(fd) };
        if !events.sinks().room_for_destination(destination) {
            return Err(fd);
        }
        let deadline = self.stages.limits.events.expect("producer has configured ownership").write_deadline;
        let io = &mut self.frame.io;
        let token = if append { io.adopt_append(fd, deadline)? } else { io.adopt_write_pipe(fd, Some(deadline))? };
        events.sinks().add(destination, token, delivery).expect("destination room checked before descriptor adoption");
        Ok(())
    }

    /// Record the shell's startup before any service pass.
    pub fn start_session(&mut self, now: Time, header: smith_events::SessionStarted) {
        if let Some(events) = &mut self.stages.events {
            events.started(now, header);
        }
    }

    /// One destination IO gave up, and why, for the shell's operator line
    /// (smith/docs/design/protocol/events.md, section 5.2).
    pub fn pop_given_up(&mut self) -> Option<smith_protocol_events::sinks::GivenUp> {
        match &mut self.stages.events {
            Some(events) => events.sinks().pop_given_up(),
            None => None,
        }
    }

    /// A typed run accounting failure for the shell operator, drained once.
    pub fn pop_accounting_overflow(&mut self) -> Option<run::Overflow> {
        match &mut self.stages.events {
            Some(events) => events.pop_overflow_notice(),
            None => None,
        }
    }

    /// An unread process usage terminal, reported once without changing the answer.
    pub fn pop_usage_unread(&mut self) -> bool {
        core::mem::replace(&mut self.stages.usage_unread, false)
    }

    /// Drain one owner and bounded `Fault::Program` for the operator's stderr log,
    /// independently of capture and the main answer.
    /// The owner token correlates it with this process's conversation events.
    pub fn pop_program_diagnostic(&mut self) -> Option<(Token, run::Fault)> {
        self.frame.program_diagnostics.pop()
    }

    /// Kernel completions arriving since the last iteration.
    pub const fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.stages.completions
    }

    /// Submissions to hand to the kernel after an iteration.
    pub const fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.stages.submissions
    }

    /// Number of projected host facts dropped at the channel's output reserve.
    #[must_use]
    pub fn lost_channel_facts(&self) -> u64 {
        self.frame.channel.lost_facts()
    }

    /// Live bytes omitted from the host channel.
    #[must_use]
    pub fn lost_live_bytes(&self) -> u64 {
        self.frame.channel.lost_live_bytes()
    }

    /// Give Skein IO the inherited channel and termination descriptors. The
    /// shell opens these before iterating; io closes them after the run.
    pub fn adopt_streams(
        &mut self,
        input: kernel::Fd,
        output: kernel::Fd,
        signals: kernel::Fd,
    ) -> Result<(), kernel::Fd> {
        let input = self.frame.io.adopt_read_pipe(input)?;
        self.stages.input = Some(input);
        let output = self.frame.io.adopt_write_pipe(output, None)?;
        self.stages.output = Some(output);
        let signals = self.frame.io.adopt_signals(signals)?;
        self.stages.signals = Some(signals);
        Ok(())
    }

    /// The next host-supplied mount path that the shell must open as a root.
    #[must_use]
    pub fn root_to_open(&self) -> Option<&[u8]> {
        self.stages.opening.next_mount()
    }

    /// Receiving ceiling for absolute host metadata at root attachment.
    #[must_use]
    pub fn host_path_bytes(&self) -> u32 {
        self.stages.limits.domain.run.host_path_bytes
    }

    /// Attach one opened mount in Start order, or refuse the Start if the
    /// shell could not open it. Absolute host metadata is retained for aliases.
    pub fn root_opened(&mut self, root: Result<kernel::Fd, ()>, host: Box<[u8]>) {
        match root {
            Ok(fd) => match self.stages.files.adopt_root(fd) {
                Some(token) => {
                    self.stages.roots.adopt(token, self.stages.limits.machine.roots);
                    self.stages.opening.attached(host, token);
                    opening::follow(&mut self.stages, &mut self.frame);
                }
                None => self.stages.refuse_start(&mut self.frame, run::Invalid::Workspace),
            },
            Err(()) => self.stages.refuse_start(&mut self.frame, run::Invalid::Workspace),
        }
    }
}

/// Install only credential forms; accounts-sources supplies the program backend.
fn credential(value: &smith_protocol_translate::GrantValue) -> Option<skein_llm::Credential> {
    let credential = match value {
        smith_protocol_translate::GrantValue::Bearer(value) => skein_llm::Credential::bearer(value.clone()),
        smith_protocol_translate::GrantValue::Key(value) => skein_llm::Credential::key(value.clone()),
        smith_protocol_translate::GrantValue::Program(_) => return None,
    };
    if credential.value.is_empty() {
        return None;
    }
    Some(credential)
}

/// Advance io, the components, and the domain once at injected clocks.
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    let stages = &mut service.stages;
    let frame = &mut service.frame;
    stages::clocks(stages, now, wall);
    up::io_up(stages, Some(frame));
    up::lower_events(stages, Some(frame));
    up::component_up(stages, Some(frame));
    up::domain_up(stages, frame);
    down::domain_down(stages, Some(frame));
    down::component_down(stages);
    teardown::follow(stages, Some(frame));
    if let Some(events) = &mut stages.events {
        events.sinks().down(stages.io_env.now, &mut stages.io_requests);
    }
    routing::io_down(stages, Some(frame));
    frame.io.reclaim();
    stages.llm.reclaim();
    frame.domain.reclaim();
    stages.owners.reclaim();
}

/// Earliest active deadline over all composed children.
#[must_use]
pub fn next_deadline(service: &Service) -> Option<Time> {
    stages::next_deadline(&service.stages, Some(&service.frame))
}
/// Earliest policy deadline, excluding IO and file settlement.
#[must_use]
pub fn next_policy_deadline(service: &Service) -> Option<Time> {
    stages::next_policy_deadline(&service.stages, Some(&service.frame))
}
/// Whether another iteration can progress without waiting for the kernel.
#[must_use]
pub fn work_pending(service: &Service, now: Time) -> bool {
    stages::work_pending(&service.stages, Some(&service.frame), now)
}
/// Whether every composed child has closed and the answer crossed the channel.
#[must_use]
pub fn done(service: &Service) -> Option<bool> {
    stages::done(&service.stages)
}
/// The first failure retained for the shell's operator line.
#[must_use]
pub const fn failure(service: &Service) -> Option<Failure> {
    stages::failure(&service.stages)
}
