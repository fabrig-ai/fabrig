//! The local host's IO hub (protocol/host.md, sections 3 and 5.6).
//! It owns common terminal, file, signal and trace resources in both child modes.
//! Delivery IO retains bounded git and plain-tree operations; store IO routes
//! durable chat requests. Policy and credential meaning stay with their domains.
//! `up` converts completions to host observations; `down` submits requested IO.

use alloc::boxed::Box;
use skein_io::file_layer::step::{self, down_until};
use skein_io::{self as io, file, file_layer, kernel};
use skein_lib::{Env, Map, Queue, Time, Token, Wall, stream};
use smith_host_domain as host;
use smith_host_protocol as protocol;
use smith_local_domain as local;
use smith_local_protocol as local_protocol;

use crate::StartValues;

mod child;
mod close;
mod delivery;
mod signals;
mod store;
mod terminal;
use child::Child;
use close::Close;
use delivery::DeliveryIo;
use signals::Signals;
use store::StoreIo;
use terminal::{TerminalInput, WindowRead};

pub use smith_protocol_program::Launch;

/// Lower limits for one agent child and its channel, supplied by the service.
#[derive(Clone, Copy, Debug)]
pub struct ProcessLimits {
    pub io: io::Limits,
    pub channel: protocol::Limits,
    pub detail_bytes: u32,
    /// Kernel and shell edge capacity, and internal drain work per iteration.
    pub queue: u32,
}

const PROGRAM_TAG: u8 = 1;

/// The one close `HostIo` itself submits, on its own tag (skein/docs/design/runtime/lib.md, 4).
fn program_root() -> Token {
    Token::tagged(PROGRAM_TAG, 0).expect("program root fits namespace")
}

#[derive(Debug)]
pub(crate) struct HostIo {
    pub(crate) live: crate::live::Live,
    limits: ProcessLimits,
    pub(crate) programs: Option<Box<crate::service::program_check::Checks>>,
    launch: Launch,
    endpoints: smith_protocol_translate::Endpoints,
    io: io::Io,
    shell_output: Option<Token>,
    shell_events: Queue<io::Event>,
    child: Child,
    delivery: DeliveryIo,
    store_io: StoreIo,
    files: file_layer::FileIo,
    file_limits: file_layer::Limits,
    file_roots: Box<[Token]>,
    file_requests: Queue<(file::Request, Time)>,
    file_events: Queue<file::Event>,
    io_submissions: Queue<kernel::Submit>,
    file_submissions: Queue<kernel::Submit>,
    in_flight: u32,
    io_completions: Queue<kernel::Complete>,
    io_events: Queue<io::Event>,
    pub(crate) io_requests: Queue<io::Request>,
    process_events: Queue<protocol::ProcessEvent>,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    send: u64,
    failed: bool,
    terminal_input: TerminalInput,
    window_read: WindowRead,
    signals: Signals,
    close: Close,
}

/// IO table capacities and the selected process channel's framing edges.
pub(crate) fn valid(limits: &ProcessLimits, framed: bool) -> bool {
    if !io::valid(&limits.io) || !local_protocol::Store::valid(&store_limits()) {
        return false;
    }
    if framed {
        let Ok(schema) = smith_protocol_channel::schema_for_start(&limits.channel.bodies, limits.channel.start_bytes)
        else {
            return false;
        };
        return schema.check(skein_channel::Role::Initiator, &limits.channel.channel).is_ok();
    }
    true
}

impl HostIo {
    pub(crate) fn new(
        limits: &ProcessLimits,
        launch: Launch,
        endpoints: smith_protocol_translate::Endpoints,
        seed: u64,
    ) -> Option<Self> {
        if !limits.io.is_usable() {
            return None;
        }
        let timeout = skein_lib::Duration::from_secs(30);
        let file_limits = file_layer::Limits { files: 66, read: 4096, entries: 8196, whole: 64 << 20, timeout };
        let mut files = file_layer::FileIo::with_limits(&file_limits);
        files.seed_randomness(seed);
        Some(Self {
            io: io::Io::new(&limits.io),
            programs: None,
            shell_output: None,
            shell_events: Queue::with_capacity(limits.queue),
            live: crate::live::Live::new(),
            limits: *limits,
            launch,
            endpoints,
            child: Child::Absent,
            delivery: DeliveryIo::new(),
            store_io: StoreIo::new(),
            files,
            file_limits,
            file_roots: Box::new([]),
            file_requests: Queue::uncapped(),
            file_events: Queue::uncapped(),
            io_submissions: Queue::uncapped(),
            file_submissions: Queue::uncapped(),
            in_flight: 0,
            io_completions: Queue::uncapped(),
            io_events: Queue::uncapped(),
            io_requests: Queue::uncapped(),
            process_events: Queue::uncapped(),
            submissions: Queue::with_capacity(limits.queue),
            completions: Queue::with_capacity(limits.queue),
            send: 1,
            failed: false,
            terminal_input: TerminalInput::Absent,
            window_read: WindowRead::Idle,
            signals: Signals::Absent,
            close: Close::Open,
        })
    }

    fn file_env(&self, now: Time, wall: Wall) -> Env<file_layer::Limits> {
        Env { now, wall, limits: self.file_limits }
    }

    // Internal handoffs count toward admission; edges never contribute.
    pub(crate) fn backlog(&self) -> u32 {
        let mut total = 0_u32;
        for entries in [
            self.io_completions.len(),
            self.io_events.len(),
            self.io_requests.len(),
            self.io_submissions.len(),
            self.file_requests.len(),
            self.file_events.len(),
            self.file_submissions.len(),
            self.store_io.store_requests.len(),
            self.store_io.store_actions.len(),
            self.process_events.len(),
        ] {
            total = total.checked_add(entries).expect("process backlog entries fit u32");
        }
        total
    }

    // O2: dispatch a store transition's actions before consuming the next file
    // terminal, so its answer precedes a later marker or plain-scan result.

