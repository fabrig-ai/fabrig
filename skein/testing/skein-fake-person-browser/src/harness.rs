//! The server and Chromium are children of one io loop (ui/web/testing.md, 2).
use crate::BrowserFace;
use browser::boundary::{Below, Down, Event, Request, Trouble};
use skein_browser::{self as browser, Browser};
use skein_fake_peers::Listening;
use skein_fake_person::Next;
use skein_io::{
    self as io, Io,
    kernel::{self, Complete, Op, Pipe, Spawn, Submit, Way},
};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall, stream};
use skein_world::{Expired, Starved};
use std::fmt::Write as _;
use std::{collections::VecDeque, ffi::OsString, os::unix::ffi::OsStrExt, path::PathBuf};

const POLL: Duration = Duration::from_millis(10);
const DEFAULT_HORIZON: Duration = Duration::from_secs(40);
const CLOSE_WINDOW: Duration = Duration::from_secs(4);
const CAPTURE_WINDOW: Duration = Duration::from_secs(5);
pub(crate) const SERVER: Token = Token::new(10);
pub(crate) const PROCESS: Token = Token::new(11);
pub(crate) const BROWSER: Token = Token::new(12);
const SNAPSHOT: Token = Token::new(u64::MAX - 3);
const IMAGE: Token = Token::new(u64::MAX - 2);
const ROOT: Token = Token::new(u64::MAX - 1);
const SERVER_ROOT: Token = Token::new(u64::MAX);

/// A peer borrowed for this run; its observations remain the test's.
pub struct Lent<'a> {
    pub name: &'a [u8],
    pub peer: &'a mut dyn Listening,
}

