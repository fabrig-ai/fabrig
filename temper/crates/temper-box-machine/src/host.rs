//! The machine on skein-io, with one bounded chunk per child pipe.
//! Kind files and removers follow temper/docs/design/protocol/setup/box.md, 1.
//! Source outcomes follow temper/docs/design/protocol/openid-provider.md, 9.
use crate::{
    layout::{self, Placed},
    limits::{Limits, Unfit},
    lines::{self, Lines, Output},
    logs::{Logs, Piece},
    machine::{Input, Machine, Order, Outcome, Relay, Said},
    pipe::{self, Command, Line, Reader},
    ready::Ready,
    records::Records,
    sink::{Sink, Terminal},
    terminal::{self, Fact, Formatter, Item, Place, Tail, Target},
};
mod front_child;
use alloc::boxed::Box;
use skein_io::{
    self as io, Event, Io, Request,
    kernel::{self, Complete, Exit, Fd, ServiceSignal, Submit},
};
use skein_lib::{
    Decimal, Duration, Env, List, Queue, Secret, Time, Token, Wall,
    stream::{Down, Read, Up},
};
use temper_forge_setup::{Kind, Stage, Step, Steps, fetch::Fetched, sign_in::Source};
/// Which diagnostic lines also reach the terminal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Echo {
    None,
    Steps,
}
/// The first link drawn by the box (protocol/setup/box.md, 2). Debug hides its credentials and URL.
pub struct FirstLink {
    pub url: Box<[u8]>,
    pub secret: Secret,
    pub local: Option<Secret>,
}
impl core::fmt::Debug for FirstLink {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FirstLink").finish_non_exhaustive()
    }
}
/// The box's TUI child (temper/docs/design/protocol/setup/box.md, 6).
pub struct Front {
    pub arguments: Box<[Box<[u8]>]>,
    pub link: Secret,
}
impl core::fmt::Debug for Front {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Front").field("arguments", &self.arguments).finish_non_exhaustive()
    }
}
/// The descriptors and command inherited from the binary.
pub struct Settings {
    pub program: Box<[u8]>,
    /// The box's own id where steps start through `internal forge-child` (protocol/setup/box.md, 1).
    pub parent: Option<u32>,
    pub environment: Box<[Box<[u8]>]>,
    pub serve: Box<[u8]>,
    pub listeners: Box<[(Fd, core::net::SocketAddr)]>,
    pub root: Fd,
    pub echo: Echo,
    pub data: Box<[u8]>,
    /// Whole lines, with line feeds and no time, that open the box log (protocol/setup/box.md, 2).
    pub log_lines: Box<[Box<[u8]>]>,
    pub first_link: Option<FirstLink>,
    pub front: Option<Front>,
    /// How the first and fresh links reach stdout (temper/docs/design/protocol/setup/box.md, 2).
    pub link_form: terminal::LinkForm,
    /// The word running this binary (temper/docs/design/protocol/setup/box.md, 2).
    pub word: Box<[u8]>,
    /// Whole untimed lines following the first block on stderr (protocol/setup/box.md, 2).
    pub after_block: Box<[u8]>,
    pub signals: Fd,
    pub stdout: Fd,
    pub stderr: Fd,
}
// Environment entries may contain credentials; Debug shows only their count (skein/docs/design/runtime/lib.md, 6).
impl core::fmt::Debug for Settings {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Settings")
            .field("program", &self.program)
            .field("parent", &self.parent)
            .field("environment_entries", &self.environment.len())
            .field("serve", &self.serve)
            .field("listeners", &self.listeners)
            .field("root", &self.root)
            .field("echo", &self.echo)
            .field("data", &self.data)
            .field("log_lines", &self.log_lines)
            .field("first_link", &self.first_link)
            .field("front", &self.front)
            .field("link_form", &self.link_form)
            .field("word", &self.word)
            .field("after_block_bytes", &self.after_block.len())
            .field("signals", &self.signals)
            .field("stdout", &self.stdout)
            .field("stderr", &self.stderr)
            .finish()
    }
}
/// Serve URL payload cap (LF excluded); diagnostic pieces share the read cap.
pub const LINE_BYTES: u32 = 4096;
#[derive(Debug)]
struct Pipe {
    token: Option<Token>,
    reading: bool,
    lines: Lines,
}
impl Pipe {
    fn new(plain: bool) -> Self {
        Self { token: None, reading: false, lines: Lines::new(plain) }
    }
    fn settled(&self) -> bool {
        self.token.is_none() && self.lines.is_empty()
    }
}
#[derive(Clone, Copy, Debug)]
enum SourceEnd {
    Exited(Exit),
    NotStarted,
}
#[derive(Debug)]
struct Child {
    owner: Option<Token>,
    process: Option<Token>,
    pipes: [Pipe; 2],
    spawn: bool,
    withdrawn: Option<Wall>,
    hold: bool,
    line: Option<Box<[u8]>>,
    stdin: Option<Sink>,
    secret: Option<Secret>,
    ready: Option<Ready>,
    ready_wall: Wall,
    tail: Tail,
    source: Option<Source>,
    source_ending: Option<SourceEnd>,
    source_said: Option<Source>,
    restarting: bool,
}
impl Child {
    fn new(serve: &[u8], step: &Step, tail_lines: u32) -> Self {
        Self {
            owner: None,
            process: None,
            pipes: [Pipe::new(lines::route(serve, step.name(), 1)), Pipe::new(false)],
            spawn: false,
            withdrawn: None,
            hold: false,
            line: None,
            stdin: None,
            secret: None,
            ready_wall: Wall::EPOCH,
            tail: Tail::new(tail_lines),
            source: None,
            source_ending: None,
            source_said: None,
            restarting: false,
            ready: match step.ready() {
                Some(address) => Some(Ready::new(address)),
                None => None,
            },
        }
    }
    fn ready_settled(&self) -> bool {
        match &self.ready {
            Some(ready) => ready.settled(),
            None => true,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    New,
    Block,
    Running,
    Closing,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Root {
    Open,
    Closing,
    Closed,
}
/// EOF requests Close; only Closed settles the reader's lifetime.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ReadPhase {
    Open,
    Ended,
    Closed,
}
#[derive(Debug)]
struct Control {
    read: Token,
    sink: Sink,
    fetched: Option<Box<[u8]>>,
    reader: Reader,
    reading: bool,
    read_phase: ReadPhase,
    line: Option<Line>,
    line_wall: Wall,
}
/// All io and policy state owned by one box process.
#[derive(Debug)]
struct Told {
    event: Event,
    wall: Wall,
}

#[derive(Debug)]
pub struct Service {
    io: Io,
    env: Env<io::Limits>,
    machine: Machine,
    steps: Box<[Placed]>,
    templates: Box<[Placed]>,
    bound_names: Box<[Option<u16>]>,
    records: Records,
    logs: Logs,
    box_lines: Queue<Item>,
    box_echo: bool,
    started_notices: Queue<(usize, Wall)>,
    record_wall: Wall,
    control: Option<Control>,
    closing_control: [Option<Token>; 2],
    line_bytes: u32,
    reply_entries: u32,
    settings: Settings,
    children: Box<[Child]>,
    front: Option<front_child::Child>,
    terminal: Terminal,
    formatter: Formatter,
    terminate_pending: bool,
    failure: Option<(Fact, Wall)>,
    deferred: Queue<Item>,
    listeners: Box<[(core::net::SocketAddr, Token)]>,
    listeners_closing: usize,
    signals: Token,
    phase: Phase,
    failed: bool,
    serial: u64,
    root: Root,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    events: Queue<Event>,
    told: Queue<Told>,
    requests: Queue<Request>,
    orders: Queue<Order>,
    order_walls: Queue<Wall>,
}
impl Service {
    /// Checks the policy limits and adopts the binary's descriptors.
    #[expect(
        clippy::too_many_lines,
        reason = "Adopt the box's descriptors and account for its front alongside the step slots."
    )]
    pub fn new(limits: &Limits, steps: Steps, settings: Settings) -> Result<Self, Unfit> {
        if settings.front.is_some() {
            let local = match &settings.first_link {
                Some(link) => link.local.is_some(),
                None => false,
            };
            assert!(local, "a front has a first local link");
        }
        let front = if settings.front.is_some() {
            Some(front_child::Child::new(limits.grace, limits.tail_lines))
        } else {
            None
        };
        let reply_entries = reply_entries(&steps);
        let mut steps = layout::flatten(steps);
        let templates = steps.clone();
        for placed in &mut steps {
            if let Some(asked) = placed.asked
                && asked.kind.is_some()
            {
                placed.step = layout::bound(&placed.step, u16::MAX);
                if let Some(label) = &mut placed.label {
                    *label = layout::bound_label(label, u16::MAX);
                }
            }
        }
        let machine = Machine::new(limits, &steps)?;
        let records = Records::new(&steps, limits.groups);
        let logs = Logs::new(&steps, &settings.data, limits.log_bytes, front.is_some());
        let count = u32::try_from(steps.len()).expect("steps fit u32");
        let mut bounds = io_limits(count);
        bounds.sockets = bounds
            .sockets
            .checked_add(u32::try_from(settings.listeners.len()).expect("bounded listeners"))
            .expect("listener slots");
        if front.is_some() {
            bounds.sockets = bounds.sockets.checked_add(5).expect("front process and pipes");
        }
        bounds.intake = bounds.intake.max(limits.line_bytes);
        bounds.output = bounds
            .output
            .max(limits.line_bytes.checked_add(10).expect("notice timestamp"))
            .max(terminal::max_bytes(&steps))
            .max(Logs::max_notice(&steps, &settings.data))
            .max(terminal::failure_bytes(&steps, &settings.data))
            .max(terminal::LINK_LINE_BYTES)
            .max(u32::try_from(settings.after_block.len()).expect("startup lines"));
        let mut children = List::with_capacity(count);
        let mut bound_names = List::with_capacity(count);
        for placed in &steps {
            bound_names.push(None).expect("one binding per step");
            let step = &placed.step;
            bounds.output = bounds.output.max(
                LINE_BYTES
                    .checked_add(u32::try_from(step.name().len()).expect("bounded name"))
                    .expect("bounded line")
                    .checked_add(3)
                    .expect("prefix"),
            );
            children.push(Child::new(&settings.serve, step, limits.tail_lines)).expect("one child per step");
        }
        let mut io = Io::new(&bounds);
        let signals = io.adopt_signals(settings.signals).expect("three adopted slots");
        let stdout = io.adopt_write_pipe(settings.stdout, None).expect("stdout slot");
        let stderr = io.adopt_write_pipe(settings.stderr, None).expect("stderr slot");
        let listeners = adopt_listeners(&mut io, &settings.listeners);
        let capacity =
            io::operations(&bounds).expect("bounded operations").checked_mul(4).expect("queue capacity").max(64);
        let entries = count.checked_mul(2).expect("terminal entries").checked_add(1).expect("terminal entries").max(4);
        let formatter = Formatter::new(settings.link_form, settings.word.clone());
        // A failed front's pipes drain before its report; their fallback lines wait behind it.
        let deferred = if front.is_some() { Queue::uncapped() } else { Queue::with_capacity(entries) };
        Ok(Self {
            io,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: bounds },
            machine,
            bound_names: bound_names.into_boxed(),
            templates,
            steps,
            records,
            logs,
            box_lines: Queue::uncapped(),
            box_echo: false,
            started_notices: Queue::uncapped(),
            record_wall: Wall::EPOCH,
            control: None,
            closing_control: [None, None],
            line_bytes: limits.line_bytes,
            reply_entries,
            settings,
            children: children.into_boxed(),
            front,
            terminal: Terminal::new(stdout, stderr, entries),
            formatter,
            terminate_pending: false,
            failure: None,
            deferred,
            listeners: listeners.into_boxed(),
            listeners_closing: 0,
            signals,
            phase: Phase::New,
            failed: false,
            serial: 1_u64 << 63,
            root: Root::Open,
            completions: Queue::with_capacity(capacity),
            submissions: Queue::with_capacity(capacity),
            events: Queue::uncapped(),
            told: Queue::uncapped(),
            requests: Queue::with_capacity(capacity),
            orders: Queue::uncapped(),
            order_walls: Queue::uncapped(),
        })
    }
    fn progress_event(&self, event: &Event) -> bool {
        match event {
            Event::Stream { owner, .. } | Event::Closed { owner } => self.terminal.owns(*owner),
            Event::Shutdown { .. }
            | Event::Spawned { .. }
            | Event::Exited { .. }
            | Event::Failed { .. }
            | Event::Listening { .. }
            | Event::Accepted { .. }
            | Event::Connecting { .. }
            | Event::Connected { .. }
            | Event::Output { .. }
            | Event::Usage { .. }
            | Event::Resized
            | Event::Window { .. } => false,
        }
    }
    /// Ordinary events keep FIFO while leaving two request slots for terminal
    /// acknowledgements to progress past a blocked ordinary front.
    fn ordinary_events_ready(&self) -> bool {
        self.requests.room() >= 6
    }
    /// A command follows the dispatch of every order from earlier policy inputs.
    fn orders_drained(&self) -> bool {
        self.orders.is_empty()
    }
    /// The record diagnostic enters the notice FIFO before its policy answer.
    fn record_notice_admitted(&self) -> bool {
        !self.records.has_notice()
    }
    /// Admitted ordinary events precede policy timers, including due retries.
    fn policy_timers_ready(&self) -> bool {
        self.told.is_empty()
    }
    fn events_runnable(&self) -> bool {
        if self.ordinary_events_ready() && !self.told.is_empty() {
            return true;
        }
        if self.requests.room() >= 2 {
            for told in &self.told {
                if self.progress_event(&told.event) {
                    return true;
                }
            }
        }
        false
    }
    fn order_ready(&self) -> bool {
        let forge = match self.orders.iter().next() {
            Some(
                Order::Say { said: Said::Started { forge } } | Order::Terminal { fact: Fact::GroupStarted { forge } },
            ) => Some(*forge),
            Some(Order::Source { step, .. }) => {
                Some(self.steps.get(*step).expect("source step").asked.expect("source group").forge)
            }
            Some(_) | None => None,
        };
        if let Some(forge) = forge
            && let Some(step) = self.source_step(forge)
        {
            for pipe in &self.children.get(step).expect("source child").pipes {
                if !pipe.settled() {
                    return false;
                }
            }
        }
        match self.orders.iter().next() {
            Some(Order::Terminal { fact: Fact::Starting { step } })
                if self.steps.get(*step).expect("step").asked.is_some() =>
            {
                true
            }
            Some(Order::Terminal { fact }) if fact.place() == Place::Log => !self.bindings_pending(),
            Some(Order::Terminal { fact }) => {
                self.failure.is_none()
                    && !self.bindings_pending()
                    && (self.front_running() || self.deferred.is_empty())
                    && (self.tail_candidate(*fact).is_some() || self.say_room(1))
            }
            Some(Order::Say { .. } | Order::Source { .. }) => self.reply_ready(),
            Some(
                Order::Spawn { .. }
                | Order::Withdraw { .. }
                | Order::Hold { .. }
                | Order::Wait { .. }
                | Order::Signal { .. }
                | Order::Close { .. }
                | Order::RunRemover { .. },
            ) => true,
            Some(Order::Record { .. } | Order::WriteGroup { .. } | Order::Unlink { .. }) => self.records.ready(),
            None => false,
        }
    }
    fn reply_ready(&self) -> bool {
        match &self.control {
            Some(control) => !control.sink.room() || control.sink.takes(),
            None => true,
        }
    }
    fn input(&mut self, input: Input) {
        policy_input(&mut self.machine, &self.env, input, &mut self.orders, &mut self.order_walls);
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Dispatch each io event to the terminal, front, control link or step that owns it."
    )]
    fn event(&mut self, event: Event) {
        for state in &mut self.children {
            if let Some(ready) = &mut state.ready
                && ready.owns(&event)
            {
                state.ready_wall = self.env.wall;
                ready.event(event, self.env.now, &mut self.requests);
                return;
            }
        }
        match event {
            Event::Shutdown { signal, .. } => match signal {
                ServiceSignal::Hangup if self.machine.outcome().is_some() => {}
                ServiceSignal::Interrupt | ServiceSignal::Terminate | ServiceSignal::Hangup => {
                    self.terminate();
                }
            },
            Event::Spawned { owner, child, pipes } => {
                self.spawned(owner, child, &pipes);
            }
            Event::Exited { owner, exit } => {
                if let Some(front) = &self.front
                    && front.owner == Some(owner)
                {
                    self.front_end(Some(exit));
                    return;
                }
                for (step, state) in self.children.iter_mut().enumerate() {
                    if state.owner == Some(owner) {
                        state.source_ending = Some(SourceEnd::Exited(exit));
                        if self.steps.get(step).expect("step").step.name() == self.settings.serve.as_ref() {
                            self.retire_control();
                        }
                        policy_input(
                            &mut self.machine,
                            &self.env,
                            Input::Exited { step, exit },
                            &mut self.orders,
                            &mut self.order_walls,
                        );
                        break;
                    }
                }
            }
            Event::Closed { owner } => {
                if let Some(front) = &mut self.front {
                    let not_started = front.owner == Some(owner) && front.process.is_none() && !front.ended;
                    front.closed(owner, self.env.wall);
                    if not_started {
                        self.front_end(None);
                    }
                }
                for token in &mut self.closing_control {
                    if *token == Some(owner) {
                        *token = None;
                    }
                }
                if let Some(control) = &mut self.control {
                    if control.sink.token == owner {
                        control.sink.closed();
                    }
                    if control.read == owner {
                        control.read_phase = ReadPhase::Closed;
                        control.reading = false;
                    }
                }
                if self.terminal.owns(owner) && self.terminal.closed(owner) {
                    self.failed = true;
                    self.terminate_pending = true;
                }
                for (step, state) in self.children.iter_mut().enumerate() {
                    if let Some(stdin) = &state.stdin
                        && stdin.token == owner
                    {
                        state.stdin = None;
                    }
                    if state.owner == Some(owner) {
                        let input = if state.process.take().is_some() {
                            Input::Closed { step }
                        } else {
                            Input::NotStarted { step }
                        };
                        if input == (Input::NotStarted { step }) {
                            state.source_ending = Some(SourceEnd::NotStarted);
                        }
                        state.owner = None;
                        state.secret = None;
                        policy_input(&mut self.machine, &self.env, input, &mut self.orders, &mut self.order_walls);
                    }
                    for pipe in &mut state.pipes {
                        if pipe.token == Some(owner) {
                            pipe.token = None;
                            pipe.reading = false;
                            pipe.lines.end(self.env.wall);
                        }
                    }
                }
            }
            Event::Stream { owner, up } => self.stream(owner, up),
            Event::Failed { owner, .. } => {
                if owner == self.signals {
                    self.failed = true;
                    self.input(Input::Terminate);
                }
            }
            Event::Listening { .. }
            | Event::Accepted { .. }
            | Event::Connecting { .. }
            | Event::Connected { .. }
            | Event::Output { .. }
            | Event::Usage { .. }
            | Event::Resized
            | Event::Window { .. } => {}
        }
    }
    fn retire_control(&mut self) {
        let Some(control) = self.control.take() else {
            return;
        };
        // Either pipe may have reported Closed before Exited. Carry only
        // outstanding Closed obligations into the replacement's spawn gate.
        let read = match control.read_phase {
            ReadPhase::Open => {
                self.requests.push(Request::Close { entity: control.read });
                Some(control.read)
            }
            ReadPhase::Ended => Some(control.read),
            ReadPhase::Closed => None,
        };
        let reply = if control.sink.is_closed() {
            None
        } else {
            self.requests.push(Request::Close { entity: control.sink.token });
            Some(control.sink.token)
        };
        self.closing_control = [read, reply];
    }
    fn spawned(&mut self, owner: Token, child: Token, pipes: &[Token]) {
        if let Some(front) = &mut self.front
            && front.owner == Some(owner)
        {
            front.spawned(child, pipes, self.env.now, &mut self.requests);
            let url = self.settings.first_link.as_ref().expect("front's first link").url.clone();
            self.front_line(crate::front::Line::Link { url });
            return;
        }
        let mut running = None;
        for (step, state) in self.children.iter_mut().enumerate() {
            if state.owner == Some(owner) {
                state.process = Some(child);
                if state.restarting {
                    state.restarting = false;
                    running =
                        Some(skein_lib::bytes::copy_of(self.steps.get(step).expect("restarted step").step.name()));
                }
                if self.steps.get(step).expect("step").step.kind() == Kind::Link {
                    let mut sink = Sink::new(*pipes.get(2).expect("link stdin"), 1);
                    sink.push(state.line.take().expect("link's first line")).expect("one entry");
                    state.stdin = Some(sink);
                }
                for (pipe, token) in state.pipes.iter_mut().zip(pipes) {
                    pipe.token = Some(*token);
                }
                if self.steps.get(step).expect("step").step.name() == self.settings.serve.as_ref() {
                    if let Some(secret) = state.secret.take() {
                        let mut sink = Sink::new(*pipes.get(4).expect("serve stdin"), 1);
                        sink.push(secret.into_exposed()).expect("one secret entry");
                        state.stdin = Some(sink);
                    }
                    self.control = Some(Control {
                        read: *pipes.get(2).expect("serve requests pipe"),
                        sink: Sink::new(*pipes.get(3).expect("serve answers pipe"), self.reply_entries),
                        fetched: None,
                        reader: Reader::new(self.line_bytes),
                        reading: false,
                        read_phase: ReadPhase::Open,
                        line: None,
                        line_wall: Wall::EPOCH,
                    });
                }
                policy_input(
                    &mut self.machine,
                    &self.env,
                    Input::Started { step },
                    &mut self.orders,
                    &mut self.order_walls,
                );
                break;
            }
        }
        if let Some(step) = running {
            self.front_line(crate::front::Line::Running { step });
        }
    }
    fn stream(&mut self, owner: Token, up: Up) {
        if let Some(front) = &mut self.front
            && front.owns_stream(owner)
        {
            front.stream(owner, up, self.env.wall, &mut self.requests);
            return;
        }
        if let Some(control) = &mut self.control {
            if control.sink.token == owner {
                control.sink.up(up, &mut self.requests);
                return;
            }
            if control.read == owner {
                control.reading = false;
                match up {
                    Up::Bytes(bytes) => {
                        assert_eq!(bytes.len(), 1, "one byte demand");
                        assert!(control.line.is_none(), "one pending control line");
                        control.line = control.reader.byte(*bytes.first().expect("one byte"));
                        if control.line.is_some() {
                            control.line_wall = self.env.wall;
                        }
                    }
                    Up::End | Up::Failed(_) => {
                        control.read_phase = ReadPhase::Ended;
                        self.requests.push(Request::Close { entity: owner });
                    }
                    Up::Room => {}
                }
                return;
            }
        }
        if self.terminal.owns(owner) {
            if self.terminal.up(owner, up, self.env.limits.output, &mut self.requests) {
                self.failed = true;
                self.terminate_pending = true;
            }
            return;
        }
        for (step, state) in self.children.iter_mut().enumerate() {
            if let Some(stdin) = &mut state.stdin
                && stdin.token == owner
            {
                // Only the link's exit decides success, even if it did not read.
                stdin.up(up, &mut self.requests);
                return;
            }
            for (fd, pipe) in state.pipes.iter_mut().enumerate() {
                if pipe.token == Some(owner) {
                    pipe.reading = false;
                    match up {
                        Up::Bytes(bytes) => pipe.lines.push(bytes, self.env.wall),
                        Up::End | Up::Failed(_) => {
                            pipe.lines.end(self.env.wall);
                            self.requests.push(Request::Close { entity: owner });
                        }
                        Up::Room => {}
                    }
                    if fd == 0
                        && let Some(line) = pipe.lines.first()
                        && self.machine.outcome().is_none()
                    {
                        state.line = Some(line);
                        policy_input(
                            &mut self.machine,
                            &self.env,
                            Input::Line { step },
                            &mut self.orders,
                            &mut self.order_walls,
                        );
                    }
                    return;
                }
            }
        }
    }
    fn started_notice(&self, fact: Fact) -> Option<usize> {
        let forge = match fact {
            Fact::GroupStarted { forge } => forge,
            Fact::Starting { .. }
            | Fact::GroupStarting { .. }
            | Fact::GroupFailed { .. }
            | Fact::GroupStopped { .. }
            | Fact::Restarting { .. }
            | Fact::GivingUp { .. }
            | Fact::Stopping
            | Fact::Stopped => return None,
        };
        for (step, placed) in self.steps.iter().enumerate() {
            if let Some(asked) = placed.asked
                && asked.forge == forge
                && !placed.step.started_notice().is_empty()
            {
                return Some(step);
            }
        }
        None
    }
    fn terminal_fact(&mut self, fact: Fact, wall: Wall) {
        if let Some(line) = crate::front::state(fact, &self.steps) {
            self.front_line(line);
        }
        if let Fact::Restarting { step, .. } = fact {
            self.children.get_mut(step).expect("restarting step").restarting = true;
        }
        if let Some(step) = self.started_notice(fact) {
            self.started_notices.push((step, wall));
        }
        if let Fact::Starting { step } = fact
            && self.steps.get(step).expect("step fact").asked.is_some()
        {
            return;
        }
        if fact.place() == Place::Log {
            self.box_lines.push(self.formatter.fact(fact, &self.steps, wall));
        } else if self.tail_candidate(fact).is_some() && (self.front_running() || !self.terminal.failed()) {
            assert!(self.failure.is_none(), "one waiting failure block");
            self.failure = Some((fact, wall));
        } else {
            self.say(self.formatter.fact(fact, &self.steps, wall));
        }
    }
    fn tail_candidate(&self, fact: Fact) -> Option<usize> {
        let step = terminal::tail_step(fact)?;
        if self.settings.echo == Echo::None && !self.logs.failed(step) { Some(step) } else { None }
    }
    fn failure_ready(&self) -> bool {
        if self.front_reporting() {
            return false;
        }
        let Some((fact, _)) = self.failure else {
            return false;
        };
        if !self.say_room(1) {
            return false;
        }
        if let Some(step) = self.tail_candidate(fact) {
            if self.logs.step() == Some(step) {
                return false;
            }
            for pipe in &self.children.get(step).expect("failure step").pipes {
                if !pipe.settled() {
                    return false;
                }
            }
        }
        true
    }
    fn failure_work(&mut self) {
        self.front_failure_work();
        if self.front_reporting() || !self.failure_ready() {
            return;
        }
        let (fact, wall) = self.failure.take().expect("ready failure");
        if self.terminal.failed() && !self.front_running() {
            return;
        }
        let mut item = self.formatter.fact(fact, &self.steps, wall);
        if let Some(step) = self.tail_candidate(fact) {
            self.children.get(step).expect("failure step").tail.append(&mut item, &self.logs.path(step));
        }
        let words = item.bytes.get(10..).expect("failure time prefix");
        // The first line is already timed; time each appended tail line too.
        self.say(self.formatter.notice(words, wall));
    }
    fn terminal_available(&self, step: usize, plain: bool) -> bool {
        if self.front_running() {
            return true;
        }
        if self.front_reporting() {
            return false;
        }
        match self.failure {
            None => self.deferred.is_empty(),
            // Output retained from the failed step's old stdout precedes its
            // failure. Drain it before the block, including legacy fragments.
            Some((fact, _)) => plain && terminal::tail_step(fact) == Some(step),
        }
    }
    fn source_step(&self, forge: u16) -> Option<usize> {
        for (step, placed) in self.steps.iter().enumerate() {
            if placed.step.ignores_ending()
                && let Some(asked) = placed.asked
                && asked.forge == forge
            {
                return Some(step);
            }
        }
        None
    }
    fn source_line(&mut self, forge: u16, wall: Wall) -> Option<Box<[u8]>> {
        let step = self.source_step(forge)?;
        let state = self.children.get_mut(step).expect("source child");
        if let Some(outcome) = &state.source_said {
            return Some(pipe::source(forge, outcome));
        }
        Some(self.source_ending(step, forge, wall))
    }
    fn source_ending(&mut self, step: usize, forge: u16, wall: Wall) -> Box<[u8]> {
        let state = self.children.get_mut(step).expect("source child");
        if state.source.is_none() {
            let why = match state.source_ending {
                Some(SourceEnd::Exited(exit)) => pipe::exited(b"source", exit),
                Some(SourceEnd::NotStarted) => pipe::not_started(b"source"),
                None => b"source ended without an outcome".as_slice().into(),
            };
            state.source = Some(Source::Failed(why));
        }
        let outcome = state.source.as_ref().expect("source outcome");
        if state.source_said.as_ref() != Some(outcome) {
            state.source_said = Some(outcome.clone());
            if let Source::Failed(why) = outcome {
                let label = layout::label(&self.steps, forge).expect("source group's label");
                let line = lines::joined(&[label, b": no sign-in through temper: ", why, b"\n"]);
                self.box_lines.push(self.formatter.notice(&line, wall));
            }
        }
        pipe::source(forge, outcome)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Dispatch each lifecycle order with its source wall, including drained source outcomes."
    )]
    fn order(&mut self, order: Order, wall: Wall) {
        match order {
            Order::Withdraw { step } => {
                let state = self.children.get_mut(step).expect("machine step");
                if state.spawn && state.owner.is_none() {
                    state.spawn = false;
                    state.hold = false;
                    state.line = None;
                    state.withdrawn = Some(wall);
                }
            }
            Order::Terminal { fact } => {
                if let Fact::GroupStarted { forge } = fact {
                    self.source_line(forge, wall);
                }
                self.terminal_fact(fact, wall);
            }
            Order::Source { step, first } => {
                let forge = self.steps.get(step).expect("source step").asked.expect("source group").forge;
                let state = self.children.get_mut(step).expect("source child");
                if first {
                    state.source_said = None;
                }
                let previous = state.source_said.clone();
                let line = self.source_ending(step, forge, wall);
                let state = self.children.get(step).expect("source child");
                let added = state.source == Some(Source::Added);
                let changed = match previous {
                    Some(previous) => pipe::source(forge, &previous) != line,
                    None => true,
                };
                if !first
                    && changed
                    && let Some(control) = &mut self.control
                    && control.sink.room()
                {
                    control.sink.push(line).expect("source reply room checked");
                }
                self.input(Input::SourceEnded { step, added });
            }
            Order::Record { forge } => self.records.write(forge),
            Order::WriteGroup { forge, kind } => self.records.write_group(forge, kind),
            Order::Unlink { forge } => self.records.unlink(forge),
            Order::RunRemover { slot, .. } => {
                let step = self.machine.remover(slot).expect("slot remover");
                self.children.get_mut(step).expect("remover child").spawn = true;
            }
            Order::Say { said } => {
                let source = match said {
                    Said::Started { forge } => self.source_line(forge, wall),
                    Said::Step { .. } | Said::Stopped { .. } | Said::Removed { .. } | Said::Failed { .. } => None,
                };
                if let Some(control) = &mut self.control {
                    control.fetched = None;
                    let line = match said {
                        Said::Step { step } => {
                            let asked = self.steps.get(step).expect("step").asked.expect("asked step");
                            pipe::step(asked.forge, asked.stage)
                        }
                        Said::Started { forge } => match source {
                            Some(source) => lines::joined(&[&source, &pipe::started(forge)]),
                            None => pipe::started(forge),
                        },
                        Said::Stopped { forge } => pipe::stopped(forge),
                        Said::Removed { forge } => pipe::removed(forge),
                        Said::Failed { forge, why } => {
                            let why = terminal::why(why, &self.steps);
                            pipe::failed(forge, &why, self.line_bytes)
                        }
                    };
                    if control.sink.room() {
                        control.sink.push(line).expect("reply room checked");
                    }
                }
            }
            Order::Wait { step } => {
                self.children
                    .get_mut(step)
                    .expect("step")
                    .ready
                    .as_mut()
                    .expect("version endpoint")
                    .start(self.env.now);
            }
            Order::Hold { step } => self.children.get_mut(step).expect("machine step").hold = true,
            Order::Spawn { step } => {
                if self.steps.get(step).expect("step").step.kind() == Kind::Link {
                    let before = step.checked_sub(1).expect("link predecessor");
                    let line = self.children.get_mut(before).expect("long step").line.take();
                    self.children.get_mut(step).expect("link step").line = line;
                }
                self.children.get_mut(step).expect("machine step").spawn = true;
            }
            Order::Signal { step } => {
                if let Some(ready) = &mut self.children.get_mut(step).expect("step").ready {
                    ready.cancel(&mut self.requests);
                }
                if let Some(child) = self.children.get(step).expect("machine step").process {
                    self.requests.push(Request::Signal {
                        child,
                        signal: kernel::Signal::Terminate,
                        to: kernel::Target::Child,
                    });
                }
            }
            Order::Close { step } => {
                if let Some(ready) = &mut self.children.get_mut(step).expect("step").ready {
                    ready.cancel(&mut self.requests);
                }
                if self.steps.get(step).expect("step").step.name() == self.settings.serve.as_ref() {
                    self.retire_control();
                }
                if let Some(child) = self.children.get(step).expect("machine step").process {
                    self.requests.push(Request::Close { entity: child });
                }
            }
        }
    }
    fn order_work(&mut self) {
        let mut blocked = false;
        for _ in 0..self.orders.len() {
            if self.requests.room() < 6 {
                break;
            }
            if self.failure.is_some()
                && let Some(Order::Terminal { fact }) = self.orders.iter().next()
                && fact.place() == Place::Terminal
            {
                let order = self.orders.pop().expect("counted fact");
                let wall = self.order_walls.pop().expect("fact wall");
                self.orders.push(order);
                self.order_walls.push(wall);
                continue;
            }
            if blocked || !self.order_ready() {
                if self.failure.is_none() {
                    break;
                }
                blocked = true;
                let order = self.orders.pop().expect("counted order");
                let wall = self.order_walls.pop().expect("order wall");
                self.orders.push(order);
                self.order_walls.push(wall);
                continue;
            }
            let order = self.orders.pop().expect("counted order");
            let wall = self.order_walls.pop().expect("order wall");
            self.order(order, wall);
        }
    }
    fn orders_runnable(&self) -> bool {
        if self.failure.is_none() {
            return self.order_ready();
        }
        for order in &self.orders {
            match order {
                Order::Terminal { fact } => {
                    if fact.place() == Place::Log && !self.bindings_pending() {
                        return true;
                    }
                }
                Order::Say { .. } | Order::Source { .. } => return self.reply_ready(),
                Order::Spawn { .. }
                | Order::Withdraw { .. }
                | Order::Hold { .. }
                | Order::Wait { .. }
                | Order::Signal { .. }
                | Order::Close { .. }
                | Order::RunRemover { .. } => return true,
                Order::Record { .. } | Order::WriteGroup { .. } | Order::Unlink { .. } => return self.records.ready(),
            }
        }
        false
    }
    fn withdrawn(&mut self) {
        for (step, state) in self.children.iter_mut().enumerate() {
            if let Some(wall) = state.withdrawn.take() {
                policy_input_at(
                    &mut self.machine,
                    self.env.now,
                    wall,
                    Input::Withdrawn { step },
                    &mut self.orders,
                    &mut self.order_walls,
                );
            }
        }
    }
    fn child_ready(&mut self, step: usize) {
        let state = self.children.get_mut(step).expect("child step");
        if let Some(ready) = &mut state.ready {
            if self.machine.outcome().is_some() {
                ready.cancel(&mut self.requests);
            }
            if let Some(ok) = ready.tick(self.env.now, &mut self.serial, &mut self.requests) {
                let wall = if ok { state.ready_wall } else { self.env.wall };
                policy_input_at(
                    &mut self.machine,
                    self.env.now,
                    wall,
                    if ok { Input::Ready { step } } else { Input::NotReady { step } },
                    &mut self.orders,
                    &mut self.order_walls,
                );
            }
        }
        if let Some(stdin) = &mut state.stdin {
            stdin.tick(true, &mut self.requests);
        }
    }
    fn bindings_pending(&self) -> bool {
        for step in 0..self.steps.len() {
            if let Some(forge) = self.machine.bound(step)
                && self.bound_names.get(step).expect("binding step") != &Some(forge)
            {
                return true;
            }
        }
        false
    }
    fn sync_bindings(&mut self) {
        if !self.bindings_pending() || !self.logs.settled() || !self.started_notices.is_empty() {
            return;
        }
        for step in 0..self.steps.len() {
            if let Some(forge) = self.machine.bound(step)
                && self.bound_names.get(step).expect("binding step") != &Some(forge)
            {
                let placed = self.steps.get_mut(step).expect("binding step");
                placed.step = layout::bound(&self.templates.get(step).expect("template step").step, forge);
                if let Some(label) = &self.templates.get(step).expect("template step").label {
                    placed.label = Some(layout::bound_label(label, forge));
                }
                if let Some(asked) = &mut placed.asked {
                    asked.forge = forge;
                }
                self.logs.rebind(step, placed.step.name());
                *self.bound_names.get_mut(step).expect("binding step") = Some(forge);
            }
        }
    }
    #[expect(clippy::too_many_lines, reason = "Child admission and its fetch output share one per-step pass.")]
    fn children(&mut self) {
        self.withdrawn();
        let failure = self.failure;
        let bindings_pending = self.bindings_pending();
        let deferred = !self.deferred.is_empty();
        let front_running = self.front_running();
        let front_reporting = self.front_reporting();
        for step in 0..self.children.len() {
            if self.requests.room() < 8 {
                break;
            }
            self.child_ready(step);
            let notice_room = self.notice_room();
            let mut unknown = None;
            let mut said = List::uncapped();
            let placed = self.steps.get(step).expect("child step");
            let fetch = match placed.asked {
                Some(asked) if asked.stage == Stage::Fetch => Some(asked),
                Some(_) | None => None,
            };
            let state = self.children.get_mut(step).expect("child step");
            let description = &self.steps.get(step).expect("step state").step;
            for (fd, pipe) in state.pipes.iter_mut().enumerate() {
                let plain = fd == 0 && lines::route(&self.settings.serve, description.name(), 1);
                let entries = if plain { self.formatter.block_entries() } else { 1 };
                let terminal_available = front_running
                    || !front_reporting
                        && match failure {
                            None => !deferred,
                            Some((fact, _)) => plain && terminal::tail_step(fact) == Some(step),
                        };
                let quiet = !plain && self.settings.echo == Echo::None && !self.logs.failed(step);
                let diagnostic_ready = (plain || self.logs.takes()) && (fd != 0 || fetch.is_none() || notice_room);
                if diagnostic_ready
                    && (front_running
                        || quiet
                        || terminal_available && self.terminal.takes(entries)
                        || self.terminal.failed())
                    && let Some(output) = pipe.lines.take(description.name())
                {
                    match output {
                        Output::Piece { bytes, wall } => {
                            let mut log_only = false;
                            if fd == 0
                                && let Some(asked) = fetch
                            {
                                match (Fetched::read(&bytes), self.machine.relay(step)) {
                                    (Some(fetched), Relay::Send) => {
                                        if let Some(control) = &mut self.control {
                                            control.fetched = Some(pipe::fetched(asked.forge, fetched));
                                        }
                                        log_only = true;
                                    }
                                    (Some(_), Relay::Exited) => log_only = true,
                                    (Some(_), Relay::Unasked)
                                    | (None, Relay::Send | Relay::Unasked | Relay::Exited) => {
                                        unknown = Some(wall);
                                    }
                                }
                            }
                            if fd == 0 && description.ignores_ending() {
                                if let Some(outcome) = Source::read(&bytes)
                                    && state.source.is_none()
                                {
                                    state.source = Some(outcome);
                                }
                                log_only = true;
                            }
                            self.logs.put(Piece { step, bytes, wall, log_only });
                        }
                        Output::Line { bytes, .. } if plain && front_running => {
                            let url = skein_lib::bytes::copy_of(bytes.strip_suffix(b"\n").expect("link's LF"));
                            if let Some(front) = &mut self.front
                                && let Some(sink) = &mut front.lines
                            {
                                drop(sink.push(crate::front::Line::Link { url }.bytes()));
                            }
                        }
                        Output::Line { bytes, .. } if plain && !self.terminal.failed() => {
                            for item in self.formatter.block(bytes).into_boxed() {
                                said.push(item).expect("pipe output");
                            }
                            self.formatter.admitted_block();
                        }
                        Output::Legacy { bytes } if front_running || !self.terminal.failed() => {
                            said.push(if front_running {
                                self.formatter.notice(&bytes, self.env.wall)
                            } else {
                                Item { target: Target::Stdout, bytes }
                            })
                            .expect("pipe output");
                        }
                        Output::Line { bytes, wall } if front_running || !self.terminal.failed() => {
                            said.push(if front_running {
                                self.formatter.notice(&bytes, wall)
                            } else {
                                Item { target: Target::Stderr, bytes }
                            })
                            .expect("pipe output");
                        }
                        Output::Refused { wall } if front_running || !self.terminal.failed() => {
                            said.push(self.formatter.notice(terminal::REFUSAL, wall)).expect("pipe refusal");
                        }
                        Output::Line { .. } | Output::Legacy { .. } | Output::Refused { .. } => {}
                    }
                }
                if (plain || self.logs.takes())
                    && pipe.lines.wants_read()
                    && !pipe.reading
                    && (front_running
                        || quiet
                        || terminal_available && self.terminal.takes(1)
                        || self.terminal.failed())
                    && let Some(stream) = pipe.token
                {
                    self.requests.push(Request::Stream {
                        stream,
                        down: Down::Demand { read: Read::Arrived { max: LINE_BYTES }, room: 0 },
                    });
                    pipe.reading = true;
                }
            }
            let mut settled = state.stdin.is_none() && state.ready_settled();
            for pipe in &state.pipes {
                settled &= pipe.settled();
            }
            if description.name() == self.settings.serve.as_ref() {
                settled &= self.closing_control == [None, None];
            }
            if state.spawn
                && settled
                && self.logs.takes()
                && !bindings_pending
                && failure.is_none()
                && !withdraw_queued(&self.orders, step)
            {
                state.spawn = false;
                state.tail.reset();
                state.source = None;
                state.source_ending = None;
                self.serial = self.serial.checked_add(1).expect("spawn serial");
                let owner = Token::new(self.serial);
                state.owner = Some(owner);
                if description.kind() != Kind::Link {
                    state.line = None;
                }
                for (fd, pipe) in state.pipes.iter_mut().enumerate() {
                    *pipe = Pipe::new(fd == 0 && lines::route(&self.settings.serve, description.name(), 1));
                }
                if state.hold {
                    state.pipes.get_mut(0).expect("stdout").lines.hold();
                    state.hold = false;
                }
                let secret = description.name() == self.settings.serve.as_ref() && self.settings.first_link.is_some();
                if secret {
                    let first = self.settings.first_link.take().expect("first serve spawn");
                    state.secret = Some(match first.local {
                        Some(local) => {
                            Secret::new(lines::joined(&[&first.secret.into_exposed(), &local.into_exposed()]))
                        }
                        None => first.secret,
                    });
                }
                self.requests.push(Request::Spawn {
                    owner,
                    spawn: kernel::Spawn {
                        program: self.settings.program.clone(),
                        args: spawn_arguments(&self.settings, description.arguments()),
                        env: spawn_environment(&self.settings.environment, description.listeners().len()),
                        root: self.settings.root,
                        dir: Box::new([]),
                        pipes: spawn_pipes(description, &self.settings.serve, &self.listeners, secret),
                    },
                });
            }
            for item in said.into_boxed() {
                self.say(item);
            }
            if let Some(wall) = unknown {
                self.notice(self.formatter.notice(b"box: fetch: unknown line\n", wall));
            }
        }
    }
    fn start_ready(&self) -> bool {
        let entries = if self.settings.first_link.is_some() { 3_u32 } else { 0_u32 }
            .checked_add(u32::from(!self.settings.after_block.is_empty()))
            .expect("startup items");
        self.phase == Phase::New
            && self.logs.ready()
            && (self.front.is_none() || self.requests.room() > 0)
            && self.records.ready()
            && (entries == 0 || self.say_room(entries))
    }
    fn start_work(&mut self) {
        if !self.start_ready() {
            return;
        }
        let wall = self.env.wall;
        for line in &self.settings.log_lines {
            self.box_lines.push(self.formatter.notice(line, wall));
        }
        let logs = lines::joined(&[b"Logs: ", &self.logs.directory(), b"\n"]);
        self.box_lines.push(self.formatter.notice(&logs, wall));
        self.box_lines.push(self.formatter.notice(terminal::CTRL_C, wall));
        for group in self.records.groups() {
            self.machine.restore(group.forge, group.kind, group.started);
        }
        if let Some(front) = self.settings.front.take() {
            let link = self.settings.first_link.as_ref().expect("front's first link");
            let words = crate::front::block_words(&self.formatter, &link.url);
            self.box_lines.push(self.formatter.notice(&words, wall));
            if !self.settings.after_block.is_empty() {
                self.box_lines.push(self.formatter.notice(&self.settings.after_block, wall));
            }
            self.formatter.admitted_block();
            self.spawn_front(front);
        } else if !self.terminal.failed()
            && let Some(link) = &self.settings.first_link
        {
            let url = lines::joined(&[&link.url, b"\n"]);
            for item in self.formatter.block(url).into_boxed() {
                self.say(item);
            }
            self.formatter.admitted_block();
        }
        if self.front.is_none() && !self.terminal.failed() && !self.settings.after_block.is_empty() {
            self.say(Item { target: Target::Stderr, bytes: self.settings.after_block.clone() });
        }
        self.phase = Phase::Block;
    }
    fn block_ready(&self) -> bool {
        self.phase == Phase::Block
            && match &self.front {
                Some(front) => front.process.is_some() || front.ended,
                None => true,
            }
            && (self.settings.first_link.is_none() || self.terminal.drained() || self.terminal.failed())
    }
    fn holding_start_notices(&self) -> bool {
        self.phase == Phase::New && self.settings.first_link.is_some()
    }
    fn say_room(&self, entries: u32) -> bool {
        self.front_running() || !self.front_reporting() && self.terminal.takes(entries) || self.terminal.failed()
    }
    fn say(&mut self, item: Item) {
        if self.front_running() {
            self.box_lines.push(item);
        } else if !self.terminal.failed() {
            self.terminal.push(item);
        }
    }
    fn notice_room(&self) -> bool {
        self.front_running()
            || self.terminal.failed()
            || if self.holding_start_notices()
                || self.front_reporting()
                || self.failure.is_some()
                || !self.deferred.is_empty()
            {
                self.deferred.room() > 0
            } else {
                self.terminal.takes(1)
            }
    }
    fn notice(&mut self, item: Item) {
        if self.front_running() {
            self.say(item);
            return;
        }
        if self.terminal.failed() {
            return;
        }
        if self.holding_start_notices() || self.front_reporting() || self.failure.is_some() || !self.deferred.is_empty()
        {
            self.deferred.push(item);
        } else {
            self.say(item);
        }
    }
    fn deferred_work(&mut self) {
        if self.holding_start_notices() || self.front_reporting() || self.failure.is_some() && !self.front_running() {
            return;
        }
        for _ in 0..self.deferred.len() {
            if !self.say_room(1) {
                break;
            }
            let item = self.deferred.pop().expect("deferred notice");
            self.say(item);
        }
    }
    fn log_work(&mut self) {
        if self.notice_room()
            && let Some(notice) = self.logs.take_notice()
        {
            self.notice(self.formatter.notice(&notice, self.env.wall));
        }
        if self.logs.completed() {
            let step = self.logs_step();
            let own = step == self.logs.box_step();
            let echo = self.log_echo();
            if (!echo || self.notice_room())
                && let Some(piece) = self.logs.take()
            {
                if own {
                    if echo && !piece.bytes.ends_with(terminal::CTRL_C) {
                        self.notice(Item { target: Target::Stderr, bytes: piece.bytes });
                    }
                } else {
                    let name = if self.logs.front_step() == Some(piece.step) {
                        b"tui"
                    } else {
                        self.steps.get(piece.step).expect("log step").step.name()
                    };
                    if echo {
                        let words = piece.echo(name);
                        let item = if self.front_running() {
                            self.formatter.notice(&words, piece.wall)
                        } else {
                            Item { target: Target::Stderr, bytes: words }
                        };
                        self.notice(item);
                    } else if !piece.log_only && !self.logs.failed(piece.step) {
                        if self.logs.front_step() == Some(piece.step) {
                            self.front.as_mut().expect("front's log").tail.push(piece.echo(name));
                        } else {
                            self.children.get_mut(piece.step).expect("log step").tail.push(piece.echo(name));
                        }
                    }
                }
            }
        }
        if self.box_log_ready()
            && let Some(item) = self.box_lines.pop()
        {
            for line in crate::front::logs(&item.bytes).into_boxed() {
                self.front_line(line);
            }
            self.box_echo = !self.front_running();
            self.logs.put(Piece { step: self.logs.box_step(), bytes: item.bytes, wall: self.env.wall, log_only: true });
        }
        if self.logs.takes()
            && let Some((step, wall)) = self.started_notices.pop()
        {
            self.logs.put(Piece {
                step,
                bytes: skein_lib::bytes::copy_of(self.steps.get(step).expect("notice step").step.started_notice()),
                wall,
                log_only: true,
            });
        }
        self.logs.tick(self.settings.root, &mut self.submissions);
    }
    fn log_echo(&self) -> bool {
        let step = self.logs_step();
        if step == self.logs.box_step() {
            self.logs.failed(step) && self.box_echo
        } else {
            !self.logs.log_only() && (self.settings.echo == Echo::Steps || self.logs.failed(step))
        }
    }
    fn logs_step(&self) -> usize {
        self.logs.step().expect("completed piece")
    }
    fn record_work(&mut self) {
        if self.notice_room()
            && let Some(notice) = self.records.take_notice()
        {
            self.notice(self.formatter.notice(&notice, self.env.wall));
        }
        if self.record_notice_admitted()
            && let Some(answer) = self.records.take_answer()
        {
            policy_input_at(
                &mut self.machine,
                self.env.now,
                self.record_wall,
                answer,
                &mut self.orders,
                &mut self.order_walls,
            );
        }
        self.records.tick(self.settings.root, self.env.now, &mut self.submissions);
        if self.records.closed() {
            self.root = Root::Closed;
        }
    }
    fn control_ready(&self) -> bool {
        let Some(control) = &self.control else {
            return false;
        };
        control.sink.drained()
            && self.orders_drained()
            && self.failure.is_none()
            && (self.front_running() || self.deferred.is_empty())
            && self.say_room(1)
            && self.phase == Phase::Running
    }
    fn take_control_line(&mut self) {
        if !self.control_ready() {
            return;
        }
        let Some(line) = self.control.as_mut().expect("serve").line.take() else {
            return;
        };
        let wall = self.control.as_ref().expect("serve").line_wall;
        let current_wall = self.env.wall;
        self.env.wall = wall;
        match line {
            Line::Whole(line) => match pipe::command(&line) {
                Command::Start(forge) => self.input(Input::Ask { forge }),
                Command::Stop(forge) => self.input(Input::Stop { forge }),
                Command::StartKind { forge, kind } => self.input(Input::AskKind { forge, kind }),
                Command::Remove(forge) => self.input(Input::Remove { forge }),
                Command::Unknown => {
                    self.say(self.formatter.notice(b"box: serve: unknown pipe line\n", self.env.wall));
                }
            },
            Line::TooLong => {
                self.say(self.formatter.notice(b"box: serve: pipe line too long\n", self.env.wall));
            }
        }
        self.env.wall = current_wall;
    }
    fn can_read_control(&self) -> bool {
        let Some(control) = &self.control else {
            return false;
        };
        control.read_phase == ReadPhase::Open
            && !control.reading
            && control.line.is_none()
            && self.control_ready()
            && self.say_room(1)
            && self.phase == Phase::Running
    }
    fn control_work(&mut self) {
        if self.requests.room() < 5 {
            return;
        }
        self.take_control_line();
        if self.can_read_control() {
            let control = self.control.as_mut().expect("live serve");
            self.requests.push(Request::Stream {
                stream: control.read,
                down: Down::Demand { read: Read::Arrived { max: 1 }, room: 0 },
            });
            control.reading = true;
        }
        if let Some(control) = &mut self.control {
            if control.sink.drained()
                && let Some(fetched) = control.fetched.take()
            {
                control.sink.push(fetched).expect("drained sink holds one fetch line");
            }
            control.sink.tick(false, &mut self.requests);
            control.sink.flush(self.env.limits.output, &mut self.requests);
        }
    }
    fn front_settled(&self) -> bool {
        match &self.front {
            Some(front) => front.settled(),
            None => true,
        }
    }
    fn finish(&mut self) {
        if !self.front_settled() {
            return;
        }
        if !self.machine.is_idle()
            || self.machine.outcome().is_none()
            || !self.records.ready()
            || !self.orders.is_empty()
            || self.failure.is_some()
            || !self.deferred.is_empty()
            || !self.started_notices.is_empty()
            || !self.box_lines.is_empty()
        {
            return;
        }
        for state in &self.children {
            if !state.ready_settled() {
                return;
            }
            for pipe in &state.pipes {
                if !pipe.settled() {
                    return;
                }
            }
        }
        if !self.logs.closed() {
            if self.logs.settled() {
                self.logs.close();
            }
            return;
        }
        if self.phase != Phase::Closing {
            if self.requests.room() == 0 {
                return;
            }
            self.requests.push(Request::Close { entity: self.signals });
            self.phase = Phase::Closing;
        }
        while self.listeners_closing < self.listeners.len() && self.requests.room() > 0 {
            let (_, token) = self.listeners.get(self.listeners_closing).expect("held listener");
            self.requests.push(Request::Close { entity: *token });
            self.listeners_closing = self.listeners_closing.saturating_add(1);
        }
        if self.root == Root::Open && self.submissions.room() > 0 {
            self.records.close_root();
            self.root = Root::Closing;
        }
    }
}
fn reply_entries(steps: &Steps) -> u32 {
    let mut longest = 0_u32;
    for group in &steps.asked {
        longest = longest.max(u32::try_from(group.steps.len()).expect("bounded group"));
    }
    longest.checked_add(2).expect("bounded reply queue")
}
// A terminal fact or pipe reply can hold a Withdraw in the order queue after
// its Spawn has been consumed. Admission must already respect that withdrawal.
fn withdraw_queued(orders: &Queue<Order>, step: usize) -> bool {
    for order in orders {
        if *order == (Order::Withdraw { step }) {
            return true;
        }
    }
    false
}
// Both queues grow together. Each policy entry stamps its newly emitted
// orders before another entry or consumption; pops and rotations keep the
// source wall paired with its order.
fn stamp_orders(orders: &Queue<Order>, walls: &mut Queue<Wall>, wall: Wall) {
    let added = orders.len().checked_sub(walls.len()).expect("paired policy orders");
    for _ in 0..added {
        walls.push(wall);
    }
    assert_eq!(orders.len(), walls.len(), "paired policy orders");
}
fn policy_input(
    machine: &mut Machine,
    env: &Env<io::Limits>,
    input: Input,
    orders: &mut Queue<Order>,
    walls: &mut Queue<Wall>,
) {
    policy_input_at(machine, env.now, env.wall, input, orders, walls);
}
fn policy_input_at(
    machine: &mut Machine,
    now: Time,
    wall: Wall,
    input: Input,
    orders: &mut Queue<Order>,
    walls: &mut Queue<Wall>,
) {
    machine.input(now, input, orders);
    stamp_orders(orders, walls, wall);
}
fn adopt_listeners(io: &mut Io, held: &[(Fd, core::net::SocketAddr)]) -> List<(core::net::SocketAddr, Token)> {
    let mut listeners = List::with_capacity(u32::try_from(held.len()).expect("bounded listeners"));
    for (fd, address) in held {
        let token = io.adopt_listener(*fd, *address).expect("unique listeners and counted slots");
        listeners.push((*address, token)).expect("counted listener tokens");
    }
    listeners
}
/// The same exec boundary preserves each step's descriptors, environment and process group (protocol/setup/box.md, 1).
fn spawn_arguments(settings: &Settings, arguments: &[Box<[u8]>]) -> Box<[Box<[u8]>]> {
    let Some(parent) = settings.parent else { return Box::from(arguments) };
    let mut args = List::uncapped();
    for argument in [b"internal".as_slice(), b"forge-child", b"--parent"] {
        args.push(Box::from(argument)).expect("uncapped arguments");
    }
    args.push(Box::from(Decimal::of(u64::from(parent)).as_bytes())).expect("uncapped arguments");
    for argument in [b"--signal".as_slice(), b"terminate", b"--", &settings.program] {
        args.push(Box::from(argument)).expect("uncapped arguments");
    }
    for argument in arguments {
        args.push(argument.clone()).expect("uncapped arguments");
    }
    args.into_boxed()
}

