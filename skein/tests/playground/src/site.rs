//! The visitor against a scripted HTTP host (skein/docs/design/playground.md, 8).
mod server;
use server::Server;
use skein_http::{Header, client as http};
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_playground::visitor::{Config, Visitor};
use skein_shell::{Delegate, Host};
use skein_ui_web::{bridge::Input, fetch::Report};
use skein_world::{Memory, Outcome, Referee, World};
use std::net::{Ipv4Addr, SocketAddr};

/// Small limits shared by the visitor and its scripted peer.
#[must_use]
pub fn config() -> Config {
    Config {
        listener: SocketAddr::from((Ipv4Addr::LOCALHOST, 32198)),
        host: b"application.example:8080".as_slice().into(),
        exchanges: 2,
        cookies: 2,
        cookie_bytes: 32,
        io: io::Limits { sockets: 8, intake: 2048, receive: 256, output: 4096, ..io::Limits::TEST },
        http: http::Limits { request: 2048, head: 2048, headers: 16, read: 256, send: 256 },
    }
}

/// A route's body, independent of the visitor.
#[derive(Clone, Debug)]
pub enum Body {
    Fixed(Box<[u8]>),
    Headers,
    Echo,
    /// Each piece is sent after its delay from the response head; optionally never finishes.
    Pieces {
        pieces: Vec<(Duration, Box<[u8]>)>,
        finish: bool,
    },
}
#[derive(Clone, Debug)]
pub struct Route {
    pub path: Box<[u8]>,
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Body,
}
impl Route {
    #[must_use]
    pub fn new(path: &[u8], status: u16, body: Body) -> Self {
        Self { path: path.into(), status, headers: Vec::new(), body }
    }
}
/// Input is released once these outside observations and the instant have arrived.
#[derive(Clone, Debug)]
pub struct Stage {
    pub reports: usize,
    pub requests: usize,
    pub at: Time,
    pub inputs: Vec<Input>,
}
impl Stage {
    #[must_use]
    pub fn new(reports: usize, inputs: Vec<Input>) -> Self {
        Self { reports, requests: 0, at: Time::ZERO, inputs }
    }
}

/// A scenario of the scripted site and its referee.
pub struct Script {
    pub config: Config,
    pub routes: Vec<Route>,
    pub stages: Vec<Stage>,
    pub expected: Vec<Report>,
    pub listener: bool,
    pub closed: usize,
    pub until: Time,
    pub reset_at: Option<Time>,
}
impl Script {
    #[must_use]
    pub fn new(routes: Vec<Route>, stages: Vec<Stage>, expected: Vec<Report>) -> Self {
        Self {
            config: config(),
            routes,
            stages,
            expected,
            listener: true,
            closed: 0,
            until: Time::ZERO,
            reset_at: None,
        }
    }
}

/// Inputs are allocated and reports taken inside the process's metered turn.
pub struct Browser {
    visitor: Visitor,
    inputs: std::collections::VecDeque<Vec<Input>>,
    allowed: usize,
    pub reports: Vec<Report>,
}
impl Host for Browser {
    fn iterate(&mut self, now: Time, wall: Wall) {
        for _ in 0..self.allowed {
            for input in self.inputs.pop_front().expect("scripted input") {
                self.visitor.enter(input);
            }
        }
        self.allowed = 0;
        self.visitor.iterate(now, wall);
        self.visitor.take(&mut self.reports);
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        self.visitor.completions()
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        self.visitor.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.allowed > 0 || self.visitor.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.visitor.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.allowed == 0 && self.inputs.is_empty() && self.visitor.is_empty()
    }
    fn operations(&self) -> u32 {
        self.visitor.operations()
    }
}