#[derive(Clone, Copy)]
pub struct Printed<'a> {
    pub sent: Sent,
    pub prefix: &'a [u8],
    pub on: Output,
    pub names: Names,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sent {
    Page,
    Command,
}
pub struct Command {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub stdin: Stdin,
    pub opens: Option<Opens>,
    pub within: Duration,
}
pub enum Stdin {
    Closed,
    Printed,
}
pub struct Opens {
    pub prefix: Box<[u8]>,
    pub on: Output,
    pub keep: Box<[u8]>,
}
pub(crate) enum CommandAction {
    Run(Command),
    Exit { code: i32, within: Duration },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Output {
    Stdout,
    Stderr,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Names {
    Address,
    Url,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Opened {
    pub url: String,
    pub address: String,
}

pub use crate::origins::{Origin, Page};
pub struct Setup<'a> {
    pub server: PathBuf,
    pub args: Vec<OsString>,
    pub printed: Printed<'a>,
    pub restart: Vec<OsString>,
    pub chromium: PathBuf,
    pub downloads: Option<PathBuf>,
    pub expected: Vec<Box<[u8]>>,
    pub origins: Vec<Origin>,
}
pub struct Asked {
    pub method: skein_http::Method,
    pub path: Box<[u8]>,
    pub host: Box<[u8]>,
    pub headers: Vec<skein_ui_web::transport::Header>,
    pub body: Box<[u8]>,
}
pub trait Play {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>>;
    fn expected(&self) -> Vec<Box<[u8]>> {
        Vec::new()
    }
    fn request(&mut self) -> Option<Asked> {
        None
    }
    fn answered(&mut self, _status: u16) {}
    /// Current zero-based script position. A step's account stays armed until the next step begins.
    fn at(&self) -> u32 {
        0
    }
    /// Also called while an operation is pending. Report unanswered waits through `face.expire`.
    fn check_deadline(&mut self, _face: &mut BrowserFace<'_>, _now: Time) {}
    fn server_action(&mut self) -> Option<ServerAction> {
        None
    }
}
mod account;
use account::{Account, Expiry};
mod commands;
mod run;
mod scripted;
pub use run::Hosted;
pub use scripted::{Element, Scripted, Shown};
mod step_deadline;
use closing::{Capture, CloseWindow};
use step_deadline::{StepAccount, StepFailure, face_failure, step_failure};
mod request;
mod script;
pub mod session;
pub use script::{NestedHas, Script, Step, TabSeen};
#[derive(Clone, Copy)]
pub enum ServerAction {
    Stop,
    Start,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Outcome {
    Done,
    Failed { person: u32, step: u32, report: String },
}
#[derive(Debug, PartialEq, Eq)]
pub struct Kept {
    pub person: u32,
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
pub struct Harness {
    horizon: Duration,
    kept: Vec<Kept>,
    server: PathBuf,
    args: Vec<OsString>,
    sent: Sent,
    page_query: Box<[u8]>,
    page_addresses: fn(&[u8]) -> bool,
    prefix: Box<[u8]>,
    output: Output,
    names: Names,
    restart: Vec<OsString>,
    chromium: PathBuf,
    downloads: Option<PathBuf>,
    expected: Vec<Box<[u8]>>,
    origins: Vec<Origin>,
}
impl Harness {
    #[must_use]
    pub fn new(setup: Setup<'_>) -> Self {
        Self {
            horizon: DEFAULT_HORIZON,
            kept: Vec::new(),
            server: setup.server,
            args: setup.args,
            sent: setup.printed.sent,
            page_query: Box::default(),
            page_addresses: lines::every_address,
            prefix: setup.printed.prefix.into(),
            output: setup.printed.on,
            names: setup.printed.names,
            restart: setup.restart,
            chromium: setup.chromium,
            downloads: setup.downloads,
            expected: setup.expected,
            origins: setup.origins,
        }
    }
    /// Adds query parameters to pages opened from server or command output,
    /// before any fragment. Kept output and command stdin remain as printed.
    #[must_use]
    pub fn page_query(mut self, query: &[u8]) -> Self {
        self.page_query = query.into();
        self
    }
    /// Names the addresses that carry the page query. By default, every address does.
    /// The predicate receives the opening address unchanged, absolute or path-only.
    #[must_use]
    pub fn page_addresses(mut self, page: fn(&[u8]) -> bool) -> Self {
        self.page_addresses = page;
        self
    }
    /// Sets the whole run's horizon for a program with bounds of its own on each step.
    /// No test sets it: every test stands on the forty-second horizon
    /// (skein/docs/design/ui/web/testing.md, 2).
    #[must_use]
    pub fn horizon(mut self, horizon: Duration) -> Self {
        bounds(horizon);
        self.horizon = horizon;
        self
    }
    #[must_use]
    pub fn kept(&self) -> &[Kept] {
        &self.kept
    }
    pub fn run(&mut self, people: &mut [&mut dyn Play]) -> Outcome {
        self.run_lending(people, &mut [])
    }
    pub fn run_lending(&mut self, people: &mut [&mut dyn Play], peers: &mut [Lent<'_>]) -> Outcome {
        run::native(self, people, peers)
    }

    /// Makes a run for a simulated world; give it the world's launch root with [`Hosted::at_root`].
    #[must_use]
    pub fn hosted<'a, 'p, 'l>(
        &'a mut self,
        people: &'a mut [&'p mut dyn Play],
        peers: &'a mut [Lent<'l>],
        browser: Scripted,
    ) -> Hosted<'a, 'p, 'l> {
        Hosted::new(self, people, peers, run::BrowserStart::Scripted(browser))
    }
}
fn bounds(horizon: Duration) -> (Duration, Duration) {
    assert!(horizon >= CLOSE_WINDOW, "horizon {horizon:?} is shorter than the close window {CLOSE_WINDOW:?}");
    (Duration::from_nanos(horizon.as_nanos().saturating_sub(CLOSE_WINDOW.as_nanos())), horizon)
}
pub(crate) mod capacity;
pub(crate) mod closing;
mod lines;
mod peers;
pub(crate) use lines::{opening, restart, server_pipes, substitute};
pub use lines::{page_with_query, printed, with_query};
pub(crate) use peers::fixture as peer_fixture;
#[derive(PartialEq, Eq, Debug)]
enum ServerState {
    Running,
    Stopping,
    Stopped,
    Starting,
}
enum BrowserEnd {
    Chromium(Box<Browser>),
    Scripted(Scripted),
}

struct Runtime {
    kept: Vec<Kept>,
    commands: Vec<Option<commands::Process>>,
    command_serial: u64,
    pending_commands: Vec<Option<Command>>,
    held: Vec<Option<Box<[u8]>>>,
    raw_lines: Vec<Box<[u8]>>,
    request_serial: u64,
    exchanges: Vec<request::Exchange>,
    expected: Vec<Vec<Box<[u8]>>>,
    origins: Vec<crate::origins::Pages>,
    browser: BrowserEnd,
    io: Io,
    startup: VecDeque<io::Request>,
    peer_addresses: Vec<(Box<[u8]>, String)>,
    benv: Env<browser::Limits>,
    ienv: Env<io::Limits>,
    complete: Queue<Complete>,
    submits: Queue<Submit>,
    events: Queue<Event>,
    io_events: Queue<io::Event>,
    down: Queue<Down>,
    requests: VecDeque<io::Request>,
    roots: [Option<kernel::Fd>; 2],
    pipes: Option<[Token; 3]>,
    server_pipes: Vec<Token>,
    output: Vec<u8>,
    lines: Vec<Opened>,
    closed: u32,
    spawned: u32,
    server_state: ServerState,
    stderr: Vec<u8>,
    server_stderr: Vec<u8>,
    address: Option<String>,
    ready: bool,
    context: bool,
    opening: usize,
    opened: Vec<bool>,
    // The last request asked of the browser, for a deadline's report.
    asked: String,
    outcome: Option<Outcome>,
    capture: Option<Capture>,
    snapshot: Option<String>,
    screenshot: Option<PathBuf>,
    closing: closing::Closing,
    close_window: Option<CloseWindow>,
    deadline: Option<Account>,
    step_accounts: Vec<Option<StepAccount>>,
    starved: Option<Starved>,
    spawned_pids: VecDeque<Option<u32>>,
    server_pid: Option<u32>,
    browser_pid: Option<u32>,
}
impl Runtime {
    fn origins(setup: &Harness) -> Vec<crate::origins::Pages> {
        setup
            .origins
            .iter()
            .enumerate()
            .map(|(i, origin)| {
                crate::origins::Pages::numbered(
                    origin.pages.clone(),
                    Time::ZERO,
                    Wall::EPOCH,
                    u64::try_from(i).expect("origin fits"),
                )
            })
            .collect()
    }
    fn browser_spawn(command: browser::command::Command, root: kernel::Fd) -> Spawn {
        Spawn {
            reach: None,
            program: command.program,
            args: command.args.into_boxed(),
            env: command
                .env
                .iter()
                .map(|entry| {
                    let mut bytes = entry.name.to_vec();
                    bytes.push(b'=');
                    bytes.extend_from_slice(&entry.value);
                    bytes.into()
                })
                .collect(),
            root,
            dir: b".".as_slice().into(),
            pipes: command
                .pipes
                .into_iter()
                .map(|pipe| Pipe {
                    child: pipe.descriptor,
                    way: match pipe.direction {
                        browser::command::Direction::Read => Way::In,
                        browser::command::Direction::Write => Way::Out,
                    },
                    parent: None,
                })
                .collect(),
        }
    }
    fn new(setup: &Harness, people: &[&mut dyn Play], start: run::BrowserStart<'_>) -> (Self, u32) {
        let bl = browser::Limits::default();
        let capacity = capacity::Capacity::new(
            u32::try_from(people.len()).expect("person count fits"),
            u32::try_from(setup.origins.len()).expect("origin count fits"),
            setup.output,
            &bl,
        )
        .expect("harness capacity fits");
        let il = capacity.io;
        let mut startup = VecDeque::new();
        let (browser, roots) = match start {
            run::BrowserStart::Chromium { profile, root, browser_root } => {
                let command =
                    browser::command::command(setup.chromium.as_os_str().as_bytes(), profile.as_os_str().as_bytes());
                startup.push_back(io::Request::Spawn {
                    owner: PROCESS,
                    spawn: Self::browser_spawn(command, browser_root),
                });
                (BrowserEnd::Chromium(Box::new(Browser::new(BROWSER, &bl))), [Some(root), Some(browser_root)])
            }
            run::BrowserStart::Scripted(browser) => (BrowserEnd::Scripted(browser), [None, None]),
        };
        let mut runtime = Self {
            kept: Vec::new(),
            commands: (0..people.len()).map(|_| None).collect(),
            command_serial: 0,
            pending_commands: (0..people.len()).map(|_| None).collect(),
            held: vec![None; people.len()],
            raw_lines: Vec::new(),
            request_serial: 0,
            exchanges: Vec::new(),
            expected: people.iter().map(|play| play.expected()).collect(),
            origins: Self::origins(setup),
            browser,
            io: Io::new(&il),
            startup,
            peer_addresses: Vec::new(),
            benv: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: bl },
            ienv: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: il },
            complete: Queue::with_capacity(capacity.queue),
            submits: Queue::with_capacity(capacity.queue),
            events: Queue::with_capacity(capacity.browser_queue),
            io_events: Queue::with_capacity(capacity.queue),
            down: Queue::with_capacity(capacity.browser_queue),
            requests: VecDeque::new(),
            roots,
            pipes: None,
            server_pipes: Vec::new(),
            output: Vec::new(),
            lines: Vec::new(),
            closed: 0,
            spawned: 0,
            server_state: ServerState::Running,
            stderr: Vec::new(),
            server_stderr: Vec::new(),
            address: None,
            ready: false,
            context: false,
            opening: 0,
            opened: vec![false; people.len()],
            asked: String::new(),
            outcome: None,
            capture: None,
            snapshot: None,
            screenshot: None,
            closing: closing::Closing::new(),
            close_window: None,
            deadline: None,
            step_accounts: (0..people.len()).map(|_| None).collect(),
            starved: None,
            spawned_pids: VecDeque::new(),
            server_pid: None,
            browser_pid: None,
        };
        if !runtime.native() {
            runtime.events.push(Event::Ready { version: b"scripted".as_slice().into() });
        }
        (runtime, capacity.operations)
    }
    fn native(&self) -> bool {
        matches!(self.browser, BrowserEnd::Chromium(_))
    }
    fn ask(&mut self, request: Request) {
        self.asked = format!("{request:?}").chars().take(200).collect();
        match &mut self.browser {
            BrowserEnd::Chromium(browser) => {
                browser::down(browser, &self.benv, request, &mut self.events, &mut self.down);
            }
            BrowserEnd::Scripted(browser) => {
                self.context |= browser.request(request, self.benv.now, self.benv.limits.matches, &mut self.events);
            }
        }
    }
    fn read_server(&mut self, pipe: Token) {
        self.requests.push_back(io::Request::Stream {
            stream: pipe,
            down: stream::Down::Demand { read: stream::Read::Arrived { max: 32768 }, room: 0 },
        });
    }
    /// Where each person stood when the run's deadline passed.
    fn stuck(
        &self,
        people: &[&mut dyn Play],
        faces: &[BrowserFace<'_>],
        done: &[bool],
        began: bool,
        peers_empty: bool,
        operations_empty: bool,
    ) -> String {
        let each = people.iter().enumerate().map(|(index, play)| {
            format!(
                "\nperson {index}: step {}, opened {}, done {}, pending {}",
                play.at(),
                self.opened[index],
                done[index],
                faces.get(index).is_some_and(BrowserFace::pending)
            )
        });
        let closing = format!(
            "\nclosing {}: closed {} of {} spawned, io empty {}, ring empty {}, submits empty {}, origins closed {}, \
             peers empty {peers_empty}",
            self.closing.active(),
            self.closed,
            self.spawned,
            self.io.is_empty(),
            operations_empty,
            self.submits.is_empty(),
            self.origins.iter().all(crate::origins::Pages::is_closed),
        );
        format!(
            "browser harness deadline passed: began {began}, ready {}, server {:?}, outcome before {:?}, last request {}{}{closing}",
            self.ready,
            self.server_state,
            self.outcome,
            self.asked,
            each.collect::<String>()
        )
    }
    fn fail(&mut self, person: usize, step: u32, report: String) {
        if !matches!(self.outcome, Some(Outcome::Failed { .. })) {
            self.outcome = Some(Outcome::Failed { person: u32::try_from(person).expect("person fits"), step, report });
        }
    }
    fn expire(&mut self, person: usize, step: u32, report: &str, expiry: Expiry) {
        let expired = match expiry {
            Expiry::Native(expired) => expired,
            Expiry::Hosted { what, bound, waited } => {
                self.fail(
                    person,
                    step,
                    format!(
                        "{report}\n{what}: bound {} ns, waited {} ns in simulated time",
                        bound.as_nanos(),
                        waited.as_nanos(),
                    ),
                );
                return;
            }
        };
        let (figures, starved) = match &expired {
            Expired::Failed(figures) => (figures, false),
            Expired::Starved(starved) => (starved.figures(), true),
        };
        let decision = deadline_outcome(
            self.outcome.take(),
            u32::try_from(person).expect("person fits"),
            step,
            report,
            figures,
            starved,
        );
        self.outcome = Some(decision.outcome);
        if decision.close_starved
            && let Expired::Starved(starved) = expired
        {
            self.starved = Some(starved);
            if !self.closing.active() {
                self.close();
            }
        }
    }
    fn spawned_pid(&mut self, owner: Token) {
        if !self.native() {
            return;
        }
        let pid = self.spawned_pids.pop_front().expect("one pid lookup per successful spawn");
        if owner == SERVER || owner == PROCESS {
            if owner == SERVER {
                self.server_pid = pid;
            } else {
                self.browser_pid = pid;
            }
            if let Some(pid) = pid {
                self.deadline.as_mut().expect("run deadline armed").watch_process(pid);
                for account in self.step_accounts.iter_mut().flatten() {
                    account.deadline.watch_process(pid);
                }
                for command in self.commands.iter_mut().flatten() {
                    command.watch_process(pid);
                }
            }
        } else if let Some(index) =
            self.commands.iter().position(|command| command.as_ref().is_some_and(|command| command.owner == owner))
        {
            let command = self.commands[index].as_mut().expect("matching command");
            command.pid = pid;
            if let Some(pid) = pid {
                command.watch_process(pid);
                if let Some(account) = &mut self.step_accounts[index] {
                    account.deadline.watch_process(pid);
                }
            }
        }
    }
    fn origin_event(&mut self, event: io::Event) -> Option<io::Event> {
        for pages in &mut self.origins {
            let owner = match &event {
                io::Event::Listening { owner, .. }
                | io::Event::Accepted { owner, .. }
                | io::Event::Stream { owner, .. }
                | io::Event::Closed { owner }
                | io::Event::Failed { owner, .. } => Some(*owner),
                io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Output { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. } => None,
            };
            if owner.is_some_and(|owner| pages.owns(owner)) {
                pages.event(event);
                return None;
            }
        }
        Some(event)
    }
    fn read_output(&mut self, owner: Token, bytes: &[u8], setup: &Harness) {
        let selected = usize::from(setup.output == Output::Stdout);
        if self.server_pipes.get(selected) == Some(&owner) {
            self.output.extend_from_slice(bytes);
            self.raw_lines.extend(
                self.output
                    .split_inclusive(|byte| *byte == b'\n')
                    .filter(|line| {
                        line.ends_with(b"\n")
                            && !printed(
                                line,
                                &Printed {
                                    sent: setup.sent,
                                    prefix: &setup.prefix,
                                    on: setup.output,
                                    names: setup.names,
                                },
                            )
                            .is_empty()
                    })
                    .map(Box::from),
            );
            self.lines.extend(printed(
                &self.output,
                &Printed { sent: setup.sent, prefix: &setup.prefix, on: setup.output, names: setup.names },
            ));
            if let Some(end) = self.output.iter().rposition(|b| *b == b'\n') {
                self.output.drain(..=end);
            }
            if let Some(line) = self.lines.last() {
                for pages in &mut self.origins {
                    pages.address(&line.address);
                }
                self.address = Some(line.address.clone());
                self.server_state = ServerState::Running;
            }
        }
    }
    fn child_spawned(&mut self, owner: Token, pipes: Box<[Token]>) {
        self.spawned += 1;
        if owner == SERVER {
            self.server_pipes = pipes.into_vec();
            for pipe in self.server_pipes.clone() {
                self.read_server(pipe);
            }
        } else {
            let [commands, replies, errors] = pipes.as_ref() else { panic!("Chromium pipes") };
            self.pipes = Some([*commands, *replies, *errors]);
            self.close_root(1);
        }
    }
    fn io_event(&mut self, event: io::Event, setup: &Harness) {
        if let io::Event::Spawned { owner, .. } = &event {
            self.spawned_pid(*owner);
        }
        if self.command_event(&event) {
            let index = self
                .commands
                .iter()
                .position(|command| {
                    command
                        .as_ref()
                        .is_some_and(|command| command.owns(event_owner(&event).expect("command event owner")))
                })
                .expect("owned command event");
            self.commands[index].as_mut().expect("command").event(event);
            return;
        }
        let Some(event) = request::route(&mut self.exchanges, event).and_then(|event| self.origin_event(event)) else {
            return;
        };
        self.closing.io_event(&event, &mut self.requests);
        match event {
            io::Event::Spawned { owner, pipes, .. } => self.child_spawned(owner, pipes),
            io::Event::Stream { owner, up } => {
                if self.server_pipes.contains(&owner) {
                    if let stream::Up::Bytes(bytes) = up {
                        if self.server_pipes.first() == Some(&owner) {
                            tail(&mut self.server_stderr, &bytes);
                        }
                        self.read_output(owner, &bytes, setup);
                        self.read_server(owner);
                    }
                    return;
                }
                let Some([commands, replies, errors]) = self.pipes else { return };
                let below = if owner == commands {
                    Below::Commands(up)
                } else if owner == replies {
                    if let stream::Up::Bytes(bytes) = &up {
                        self.context |= bytes.windows(16).any(|part| part == b"browserContextId");
                    }
                    Below::Replies(up)
                } else if owner == errors {
                    if let stream::Up::Bytes(bytes) = &up {
                        tail(&mut self.stderr, bytes);
                    }
                    Below::Errors(up)
                } else {
                    return;
                };
                let BrowserEnd::Chromium(browser) = &mut self.browser else { unreachable!("only Chromium has pipes") };
                browser::up(browser, &self.benv, below, &mut self.events, &mut self.down);
            }
            io::Event::Exited { owner, exit } => {
                if !(self.closing.active() || owner == SERVER && self.server_state == ServerState::Stopping) {
                    self.fail(0, 0, format!("child {owner:?} exited: {exit:?}"));
                }
                if owner == PROCESS {
                    self.browser_pid = None;
                    if let Some(pipes) = self.pipes {
                        for entity in pipes {
                            self.requests.push_back(io::Request::Close { entity });
                        }
                    }
                } else {
                    self.server_pid = None;
                    for entity in &self.server_pipes {
                        self.requests.push_back(io::Request::Close { entity: *entity });
                    }
                }
            }
            io::Event::Closed { owner } => {
                if owner == PROCESS || owner == SERVER {
                    self.closed += 1;
                    if owner == SERVER {
                        self.server_state = ServerState::Stopped;
                    }
                }
            }
            io::Event::Failed { owner, error } => self.fail(0, 0, format!("io {owner:?}: {error:?}")),
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Output { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => {}
        }
    }
    fn event(&mut self, event: &Event, setup: &Harness, people: &[&mut dyn Play]) {
        match event {
            Event::Ready { .. } => {
                self.ready = true;
            }
            Event::Opened { page } => {
                let index = usize::try_from(page.raw() - 100).expect("page index");
                if let Some(opened) = self.opened.get_mut(index) {
                    *opened = true;
                }
                self.opening += 1;
                self.context = false;
                if self.opening < self.opened.len() {
                    self.ask(Request::Person {
                        person: person(self.opening),
                        downloads: setup.downloads.as_ref().map(|path| path.as_os_str().as_bytes().into()),
                    });
                }
            }
            Event::Trouble { page, trouble, text, url } => {
                let (person, step) = trouble_step(*page, people);
                if !expected_trouble(&setup.expected, text)
                    && !expected_trouble(self.expected.get(person).map_or(&[], Vec::as_slice), text)
                {
                    self.fail(person, step, trouble_report(*trouble, text, url));
                }
            }
            Event::Snapshot { op: SNAPSHOT, text } if self.capture_answer(SNAPSHOT) => {
                self.snapshot = Some(String::from_utf8_lossy(text).into_owned());
            }
            Event::Screenshot { op: IMAGE, png } if self.capture_answer(IMAGE) => {
                let path = std::env::temp_dir().join(format!(
                    "skein-ui-failure-{}-{}.png",
                    std::process::id(),
                    self.benv.now.as_nanos()
                ));
                if std::fs::write(&path, png).is_ok() {
                    self.screenshot = Some(path);
                }
            }
            Event::Refused { op, .. } if *op == SNAPSHOT || *op == IMAGE => {
                self.capture_answer(*op);
            }
            Event::Closed { owner } => self.closing.browser_closed(*owner, &mut self.requests),
            Event::Sent { .. }
            | Event::Answered { .. }
            | Event::Downloaded { .. }
            | Event::Warned { .. }
            | Event::Found { .. }
            | Event::Met { .. }
            | Event::Missed { .. }
            | Event::Done { .. }
            | Event::Refused { .. }
            | Event::Attribute { .. }
            | Event::Tabs { .. }
            | Event::Address { .. }
            | Event::Snapshot { .. }
            | Event::Screenshot { .. } => {}
        }
    }
    fn capture_answer(&mut self, op: Token) -> bool {
        !self.closing.active() && self.capture.as_mut().is_some_and(|capture| capture.answer(op, self.benv.now))
    }
    fn close_root(&mut self, index: usize) {
        if let Some(fd) = self.roots[index].take() {
            let op = if index == 0 { SERVER_ROOT } else { ROOT };
            self.submits.push(Submit { op, kind: Op::Close { fd } });
        }
    }
    fn close(&mut self) {
        self.close_window = Some(CloseWindow::new(self.benv.now, self.ienv.limits.close_timeout));
        let native = self.native();
        let close = self.closing.begin(!native || !self.startup.is_empty());
        for pages in &mut self.origins {
            pages.stop();
        }
        for exchange in &mut self.exchanges {
            exchange.stop();
        }
        for command in self.commands.iter_mut().flatten() {
            command.stop();
        }
        self.close_root(0);
        if let Some(close) = close {
            self.ask(close);
        } else {
            self.startup.clear();
            self.close_root(1);
            if !native {
                self.ask(Request::Close { entity: BROWSER });
            }
        }
    }

    fn server_action(&mut self, action: ServerAction, setup: &Harness) {
        match action {
            ServerAction::Stop => {
                assert!(self.server_state == ServerState::Running, "server running before stop");
                self.server_state = ServerState::Stopping;
                self.requests.push_back(io::Request::Signal {
                    child: self.closing.server_child().expect("server is running"),
                    signal: kernel::Signal::Terminate,
                    to: kernel::Target::Child,
                    reason: kernel::Reason::Unknown,
                });
            }
            ServerAction::Start => {
                assert!(self.server_state == ServerState::Stopped, "server stopped before start");
                self.server_state = ServerState::Starting;
                self.server_stderr.clear();
                self.output.clear();
                self.lines.clear();
                self.raw_lines.clear();
                let args =
                    restart(&setup.restart, self.address.as_deref().expect("server address"), &self.peer_addresses);
                self.requests.push_back(io::Request::Spawn {
                    owner: SERVER,
                    spawn: Spawn {
                        reach: None,
                        program: setup.server.as_os_str().as_bytes().into(),
                        args: args.into(),
                        env: Box::new([]),
                        root: self.roots[0].expect("launch root"),
                        dir: b".".as_slice().into(),
                        pipes: server_pipes(setup.output),
                    },
                });
            }
        }
    }
    fn play(
        &mut self,
        people: &mut [&mut dyn Play],
        faces: &mut [BrowserFace<'_>],
        done: &mut [bool],
        now: Time,
        setup: &Harness,
    ) {
        if matches!(self.server_state, ServerState::Stopping | ServerState::Starting) {
            return;
        }
        for (index, (play, face)) in people.iter_mut().zip(faces).enumerate() {
            if !self.opened[index] || done[index] {
                continue;
            }
            if self.exchanges.iter().filter(|exchange| exchange.person == index).count() >= 2 {
                continue;
            }
            for request in face.requests() {
                self.ask(request);
            }
            if face.pending() {
                continue;
            }
            let step = play.at();
            let command =
                self.commands[index].as_ref().filter(|command| !command.settled).and_then(|command| command.pid);
            let native = self.native();
            StepAccount::begin(
                &mut self.step_accounts[index],
                step,
                [self.server_pid, self.browser_pid, command],
                now,
                native,
            );
            let next = play.next(face, now);
            if let Some(failure) = face_failure(face, step) {
                self.step_failed(index, failure);
                break;
            }
            match next {
                Next::Do(doing) => {
                    if let Err(why) = face.perform(doing) {
                        self.fail(index, step, why);
                        break;
                    }
                }
                Next::Wait { .. } => {}
                Next::Done => {
                    if let Some(command) = &mut self.commands[index]
                        && command.exit.is_none()
                    {
                        let name = command.name.clone();
                        command.stop();
                        self.fail(index, play.at(), format!("command {name} still running at story end"));
                    } else {
                        done[index] = true;
                    }
                }
                Next::Failed { step, why } => {
                    self.fail(index, step, format!("{why:?}"));
                    break;
                }
            }
            if let Some(action) = face.command_action() {
                self.command_action(index, action, face, step);
            }
            if let Some(asked) = play.request() {
                self.exchanges.push(request::Exchange::new(
                    index,
                    self.request_serial,
                    asked,
                    self.address.as_ref().expect("server address"),
                    now,
                ));
                self.request_serial += 1;
            }
            if let Some(action) = play.server_action() {
                self.server_action(action, setup);
            }
            for request in face.requests() {
                self.ask(request);
            }
            if self.outcome.is_some() || matches!(self.server_state, ServerState::Stopping | ServerState::Starting) {
                break;
            }
        }
        if self.outcome.is_none() && done.iter().all(|done| *done) {
            self.outcome = Some(Outcome::Done);
        }
    }
    fn tick(&mut self, setup: &Harness, faces: &mut [BrowserFace<'_>], people: &[&mut dyn Play], now: Time) -> bool {
        let mut opened = false;
        io::stage::up(&mut self.io, &self.ienv, &mut self.complete, &mut self.io_events, &mut self.submits, now);
        while let Some(event) = self.io_events.pop() {
            self.io_event(event, setup);
        }
        match &mut self.browser {
            BrowserEnd::Chromium(browser) if self.pipes.is_some() => {
                for _ in 0..32 {
                    if !browser.work_pending() && browser.next_deadline().is_none_or(|at| at > now) {
                        break;
                    }
                    browser::fire(browser, &self.benv, now, &mut self.events, &mut self.down);
                }
            }
            BrowserEnd::Chromium(_) => {}
            BrowserEnd::Scripted(browser) => browser.fire(now, self.benv.limits.matches, &mut self.events),
        }
        while let Some(event) = self.events.pop() {
            for (index, face) in faces.iter_mut().enumerate() {
                let navigating = face.navigating();
                if !matches!(&event, Event::Downloaded { person: owner, .. } if *owner != person(index)) {
                    BrowserFace::event(face, &event);
                }
                if navigating
                    && !face.navigating()
                    && let Some(command) = &mut self.commands[index]
                    && command.opening()
                    && command.error.is_none()
                {
                    if let commands::Progress::Kept { name, value } = command.advance(now, false, face.failed()) {
                        self.kept.push(Kept { person: u32::try_from(index).expect("person fits"), name, value });
                    }
                    face.command_complete();
                }
                if let Some((name, value)) = face.kept() {
                    self.kept.push(Kept { person: u32::try_from(index).expect("person fits"), name, value });
                }
            }
            if let Event::Opened { .. } = event {
                opened = true;
            }
            self.event(&event, setup, people);
        }
        opened
    }
    fn open_page(&mut self, setup: &Harness, faces: &mut [BrowserFace<'_>], pending: bool) -> bool {
        if self.context
            && !pending
            && self.opening < faces.len()
            && let Some(line) = opening(&self.lines, setup.names, self.opening)
        {
            let raw_index = if setup.names == Names::Address { 0 } else { self.opening };
            self.held[self.opening] = self.raw_lines.get(raw_index).cloned();
            let url = if setup.sent == Sent::Command {
                "about:blank".as_bytes().into()
            } else {
                page_with_query(line.url.as_bytes(), &setup.page_query, setup.page_addresses)
            };
            faces[self.opening].first_url = String::from_utf8(url.to_vec()).expect("printed URL is UTF-8");
            self.ask(Request::Page { person: person(self.opening), page: page(self.opening), url, requests: false });
            return true;
        }
        false
    }
    fn faces(&self, setup: &Harness, count: usize) -> Vec<BrowserFace<'static>> {
        (0..count)
            .map(|index| {
                let mut face = BrowserFace::new(page(index), self.address.clone().expect("listening"));
                face.origins = setup
                    .origins
                    .iter()
                    .zip(&self.origins)
                    .map(|(origin, pages)| {
                        format!(
                            "http://{}:{}",
                            String::from_utf8_lossy(&origin.host),
                            pages.addr.expect("listening origin").port()
                        )
                    })
                    .collect();
                face
            })
            .collect()
    }
    fn origin_requests(&mut self, now: Time, wall: Wall) {
        for exchange in &mut self.exchanges {
            exchange.update(now, wall);
            self.requests.append(&mut exchange.requests);
        }
        for pages in &mut self.origins {
            pages.update_time(now, wall);
            while let Some(request) = pages.take() {
                self.requests.push_back(request);
            }
        }
    }
    fn close_point(&mut self, report: &str, classified: &mut bool) {
        if !*classified {
            *classified = true;
            if self.starved.is_none() && !matches!(self.outcome, Some(Outcome::Failed { .. })) {
                self.expire(0, 0, report, self.deadline.as_ref().expect("run deadline armed").expired(self.benv.now));
            }
        }
        if !self.closing.active() {
            self.close();
        }
        self.closing.force(&mut self.requests);
    }
    fn capture_or_close(&mut self, opening_page: bool, now: Time) {
        if self.closing.active() {
            return;
        }
        match self.outcome {
            Some(Outcome::Failed { person, .. }) => {
                let index = usize::try_from(person).expect("person fits");
                if self.opened.get(index) != Some(&true)
                    || self.capture.as_ref().is_some_and(|capture| capture.done(now))
                {
                    self.close();
                } else if self.capture.is_none() {
                    self.capture = Some(Capture::new(now));
                    let page = page(index);
                    self.ask(Request::Snapshot { page, op: SNAPSHOT });
                    self.ask(Request::Screenshot { page, op: IMAGE });
                }
            }
            Some(Outcome::Done) if !opening_page => self.close(),
            Some(Outcome::Done) | None => {}
        }
    }
    fn read_statuses(&mut self, people: &mut [&mut dyn Play]) {
        for exchange in &mut self.exchanges {
            if let Some(status) = exchange.status.take() {
                people[exchange.person].answered(status);
            }
        }
        self.exchanges.retain(|exchange| !exchange.closed);
    }
    fn check_steps(
        &mut self,
        people: &mut [&mut dyn Play],
        faces: &mut [BrowserFace<'_>],
        done: &[bool],
        now: Time,
    ) -> Option<(usize, StepFailure)> {
        let mut expiry = None;
        for (index, (play, face)) in people.iter_mut().zip(faces).enumerate() {
            if done[index] {
                continue;
            }
            let Some(account) = &self.step_accounts[index] else { continue };
            if let Some(failure) = step_failure(*play, face, now, account.step) {
                if failure.bound.is_some() {
                    expiry.get_or_insert((index, failure));
                } else {
                    self.fail(index, failure.step, failure.why);
                    break;
                }
            }
        }
        expiry
    }
    fn step_failed(&mut self, index: usize, failure: StepFailure) {
        let account = self.step_accounts[index].as_ref().expect("failed step has an account");
        if let Some(expired) =
            failure.classify(|bound| account.deadline.expired_for("the browser step", bound, self.benv.now))
        {
            self.expire(index, failure.step, &failure.why, expired);
        } else {
            self.fail(index, failure.step, failure.why);
        }
    }
    fn judge_steps(
        &mut self,
        setup: &Harness,
        people: &mut [&mut dyn Play],
        faces: &mut [BrowserFace<'_>],
        done: &[bool],
        now: Time,
    ) {
        self.read_statuses(people);
        let command_expiry = self.drive_commands(faces, people, now, &setup.page_query, setup.page_addresses);
        if self.outcome.is_none() {
            let step_expiry = self.check_steps(people, faces, done, now);
            if self.outcome.is_none() {
                if let Some((index, failure)) = step_expiry {
                    self.step_failed(index, failure);
                } else if let Some(expiry) = command_expiry {
                    self.expire(expiry.person, expiry.step, &expiry.report, expiry.deadline.expired(now));
                }
            }
        }
    }
    fn close_window_passed(&mut self, now: Time) -> bool {
        if (self.starved.is_some() || matches!(self.outcome, Some(Outcome::Failed { .. })))
            && let Some(window) = &self.close_window
        {
            if window.force(now) {
                self.closing.force(&mut self.requests);
            }
            window.done(now)
        } else {
            false
        }
    }
    fn flush(&mut self, started: bool, now: Time, wall: Wall) {
        self.closing.advance(&mut self.requests);
        while let Some(down) = self.down.pop() {
            let [commands, replies, errors] = self.pipes.expect("browser spawned");
            let (pipe, down) = match down {
                Down::Commands(d) => (commands, d),
                Down::Replies(d) => (replies, d),
                Down::Errors(d) => (errors, d),
            };
            self.requests.push_back(io::Request::Stream { stream: pipe, down });
        }
        if started {
            self.origin_requests(now, wall);
        }
        for command in self.commands.iter_mut().flatten() {
            self.requests.append(&mut command.requests);
        }
        while self.io.takes() {
            let Some(request) = self.requests.pop_front() else { break };
            io::down(&mut self.io, &self.ienv, request, &mut self.submits);
        }
        self.io.reclaim();
    }
    fn start(&mut self, setup: &Harness, peers: &peers::Peers<'_, '_>) {
        self.peer_addresses = peers.addresses();
        for pages in &mut self.origins {
            pages.peers(&self.peer_addresses);
        }
        self.requests.push_back(io::Request::Spawn {
            owner: SERVER,
            spawn: Spawn {
                reach: None,
                program: setup.server.as_os_str().as_bytes().into(),
                args: setup.args.iter().map(|arg| substitute(arg.as_bytes(), None, &self.peer_addresses)).collect(),
                env: Box::new([]),
                root: self.roots[0].expect("launch root"),
                dir: b".".as_slice().into(),
                pipes: server_pipes(setup.output),
            },
        });
        self.requests.append(&mut self.startup);
    }
    fn report(self) -> (Outcome, Vec<Kept>, Option<Starved>) {
        let mut outcome = self.outcome.expect("terminal outcome");
        if let Outcome::Failed { report, .. } = &mut outcome {
            write!(
                report,
                "\nSnapshot:\n{}\nScreenshot: {}\nChromium stderr:\n{}\nServer stderr:\n{}",
                self.snapshot.as_deref().unwrap_or("unavailable"),
                self.screenshot.as_ref().map_or_else(|| "unavailable".into(), |p| p.display().to_string()),
                String::from_utf8_lossy(&self.stderr),
                String::from_utf8_lossy(&self.server_stderr)
            )
            .expect("string write");
            for (index, command) in self.commands.iter().enumerate() {
                if let Some(command) = command {
                    write!(
                        report,
                        "\nPerson {index} command {} stderr:\n{}",
                        command.name,
                        String::from_utf8_lossy(&command.stderr)
                    )
                    .expect("string write");
                }
            }
        }
        (outcome, self.kept, self.starved)
    }
}
#[derive(Debug, PartialEq, Eq)]
struct DeadlineDecision {
    outcome: Outcome,
    close_starved: bool,
}
fn deadline_outcome(
    current: Option<Outcome>,
    person: u32,
    step: u32,
    report: &str,
    figures: &skein_world::Figures,
    starved: bool,
) -> DeadlineDecision {
    if let Some(failed @ Outcome::Failed { .. }) = current {
        DeadlineDecision { outcome: failed, close_starved: false }
    } else if starved {
        DeadlineDecision { outcome: Outcome::Done, close_starved: true }
    } else {
        let outcome = Outcome::Failed { person, step, report: format!("{report}\n{figures}") };
        DeadlineDecision { outcome, close_starved: false }
    }
}

/// io keeps process identity opaque; Linux exposes our owned child's PID on its pidfd.
fn pidfd_pid(pidfd: kernel::Fd) -> Option<u32> {
    let source = std::fs::read_to_string(format!("/proc/self/fdinfo/{}", pidfd.raw())).ok()?;
    parse_pidfd_pid(&source)
}
fn parse_pidfd_pid(source: &str) -> Option<u32> {
    for line in source.lines() {
        if let Some(pid) = line.strip_prefix("Pid:") {
            return pid.trim().parse::<u32>().ok().filter(|pid| *pid != 0);
        }
    }
    None
}

#[cfg(test)]
mod deadline_tests;
#[cfg(test)]
mod failure_tests;
fn page(index: usize) -> Token {
    Token::new(100 + u64::try_from(index).expect("person fits"))
}
fn person(index: usize) -> Token {
    Token::new(100_000 + u64::try_from(index).expect("person fits"))
}
fn tail(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(bytes);
    if buffer.len() > 32768 {
        buffer.drain(..buffer.len() - 32768);
    }
}
fn event_owner(event: &io::Event) -> Option<Token> {
    match event {
        io::Event::Listening { owner, .. }
        | io::Event::Accepted { owner, .. }
        | io::Event::Connecting { owner, .. }
        | io::Event::Connected { owner }
        | io::Event::Stream { owner, .. }
        | io::Event::Output { owner, .. }
        | io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Usage { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => Some(*owner),
        io::Event::Resized | io::Event::Window { .. } | io::Event::Shutdown { .. } => None,
    }
}

fn trouble_report(trouble: Trouble, text: &[u8], url: &[u8]) -> String {
    let mut report = format!("unexpected {trouble:?}: {}", String::from_utf8_lossy(text));
    if !url.is_empty() {
        write!(report, " ({})", String::from_utf8_lossy(url)).expect("string write");
    }
    report
}

fn expected_trouble(expected: &[Box<[u8]>], text: &[u8]) -> bool {
    expected.iter().any(|words| !words.is_empty() && text.windows(words.len()).any(|part| part == words.as_ref()))
}

fn trouble_step(page: Token, people: &[&mut dyn Play]) -> (usize, u32) {
    let person = usize::try_from(page.raw().saturating_sub(100)).unwrap_or(0);
    (person, people.get(person).map_or(0, |play| play.at()))
}