fn spawn_environment(environment: &[Box<[u8]>], listeners: usize) -> Box<[Box<[u8]>]> {
    let mut out = List::with_capacity(
        u32::try_from(environment.len()).expect("bounded environment").checked_add(1).expect("bounded startup memory"),
    );
    for entry in environment {
        if !entry.starts_with(b"LISTEN_FDS=") && !entry.starts_with(b"LISTEN_PID=") {
            out.push(entry.clone()).expect("environment counted");
        }
    }
    if listeners > 0 {
        let count = Decimal::of(u64::try_from(listeners).expect("bounded listeners"));
        out.push(lines::joined(&[b"LISTEN_FDS=", count.as_bytes()])).expect("count slot");
    }
    out.into_boxed()
}
fn spawn_pipes(
    step: &Step,
    serve: &[u8],
    listeners: &[(core::net::SocketAddr, Token)],
    secret: bool,
) -> Box<[kernel::Pipe]> {
    let n = u32::try_from(step.listeners().len()).expect("bounded listeners");
    let mut pipes = List::with_capacity(n.checked_add(5).expect("bounded spawn table"));
    for child in [1, 2] {
        pipes.push(kernel::Pipe { child, way: kernel::Way::Out, parent: None }).expect("stdio");
    }
    for (i, address) in step.listeners().iter().enumerate() {
        let mut found = None;
        for (addr, token) in listeners {
            if addr == address {
                found = Some(*token);
                break;
            }
        }
        let token = found.expect("binary opened every step listener");
        let child = 3_u32.checked_add(u32::try_from(i).expect("bounded listener index")).expect("bounded child fd");
        pipes.push(kernel::Pipe { child, way: kernel::Way::Given(token), parent: None }).expect("listener slot");
    }
    if step.name() == serve {
        let requests = n.checked_add(3).expect("bounded child fd");
        let answers = n.checked_add(4).expect("bounded child fd");
        pipes.push(kernel::Pipe { child: requests, way: kernel::Way::Out, parent: None }).expect("serve requests");
        pipes.push(kernel::Pipe { child: answers, way: kernel::Way::In, parent: None }).expect("serve answers");
    } else if step.kind() == Kind::Link {
        pipes.push(kernel::Pipe { child: 0, way: kernel::Way::In, parent: None }).expect("link stdin");
    }
    if secret {
        pipes.push(kernel::Pipe { child: 0, way: kernel::Way::In, parent: None }).expect("serve stdin");
    }
    pipes.into_boxed()
}
/// Up to five entities per step, three inherited streams and two serve control pipes.
/// Names and the configured control line bound extend the byte limits at construction.
#[must_use]
pub fn io_limits(steps: u32) -> io::Limits {
    io::Limits {
        sockets: steps.checked_mul(5).expect("bounded step count").checked_add(5).expect("inherited streams"),
        names: 0,
        refusals: steps.max(1),
        intake: LINE_BYTES,
        receive: 1,
        output: LINE_BYTES.saturating_add(3),
        sends: 1,
        accepts: 1,
        backlog: 1,
        close_timeout: Duration::from_secs(5),
        retry: Duration::from_millis(10),
    }
}
impl skein_shell::Host for Service {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        for _ in 0..self.env.limits.sockets.saturating_add(self.env.limits.refusals) {
            if !self.io.is_ready() || self.submissions.room() < io::MAX_OUT_RESUME.submissions {
                break;
            }
            io::resume(&mut self.io, &self.env, &mut self.events, &mut self.submissions);
        }
        if !self.io.is_ready() {
            for _ in 0..self.completions.len() {
                if self.submissions.room() < io::MAX_OUT_UP.submissions {
                    break;
                }
                let complete = self.completions.pop().expect("counted completion");
                if Logs::owns(complete.op) {
                    self.logs.complete(complete);
                } else if Records::owns(complete.op) {
                    self.record_wall = wall;
                    self.records.complete(complete);
                } else {
                    io::up(&mut self.io, &self.env, complete, &mut self.events, &mut self.submissions);
                }
            }
            if self.completions.is_empty() {
                for _ in 0..self.env.limits.sockets.saturating_mul(2) {
                    if !self.io.is_due(now) || self.submissions.room() < io::MAX_OUT_FIRE.submissions {
                        break;
                    }
                    io::fire(&mut self.io, &self.env, &mut self.events, &mut self.submissions);
                }
            }
        }
        self.record_work();
        self.log_work();
        self.start_work();
        // Capture wall time on intake, before an ordinary event can wait behind
        // request credit. Scan once: progress bypasses a blocked ordinary front,
        // while all ordinary events keep their relative order.
        for _ in 0..self.events.len() {
            let event = self.events.pop().expect("captured event");
            self.told.push(Told { event, wall });
        }
        // An ordinary event can issue four requests. Leave two slots for the
        // active terminal's Send and full-budget Demand, even if policy stalls.
        let mut blocked = false;
        for _ in 0..self.told.len() {
            let told = self.told.pop().expect("counted event");
            let progress = self.progress_event(&told.event);
            let admitted = if progress { self.requests.room() >= 2 } else { !blocked && self.ordinary_events_ready() };
            if !admitted {
                if !progress {
                    blocked = true;
                }
                self.told.push(told);
                continue;
            }
            self.env.wall = told.wall;
            self.event(told.event);
            self.env.wall = wall;
        }
        if self.terminate_pending && (self.front.is_none() || self.requests.room() > 0) {
            self.terminate_pending = false;
            self.terminate();
        }
        if self.block_ready() {
            self.machine.start(now, self.records.recorded(), &mut self.orders);
            stamp_orders(&self.orders, &mut self.order_walls, wall);
            self.phase = Phase::Running;
        }
        if self.policy_timers_ready() {
            self.machine.fire(now, &mut self.orders);
            stamp_orders(&self.orders, &mut self.order_walls, wall);
        }
        if self.policy_timers_ready()
            && self.requests.room() >= 8
            && let Some(front) = &mut self.front
        {
            front.fire(now, &mut self.requests);
        }
        self.failure_work();
        self.deferred_work();
        self.order_work();
        self.sync_bindings();
        self.front_work();
        self.children();
        self.control_work();
        self.record_work();
        self.log_work();
        self.failure_work();
        self.deferred_work();
        self.finish();
        if self.requests.room() >= 2 {
            self.terminal.tick(self.phase == Phase::Closing, &mut self.requests);
        }
        for _ in 0..self.requests.len() {
            if !self.io.takes() || self.submissions.room() < io::MAX_OUT_DOWN.submissions {
                break;
            }
            let request = self.requests.pop().expect("counted request");
            io::down(&mut self.io, &self.env, request, &mut self.submissions);
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    #[expect(clippy::too_many_lines, reason = "Runnability checks each host queue against its destination's credit.")]
    fn work_pending(&self, now: Time) -> bool {
        if self.start_ready()
            || self.block_ready()
            || !self.completions.is_empty() && self.submissions.room() >= io::MAX_OUT_UP.submissions
            || !self.events.is_empty()
            || self.events_runnable()
            || !self.requests.is_empty() && self.io.takes() && self.submissions.room() >= io::MAX_OUT_DOWN.submissions
            || !self.orders.is_empty() && self.orders_runnable() && self.requests.room() >= 6
            || self.terminate_pending && (self.front.is_none() || self.requests.room() > 0)
            || self.io.is_ready() && self.submissions.room() >= io::MAX_OUT_RESUME.submissions
            || self.io.is_due(now) && self.submissions.room() >= io::MAX_OUT_FIRE.submissions
        {
            return true;
        }
        if let Some(at) = self.next_deadline()
            && at <= now
        {
            return true;
        }
        if self.bindings_pending() && self.logs.settled() && self.started_notices.is_empty()
            || !self.started_notices.is_empty() && self.logs.takes()
            || !self.box_lines.is_empty() && self.box_log_ready()
            || self.failure_ready()
            || self.front_failure_ready()
            || self.logs.pending()
            || self.logs.has_notice() && self.notice_room()
            || !self.holding_start_notices()
                && !self.deferred.is_empty()
                && (self.failure.is_none() || self.front_running())
                && !self.front_reporting()
                && self.say_room(1)
            || self.logs.completed() && (!self.log_echo() || self.notice_room())
        {
            return true;
        }
        if self.records.pending() && (!self.records.has_notice() || self.notice_room()) {
            return true;
        }
        if let Some(control) = &self.control
            && self.requests.room() >= 5
            && (control.fetched.is_some() && control.sink.drained()
                || control.sink.pending()
                || control.sink.needs_flush()
                || self.can_read_control()
                || (control.line.is_some() && self.control_ready()))
        {
            return true;
        }
        if self.terminal.pending(self.phase == Phase::Closing) && self.requests.room() >= 2 {
            return true;
        }
        if self.requests.room() >= 8
            && let Some(front) = &self.front
            && front.pending(self.logs.takes())
        {
            return true;
        }
        for state in &self.children {
            if state.withdrawn.is_some() {
                return true;
            }
        }
        if self.requests.room() < 8 {
            return false;
        }
        for (step, state) in self.children.iter().enumerate() {
            if let Some(stdin) = &state.stdin
                && (stdin.pending() || stdin.needs_flush())
            {
                return true;
            }
            let description = &self.steps.get(step).expect("step state").step;
            for (fd, pipe) in state.pipes.iter().enumerate() {
                let plain = fd == 0 && lines::route(&self.settings.serve, description.name(), 1);
                let fetch = match self.steps.get(step).expect("step").asked {
                    Some(asked) => asked.stage == Stage::Fetch,
                    None => false,
                };
                let entries = if plain { self.formatter.block_entries() } else { 1 };
                let quiet = !plain && self.settings.echo == Echo::None && !self.logs.failed(step);
                let visible = self.terminal_available(step, plain);
                if self.requests.room() >= 8
                    && (plain || self.logs.takes())
                    && ((pipe.lines.ready()
                        && (fd != 0 || !fetch || self.notice_room())
                        && (quiet || visible && self.say_room(entries) || self.terminal.failed()))
                        || (pipe.lines.wants_read()
                            && !pipe.reading
                            && pipe.token.is_some()
                            && (quiet || visible && self.say_room(1) || self.terminal.failed())))
                {
                    return true;
                }
            }
            let mut settled = state.stdin.is_none() && state.ready_settled();
            for pipe in &state.pipes {
                settled &= pipe.settled();
            }
            if description.name() == self.settings.serve.as_ref() {
                settled &= self.closing_control == [None, None];
            }
            if state.spawn
                && settled
                && self.logs.takes()
                && !self.bindings_pending()
                && self.failure.is_none()
                && !withdraw_queued(&self.orders, step)
            {
                return true;
            }
        }
        false
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        if self.submissions.room() >= io::MAX_OUT_FIRE.submissions {
            match (self.io.next_deadline(), self.records.deadline()) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
            }
        } else {
            None
        }
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        if !self.policy_timers_ready() {
            return None;
        }
        let mut deadline = self.machine.next_deadline();
        if self.requests.room() >= 8
            && let Some(front) = &self.front
            && let Some(at) = front.deadline()
        {
            deadline = Some(deadline.unwrap_or(at).min(at));
        }
        for state in &self.children {
            if self.requests.room() >= 8
                && let Some(ready) = &state.ready
                && let Some(at) = ready.next_deadline()
            {
                deadline = Some(deadline.unwrap_or(at).min(at));
            }
        }
        deadline
    }
    fn is_empty(&self) -> bool {
        self.phase == Phase::Closing
            && self.front_settled()
            && self.root == Root::Closed
            && self.logs.closed()
            && self.failure.is_none()
            && self.deferred.is_empty()
            && self.box_lines.is_empty()
            && self.machine.is_idle()
            && self.io.is_empty()
            && self.requests.is_empty()
            && self.orders_drained()
            && self.order_walls.is_empty()
            && self.events.is_empty()
            && self.told.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        if !self.is_empty() {
            return None;
        }
        Some(Exit::Code(u8::from(self.failed || self.machine.outcome() != Some(Outcome::Stopped))))
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits)
            .expect("io operations")
            .checked_add(5)
            .expect("root, records, group files, logger")
    }
}

