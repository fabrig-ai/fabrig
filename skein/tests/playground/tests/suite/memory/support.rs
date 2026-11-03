//! A native client's shell: its report type, inbox, transport and chosen browser instants.

use skein_lib::{Time, Wall};
use skein_playground::{
    memory::Memory,
    slices::{Slices, Wake},
};
use skein_playground_world::{
    native::{self, Process},
    site::{Body, Route},
};
use skein_sim_world::NoMachine;
use skein_ui_web::{
    bridge, fetch,
    inbox::Inbox,
    platform::worker::Heard,
    store::Store,
    transport::{self, Requests},
};
use std::{cell::RefCell, rc::Rc};

pub const SEEDS: [u64; 3] = [7, 19, 43];
// Enough steps for a short HTTP exchange per slice; busy work may span several slices.
pub const BUDGET: u32 = 64;
// A short session's textual inputs and digests fit with room for its header and end.
pub const LOG_BOUND: u32 = 16_384;
// A finite scripted exchange must finish within this many slices.
const SLICES: usize = 1000;
// The site's small response bodies fit the client's bound.
const ANSWER_BOUND: u32 = 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum ShellReport {
    Fetch(fetch::Report),
    World(Heard),
}

pub struct Shell {
    pub memory: Memory<Process, NoMachine, ShellReport>,
    pub store: Store,
    reported: Rc<RefCell<Vec<ShellReport>>>,
    browser: Time,
    seed: u64,
}

impl Shell {
    pub fn new(seed: u64, routes: Vec<Route>) -> Self {
        let start = start(seed);
        let (world, reach) = native::world(&start, routes);
        let inbox = Inbox::new();
        let reported = Rc::new(RefCell::new(Vec::new()));
        let reports = reported.clone();
        inbox.start(Box::new(move |inbox| {
            while let Some(report) = inbox.next() {
                reports.borrow_mut().push(report);
            }
        }));
        Self {
            memory: Memory::new(
                Slices::new(world, reach, BUDGET, Time::ZERO),
                start,
                LOG_BOUND,
                inbox,
                ShellReport::Fetch,
                ShellReport::World,
            ),
            store: Store::new(b"client/", None),
            reported,
            browser: Time::ZERO,
            seed,
        }
    }

    pub fn carry(&mut self, requests: Requests) {
        let mut reports = Vec::new();
        transport::carry(requests, &mut self.store, &mut self.memory, &mut reports);
        for report in reports {
            match report {
                transport::Carried::Written { .. } => {}
                transport::Carried::Refused { .. } | transport::Carried::Failed { .. } => {
                    panic!("seed {}: transport failed: {report:?}", self.seed)
                }
            }
        }
    }

    pub fn until_message(&mut self) {
        for _ in 0..SLICES {
            match self.memory.slice(self.browser) {
                Wake::Now => {}
                Wake::At(at) => self.browser = at,
                Wake::Message => return,
            }
        }
        panic!("seed {}: the site did not become idle", self.seed);
    }

    pub fn fetches(&self) -> Vec<fetch::Report> {
        self.reported
            .borrow_mut()
            .drain(..)
            .filter_map(|report| match report {
                ShellReport::Fetch(report) => Some(report),
                ShellReport::World(_) => None,
            })
            .collect()
    }
}

pub fn start(seed: u64) -> bridge::Start {
    bridge::Start { story: Box::from([]), seed, start: Wall::from_nanos(1_000_000) }
}

pub fn exchange(id: u32) -> fetch::Exchange {
    fetch::Exchange {
        id,
        path: b"/answer".as_slice().into(),
        method: "GET".into(),
        headers: Vec::new(),
        body: Box::from([]),
        bound: ANSWER_BOUND,
        cache: skein_ui_web::fetch::Cache::Browser,
        keep: &[],
    }
}

pub fn fixed() -> Route {
    Route::new(b"/answer", 201, Body::Fixed(b"the site's answer".as_slice().into()))
}

pub fn answer(exchange: u32) -> fetch::Report {
    fetch::Report::Answered { exchange, status: 201, body: b"the site's answer".as_slice().into(), headers: Vec::new() }
}
