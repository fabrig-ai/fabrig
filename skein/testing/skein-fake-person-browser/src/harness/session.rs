//! Incremental harness transport for a program with its own session policy
//! (ui/web/testing.md, 2). The caller supplies every writable directory.
use super::{BROWSER, PROCESS, capacity, ending_report};
use crate::BrowserFace;
use skein_browser::{
    self as browser, Browser,
    boundary::{Below, Down, Ending, Event, Request, Trouble},
};
use skein_io::{
    self as io, Io,
    kernel::{self, Complete, Fd, Pipe, Spawn, Submit, Way},
};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_shell::{Clock, Wait};
use skein_world::ring::Ring;
use std::{
    collections::VecDeque,
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
const PERSON: Token = Token::new(100_000);
const PAGE: Token = Token::new(100);
const ROOT_CLOSE: Token = Token::new(u64::MAX);
const TRANSPORT: usize = 0;
const EXTERNAL: usize = 1;
/// Page observations stay apart from transport failures (ui/web/testing.md, 2).
#[derive(Debug)]
pub enum Report {
    Io(io::Event),
    Opened,
    BrowserClosed,
    Page {
        trouble: Trouble,
        text: Box<[u8]>,
        url: Box<[u8]>,
    },
    Sent {
        id: Box<[u8]>,
        load: Box<[u8]>,
        document: bool,
        method: Box<[u8]>,
        url: Box<[u8]>,
    },
    Answered {
        id: Box<[u8]>,
    },
    /// A transport failure; a kit ending or unsolicited Chromium exit carries its stderr tail.
    Trouble {
        why: String,
        chromium_stderr: Option<Box<[u8]>>,
    },
}
impl Report {
    fn trouble(why: String) -> Self {
        Self::Trouble { why, chromium_stderr: None }
    }
}
fn page_report(event: Event) -> Result<Report, Event> {
    match event {
        Event::Trouble { trouble, text, url, .. } => Ok(Report::Page { trouble, text, url }),
        Event::Sent { id, load, document, method, url, .. } => Ok(Report::Sent { id, load, document, method, url }),
        Event::Answered { id, .. } => Ok(Report::Answered { id }),
        event => Err(event),
    }
}

fn session_page(url: Box<[u8]>) -> Request {
    Request::Page { person: PERSON, page: PAGE, url, requests: true }
}

// Each pop reserves the room for putting an ordinary completion back.
fn take_root_close(complete: &mut Queue<Complete>, reports: &mut Vec<Report>) {
    for _ in 0..complete.len() {
        let entry = complete.pop().expect("each queued completion is taken once");
        if entry.op == ROOT_CLOSE {
            if let Err(error) = entry.result {
                reports.push(Report::trouble(format!("closing session root: {error:?}")));
            }
        } else {
            complete.push(entry);
        }
    }
}

enum Opening {
    Idle,
    Starting(Box<[u8]>),
    Context(Box<[u8]>),
    Page,
    Open,
    Closing,
    Ended,
    Exited,
    Closed,
}
impl Opening {
    fn ended(&mut self, why: Ending, detail: &[u8], browser: &Browser, reports: &mut Vec<Report>) {
        if !matches!(self, Self::Ended | Self::Exited | Self::Closed) {
            *self = Self::Ended;
            reports.push(Report::Trouble {
                why: ending_report(why, detail),
                chromium_stderr: Some(browser.stderr_tail().into()),
            });
        }
    }

    fn exited(&mut self, exit: kernel::Exit, browser: &Browser, reports: &mut Vec<Report>) {
        if !matches!(self, Self::Closing | Self::Ended | Self::Exited | Self::Closed) {
            reports.push(Report::Trouble {
                why: format!("Chromium exited: {exit:?}"),
                chromium_stderr: Some(browser.stderr_tail().into()),
            });
        }
        *self = Self::Exited;
    }

    fn browser_closed(
        &self,
        owner: Token,
        child: Option<Token>,
        requests: &mut VecDeque<io::Request>,
        reports: &mut Vec<Report>,
    ) {
        if owner == BROWSER {
            if !matches!(self, Self::Exited | Self::Closed)
                && let Some(entity) = child
            {
                // io closes a child by killing its process group if it is still running.
                requests.push_back(io::Request::Close { entity });
            }
        } else if (owner == PERSON || owner == PAGE)
            && !matches!(self, Self::Closing | Self::Ended | Self::Exited | Self::Closed)
        {
            reports.push(Report::trouble("browser could not open the session page".into()));
        }
    }
}
/// One ring for a browser and caller-owned processes. No program, profile or
/// scratch directory is allocated implicitly; the caller owns deadline policy.
pub struct Pump {
    browser: Browser,
    io: Io,
    ring: Ring,
    clock: Clock,
    benv: Env<browser::Limits>,
    ienv: Env<io::Limits>,
    complete: Queue<Complete>,
    submits: Queue<Submit>,
    io_events: Queue<io::Event>,
    events: Queue<Event>,
    down: Queue<Down>,
    requests: VecDeque<io::Request>,
    reports: Vec<Report>,
    root: Option<Fd>,
    closing_root: bool,
    chromium: PathBuf,
    profile: PathBuf,
    environment: Vec<(OsString, OsString)>,
    child: Option<Token>,
    pipes: Option<[Token; 3]>,
    opening: Opening,
}
impl Pump {
    pub fn new(chromium: PathBuf, profile: PathBuf, environment: Vec<(OsString, OsString)>) -> Result<Self, String> {
        Self::with_external(chromium, profile, environment, 0)
    }
    /// Reserves raw operations for one caller-owned device on the same ring.
    pub fn with_external(
        chromium: PathBuf,
        profile: PathBuf,
        environment: Vec<(OsString, OsString)>,
        external: u32,
    ) -> Result<Self, String> {
        let bl = browser::Limits::default();
        let cap = capacity::Capacity::new(2, 0, super::Output::Stdout, &bl).ok_or("session capacity does not fit")?;
        let transport = cap.operations.checked_add(1).ok_or("session root capacity does not fit")?;
        let operations = transport.checked_add(external).ok_or("session operation capacity does not fit")?;
        let root = skein_shell::open_root(Path::new("/")).map_err(|error| format!("session child root: {error}"))?;
        let ring = match Ring::new(operations, vec![transport, external]) {
            Ok(ring) => ring,
            Err(error) => {
                skein_shell::close_startup_fd(root);
                return Err(format!("session ring: {error:?}"));
            }
        };
        Ok(Self {
            browser: Browser::new(BROWSER, &bl),
            io: Io::new(&cap.io),
            ring,
            clock: Clock::new(),
            benv: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: bl },
            ienv: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: cap.io },
            complete: Queue::with_capacity(cap.queue),
            submits: Queue::with_capacity(cap.queue),
            io_events: Queue::with_capacity(cap.queue),
            events: Queue::with_capacity(cap.browser_queue),
            down: Queue::with_capacity(cap.browser_queue),
            requests: VecDeque::new(),
            reports: vec![],
            root: Some(root),
            closing_root: false,
            chromium,
            profile,
            environment,
            child: None,
            pipes: None,
            opening: Opening::Idle,
        })
    }
    #[must_use]
    pub fn root(&self) -> Fd {
        self.root.expect("session root is open")
    }
    /// A spawn's `dir` beneath the session's root, `/`: the path's leading
    /// slashes dropped, since a spawn opens `dir` beneath its root and refuses
    /// an absolute one (runtime/io/processes.md, 3).
    #[must_use]
    pub fn beneath(path: &Path) -> Box<[u8]> {
        let bytes = path.as_os_str().as_bytes();
        let start = bytes.iter().position(|byte| *byte != b'/').unwrap_or(bytes.len());
        bytes[start..].into()
    }
    #[must_use]
    pub fn now(&self) -> Time {
        self.clock.now().now
    }
    pub fn request(&mut self, request: io::Request) {
        self.requests.push_back(request);
    }
    pub fn input(&mut self, fd: Fd) -> Result<Token, Fd> {
        self.io.adopt_read_pipe(fd)
    }
    fn chromium_spawn(chromium: &Path, profile: &Path, environment: &[(OsString, OsString)], root: Fd) -> Spawn {
        let command = browser::command::command(chromium.as_os_str().as_bytes(), profile.as_os_str().as_bytes());
        let mut env = environment.to_vec();
        // As in the browser tier, leave Chromium's singleton socket to the host's
        // temporary directory. A deep scratch directory can make its path too long.
        env.retain(|(key, _)| key != "TMPDIR");
        for entry in &command.env {
            env.retain(|(key, _)| key.as_bytes() != entry.name.as_ref());
            env.push((OsStr::from_bytes(&entry.name).into(), OsStr::from_bytes(&entry.value).into()));
        }
        Spawn {
            reach: None,
            program: command.program,
            args: command.args.into_boxed(),
            env: env.iter().map(|(key, value)| [key.as_bytes(), b"=", value.as_bytes()].concat().into()).collect(),
            root,
            dir: Self::beneath(profile),
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
    pub fn open(&mut self, url: Box<[u8]>, face: &mut BrowserFace<'_>) {
        if matches!(self.opening, Opening::Idle) {
            let spawn = Self::chromium_spawn(&self.chromium, &self.profile, &self.environment, self.root());
            self.opening = Opening::Starting(url);
            self.request(io::Request::Spawn { owner: PROCESS, spawn });
        } else {
            face.go_to(url);
        }
    }
    pub fn close_browser(&mut self) {
        match self.opening {
            Opening::Idle | Opening::Closed => {
                self.opening = Opening::Closed;
                self.reports.push(Report::BrowserClosed);
            }
            Opening::Starting(_) | Opening::Context(_) | Opening::Page | Opening::Open => {
                self.opening = Opening::Closing;
                self.ask(Request::Close { entity: BROWSER });
            }
            Opening::Closing | Opening::Ended | Opening::Exited => {}
        }
    }
    pub fn force_browser(&mut self) {
        match self.opening {
            Opening::Idle => self.opening = Opening::Closed,
            Opening::Starting(_) | Opening::Context(_) | Opening::Page | Opening::Open | Opening::Closing => {
                self.opening = Opening::Closing;
                if let Some(entity) = self.child {
                    self.request(io::Request::Close { entity });
                }
            }
            Opening::Ended | Opening::Exited | Opening::Closed => {}
        }
    }
    fn ask(&mut self, request: Request) {
        browser::down(&mut self.browser, &self.benv, request, &mut self.events, &mut self.down);
    }
    /// Queues a device's submission; its operation names are local to it.
    pub fn submit_external(&mut self, submit: Submit) {
        let submit = self.ring.register(EXTERNAL, submit);
        self.ring.queue(submit);
    }
    pub fn external_completions(&mut self, out: &mut Queue<Complete>) {
        self.ring.deliver(EXTERNAL, out);
    }
    /// After the device's owner cancels outstanding operations, give up their
    /// buffers before releasing the descriptors. Retain other hosts' completions.
    pub fn drain_external(&mut self, out: &mut Queue<Complete>) {
        loop {
            self.reap();
            self.external_completions(out);
            out.clear();
            if self.ring.count(EXTERNAL) == 0 {
                break;
            }
            self.ring.enter(Wait::Forever);
        }
    }
    fn reap(&mut self) {
        while let Some(complete) = self.ring.reap().pop() {
            let (host, complete) = self.ring.complete(complete);
            self.ring.returned(host, complete);
        }
    }
    /// Reap, advance and flush once. Wait for a completion or the earliest
    /// deadline only if neither the transport nor the caller has work to take.
    pub fn poll(&mut self, face: &mut BrowserFace<'_>, until: Option<Time>) -> Vec<Report> {
        self.reap();
        self.ring.deliver(TRANSPORT, &mut self.complete);
        let time = self.clock.now();
        self.benv.now = time.now;
        self.benv.wall = time.wall;
        self.ienv.now = time.now;
        self.ienv.wall = time.wall;
        take_root_close(&mut self.complete, &mut self.reports);
        io::stage::up(&mut self.io, &self.ienv, &mut self.complete, &mut self.io_events, &mut self.submits, time.now);
        let mut face_activity = false;
        while let Some(event) = self.io_events.pop() {
            // A pipe fault already observed by the kit takes precedence over a
            // subsequent process exit, including when both arrive in this poll.
            if matches!(event, io::Event::Exited { owner: PROCESS, .. }) && !self.events.is_empty() {
                face_activity |= self.browser_events(face, time.now);
            }
            self.io_event(event);
        }
        if self.browser_live() {
            browser::fire(&mut self.browser, &self.benv, time.now, &mut self.events, &mut self.down);
        }
        face_activity |= self.browser_events(face, time.now);
        // With no page yet, a settled context has no remaining command deadline.
        if matches!(self.opening, Opening::Context(_)) && self.browser.next_deadline().is_none() {
            let Opening::Context(url) = std::mem::replace(&mut self.opening, Opening::Page) else {
                unreachable!("context")
            };
            self.ask(session_page(url));
        }
        face.tick(time.now);
        for request in face.requests() {
            self.ask(request);
        }
        while let Some(down) = self.down.pop() {
            if self.browser_live()
                && let Some([commands, replies, errors]) = self.pipes
            {
                let (stream, down) = match down {
                    Down::Commands(d) => (commands, d),
                    Down::Replies(d) => (replies, d),
                    Down::Errors(d) => (errors, d),
                };
                self.request(io::Request::Stream { stream, down });
            }
        }
        while self.io.takes() {
            let Some(request) = self.requests.pop_front() else { break };
            io::down(&mut self.io, &self.ienv, request, &mut self.submits);
        }
        self.io.reclaim();
        self.browser.reclaim();
        if self.closing_root
            && self.io.is_empty()
            && self.requests.is_empty()
            && let Some(fd) = self.root.take()
        {
            self.submits.push(Submit { op: ROOT_CLOSE, kind: kernel::Op::Close { fd } });
        }
        while let Some(submit) = self.submits.pop() {
            let submit = self.ring.register(TRANSPORT, submit);
            self.ring.queue(submit);
        }
        let pending = self.io.is_ready()
            || (self.browser_live() && self.browser.work_pending())
            || !self.requests.is_empty()
            || !self.complete.is_empty()
            || self.ring.has_ready()
            || face_activity
            || face.failed()
            || !self.reports.is_empty();
        let deadline = self
            .io
            .next_deadline()
            .into_iter()
            .chain(self.browser_live().then(|| self.browser.next_deadline()).flatten())
            .chain(face.press_wake())
            .chain(until)
            .min();
        let wait = match deadline {
            _ if pending => Wait::No,
            Some(at) if at <= time.now => Wait::No,
            Some(at) => Wait::Until(at),
            None if self.ring.count(TRANSPORT) != 0 || self.ring.count(EXTERNAL) != 0 => Wait::Forever,
            None => Wait::No,
        };
        self.ring.enter(wait);
        std::mem::take(&mut self.reports)
    }
    fn browser_live(&self) -> bool {
        self.pipes.is_some() && !matches!(self.opening, Opening::Exited | Opening::Closed)
    }
    fn browser_events(&mut self, face: &mut BrowserFace<'_>, now: Time) -> bool {
        let mut activity = false;
        loop {
            while let Some(event) = self.events.pop() {
                activity = true;
                face.event_at(&event, now);
                self.browser_event(event);
            }
            if !self.browser_live() || !self.browser.work_pending() {
                return activity;
            }
            browser::fire(&mut self.browser, &self.benv, now, &mut self.events, &mut self.down);
        }
    }
    fn io_event(&mut self, event: io::Event) {
        match event {
            io::Event::Spawned { owner: PROCESS, child, pipes } => {
                self.child = Some(child);
                let [commands, replies, errors] = pipes.as_ref() else {
                    self.reports.push(Report::trouble("Chromium pipe layout".into()));
                    return;
                };
                self.pipes = Some([*commands, *replies, *errors]);
                if matches!(self.opening, Opening::Closing) {
                    self.request(io::Request::Close { entity: child });
                }
            }
            io::Event::Stream { owner, up } if self.pipes.is_some_and(|pipes| pipes.contains(&owner)) => {
                let [commands, replies, errors] = self.pipes.expect("browser pipes");
                if !self.browser_live() && owner != errors {
                    return;
                }
                let below = if owner == commands {
                    Below::Commands(up)
                } else if owner == replies {
                    Below::Replies(up)
                } else {
                    Below::Errors(up)
                };
                browser::up(&mut self.browser, &self.benv, below, &mut self.events, &mut self.down);
            }
            io::Event::Exited { owner: PROCESS, exit } => {
                self.opening.exited(exit, &self.browser, &mut self.reports);
                if let Some(pipes) = self.pipes {
                    for entity in pipes {
                        self.request(io::Request::Close { entity });
                    }
                }
                if let Some(child) = self.child {
                    self.request(io::Request::Close { entity: child });
                }
            }
            io::Event::Closed { owner: PROCESS } => {
                self.child = None;
                self.pipes = None;
                self.opening = Opening::Closed;
                self.reports.push(Report::BrowserClosed);
            }
            io::Event::Failed { owner: PROCESS, error } => {
                self.reports.push(Report::trouble(format!("Chromium: {error:?}")));
            }
            event => self.reports.push(Report::Io(event)),
        }
    }
    fn browser_event(&mut self, event: Event) {
        let event = match page_report(event) {
            Ok(report) => {
                self.reports.push(report);
                return;
            }
            Err(event) => event,
        };
        match event {
            Event::Ready { .. } => {
                if matches!(self.opening, Opening::Starting(_)) {
                    let Opening::Starting(url) = std::mem::replace(&mut self.opening, Opening::Idle) else {
                        unreachable!("starting session")
                    };
                    self.opening = Opening::Context(url);
                    self.ask(Request::Person {
                        person: PERSON,
                        downloads: Some(self.profile.as_os_str().as_bytes().into()),
                    });
                }
            }
            Event::Opened { page } => {
                if page == PAGE && matches!(self.opening, Opening::Page) {
                    self.opening = Opening::Open;
                    self.reports.push(Report::Opened);
                }
            }
            Event::Ended { why, detail } => self.opening.ended(why, &detail, &self.browser, &mut self.reports),
            Event::Closed { owner } => {
                self.opening.browser_closed(owner, self.child, &mut self.requests, &mut self.reports)
            }
            Event::Found { .. }
            | Event::Met { .. }
            | Event::Missed { .. }
            | Event::Warned { .. }
            | Event::Done { .. }
            | Event::Refused { .. }
            | Event::Snapshot { .. }
            | Event::Screenshot { .. }
            | Event::Address { .. }
            | Event::Tabs { .. }
            | Event::Attribute { .. }
            | Event::Downloaded { .. }
            | Event::Trouble { .. }
            | Event::Sent { .. }
            | Event::Answered { .. } => {}
        }
    }
    pub fn close_root(&mut self) {
        self.closing_root = true;
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        (!self.closing_root || self.root.is_none())
            && self.io.is_empty()
            && self.ring.is_empty()
            && self.submits.is_empty()
            && self.complete.is_empty()
            && self.requests.is_empty()
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