    pub(crate) fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }

    pub(crate) fn io(&mut self) -> &mut io::Io {
        &mut self.io
    }

    pub(crate) fn adopt_writer(&mut self, fd: kernel::Fd) -> Result<Token, kernel::Fd> {
        let token = self.io.adopt_write_pipe(fd, None)?;
        self.shell_output = Some(token);
        Ok(token)
    }

    pub(crate) fn io_requests(&mut self) -> &mut Queue<io::Request> {
        &mut self.io_requests
    }

    pub(crate) fn adopt_terminal(
        &mut self,
        input: Option<kernel::Fd>,
        signals: kernel::Fd,
        read_input: bool,
    ) -> Result<(), kernel::Fd> {
        if let Some(input) = input {
            let stream = self.io.adopt_read_pipe(input)?;
            self.terminal_input.adopted(stream, read_input, &mut self.io_requests);
        }
        let signal = self.io.adopt_signals(signals)?;
        self.signals.adopted(signal);
        Ok(())
    }

    pub(crate) fn read_window(&mut self) {
        if self.terminal_input.open().is_some() {
            self.window_read.asked();
        }
    }

    pub(crate) fn input(&mut self, reading: local::Reading) {
        self.terminal_input.reading(reading, &mut self.io_requests);
    }

    pub(crate) fn at(&self, now: Time, wall: Wall) -> Env<io::Limits> {
        Env { now, wall, limits: self.limits.io }
    }

    pub(crate) fn adopt_trace(
        &mut self,
        fd: kernel::Fd,
        write_deadline: skein_lib::Duration,
    ) -> Result<Token, kernel::Fd> {
        self.io.adopt_append(fd, write_deadline)
    }

    pub(crate) fn adopt_output(
        &mut self,
        descriptor: kernel::Fd,
        append: bool,
        write_deadline: skein_lib::Duration,
    ) -> Result<Token, kernel::Fd> {
        if append {
            self.io.adopt_append(descriptor, write_deadline)
        } else {
            self.io.adopt_write_pipe(descriptor, Some(write_deadline))
        }
    }

    pub(crate) fn endpoint_name(&self, number: u32) -> Option<&[u8]> {
        self.endpoints.name_for(number)
    }

    pub(crate) fn close(&mut self) {
        let roots = u32::try_from(self.file_roots.len()).expect("admitted root count");
        if self.close.begin(roots) {
            self.cancel_programs();
            self.terminal_input.close(&mut self.io_requests);
        }
    }

    pub(crate) fn signal_streams(&self) -> u32 {
        self.signals.streams()
    }

    pub(crate) fn close_signals(&mut self) {
        self.signals.close(&mut self.io_requests);
    }

    pub(crate) fn take_shutdown(&mut self) -> bool {
        self.close.take_shutdown()
    }

    pub(crate) fn abort_stream(&mut self, trace: &mut Option<crate::stream::Spawned>) {
        if let Some(trace) = trace {
            trace.producer.sinks().abort(&mut self.io_requests);
        }
    }

    pub(crate) fn abort(&mut self) {
        self.terminal_input.abort(&mut self.io_requests);
    }

    pub(crate) fn closed(&self) -> bool {
        self.settled(0) && self.io.is_empty()
    }

    pub(crate) fn settled(&self, streams: u32) -> bool {
        self.close.closing()
            && match &self.programs {
                Some(checks) => checks.closed(),
                None => true,
            }
            && self.close.roots_left() == 0
            && self.io.sockets() == streams
            && self.store_io.store.is_none()
            && self.store_io.store_requests.is_empty()
            && self.store_io.store_actions.is_empty()
            && self.files.takes()
            && self.files.open_files() == 0
            && self.in_flight == self.signals.reads()
            && self.io_completions.is_empty()
            && self.file_requests.is_empty()
            && self.file_events.is_empty()
            && self.io_requests.is_empty()
            && self.io_events.is_empty()
            && self.io_submissions.is_empty()
            && self.file_submissions.is_empty()
            && self.submissions.is_empty()
            && self.completions.is_empty()
    }

    pub(crate) fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }

    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let git_deadline = match &self.delivery.git {
            Some(git) => git.next_deadline(),
            None => None,
        };
        let process_deadline = match &self.child {
            Child::Spawned { process, .. } => process.next_deadline(),
            Child::Absent => None,
        };
        let mut next: Option<Time> = None;
        for deadline in [
            self.io.next_deadline(),
            process_deadline,
            git_deadline,
            self.files.next_deadline(),
            match &self.programs {
                Some(checks) => checks.next_deadline(),
                None => None,
            },
        ]
        .into_iter()
        .flatten()
        {
            next = Some(match next {
                Some(current) => current.min(deadline),
                None => deadline,
            });
        }
        next
    }

    pub(crate) fn work_pending(&self, now: Time) -> bool {
        match &self.programs {
            Some(checks) if checks.work_pending(now, self.close.closing()) => return true,
            Some(_) | None => {}
        }
        self.io.is_ready()
            || self.io.is_due(now)
            || !self.io_completions.is_empty()
            || !self.io_events.is_empty()
            || !self.shell_events.is_empty()
            || !self.io_requests.is_empty()
            || self.window_read.requested()
            || self.process_work_pending()
            || !self.completions.is_empty()
            || (self.store_actions_dispatched() && !self.file_events.is_empty())
            || !self.store_io.store_actions.is_empty()
            || (!self.store_io.store_requests.is_empty()
                && match &self.store_io.store {
                    Some(store) => store.takes(),
                    None => false,
                })
            || (!self.file_requests.is_empty() && self.files.takes())
            || !self.io_submissions.is_empty()
            || !self.file_submissions.is_empty()
            || self.files.is_due(now)
            || (self.close.root_due() && self.files.takes())
    }

    pub(crate) const fn failed(&self) -> bool {
        self.failed
    }

    // A completion's token names its lane (skein/docs/design/runtime/io/files.md).
    fn split(&mut self, complete: kernel::Complete) {
        self.signals.completed(complete.op);
        self.in_flight = self.in_flight.checked_sub(1).expect("one completion per submitted operation");
        if complete.op == program_root() {
            self.programs.as_mut().expect("program root close").root_closed();
        } else if file_layer::owns(complete.op) {
            let env = self.file_env(Time::ZERO, Wall::EPOCH);
            step::up(&mut self.files, &env, complete, &mut self.file_events, &mut self.file_submissions);
        } else {
            self.io_completions.push(complete);
        }
    }

    pub(crate) fn io_up(&mut self, env: &Env<io::Limits>) {
        for _ in 0..self.completions.len() {
            let Some(complete) = self.completions.pop() else { break };
            self.split(complete);
        }
        io::stage::up(
            &mut self.io,
            env,
            &mut self.io_completions,
            &mut self.io_events,
            &mut self.io_submissions,
            env.now,
        );
    }

    pub(crate) fn shell_events(&mut self) -> &mut Queue<io::Event> {
        &mut self.shell_events
    }

    pub(crate) fn effects_up(&mut self, effects: &mut smith_agent_service::Effects) {
        for _ in 0..self.io_events.len() {
            let event = self.io_events.pop().expect("bounded native terminals");
            if effects.owns_io(&event) {
                effects.io_event(event);
            } else {
                self.io_events.push(event);
            }
        }
    }

    pub(crate) fn up(
        &mut self,
        env: &Env<io::Limits>,
        host_events: &mut Queue<host::Input>,
        terminal: &mut local_protocol::Terminal,
        terminal_events: &mut Queue<local_protocol::TerminalEvent>,
        local_events: &mut Queue<smith_local_domain::Event>,
        trace: &mut Option<crate::stream::Spawned>,
    ) {
        let now = env.now;
        self.markers_up(now, local_events);
        for _ in 0..self.limits.queue {
            if self.shell_events.room() == 0 {
                break;
            }
            let Some(event) = self.io_events.pop() else { break };
            let Some(event) = trace_up(trace, now, event) else { continue };
            if self.shell_output.is_some() && io_owner(&event) == self.shell_output {
                self.writer_up(event);
                continue;
            }

            if self.program_owns(io_owner(&event)) {
                self.programs.as_mut().expect("program route").receive(event, &mut self.io_requests);
                continue;
            }
            let git_owned = match &self.delivery.git {
                Some(git) => match io_owner(&event) {
                    Some(owner) => git.owns(owner),
                    None => false,
                },
                None => false,
            };
            if git_owned {
                self.delivery.git.as_mut().expect("git route exists").from_io(
                    event,
                    local_events,
                    &mut self.io_requests,
                );
                continue;
            }
            match event {
                io::Event::Resized => self.shell_events.push(io::Event::Resized),
                event @ io::Event::Window { .. } => {
                    self.window_read.answered();
                    self.shell_events.push(event);
                }
                io::Event::Shutdown { .. } => {
                    if !self.close.latch_shutdown() {
                        terminal.interrupt(terminal_events);
                    }
                }
                io::Event::Stream { owner, up } if self.terminal_input.owns(owner) => match up {
                    stream::Up::Bytes(bytes) => {
                        for byte in bytes {
                            terminal.feed(byte, terminal_events);
                        }
                        self.terminal_input.bytes(&mut self.io_requests);
                    }
                    stream::Up::End | stream::Up::Failed(_) => terminal.closed(terminal_events),
                    stream::Up::Room => unreachable!("read-only terminal asks for no output room"),
                },
                io::Event::Closed { owner } if self.terminal_input.owns(owner) => self.terminal_input.closed(),
                io::Event::Closed { owner } if self.signals.owns(owner) => self.signals.closed(),
                other @ (io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Stream { .. }
                | io::Event::Output { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Failed { .. }
                | io::Event::Closed { .. }) => match &mut self.child {
                    Child::Spawned { process, .. } => {
                        process.from_io(now, other, &mut self.process_events, &mut self.io_requests);
                    }
                    Child::Absent => self.shell_events.push(other),
                },
            }
        }
        if let Some(git) = &mut self.delivery.git {
            git.fire(now, &mut self.io_requests);
        }
        let git_done = match &self.delivery.git {
            Some(git) => git.done(),
            None => false,
        };
        if git_done {
            self.delivery.git = None;
        }
        match &mut self.child {
            Child::Spawned { process, .. } => process.fire(now, &mut self.process_events, &mut self.io_requests),
            Child::Absent => {}
        }
        self.drain_process(host_events);
    }

    fn writer_up(&mut self, event: io::Event) {
        match &event {
            io::Event::Closed { .. } => self.shell_output = None,
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. }
            | io::Event::Failed { .. } => {}
        }
        self.shell_events.push(event);
    }

    fn process_work_pending(&self) -> bool {
        for event in &self.process_events {
            if self.child.read_held() || !read_terminal(event) {
                return true;
            }
        }
        false
    }

    fn drain_process(&mut self, host_events: &mut Queue<host::Input>) {
        for _ in 0..self.process_events.len() {
            let event = self.process_events.pop().expect("bounded queued events");
            match &event {
                protocol::ProcessEvent::Channel { event, .. } => match event.as_ref() {
                    protocol::Received::WriteFailed => continue,
                    protocol::Received::Opened { .. }
                    | protocol::Received::Sent { .. }
                    | protocol::Received::Unsent { .. }
                    | protocol::Received::Hangup { .. }
                    | protocol::Received::Answer { .. }
                    | protocol::Received::Message { .. }
                    | protocol::Received::Live { .. } => {}
                },
                protocol::ProcessEvent::Spawned { .. }
                | protocol::ProcessEvent::Unspawned { .. }
                | protocol::ProcessEvent::Exited { .. }
                | protocol::ProcessEvent::Reaped { .. } => {}
            }
            if read_terminal(&event) {
                if !self.child.read_held() {
                    self.process_events.push(event);
                    continue;
                }
                self.child.answered();
            }
            host_events.push(to_host_event(event, &mut self.live));
        }
    }

    pub(crate) fn down(
        &mut self,
        env: &Env<io::Limits>,
        requests: &mut Queue<host::process::Request>,
        start_values: &mut Option<StartValues>,
        grant_values: &Map<u32, smith_protocol_translate::GrantValue>,
        host_events: &mut Queue<host::Input>,
        trace: &mut Option<crate::stream::Spawned>,
    ) {
        let now = env.now;
        self.window_read.follow(self.terminal_input.open(), &mut self.io_requests);
        if let Some(trace) = trace {
            if self.settled(trace.producer.streams().checked_add(self.signal_streams()).expect("bounded signal stream"))
            {
                trace.measure(&mut self.io_requests);
            }
            trace.producer.sinks().down(now, &mut self.io_requests);
        }
        for _ in 0..self.limits.queue {
            let Some(request) = requests.pop() else { break };
            self.request(request, start_values, grant_values, host_events);
        }
        match &mut self.child {
            Child::Spawned { process, .. } => {
                for _ in 0..self.limits.queue {
                    let Some(request) = process.next_channel_down() else { break };
                    process.channel_down(request, &mut self.io_requests);
                }
            }
            Child::Absent => {}
        }
        for _ in 0..self.limits.queue {
            if !self.io.takes() {
                break;
            }
            let Some(request) = self.io_requests.pop() else { break };
            io::down(&mut self.io, env, request, &mut self.io_submissions);
        }
        if self.files.takes()
            && let Some((request, deadline)) = self.file_requests.pop()
        {
            let env = env.with(self.file_limits);
            down_until(&mut self.files, &env, deadline, request, &mut self.file_events, &mut self.file_submissions);
        }
        self.follow_close(now);
        self.flush_submissions();
        self.io.reclaim();
    }

    fn flush_submissions(&mut self) {
        for _ in 0..self.limits.queue {
            if !self.edge_room() {
                break;
            }
            let Some(submit) = self.io_submissions.pop() else { break };
            self.forward(submit);
        }
        for _ in 0..self.limits.queue {
            if !self.edge_room() {
                break;
            }
            let Some(submit) = self.file_submissions.pop() else { break };
            self.forward(submit);
        }
    }

    fn edge_room(&self) -> bool {
        self.submissions.room() > 0 && self.in_flight < self.limits.queue
    }

    fn forward(&mut self, submit: kernel::Submit) {
        if let kernel::Op::ReadSignal { .. } = &submit.kind {
            self.signals.reading(submit.op);
        }
        self.in_flight = self.in_flight.checked_add(1).expect("kernel edge bounds operations in flight");
        self.submissions.push(submit);
    }

    fn request(
        &mut self,
        request: host::process::Request,
        start_values: &mut Option<StartValues>,
        grant_values: &Map<u32, smith_protocol_translate::GrantValue>,
        host_events: &mut Queue<host::Input>,
    ) {
        match request {
            host::process::Request::Spawn { owner, deadline, .. } => {
                let Ok(mut process) = protocol::Process::new(
                    owner,
                    &self.limits.channel,
                    self.limits.detail_bytes,
                    self.endpoints.clone(),
                ) else {
                    self.failed = true;
                    host_events.push(host::Input::Process(host::process::Event::Unspawned {
                        owner,
                        detail: Box::from(&b"invalid channel"[..]),
                    }));
                    return;
                };
                process.spawn(self.launch.clone(), deadline, &mut self.io_requests);
                self.child.spawned(process);
            }
            host::process::Request::Send { owner, message, .. } => {
                let send_token = Token::new(self.send);
                self.send = self.send.checked_add(1).expect("bounded send tokens");
                let process = self.child.process_mut();
                let sent = match message {
                    host::Down::Start { start, window } => {
                        let asked = match start.live {
                            Some(ask) => ask.text,
                            None => false,
                        };
                        self.live.start(owner, asked);
                        let values = start_values.take().expect("Start values paired before Spawn");
                        process.send_start(
                            start,
                            window,
                            protocol::Values { paths: values.paths, credentials: values.credentials },
                            send_token,
                            &mut self.process_events,
                            &mut self.io_requests,
                        )
                    }
                    host::Down::Message { finish_requested, name, label, text } => process.send_message(
                        host::Message { finish_requested, name, label, text },
                        send_token,
                        &mut self.process_events,
                        &mut self.io_requests,
                    ),
                    host::Down::Answer { call, reply } => {
                        process.send_reply(call, reply, send_token, &mut self.process_events, &mut self.io_requests)
                    }
                    host::Down::Acknowledge { turn } => {
                        process.send_acknowledge(turn, send_token, &mut self.process_events, &mut self.io_requests)
                    }
                    host::Down::Grant { grant } => {
                        let value = grant_values.get(&grant.account).expect("grant value supplied before send");
                        process.send_grant(
                            grant,
                            value.clone(),
                            send_token,
                            &mut self.process_events,
                            &mut self.io_requests,
                        )
                    }
                    host::Down::Cancel => {
                        process.send_cancel(send_token, &mut self.process_events, &mut self.io_requests)
                    }
                    host::Down::Drain => {
                        process.send_drain(send_token, &mut self.process_events, &mut self.io_requests)
                    }
                };
                if sent.is_err() {
                    self.failed = true;
                    host_events.push(host::Input::Process(host::process::Event::Unsent { owner }));
                }
            }
            host::process::Request::Signal { owner, signal, .. } => {
                let signal = match signal {
                    host::Signal::Terminate => kernel::Signal::Terminate,
                    host::Signal::Kill => kernel::Signal::Kill,
                };
                self.child.process_mut().signal(signal, &mut self.io_requests);
                host_events.push(host::Input::Process(host::process::Event::Signalled { owner }));
            }
            host::process::Request::Read { .. } => {
                self.child.hold();
                self.child.process_mut().read(&mut self.process_events, &mut self.io_requests);
            }
            host::process::Request::Wait { .. } | host::process::Request::Reap { .. } => {}
        }
    }
}