pub enum Process {
    Browser(Box<Browser>),
    Server(Box<Server>),
}
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Browser(browser) => browser.as_ref(),
            Self::Server(server) => server.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Browser(browser) => browser.as_mut(),
            Self::Server(server) => server.as_mut(),
        }
    }
}
struct Judge {
    seed: u64,
    stages: std::collections::VecDeque<Stage>,
    expected: Vec<Report>,
    listener: bool,
    closed: usize,
    until: Time,
    passed: bool,
    now: Time,
    wake: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let ready = !self.listener
            || procs.iter().any(|process| match process {
                Process::Server(server) => server.listening(),
                Process::Browser(_) => false,
            });
        let reports = procs
            .iter()
            .find_map(|process| match process {
                Process::Browser(browser) => Some(browser.reports.len()),
                Process::Server(_) => None,
            })
            .expect("browser");
        let requests = procs
            .iter()
            .find_map(|process| match process {
                Process::Server(server) => Some(server.requests_seen),
                Process::Browser(_) => None,
            })
            .unwrap_or(0);
        if ready
            && self
                .stages
                .front()
                .is_some_and(|stage| stage.at <= now && stage.reports <= reports && stage.requests <= requests)
        {
            self.stages.pop_front();
            for process in procs.iter_mut() {
                if let Process::Browser(browser) = process {
                    browser.allowed = browser.allowed.checked_add(1).expect("finite stages");
                }
            }
        }
        if self.passed {
            for process in procs {
                if let Process::Server(server) = process {
                    server.stop = true;
                }
            }
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        self.now = now;
        let browser = procs
            .iter()
            .find_map(|process| match process {
                Process::Browser(browser) => Some(browser),
                Process::Server(_) => None,
            })
            .expect("browser");
        assert!(
            self.expected.starts_with(&browser.reports),
            "seed {}: unexpected reports {:?}, expected {:?}",
            self.seed,
            browser.reports,
            self.expected
        );
        if let Some(stage) = self.stages.front().filter(|stage| stage.at > now) {
            assert!(
                browser.reports.len() <= stage.reports,
                "seed {}: a report before the scripted read-on: {:?}",
                self.seed,
                browser.reports
            );
        }
        let closed = procs
            .iter()
            .find_map(|process| match process {
                Process::Server(server) => Some(server.closed),
                Process::Browser(_) => None,
            })
            .unwrap_or(0);
        let ready = !self.listener
            || procs.iter().any(|process| match process {
                Process::Server(server) => server.listening(),
                Process::Browser(_) => false,
            });
        let requests = procs
            .iter()
            .find_map(|process| match process {
                Process::Server(server) => Some(server.requests_seen),
                Process::Browser(_) => None,
            })
            .unwrap_or(0);
        self.wake = ready
            && self
                .stages
                .front()
                .is_some_and(|stage| stage.reports <= browser.reports.len() && stage.requests <= requests);
        self.passed = self.stages.is_empty()
            && browser.is_empty()
            && browser.reports == self.expected
            && closed >= self.closed
            && now >= self.until;
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.passed {
            None
        } else if self.wake {
            Some(self.stages.front().expect("stage to release").at.max(self.now))
        } else if self.until > self.now && self.stages.is_empty() {
            Some(self.until)
        } else {
            Some(Time::from_nanos(10_000_000_000))
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= Time::from_nanos(10_000_000_000) && !self.passed)
            .then(|| format!("seed {}: visitor expectations unmet", self.seed))
    }
    fn passed(&self) -> bool {
        self.passed
    }
}

/// Runs the visitor and independent server under the world's clock and heap accounts.
#[must_use]
pub fn run(seed: u64, script: &Script, memory: Memory) -> Outcome<Process> {
    let judge = Judge {
        seed,
        stages: script.stages.clone().into(),
        expected: script.expected.clone(),
        listener: script.listener,
        closed: script.closed,
        until: script.until,
        passed: false,
        now: Time::ZERO,
        wake: false,
    };
    let mut world = World::new(seed, skein_sim::Config::calm(), judge, memory);
    if script.listener {
        world.spawn(|| Process::Server(Box::new(Server::new(&script.config, script.routes.clone()))));
    }
    world.spawn(|| {
        Process::Browser(Box::new(Browser {
            visitor: Visitor::new(&script.config),
            inputs: script.stages.iter().map(|stage| stage.inputs.clone()).collect(),
            allowed: 0,
            reports: Vec::new(),
        }))
    });
    world.run_with_faults(|_, now, _| {
        script.reset_at.filter(|at| now >= *at).map(|_| skein_sim::Faults { reset: 1000, ..skein_sim::Faults::NONE })
    })
}
