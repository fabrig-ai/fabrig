//! A tab against the site's independent HTTP servers (playground.md, 4).
use super::{Route, config, server::Server};
use skein_io::kernel;
use skein_lib::{Queue, Time, Wall};
use skein_playground::tab::{Shown, Tab};
use skein_shell::{Delegate, Host};
use skein_world::{Memory, Referee, World};
use std::{collections::VecDeque, net::SocketAddr};

/// An input released after this many requests have reached the servers.
pub struct Point {
    pub requests: usize,
    /// None closes the tab.
    pub to: Option<Box<[u8]>>,
}
/// Outside observations expected from one visit, possibly replaced in flight.
pub struct Script {
    pub hosts: Vec<(SocketAddr, Vec<Route>)>,
    pub points: Vec<Point>,
    pub bound: u32,
    pub expected: Option<Shown>,
    pub requests: usize,
    pub until: Time,
}
struct Browser {
    tab: Tab,
    points: VecDeque<Option<Box<[u8]>>>,
    allowed: bool,
}
impl Host for Browser {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.allowed {
            match self.points.pop_front().expect("scripted input") {
                Some(address) => self.tab.point(&address),
                None => self.tab.close(),
            }
            self.allowed = false;
        }
        self.tab.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> { self.tab.completions() }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> { self.tab.submissions() }
    fn work_pending(&self, now: Time) -> bool { self.allowed || self.tab.work_pending(now) }
    fn next_mechanism_deadline(&self) -> Option<Time> { self.tab.next_mechanism_deadline() }
    fn next_policy_deadline(&self) -> Option<Time> { self.tab.next_policy_deadline() }
    fn is_empty(&self) -> bool { !self.allowed && self.points.is_empty() && self.tab.is_empty() }
    fn operations(&self) -> u32 { self.tab.operations() }
}
enum Process { Browser(Box<Browser>), Server(Box<Server>) }
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self { Self::Browser(browser) => browser.as_ref(), Self::Server(server) => server.as_ref() }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self { Self::Browser(browser) => browser.as_mut(), Self::Server(server) => server.as_mut() }
    }
}
struct Judge<'a> { script: &'a Script, points: VecDeque<usize>, passed: bool, now: Time, wake: bool }
fn browser(processes: &[Process]) -> &Browser {
    processes.iter().find_map(|p| if let Process::Browser(b) = p { Some(b.as_ref()) } else { None }).expect("tab")
}
fn requests(processes: &[Process]) -> usize {
    processes.iter().map(|p| if let Process::Server(s) = p { s.requests_seen } else { 0 }).sum()
}
impl Referee<Process> for Judge<'_> {
    fn act(&mut self, _: Time, processes: &mut [Process]) {
        let ready = processes.iter().all(|p| match p {
            Process::Server(server) => server.listening(), Process::Browser(_) => true,
        });
        if ready && self.points.front().is_some_and(|count| *count <= requests(processes)) {
            self.points.pop_front();
            for process in processes.iter_mut() {
                if let Process::Browser(browser) = process { browser.allowed = true; }
            }
        }
        if self.passed {
            for process in processes {
                if let Process::Server(server) = process { server.stop = true; }
            }
        }
    }
    fn watch(&mut self, now: Time, processes: &[Process]) -> Result<(), String> {
        self.now = now;
        let browser = browser(processes);
        let seen = requests(processes);
        self.wake = self.points.front().is_some_and(|count| *count <= seen)
            && processes.iter().all(|p| match p {
                Process::Server(server) => server.listening(), Process::Browser(_) => true,
            });
        if seen > self.script.requests { return Err(format!("unexpected request {seen}")); }
        if self.points.is_empty() && browser.is_empty() {
            if browser.tab.shown() != self.script.expected.as_ref() {
                return Err(format!("shown {:?}, expected {:?}", browser.tab.shown(), self.script.expected));
            }
            self.passed = seen == self.script.requests && now >= self.script.until;
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.passed || self.wake { Some(self.now) }
        else if self.script.until > self.now { Some(self.script.until) }
        else { Some(Time::from_nanos(2_000_000_000)) }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= Time::from_nanos(2_000_000_000) && !self.passed).then(|| "tab expectations unmet".into())
    }
    fn passed(&self) -> bool { self.passed }
}

/// Runs only simulated sockets, with the same memory checks as the visitor's tests.
pub fn run(script: &Script) {
    let judge = Judge { script, points: script.points.iter().map(|p| p.requests).collect(), passed: false, now: Time::ZERO, wake: false };
    let mut world = World::new(7, skein_sim::Config::calm(), judge, Memory::Checked);
    for (address, routes) in &script.hosts {
        world.spawn(|| {
            let mut config = config();
            config.listener = *address;
            Process::Server(Box::new(Server::new(&config, routes.clone())))
        });
    }
    world.spawn(|| {
        let config = config();
        Process::Browser(Box::new(Browser {
            tab: Tab::new(&config.io, &config.http, script.bound),
            points: script.points.iter().map(|p| p.to.clone()).collect(), allowed: false,
        }))
    });
    let outcome = world.run();
    assert!(outcome.iterations > 0);
}
