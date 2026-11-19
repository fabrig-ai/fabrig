//! The web client's native page (temper/docs/design/playground.md, 6 and 9).

use jig_client_shell_web::{adapter, sign_in};
use skein_fake_person::{Doing, Face, Find, Found, Next, Person, TreeFace, Why};
use skein_lib::{Duration, Time};
use skein_playground::{memory::Memory, slices::Wake};
use skein_ui::{Act, NodeId, Role, Tree};
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
use temper_playground::{machine::Machine, programs, programs::Process, story};

// A whole story's HTTP inputs and stream read credits, with room for the end digest.
const LOG_BYTES: u32 = 1_048_576;
// Bound zero-time turns as well as world time when the host keeps asking at once.
const TURNS_AT_ONE_INSTANT: u32 = 128;

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
    memory: Memory<Process<crate::Journey>, Machine, Report>,
    inbox: Rc<Inbox<Report>>,
    store: Store,
    surface: skein_ui_web::page::Page<'static>,
    browser: Time,
    pending_input: bool,
    seed: u64,
    ways: temper_client_view::sign_in::Ways,
}

/// A bounded driver operation could not finish (journey-walks.md, 3.2).
#[derive(Debug, PartialEq, Eq)]
pub enum Failed {
    Bound,
    World(Box<[u8]>),
    Stopped,
    Turns,
    NoLink,
}

/// The person's terminal result.
#[derive(Debug, PartialEq, Eq)]
pub enum Walked {
    Done,
    Failed { step: u32, why: Why },
}