fn read_terminal(event: &protocol::ProcessEvent) -> bool {
    match event {
        protocol::ProcessEvent::Channel { event, .. } => match event.as_ref() {
            protocol::Received::Opened { .. }
            | protocol::Received::Sent { .. }
            | protocol::Received::Unsent { .. }
            | protocol::Received::WriteFailed => false,
            protocol::Received::Hangup { .. }
            | protocol::Received::Answer { .. }
            | protocol::Received::Message { .. }
            | protocol::Received::Live { .. } => true,
        },
        protocol::ProcessEvent::Spawned { .. }
        | protocol::ProcessEvent::Unspawned { .. }
        | protocol::ProcessEvent::Exited { .. }
        | protocol::ProcessEvent::Reaped { .. } => false,
    }
}

fn to_host_event(event: protocol::ProcessEvent, live: &mut crate::live::Live) -> host::Input {
    match event {
        protocol::ProcessEvent::Spawned { agent, process } => {
            host::Input::Process(host::process::Event::Spawned { owner: agent, process })
        }
        protocol::ProcessEvent::Unspawned { agent, detail } => {
            host::Input::Process(host::process::Event::Unspawned { owner: agent, detail })
        }
        protocol::ProcessEvent::Exited { agent, exit } => {
            host::Input::Process(host::process::Event::Exited { owner: agent, exit: to_host_exit(exit) })
        }
        protocol::ProcessEvent::Reaped { agent, detail } => {
            host::Input::Process(host::process::Event::Reaped { owner: agent, detail })
        }
        protocol::ProcessEvent::Channel { agent, event } => live.receive(agent, *event),
    }
}

