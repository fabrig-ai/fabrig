//! The web client's native page (temper/docs/design/playground.md, 6 and 9).

use jig_client_shell_web::{adapter, sign_in};
use skein_fake_person::{Doing, Next, Person, TreeFace, Why};
use skein_lib::{Duration, Time, Wall};
use skein_playground::{memory::Memory, slices::Wake};
use skein_ui_web::{
    bridge, fetch,
    inbox::Inbox,
    mapping::Primary,
    platform::{Platform, worker::Heard},
    store::Store,
    transport,
};
use std::rc::Rc;
use temper_client_domain::{Address, address};
use temper_client_shell_web::host;
use temper_playground::{machine::Machine, programs, programs::Process, stories::Story, story};

// A whole story's HTTP inputs and stream read credits, with room for the end digest.
const LOG_BYTES: u32 = 1_048_576;

/// What enters the page's inbox: the bridge's raises and the page's own.
pub enum Report {
    Fetch(fetch::Report),
    World(Heard),
    Carried { reports: Vec<transport::Carried>, writes: u32 },
    Due,
}

/// One page's life: the story's world behind the bridge, the shipped host in front.
pub struct Page {
    host: host::Host,
    memory: Memory<Process<()>, Machine, Report>,
    inbox: Rc<Inbox<Report>>,
    store: Store,
    surface: skein_ui_web::page::Page<'static>,
    browser: Time,
}

/// The person's terminal result.
#[derive(Debug, PartialEq, Eq)]
pub enum Walked {
    Done,
    Failed { step: u32, why: Why },
}

impl Page {
    /// Start the world, await its printed link, and spend it through the web host.
    #[must_use]
    pub fn start(scenario: &Story, seed: u64, wall: Wall) -> Page {
        let inbox = Inbox::new();
        let mut memory = Memory::new(
            story::slices(scenario.start(seed, wall), story::SLICE_STEPS, Time::ZERO),
            bridge::Start { story: scenario.name.as_bytes().into(), seed, start: wall },
            LOG_BYTES,
            Rc::clone(&inbox),
            Report::Fetch,
            Report::World,
        );
        let mut browser = Time::ZERO;
        let link = loop {
            let parameters = story::parameters(memory.slices().world());
            let wake = programs::with(parameters, || memory.slice(browser));
            let mut link = None;
            while let Some(report) = inbox.next() {
                match report {
                    Report::World(Heard::Printed(printed)) => {
                        if printed.process.as_ref() == b"temper serve"
                            && printed.stream == bridge::Stream::Output
                            && skein_lib::bytes::find(&printed.line, b"/jig/v1/sign-in/link?secret=").is_some()
                        {
                            link = Some(printed.line);
                        }
                    }
                    Report::World(Heard::Clock) => {}
                    Report::World(Heard::Failed { words }) => {
                        panic!("world failed: {}", String::from_utf8_lossy(&words))
                    }
                    Report::World(Heard::Stopped) => panic!("world stopped before printing its link"),
                    Report::Fetch(_) | Report::Carried { .. } | Report::Due => {
                        panic!("no client reports before the printed link")
                    }
                }
            }
            if let Some(link) = link {
                break link;
            }
            assert!(memory.slices().world().steps() < super::STARTUP_STEPS, "no printed link within startup's bound");
            match wake {
                Wake::Now => {}
                Wake::At(at) => browser = at,
                Wake::Message => panic!("world waits without printing its link"),
            }
        };
        let link = std::str::from_utf8(&link).expect("printed link is UTF-8");
        let (_, path) = link.strip_prefix("http://").expect("plaintext world").split_once('/').expect("link path");
        let path = format!("/{path}");
        let taken =
            sign_in::take(&sign_in::Route { path: b"/jig/v1/sign-in/link", home: b"/welcome" }, path.as_bytes())
                .expect("the service printed a sign-in link");
        let (now, wall) = memory.clocks(browser);
        let mut host = host::Host::new(seed, now, wall, temper_client_view::sign_in::Ways::NONE);
        let surface = host::page(&host.bounds.tree, Primary::Control);
        let store = Store::new(b"temper.", None);
        host.start(&taken.address, store.read_all(&memory), Some(taken.secret));
        let mut page = Page { host, memory, inbox, store, surface, browser };
        page.give(host::Report::Start, false);
        page.turn();
        page
    }