impl Page {
    /// Start the world, await its printed link, and spend it through the web host.
    pub fn start(start: story::Start, name: &[u8]) -> Result<Page, Failed> {
        let seed = start.seed;
        let wall = start.wall;
        let inbox = Inbox::new();
        let mut world = story::world_harness::<crate::Journey, _>(start, skein_sim_world::Bare);
        world.host(skein_sim_world::HostedProgram {
            program: crate::box_peer::PROGRAM.into(),
            make: crate::box_peer::BoxPeer::program,
            instances: 1,
            operations: 16,
        });
        let mut memory = Memory::new(
            skein_playground::slices::Slices::new(world, story::reach(), story::SLICE_STEPS, Time::ZERO),
            bridge::Start { story: name.into(), seed, start: wall },
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
                    Report::World(Heard::Failed { words }) => return Err(Failed::World(words)),
                    Report::World(Heard::Stopped) => return Err(Failed::Stopped),
                    Report::Fetch(_) | Report::Carried { .. } | Report::Due => {
                        return Err(Failed::World(Box::from(b"client report before the printed link".as_slice())));
                    }
                }
            }
            if let Some(link) = link {
                break link;
            }
            if memory.slices().world().steps() >= crate::STARTUP_STEPS {
                return Err(Failed::NoLink);
            }
            match wake {
                Wake::Now => {}
                Wake::At(at) => browser = at,
                Wake::Message => return Err(Failed::NoLink),
            }
        };
        let link = std::str::from_utf8(&link).map_err(|_| Failed::NoLink)?;
        let (_, path) = link.strip_prefix("http://").and_then(|link| link.split_once('/')).ok_or(Failed::NoLink)?;
        let path = format!("/{path}");
        let taken = sign_in::take(
            &sign_in::Route { path: b"/jig/v1/sign-in/link", page: None, home: b"/welcome" },
            path.as_bytes(),
        )
        .ok_or(Failed::NoLink)?;
        let (now, wall) = memory.clocks(browser);
        let ways = temper_client_view::sign_in::Ways::NONE;
        let mut host = host::Host::new(temper_client_shell_web::limits::client(), seed, now, wall, 0, ways.clone());
        let surface = host::page(&host.bounds.tree, Primary::Control);
        let store = Store::new(b"temper.", None);
        host.start(&taken.address, store.read_all(&memory), Some(taken.secret));
        let mut page = Page { host, memory, inbox, store, surface, browser, pending_input: false, seed, ways };
        page.give(host::Report::Start, false);
        page.turn()?;
        Ok(page)
    }

    /// The world's time at the last drawing.
    #[must_use]
    pub fn now(&self) -> Time {
        self.world().now()
    }

    /// The HTML of the last drawn page.
    #[must_use]
    pub fn html(&self) -> String {
        self.surface.html()
    }

    #[must_use]
    pub fn tree(&self) -> &Tree {
        self.host.tree()
    }

    /// Await a semantic target, optionally requiring its interactive semantics.
    pub fn wait(&mut self, find: &Find, act: Option<&Act>, within: Duration) -> Result<NodeId, Failed> {
        let (now, _) = self.memory.clocks(self.browser);
        let end = now.saturating_add(within);
        let mut instant = self.browser;
        let mut turns = 0;
        loop {
            if self.browser != instant {
                instant = self.browser;
                turns = 0;
            }
            turns += 1;
            if turns > TURNS_AT_ONE_INSTANT {
                return Err(Failed::Turns);
            }
            let parameters = story::parameters(self.world());
            let wake = programs::with(parameters, || self.memory.slice(self.browser));
            self.pending_input = false;
            self.turn()?;
            let mut face = TreeFace::new(self.tree());
            if let Found::Node(node) = face.find(find)
                && act.is_none_or(|act| face.allows(node, act))
            {
                return Ok(node);
            }
            let (now, _) = self.memory.clocks(self.browser);
            if now >= end {
                return Err(Failed::Bound);
            }
            if self.pending_input || wake == Wake::Now {
                continue;
            }
            let due = self.host.deadline().and_then(|at| self.memory.timer(at, self.browser));
            let world_due = match wake {
                Wake::At(at) => Some(at),
                Wake::Message => None,
                Wake::Now => unreachable!("busy slices continue at this browser instant"),
            };
            // The platform's timer refuses deadlines beyond the world's next due.
            // The driver's bound still competes with that earlier wake, at 1x.
            let bound = self.browser.saturating_add(end.saturating_since(now));
            let at = [world_due, due, Some(bound)].into_iter().flatten().min().ok_or(Failed::Bound)?;
            self.browser = at;
            if due.is_some_and(|due| due <= at) {
                self.inbox.raise(Report::Due);
            }
        }
    }

    /// Give an act to the shipped host and draw its result.
    pub fn act(&mut self, node: NodeId, act: Act) -> Result<(), Failed> {
        self.give(host::Report::Act { node, act }, false);
        self.turn()
    }

    /// Restart the host, retaining the world's cookies and the page's small store.
    pub fn reload(&mut self, path: &[u8], within: Duration) -> Result<(), Failed> {
        let (now, wall) = self.memory.clocks(self.browser);
        self.host =
            host::Host::new(temper_client_shell_web::limits::client(), self.seed, now, wall, 0, self.ways.clone());
        self.surface = host::page(&self.host.bounds.tree, Primary::Control);
        self.host.start(path, self.store.read_all(&self.memory), None);
        self.give(host::Report::Start, false);
        self.turn()?;
        self.wait(&Find::named(Role::Main, b""), None, within).map(|_| ())
    }

    /// Walk the script, moving browser time only to the world's, host's or person's next instant.
    pub fn walk(&mut self, person: &mut Person<Address>, bound: Duration) -> Walked {
        let end = self.world().now().saturating_add(bound);
        let mut instant = self.browser;
        let mut turns = 0;
        loop {
            if self.browser != instant {
                instant = self.browser;
                turns = 0;
            }
            turns += 1;
            if turns > TURNS_AT_ONE_INSTANT {
                self.failed(person.at(), "turn bound at one browser instant passed");
                panic!("story exceeded {TURNS_AT_ONE_INSTANT} turns at browser {instant:?}");
            }
            let parameters = story::parameters(self.world());
            let wake = programs::with(parameters, || self.memory.slice(self.browser));
            self.pending_input = false;
            self.walk_turn();
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
                    self.walk_turn();
                    continue;
                }
                Next::Do(Doing::Go { address }) => {
                    self.give(host::Report::Place { path: address::print(&address) }, false);
                    self.walk_turn();
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
            if person.at() != before || self.pending_input || wake == Wake::Now {
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

    fn turn(&mut self) -> Result<(), Failed> {
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
                Report::World(Heard::Stopped) => return Err(Failed::Stopped),
                Report::World(Heard::Failed { words }) => return Err(Failed::World(words)),
            };
            self.give(report, read_on);
        }
        self.host.draw(&mut self.surface, &mut Vec::new());
        Ok(())
    }

    fn walk_turn(&mut self) {
        match self.turn() {
            Ok(()) => {}
            Err(Failed::Stopped) => panic!("world stopped during the walk"),
            Err(Failed::World(words)) => panic!("world failed: {}", String::from_utf8_lossy(&words)),
            Err(Failed::Bound | Failed::Turns | Failed::NoLink) => unreachable!("turn only reports world failures"),
        }
    }

    fn give(&mut self, report: host::Report, read_on: bool) {
        let duties = self.host.report(report);
        if let Some(look) = duties.keep_look {
            // The native page keeps the look in its small store, alongside the session.
            self.store.write(temper_client_shell_web::records::LOOK_KEY, &look, &mut self.memory).ok();
        }
        let mut requests = adapter::requests(duties.asked);
        requests.navigate = duties.navigate;
        if let Some(to) = duties.replace {
            self.memory.navigate(&to);
        }
        self.pending_input |= !requests.exchanges.is_empty() || requests.stream.is_some() || read_on;
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