/// Test-only saturation of the real host's destination queues. The simulator
/// still owns every completion and ACK; no service or io behaviour is replaced.
#[cfg(test)]
pub(crate) mod pressure {
    use super::{Event, Fact, Item, Order, Service, Target, Told};
    use skein_lib::{Wall, bytes};
    #[derive(Debug)]
    #[expect(clippy::struct_excessive_bools, reason = "read-only test snapshot of independent gate predicates")]
    pub struct RestartGate {
        pub spawn: bool,
        pub pipes_settled: [bool; 2],
        pub ready_settled: bool,
        pub stdin_settled: bool,
        pub room: bool,
        pub closing_control: [Option<skein_lib::Token>; 2],
        pub logs_take: bool,
        pub failure: bool,
        pub withdraw: bool,
    }
    impl RestartGate {
        pub fn open(&self) -> bool {
            self.spawn
                && self.pipes_settled == [true, true]
                && self.ready_settled
                && self.stdin_settled
                && self.room
                && self.closing_control == [None, None]
                && self.logs_take
                && !self.failure
                && !self.withdraw
        }
    }
    pub fn restart_gate(service: &Service) -> RestartGate {
        let mut serve = None;
        for (index, placed) in service.steps.iter().enumerate() {
            if placed.step.name() == service.settings.serve.as_ref() {
                serve = Some(index);
            }
        }
        let step = serve.expect("serve step");
        let state = &service.children[step];
        RestartGate {
            spawn: state.spawn,
            pipes_settled: [state.pipes[0].settled(), state.pipes[1].settled()],
            ready_settled: state.ready_settled(),
            stdin_settled: state.stdin.is_none(),
            room: service.requests.room() >= 8,
            closing_control: service.closing_control,
            logs_take: service.logs.takes(),
            failure: service.failure.is_some(),
            withdraw: super::withdraw_queued(&service.orders, step),
        }
    }
    pub fn fill_terminal(service: &mut Service) -> u32 {
        let mut queued = 0_u32;
        while service.terminal.takes(1) {
            service.terminal.push(Item { target: Target::Stderr, bytes: bytes::copy_of(b"queued\n") });
            queued = queued.checked_add(1).unwrap();
        }
        queued
    }
    pub fn saturate(service: &mut Service) -> (u32, u32) {
        let queued = fill_terminal(service);
        let orders = 64;
        for _ in 0..orders {
            service.orders.push(Order::Terminal { fact: Fact::GivingUp { step: 0, why: crate::machine::Why::Busy } });
            service.order_walls.push(service.env.wall);
        }
        // Ordinary intake can drain while the terminal holds policy orders.
        for wall in [Wall::EPOCH, Wall::from_nanos(1)] {
            service.told.push(Told { event: Event::Resized, wall });
        }
        (queued, orders)
    }
    /// More admitted policy inputs than a single step's lifecycle burst.
    pub fn terminate_load(service: &mut Service) {
        for _ in 0_u32..64 {
            service.events.push(Event::Shutdown { signal: skein_io::kernel::ServiceSignal::Terminate, from: None });
        }
    }
    /// Valid redundant signals occupy request credit until the down pass.
    pub fn reserve_requests(service: &mut Service) {
        let child = service.children.first().expect("live child").process.expect("started child");
        for _ in 0..service.requests.room().saturating_sub(4) {
            service.requests.push(skein_io::Request::Signal {
                child,
                signal: skein_io::kernel::Signal::Terminate,
                to: skein_io::kernel::Target::Child,
            });
        }
    }
    pub fn ordinary_load(service: &mut Service) {
        for index in 0_u32..64 {
            service.told.push(Told { event: Event::Resized, wall: Wall::from_nanos(u64::from(index)) });
        }
    }
    pub fn ordinary_load_waits(service: &Service) -> bool {
        if service.told.len() != 64 {
            return false;
        }
        for (index, told) in service.told.iter().enumerate() {
            if told.wall != Wall::from_nanos(u64::try_from(index).expect("load index")) {
                return false;
            }
        }
        true
    }
    pub fn fact_waits(service: &Service, fact: Fact, wall: Wall) -> bool {
        service.orders.iter().next() == Some(&Order::Terminal { fact })
            && service.order_walls.iter().next() == Some(&wall)
    }
    pub fn outcome(service: &Service) -> Option<super::Outcome> {
        service.machine.outcome()
    }
    pub fn blocked(service: &Service) -> bool {
        !service.orders.is_empty() && !service.terminal.takes(1) && service.told.is_empty()
    }
    fn children_bounded(service: &Service) -> bool {
        for child in &service.children {
            for pipe in &child.pipes {
                if !pipe.lines.bounded() {
                    return false;
                }
            }
        }
        true
    }
    pub fn front_held(service: &Service) -> u32 {
        match &service.front {
            Some(front) => match &front.lines {
                Some(sink) => sink.held(),
                None => 0,
            },
            None => 0,
        }
    }
    pub fn bounded(service: &Service) -> bool {
        service.orders.len() <= service.orders.capacity()
            && service.orders.len() == service.order_walls.len()
            && service.order_walls.capacity() == service.orders.capacity()
            && service.told.len() <= service.told.capacity()
            && service.events.len() <= service.events.capacity()
            && service.requests.len() <= service.requests.capacity()
            && service.submissions.len() <= service.submissions.capacity()
            && service.completions.len() <= service.completions.capacity()
            && front_held(service) <= crate::front::HELD
            && service.terminal.bounded(service.env.limits.output)
            && children_bounded(service)
    }
}