    /// Walk the script, moving browser time only to the world's, host's or person's next instant.
    pub fn walk(&mut self, person: &mut Person<Address>, bound: Duration) -> Walked {
        let end = self.world().now().saturating_add(bound);
        loop {
            let parameters = story::parameters(self.world());
            let wake = programs::with(parameters, || self.memory.slice(self.browser));
            self.turn();
            let (now, _) = self.memory.clocks(self.browser);
            if now > end {
                self.failed(person.at(), "world time bound passed");
                panic!("story exceeded its world time bound at {now:?}");
            }
            let before = person.at();
            let next = person.next(&mut TreeFace::new(self.host.tree()), now);
            let until = match next {
                Next::Do(Doing::Act { node, act }) => {
                    self.give(host::Report::Act { node, act }, false);
                    self.turn();
                    continue;
                }
                Next::Do(Doing::Go { address }) => {
                    self.give(host::Report::Place { path: address::print(&address) }, false);
                    self.turn();
                    continue;
                }
                Next::Do(Doing::Reload) => panic!("the native walk has no reload"),
                Next::Done => return Walked::Done,
                Next::Failed { step, why } => {
                    self.failed(step, &format!("{why:?}"));
                    return Walked::Failed { step, why };
                }
                Next::Wait { until } => until,
            };
            if person.at() != before || wake == Wake::Now {
                continue;
            }
            let due = self.host.deadline().and_then(|at| self.memory.timer(at, self.browser));
            let person_due = until.and_then(|at| self.memory.timer(at, self.browser));
            let world_due = match wake {
                Wake::At(at) => Some(at),
                Wake::Message => None,
                Wake::Now => unreachable!("busy slices continue at this browser instant"),
            };
            let Some(at) = [world_due, due, person_due].into_iter().flatten().min() else {
                self.failed(person.at(), "no world, host or person wake");
                panic!("the person's walk has no wake");
            };
            self.browser = at;
            if due.is_some_and(|due| due <= at) {
                self.inbox.raise(Report::Due);
            }
        }
    }

    #[must_use]
    pub fn host(&self) -> &host::Host {
        &self.host
    }

    /// The session log so far, with native build `-`.
    #[must_use]
    pub fn log(&self) -> Box<[u8]> {
        self.memory.log()
    }

    /// The world for end checks, including the shared forge.
    #[must_use]
    pub fn world(&self) -> &crate::Playground {
        self.memory.slices().world()
    }

    fn turn(&mut self) {
        let (now, wall) = self.memory.clocks(self.browser);
        self.host.clocks(now, wall);
        while let Some(report) = self.inbox.next() {
            let read_on = matches!(
                &report,
                Report::Fetch(
                    fetch::Report::StreamOpened
                        | fetch::Report::StreamRefused { .. }
                        | fetch::Report::StreamPiece { .. }
                )
            );
            let report = match report {
                Report::Fetch(report) => host::Report::Heard(adapter::fetched(report)),
                Report::Carried { reports, writes } => host::Report::Carried { reports, writes },
                Report::Due => host::Report::Due,
                Report::World(Heard::Clock | Heard::Printed(_)) => continue,
                Report::World(Heard::Stopped) => panic!("world stopped during the walk"),
                Report::World(Heard::Failed { words }) => panic!("world failed: {}", String::from_utf8_lossy(&words)),
            };
            self.give(report, read_on);
        }
        self.host.draw(&mut self.surface, &mut Vec::new());
    }

    fn give(&mut self, report: host::Report, read_on: bool) {
        let duties = self.host.report(report);
        let mut requests = adapter::requests(duties.asked);
        requests.navigate = duties.navigate;
        if let Some(to) = duties.replace {
            self.memory.navigate(&to);
        }
        let mut reports = Vec::new();
        transport::carry(requests, &mut self.store, &mut self.memory, &mut reports);
        if duties.writes != 0 || !reports.is_empty() {
            self.inbox.raise(Report::Carried { reports, writes: duties.writes });
        }
        if read_on {
            self.memory.read_on();
        }
    }

    fn failed(&self, step: u32, why: &str) {
        eprintln!("story A at {} ns: person step {step}: {why}", self.world().now().as_nanos());
        for node in self.host.tree().nodes() {
            if let Some(name) = &node.name {
                eprintln!("story A page {:?}: {}", node.role, String::from_utf8_lossy(name));
            }
        }
    }
}