/// Preserve io's observed exit without inferring it from the signals sent.
#[must_use]
pub const fn to_host_exit(exit: Option<kernel::Exit>) -> Option<host::Exit> {
    match exit {
        Some(kernel::Exit::Code(code)) => Some(host::Exit::Code(code)),
        Some(kernel::Exit::Signal(signal)) => Some(host::Exit::Signal(signal)),
        None => None,
    }
}

pub(crate) fn channel_event(agent: Token, event: protocol::Received) -> host::Input {
    use protocol::Received;
    match event {
        Received::Message { message } => host::Input::Process(host::process::Event::Received { owner: agent, message }),
        Received::Opened { .. } => unreachable!("opened is emitted as Spawned"),
        Received::Hangup { .. } => host::Input::Process(host::process::Event::Hangup { owner: agent }),
        Received::Sent { .. } => host::Input::Process(host::process::Event::Sent { owner: agent }),
        Received::Unsent { .. } => host::Input::Process(host::process::Event::Unsent { owner: agent }),
        Received::Answer { answer, .. } => {
            host::Input::Process(host::process::Event::Received { owner: agent, message: host::Up::Answer { answer } })
        }

        Received::Live { .. } => host::Input::Process(host::process::Event::Progress { owner: agent }),

        Received::WriteFailed => unreachable!("write failure does not end a host read"),
    }
}

fn git_limits(limits: &ProcessLimits) -> local_protocol::GitLimits {
    local_protocol::GitLimits {
        argument_bytes: 262_144,
        output_bytes: 65_536,
        conflicts: 64,
        path_bytes: 4096,
        detail_bytes: limits.detail_bytes,
    }
}

