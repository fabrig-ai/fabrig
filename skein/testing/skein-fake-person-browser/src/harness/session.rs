//! Incremental harness transport for a program with its own session policy
//! (ui/web/testing.md, 2). The caller supplies every writable directory.
use super::{BROWSER, PROCESS, capacity};
use crate::BrowserFace;
use skein_browser::{self as browser, Browser, boundary::{Below, Down, Event, Request}};
use skein_io::{self as io, Io, kernel::{self, Complete, Fd, Pipe, Spawn, Submit, Way}};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_shell::{Clock, Wait};
use skein_world::ring::Ring;
use std::{collections::VecDeque, ffi::{OsStr, OsString}, os::unix::ffi::OsStrExt, path::{Path, PathBuf}};
const PERSON: Token = Token::new(100_000);
const PAGE: Token = Token::new(100);
const ROOT_CLOSE: Token = Token::new(u64::MAX);
#[derive(Debug)]
pub enum Report { Io(io::Event), Opened, BrowserClosed, Trouble(String) }
enum Opening { Idle, Starting(Box<[u8]>), Context(Box<[u8]>), Page, Open, Closing, Closed }
/// One ring for a browser and caller-owned processes. No program, profile or
/// scratch directory is allocated implicitly; the caller owns deadline policy.
pub struct Pump {
    browser: Browser, io: Io, ring: Ring, clock: Clock,
    benv: Env<browser::Limits>, ienv: Env<io::Limits>, complete: Queue<Complete>, submits: Queue<Submit>,
    io_events: Queue<io::Event>, events: Queue<Event>, down: Queue<Down>, requests: VecDeque<io::Request>, reports: Vec<Report>,
    root: Option<Fd>, chromium: PathBuf, profile: PathBuf, environment: Vec<(OsString, OsString)>,
    child: Option<Token>, pipes: Option<[Token; 3]>, opening: Opening,
}
impl Pump {
    pub fn new(chromium: PathBuf, profile: PathBuf, environment: Vec<(OsString, OsString)>) -> Result<Self, String> {
        let bl = browser::Limits::default();
        let cap = capacity::Capacity::new(2, 0, super::Output::Stdout, &bl).ok_or("session capacity does not fit")?;
        let operations = cap.operations.checked_add(1).ok_or("session root capacity does not fit")?;
        let root = skein_shell::open_root(Path::new("/")).map_err(|error| format!("session child root: {error}"))?;
        let ring = match Ring::new(operations, vec![operations]) { Ok(ring) => ring, Err(error) => { skein_shell::close_startup_fd(root); return Err(format!("session ring: {error:?}")); } };
        Ok(Self { browser: Browser::new(BROWSER, &bl), io: Io::new(&cap.io), ring, clock: Clock::new(), benv: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: bl }, ienv: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: cap.io }, complete: Queue::with_capacity(cap.queue), submits: Queue::with_capacity(cap.queue), io_events: Queue::with_capacity(cap.queue), events: Queue::with_capacity(cap.browser_queue), down: Queue::with_capacity(cap.browser_queue), requests: VecDeque::new(), reports: vec![], root: Some(root), chromium, profile, environment, child: None, pipes: None, opening: Opening::Idle })
    }
    #[must_use]
    pub fn root(&self) -> Fd { self.root.expect("session root is open") }
    #[must_use]
    pub fn now(&self) -> Time { self.clock.now().now }
    pub fn request(&mut self, request: io::Request) { self.requests.push_back(request); }
    pub fn input(&mut self, fd: Fd) -> Result<Token, Fd> { self.io.adopt_read_pipe(fd) }
    pub fn open(&mut self, url: Box<[u8]>, face: &mut BrowserFace<'_>) {
        if matches!(self.opening, Opening::Idle) {
            let command = browser::command::command(self.chromium.as_os_str().as_bytes(), self.profile.as_os_str().as_bytes());
            let mut env = self.environment.clone();
            for entry in &command.env { env.retain(|(key, _)| key.as_bytes() != entry.name.as_ref()); env.push((OsStr::from_bytes(&entry.name).into(), OsStr::from_bytes(&entry.value).into())); }
            let spawn = Spawn {
                program: command.program, args: command.args.into_boxed(), env: env.iter().map(|(key, value)| [key.as_bytes(), b"=", value.as_bytes()].concat().into()).collect(),
                root: self.root(), dir: self.profile.as_os_str().as_bytes().into(),
                pipes: command.pipes.into_iter().map(|pipe| Pipe { child: pipe.descriptor, way: match pipe.direction { browser::command::Direction::Read => Way::In, browser::command::Direction::Write => Way::Out }, parent: None }).collect(),
            };
            self.opening = Opening::Starting(url); self.request(io::Request::Spawn { owner: PROCESS, spawn });
        } else { face.go_to(url); }
    }
    pub fn close_browser(&mut self) {
        if matches!(self.opening, Opening::Idle | Opening::Closed) { self.opening = Opening::Closed; self.reports.push(Report::BrowserClosed); }
        else { self.opening = Opening::Closing; self.ask(Request::Close { entity: BROWSER }); }
    }
    pub fn force_browser(&mut self) { if let Some(child) = self.child { self.request(io::Request::Close { entity: child }); } }
    fn ask(&mut self, request: Request) { browser::down(&mut self.browser, &self.benv, request, &mut self.events, &mut self.down); }
    /// Reap, advance and flush once. Wait at most ten ms, leaving the caller's
    /// own deadlines and input live. Raw io observations belong to the caller.
    pub fn poll(&mut self, face: &mut BrowserFace<'_>, until: Option<Time>) -> Vec<Report> {
        while let Some(complete) = self.ring.reap().pop() { let (host, complete) = self.ring.complete(complete); self.ring.returned(host, complete); }
        self.ring.deliver(0, &mut self.complete);
        let time = self.clock.now(); self.benv.now = time.now; self.benv.wall = time.wall; self.ienv.now = time.now; self.ienv.wall = time.wall;
        let mut ordinary = Queue::with_capacity(0);
        while let Some(complete) = self.complete.pop() {
            if complete.op == ROOT_CLOSE { if let Err(error) = complete.result { self.reports.push(Report::Trouble(format!("closing session root: {error:?}"))); } }
            else { ordinary.push(complete); }
        }
        io::stage::up(&mut self.io, &self.ienv, &mut ordinary, &mut self.io_events, &mut self.submits, time.now);
        while let Some(event) = self.io_events.pop() { self.io_event(event); }
        if self.pipes.is_some() { browser::fire(&mut self.browser, &self.benv, time.now, &mut self.events, &mut self.down); }
        while let Some(event) = self.events.pop() { face.event(&event); self.browser_event(event); }
        // With no page yet, a settled context has no remaining command deadline.
        if matches!(self.opening, Opening::Context(_)) && self.browser.next_deadline().is_none() {
            let Opening::Context(url) = std::mem::replace(&mut self.opening, Opening::Page) else { unreachable!("context") };
            self.ask(Request::Page { person: PERSON, page: PAGE, url });
        }
        face.tick(time.now); for request in face.requests() { self.ask(request); }
        while let Some(down) = self.down.pop() {
            if let Some([commands, replies, errors]) = self.pipes {
                let (stream, down) = match down { Down::Commands(d) => (commands, d), Down::Replies(d) => (replies, d), Down::Errors(d) => (errors, d) };
                self.request(io::Request::Stream { stream, down });
            }
        }
        while self.io.takes() { let Some(request) = self.requests.pop_front() else { break }; io::down(&mut self.io, &self.ienv, request, &mut self.submits); }
        self.io.reclaim(); self.browser.reclaim();
        while let Some(submit) = self.submits.pop() { let submit = self.ring.register(0, submit); self.ring.queue(submit); }
        let pending = self.io.is_ready() || self.browser.work_pending() || !self.requests.is_empty() || self.ring.has_ready();
        let deadline = self.io.next_deadline().into_iter().chain(self.browser.next_deadline()).chain(until).fold(time.now.saturating_add(Duration::from_millis(10)), Time::min);
        self.ring.enter(if pending || !self.reports.is_empty() { Wait::No } else { Wait::Until(deadline) });
        std::mem::take(&mut self.reports)
    }
    fn io_event(&mut self, event: io::Event) {
        match event {
            io::Event::Spawned { owner: PROCESS, child, pipes } => { self.child = Some(child); let [commands, replies, errors] = pipes.as_ref() else { self.reports.push(Report::Trouble("Chromium pipe layout".into())); return; }; self.pipes = Some([*commands, *replies, *errors]); }
            io::Event::Stream { owner, up } if self.pipes.is_some_and(|pipes| pipes.contains(&owner)) => {
                let [commands, replies, _] = self.pipes.expect("browser pipes");
                let below = if owner == commands { Below::Commands(up) } else if owner == replies { Below::Replies(up) } else { Below::Errors(up) };
                browser::up(&mut self.browser, &self.benv, below, &mut self.events, &mut self.down);
            }
            io::Event::Exited { owner: PROCESS, exit } => {
                if !matches!(self.opening, Opening::Closing) { self.reports.push(Report::Trouble(format!("Chromium exited: {exit:?}"))); }
                if let Some(pipes) = self.pipes { for entity in pipes { self.request(io::Request::Close { entity }); } }
                if let Some(child) = self.child { self.request(io::Request::Close { entity: child }); }
            }
            io::Event::Closed { owner: PROCESS } => { self.child = None; self.opening = Opening::Closed; self.reports.push(Report::BrowserClosed); }
            io::Event::Failed { owner: PROCESS, error } => self.reports.push(Report::Trouble(format!("Chromium: {error:?}"))),
            event => self.reports.push(Report::Io(event)),
        }
    }
    fn browser_event(&mut self, event: Event) {
        match event {
            Event::Ready { .. } => {
                if let Opening::Starting(url) = std::mem::replace(&mut self.opening, Opening::Idle) { self.opening = Opening::Context(url); self.ask(Request::Person { person: PERSON, downloads: Some(self.profile.as_os_str().as_bytes().into()) }); }
            }
            Event::Opened { page: PAGE } => { self.opening = Opening::Open; self.reports.push(Report::Opened); }
            Event::Closed { owner: BROWSER } => { if let Some(child) = self.child { self.request(io::Request::Close { entity: child }); } }
            Event::Closed { owner: PERSON | PAGE } if !matches!(self.opening, Opening::Closing | Opening::Closed) => self.reports.push(Report::Trouble("browser could not open the session page".into())),
            Event::Trouble { trouble, text, .. } => self.reports.push(Report::Trouble(format!("{trouble:?}: {}", String::from_utf8_lossy(&text)))), _ => {},
        }
    }
    pub fn close_root(&mut self) { if let Some(fd) = self.root.take() { self.submits.push(Submit { op: ROOT_CLOSE, kind: kernel::Op::Close { fd } }); } }
    #[must_use]
    pub fn is_empty(&self) -> bool { self.io.is_empty() && self.ring.is_empty() && self.submits.is_empty() && self.requests.is_empty() }
}