fn io_owner(event: &io::Event) -> Option<Token> {
    match event {
        io::Event::Listening { owner, .. }
        | io::Event::Accepted { owner, .. }
        | io::Event::Connecting { owner, .. }
        | io::Event::Connected { owner }
        | io::Event::Stream { owner, .. }
        | io::Event::Output { owner, .. }
        | io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Window { owner, .. }
        | io::Event::Usage { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => Some(*owner),
        io::Event::Resized | io::Event::Shutdown { .. } => None,
    }
}

fn plain_limits() -> local_protocol::PlainLimits {
    local_protocol::PlainLimits { entries: 256, path_bytes: 4096, file_bytes: 65_536 }
}

fn trace_up(trace: &mut Option<crate::stream::Spawned>, now: Time, event: io::Event) -> Option<io::Event> {
    if let Some(trace) = trace {
        match io_owner(&event) {
            Some(owner) if trace.owns(owner) => {
                trace.up(now, event);
                return None;
            }
            Some(_) | None => {}
        }
    }
    Some(event)
}

fn store_limits() -> local_protocol::StoreLimits {
    local_protocol::StoreLimits { file_bytes: 64 << 20, turns: 4096, deliveries: 4096, scan_bytes: 16 << 20 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::List;

    #[test]
    fn a_capture_of_a_directory_with_no_root_ends_unknown_and_idle() {
        let config = crate::tests::config();
        let mut hub = HostIo::new(&config.limits.process, config.launch, config.endpoints, 1).expect("hub");
        let mut directories = List::with_capacity(1);
        directories.push(0).expect("one directory");
        hub.capture_plain(directories, Time::from_nanos(1_000_000_000));
        assert_eq!(hub.take_capture_done(), Some(false));
        assert!(hub.delivery.inspection.idle());
    }

    #[test]
    fn invalid_limits_are_refused() {
        let base = crate::tests::config().limits.process.io;
        let mut limits = crate::tests::config().limits.process;
        assert!(valid(&limits, false));
        let changes: &[fn(&mut io::Limits)] = &[
            |limits| limits.refusals = 0,
            |limits| limits.intake = 0,
            |limits| limits.receive = 0,
            |limits| limits.output = 0,
            |limits| limits.sends = 0,
            |limits| limits.accepts = 0,
            |limits| limits.retry = skein_lib::Duration::ZERO,
            |limits| limits.sockets = u32::MAX,
            |limits| limits.sockets = u32::MAX / 8 + 1,
            |limits| limits.sockets = 1 << 24_u32,
            |limits| limits.sockets = u32::MAX / 8,
        ];
        for change in changes {
            let mut invalid = base;
            change(&mut invalid);
            limits.io = invalid;
            assert!(!valid(&limits, false), "invalid native IO: {invalid:?}");
        }
        for sockets in [0, (1 << 24_u32) - 1] {
            limits.io = io::Limits { sockets, ..base };
            assert!(valid(&limits, false));
        }
    }

    #[test]
    #[expect(
        clippy::disallowed_types,
        reason = "unit-test observations use ordinary Rust (programming-model.md, section 10.2)"
    )]
    fn a_store_answer_reaches_the_chat_before_a_later_file_terminal() {
        const STORE_OWNER: Token = Token::new(u64::MAX - 1);
        const MARKER_OWNER: Token = Token::new(7);
        // Consume the store terminal, dispatch Fault + Domain, then the marker.
        const DISPATCH_PASSES: u32 = 3;
        let config = crate::tests::config();
        let mut host_io = HostIo::new(&config.limits.process, config.launch, config.endpoints, 1).expect("host_io");
        host_io.adopt_store(kernel::Fd::new(10)).expect("store root");
        host_io.adopt_delivery_roots(Box::new([kernel::Fd::new(11)]), Box::new([]));
        let mut events = Queue::uncapped();
        start_store_load(&mut host_io, Time::ZERO, &mut events);
        host_io.start_markers(
            MARKER_OWNER,
            0,
            Box::new([Box::from(b"source.rs".as_slice())]),
            Time::from_nanos(1_000_000_000),
            &mut events,
        );
        // The file driver releases its serial slot when it queues a terminal.
        // Accumulate both completed requests before the next markers_up pass;
        // the chat handoff is now uncapped (protocol/limits.md, section 3.8).
        match host_io.file_requests.pop().expect("store load") {
            (file::Request::Load { owner: STORE_OWNER, path, .. }, _) => {
                assert_eq!(path.as_ref(), b"format");
            }
            other => panic!("store's first file request: {other:?}"),
        }
        host_io.file_events.push(file::Event::Failed {
            owner: STORE_OWNER,
            error: kernel::Error::Permission,
            committed: false,
            residue: None,
        });
        match host_io.file_requests.pop().expect("marker load") {
            (file::Request::Load { owner: MARKER_OWNER, .. }, _) => {}
            other => panic!("marker's file request: {other:?}"),
        }
        host_io.file_events.push(file::Event::Loaded { owner: MARKER_OWNER, bytes: b"clean\n".as_slice().into() });

        // Collect both terminals in dispatch order across the bounded passes.
        // The assertion is about the chat's stream, not queue room.
        let mut terminals = Vec::new();
        for _ in 0..DISPATCH_PASSES {
            host_io.markers_up(Time::ZERO, &mut events);
            while let Some(event) = events.pop() {
                terminals.push(event);
            }
        }
        let [
            local::Event::StoreFailed { turn: None, reason: local::StoreFailure::Read },
            local::Event::Git { owner: MARKER_OWNER, result: local::GitResult::Markers { first: None } },
        ] = terminals.as_slice()
        else {
            panic!("the store answer must reach the chat before the later marker terminal: {terminals:?}");
        };
        assert!(host_io.file_events.is_empty(), "both file terminals settle once");
        assert!(host_io.store_io.store_actions.is_empty(), "the store answer settles once");
        assert!(host_io.delivery.inspection.idle(), "the inspection settles once");
    }

    fn start_store_load(host_io: &mut HostIo, now: Time, events: &mut Queue<local::Event>) {
        host_io.store_request(local_protocol::StoreRequest::Load);
        host_io.markers_up(now, events);
        host_io.markers_up(now, events);
    }

    // Two completed loads must both fit the dispatch work bound, so only
    // the store-action gate can hold the second terminal within that pass.
    const ORDER_TERMINALS: u32 = 2;
    // Open/stat/read/close for the loads and adopted roots settle within this bound.
    const ORDER_STEPS: u32 = 64;

    struct FileOrderWorld {
        host_io: HostIo,
        sim: skein_sim::Sim,
        pid: skein_sim::Pid,
        machine: skein_fake_machine::Machine,
    }

    impl FileOrderWorld {
        fn new(seed: u64) -> Self {
            use skein_fake_machine::{How, Item, Machine};
            let mut machine = Machine::for_user(7);
            let root =
                machine.lay(&[Item::file(b"format", b"unreadable").mode(0o000), Item::file(b"source.rs", b"clean\n")]);
            let workspace = machine.open(root, b".", How::Directory).expect("workspace root");
            let mut sim = skein_sim::Sim::new(seed, skein_sim::Config::calm());
            let pid = sim.spawn_process();
            let store = sim.root(pid, skein_sim::Handle::new(root.raw()));
            let workspace = sim.root(pid, skein_sim::Handle::new(workspace.raw()));
            let mut host_io = host_io(ORDER_TERMINALS, KERNEL_QUEUE);
            host_io.adopt_store(store).expect("store root");
            host_io.adopt_delivery_roots(Box::new([workspace]), Box::new([]));
            Self { host_io, sim, pid, machine }
        }

        // Complete real host_io file IO while its parent holds terminal dispatch.
        fn kernel_step(&mut self) {
            let env = self.host_io.at(self.sim.now(), self.sim.wall());
            self.sim.reap(self.pid, self.host_io.completions());
            self.host_io.io_up(&env);
            self.host_io.down(
                &env,
                &mut Queue::uncapped(),
                &mut None,
                &Map::with_capacity(0),
                &mut Queue::uncapped(),
                &mut None,
            );
            self.sim.submit(self.pid, &mut self.host_io.submissions);
            skein_fake_machine::serve(&mut self.machine, &mut self.sim);
            if self.sim.ready(self.pid) == 0 {
                self.sim.advance();
            }
        }

        fn complete_files(&mut self) {
            for _ in 0..ORDER_STEPS {
                self.kernel_step();
                if self.host_io.file_events.len() == ORDER_TERMINALS {
                    return;
                }
            }
            panic!("seeded file operations did not complete");
        }

        fn close(&mut self, events: &mut Queue<local::Event>) {
            self.host_io.close();
            for _ in 0..ORDER_STEPS {
                self.host_io.markers_up(self.sim.now(), events);
                self.kernel_step();
                if self.host_io.closed() {
                    assert_eq!(self.machine.open_handles(), 0, "all adopted roots close");
                    return;
                }
            }
            panic!("seeded file owner did not close");
        }
    }

    /// Order: `store_actions_dispatched` (phase C).
    #[test]
    fn simulated_store_failure_reaches_chat_before_a_later_marker() {
        const SEED: u64 = 19;
        const MARKER_OWNER: Token = Token::new(7);
        let mut world = FileOrderWorld::new(SEED);
        let mut events = Queue::uncapped();
        start_store_load(&mut world.host_io, world.sim.now(), &mut events);
        world.host_io.start_markers(
            MARKER_OWNER,
            0,
            Box::new([Box::from(b"source.rs".as_slice())]),
            world.sim.now().saturating_add(skein_lib::Duration::from_secs(1)),
            &mut events,
        );
        world.complete_files();
        assert!(world.host_io.files.takes(), "both file operations have returned their credit");
        let mut pending = world.host_io.file_events.iter();
        assert_eq!(
            pending.next(),
            Some(&file::Event::Failed {
                owner: Token::new(u64::MAX - 1),
                error: kernel::Error::Permission,
                committed: false,
                residue: None,
            })
        );
        assert_eq!(
            pending.next(),
            Some(&file::Event::Loaded { owner: MARKER_OWNER, bytes: b"clean\n".as_slice().into() })
        );
        // The first pass consumes the failed load but must retain the marker
        // behind the actions that will publish the store's failure.
        world.host_io.markers_up(world.sim.now(), &mut events);
        assert!(events.is_empty(), "the marker cannot pass the undispatched store answer");
        assert!(!world.host_io.store_actions_dispatched());
        assert_eq!(world.host_io.file_events.len(), 1, "the gate retains the completed marker");
        world.host_io.markers_up(world.sim.now(), &mut events);
        let Some(local::Event::StoreFailed { turn: None, reason: local::StoreFailure::Read }) = events.pop() else {
            panic!("the chat must see the store's read failure first");
        };
        let Some(local::Event::Git { owner: MARKER_OWNER, result: local::GitResult::Markers { first: None } }) =
            events.pop()
        else {
            panic!("the chat must see the marker result after the store failure");
        };
        assert!(events.is_empty(), "both requests have exactly one chat terminal");
        assert!(world.host_io.file_events.is_empty());
        assert!(world.host_io.store_io.store_actions.is_empty());
        assert!(world.host_io.delivery.inspection.idle());
        world.close(&mut events);
        assert!(events.is_empty(), "closing adds no request terminal");
    }

    // One entry is the smallest routing queue: terminals are handed to the
    // shell one per pass. Two ready pipes exceed that estimate.
    const ROUTING_QUEUE: u32 = 1;
    const BURST: u32 = 2;
    const KERNEL_QUEUE: u32 = 1;

    fn host_io(queue: u32, sockets: u32) -> HostIo {
        let mut config = crate::tests::config();
        config.limits.process.queue = queue;
        config.limits.process.io.sockets = sockets;
        HostIo::new(&config.limits.process, config.launch, config.endpoints, 1).expect("host_io")
    }

    fn close_submission(number: u64) -> kernel::Submit {
        kernel::Submit { op: Token::new(number), kind: kernel::Op::Close { fd: kernel::Fd::new(90) } }
    }

    fn close_completion() -> kernel::Complete {
        let submit = close_submission(1);
        kernel::Complete { op: submit.op, kind: submit.kind, result: Ok(kernel::Done::Nothing) }
    }

    #[test]
    fn a_host_io_drains_more_io_terminals_than_its_queue_in_one_burst() {
        let mut host_io = host_io(ROUTING_QUEUE, BURST);
        let env = host_io.at(Time::ZERO, Wall::EPOCH);
        let mut expected = Queue::with_capacity(BURST);
        for number in 0..BURST {
            let owner = host_io
                .io
                .adopt_write_pipe(kernel::Fd::new(i32::try_from(number).expect("fixture descriptor")), None)
                .expect("one socket per pipe");
            io::down(
                &mut host_io.io,
                &env,
                io::Request::Stream {
                    stream: owner,
                    down: stream::Down::Demand { read: stream::Read::Nothing, room: 1 },
                },
                &mut host_io.io_submissions,
            );
            expected.push(io::Event::Stream { owner, up: stream::Up::Room });
        }
        host_io.io.reclaim();
        host_io.io_up(&env);
        assert_eq!(host_io.io_events.len(), BURST, "one metered IO pass exceeds the internal queue estimate");
        let config = crate::tests::config();
        let mut terminal = local_protocol::Terminal::new(local_protocol::TerminalLimits {
            line_bytes: config.limits.local.line_bytes,
            show_bytes: config.limits.local.show_bytes,
        })
        .expect("fixture terminal");
        let mut host_events = Queue::with_capacity(ROUTING_QUEUE);
        let mut terminal_events = Queue::uncapped();
        let mut local_events = Queue::with_capacity(ROUTING_QUEUE);
        for remaining in [1, 0] {
            host_io.up(&env, &mut host_events, &mut terminal, &mut terminal_events, &mut local_events, &mut None);
            assert_eq!(host_io.io_events.len(), remaining, "each drain keeps its queue work bound");
            for _ in 0..ROUTING_QUEUE {
                let Some(event) = host_io.shell_events.pop() else { break };
                assert_eq!(Some(event), expected.pop(), "every IO terminal reaches the shell in order");
            }
        }
        assert!(expected.is_empty(), "no terminal lost within the bounded iterations");
        assert!(host_io.io_events.is_empty());
        assert!(host_io.shell_events.is_empty());
    }

    fn requests_down(host_io: &mut HostIo) {
        let env = host_io.at(Time::ZERO, Wall::EPOCH);
        host_io.down(
            &env,
            &mut Queue::uncapped(),
            &mut None,
            &Map::with_capacity(0),
            &mut Queue::uncapped(),
            &mut None,
        );
    }

    fn internal_io_burst() -> HostIo {
        use skein_lib::Duration;
        use smith_host_protocol::{ProgramSource, SourceAsk};
        use smith_protocol_program::Programs;

        // Two routing entries, four occupied pipes, one refusal: the check,
        // stderr read, window and channel opening exceed the queue together.
        const QUEUE: u32 = 2;
        const PIPES: u32 = 4;
        let mut host_io = host_io(QUEUE, PIPES);
        let env = host_io.at(Time::ZERO, Wall::EPOCH);
        let input = host_io.io.adopt_write_pipe(kernel::Fd::new(10), None).unwrap();
        let output = host_io.io.adopt_read_pipe(kernel::Fd::new(11)).unwrap();
        let error = host_io.io.adopt_read_pipe(kernel::Fd::new(12)).unwrap();
        let terminal = host_io.io.adopt_read_pipe(kernel::Fd::new(13)).unwrap();
        // Adopted read pipes prefetch independently of their producer demands.
        host_io.io.reclaim();
        host_io.io_up(&env);
        while host_io.io_submissions.pop().is_some() {}
        host_io.terminal_input.adopted(terminal, false, &mut host_io.io_requests);
        host_io.read_window();
        // Filling IO's one refusal slot holds its credit independently
        // of the service handoff. No kernel operation is required to release it.
        io::down(
            &mut host_io.io,
            &env,
            io::Request::Connect {
                owner: Token::new(90),
                addr: io::Addr::Ip(core::net::SocketAddr::from(([127, 0, 0, 1], 1_u16))),
            },
            &mut host_io.io_submissions,
        );
        let _check = ProgramSource::new(
            Token::new(91),
            SourceAsk::Check,
            &Programs { directory: b"/programs".as_slice().into(), claude: b"/claude".as_slice().into() },
            host::SourceName::new(b"work").unwrap(),
            &[],
            kernel::Fd::new(3),
            b".".as_slice().into(),
            Time::ZERO.saturating_add(Duration::from_secs(60)),
            host_io.limits.detail_bytes,
            &mut host_io.io_requests,
        )
        .unwrap();
        let mut process = protocol::Process::new(
            Token::new(7),
            &host_io.limits.channel,
            host_io.limits.detail_bytes,
            host_io.endpoints.clone(),
        )
        .unwrap();
        process.spawn(
            host_io.launch.clone(),
            Time::ZERO.saturating_add(Duration::from_secs(60)),
            &mut Queue::uncapped(),
        );
        process.from_io(
            Time::ZERO,
            io::Event::Spawned {
                owner: process.io_owner(),
                child: Token::new(8),
                pipes: Box::new([input, output, error]),
            },
            &mut host_io.process_events,
            &mut host_io.io_requests,
        );
        host_io.child.spawned(process);
        host_io
    }

    #[test]
    fn a_burst_of_internal_io_requests_past_the_queue_goes_down_in_order() {
        // The check spawn, stderr demand, window, and two channel requests.
        const BURST_REQUESTS: u32 = 5;
        // Two refusals each emit Failed/Closed; the channel gets one grant.
        const IO_TERMINALS: u32 = 5;
        let mut host_io = internal_io_burst();
        let env = host_io.at(Time::ZERO, Wall::EPOCH);
        requests_down(&mut host_io);
        assert!(!host_io.io.takes(), "only IO refusal credit holds the burst");
        let mut burst = List::with_capacity(BURST_REQUESTS);
        for request in &host_io.io_requests {
            burst.push(request).unwrap();
        }
        let [
            io::Request::Spawn { owner, .. },
            io::Request::Stream { down: stream::Down::Demand { read: stream::Read::Arrived { .. }, room: 0 }, .. },
            io::Request::Window { .. },
            io::Request::Stream { down: stream::Down::Demand { read: stream::Read::Fill(1), room: 0 }, .. },
            io::Request::Output { down: stream::OutputDown::Room { .. }, .. },
        ] = burst.as_slice()
        else {
            panic!("all three producers enqueue in FIFO order: {burst:?}");
        };
        assert_eq!(*owner, Token::new(91), "the check starts the burst");
        assert!(host_io.io_submissions.is_empty(), "no request crosses exhausted IO credit");

        // The full entity table also refuses the check. One pass takes that
        // spawn, the next stderr and window, and the last both channel requests.
        for remaining in [4, 2, 0] {
            host_io.io_up(&env);
            requests_down(&mut host_io);
            assert_eq!(host_io.io_requests.len(), remaining, "IO credit and the work bound alone pace FIFO drains");
        }
        host_io.io_up(&env);
        let Some(kernel::Submit { kind: kernel::Op::Window { fd }, .. }) = host_io.submissions.pop() else {
            panic!("the window request reaches IO after the check refusal");
        };
        assert_eq!(fd, kernel::Fd::new(13));
        assert!(host_io.submissions.is_empty(), "only the window needs a new kernel operation");
        let mut events = List::with_capacity(IO_TERMINALS);
        for event in &host_io.io_events {
            events.push(event).unwrap();
        }
        let [
            io::Event::Failed { owner: blocker, .. },
            io::Event::Closed { owner: blocker_closed },
            io::Event::Failed { owner: check, .. },
            io::Event::Closed { owner: check_closed },
            io::Event::Output { up: stream::OutputUp::Settled { outcome: stream::OutputOutcome::Granted, .. }, .. },
        ] = events.as_slice()
        else {
            panic!("IO answers both refusals before granting the channel output: {events:?}");
        };
        assert_eq!(
            [*blocker, *blocker_closed, *check, *check_closed],
            [Token::new(90), Token::new(90), Token::new(91), Token::new(91)]
        );
    }

    #[test]
    fn the_kernel_edge_stays_capped_at_its_queue() {
        let mut host_io = host_io(KERNEL_QUEUE, KERNEL_QUEUE);
        host_io.submissions.push(close_submission(10));
        host_io.io_submissions.push(close_submission(11));
        host_io.io_submissions.push(close_submission(12));
        host_io.file_submissions.push(close_submission(file_layer::LANE | 0x0d));
        host_io.flush_submissions();
        assert_eq!(host_io.submissions.capacity(), KERNEL_QUEUE);
        assert_eq!(host_io.completions.capacity(), KERNEL_QUEUE);
        assert_eq!(host_io.submissions.len(), KERNEL_QUEUE);
        assert_eq!(host_io.io_submissions.len(), 2, "full edge holds IO");
        assert_eq!(host_io.file_submissions.len(), 1, "full edge holds files");
        assert_eq!(host_io.in_flight, 0, "no operation crosses the full kernel edge");
        host_io.submissions.pop().expect("shell drains the occupied edge");
        for original in [11, 12, 13] {
            host_io.flush_submissions();
            assert_eq!(host_io.submissions.len(), KERNEL_QUEUE, "only one submission crosses returned credit");
            let submit = host_io.submissions.pop().expect("shell takes the routed submission");
            host_io.flush_submissions();
            assert!(host_io.submissions.is_empty(), "the kernel edge also waits for completion credit");
            // Model completion credit without sending synthetic operations to the machines.
            host_io.in_flight = host_io.in_flight.checked_sub(1).unwrap();
            if file_layer::owns(submit.op) {
                assert_eq!(original, 13, "files follow both IO submissions");
                assert_eq!(submit.op, Token::new(file_layer::LANE | original));
            } else {
                assert!(original == 11 || original == 12, "IO retains priority over files");
                assert_eq!(submit.op, Token::new(original));
            }
        }
        assert!(host_io.io_submissions.is_empty());
        assert!(host_io.file_submissions.is_empty());
        assert_eq!(host_io.in_flight, 0);
    }

    #[test]
    fn a_staged_io_completion_keeps_work_pending() {
        let mut host_io = host_io(KERNEL_QUEUE, KERNEL_QUEUE);
        host_io.io_completions.push(close_completion());
        assert!(host_io.work_pending(Time::ZERO), "a staged IO completion needs another up pass");
    }

    #[test]
    fn the_process_backlog_counts_only_internal_handoffs() {
        let mut host_io = host_io(KERNEL_QUEUE, KERNEL_QUEUE);
        host_io.submissions.push(close_submission(1));
        host_io.completions.push(close_completion());
        host_io.shell_events.push(io::Event::Resized);
        host_io.in_flight = 1;
        assert_eq!(host_io.backlog(), 0, "kernel and shell edges are outside the backlog");
        host_io.io_events.push(io::Event::Resized);
        host_io.io_submissions.push(close_submission(2));
        host_io.file_requests.push((file::Request::Close { owner: Token::new(1), file: Token::new(2) }, Time::ZERO));
        host_io.file_events.push(file::Event::Closed { owner: Token::new(1) });
        host_io.file_submissions.push(close_submission(3));
        host_io.store_io.store_requests.push(local_protocol::StoreRequest::Load);
        host_io.store_io.store_actions.push(local_protocol::Action::Closed);
        host_io.process_events.push(protocol::ProcessEvent::Exited { agent: Token::new(1), exit: None });
        host_io.io_requests.push(io::Request::Close { entity: Token::new(1) });
        // Each of the nine internal handoffs holds one entry.
        assert_eq!(host_io.backlog(), 9);
    }

    fn prefetched_host_io(owner: Token) -> HostIo {
        let config = crate::tests::config();
        let mut host_io = HostIo::new(&config.limits.process, config.launch, config.endpoints, 1).expect("host_io");
        host_io.child.spawned(
            protocol::Process::new(
                owner,
                &host_io.limits.channel,
                host_io.limits.detail_bytes,
                host_io.endpoints.clone(),
            )
            .unwrap(),
        );
        host_io
            .process_events
            .push(protocol::ProcessEvent::Channel { agent: owner, event: Box::new(protocol::Received::WriteFailed) });
        host_io.process_events.push(protocol::ProcessEvent::Channel {
            agent: owner,
            event: Box::new(protocol::Received::Message {
                message: smith_host_domain::Up::Fact { body: b"first".as_slice().into() },
            }),
        });
        host_io.process_events.push(protocol::ProcessEvent::Exited { agent: owner, exit: None });
        host_io.process_events.push(protocol::ProcessEvent::Channel {
            agent: owner,
            event: Box::new(protocol::Received::Message {
                message: smith_host_domain::Up::Fact { body: b"second".as_slice().into() },
            }),
        });
        host_io
    }

    fn take_exit(events: &mut Queue<host::Input>) {
        match events.pop() {
            Some(host::Input::Process(host::process::Event::Exited { .. })) => {}
            other @ (Some(host::Input::Process(_) | host::Input::Parent(_)) | None) => {
                panic!("exit terminal: {other:?}")
            }
        }
    }

    #[test]
    fn a_write_failure_does_not_spend_the_hosts_pending_read() {
        let owner = Token::new(7);
        let mut host_io = prefetched_host_io(owner);
        let mut events = Queue::with_capacity(1);
        host_io.drain_process(&mut events);
        take_exit(&mut events);
        assert!(events.is_empty(), "no record without a held Read");
        assert!(!host_io.work_pending(Time::ZERO), "prefetched records wait without spinning");
        for text in [b"first".as_slice(), b"second".as_slice()] {
            // This fixture tests the outer terminal ledger with already queued events,
            // below request forwarding: a real Read requires an opened Process.
            host_io.child.hold();
            host_io.drain_process(&mut events);
            match events.pop().expect("one read terminal") {
                host::Input::Process(host::process::Event::Received {
                    owner: seen,
                    message: host::Up::Fact { body },
                }) => {
                    assert_eq!(seen, owner);
                    assert_eq!(&*body, text);
                }
                other @ (host::Input::Parent(_) | host::Input::Process(_)) => panic!("fact terminal: {other:?}"),
            }
            assert!(events.is_empty(), "one terminal per Read");
        }
        assert!(host_io.process_events.is_empty());
        host_io.child.hold();
        host_io
            .process_events
            .push(protocol::ProcessEvent::Channel { agent: owner, event: Box::new(protocol::Received::WriteFailed) });
        host_io.drain_process(&mut events);
        assert!(host_io.child.read_held(), "an independent write failure leaves the read owed");
        assert!(events.is_empty());
    }
}
