//! A scripted stand-in for the visitor, for the slices' tests.
//! The world and its trace follow skein/docs/design/playground.md, 8.

use std::{collections::VecDeque, io::Write};

use skein_io::kernel::{Complete, Submit};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_playground::slices::Reach;
use skein_shell::{Delegate, Host};
use skein_sim::{Config, Trace};
use skein_sim_world::{Digests, Printer, Stream, World};
use skein_ui_web::{bridge::Input, fetch};

/// One input's work before its answer.
#[derive(Clone, Copy, Debug, Hash)]
pub struct Plan {
    pub steps: u32,
    pub delay: Duration,
}

#[derive(Debug, Hash)]
struct Job {
    exchange: u32,
    left: u32,
    delay: Duration,
    due: Option<Time>,
}

/// The visitor and a process that prints on both streams.
#[derive(Debug)]
pub enum Process {
    Visitor(StandIn),
    Printing(StandIn),
}

/// A scripted host: no private loop or clock.
#[derive(Debug)]
pub struct StandIn {
    seed: u64,
    plans: VecDeque<Plan>,
    jobs: VecDeque<Job>,
    reports: Vec<fetch::Report>,
    answered: Vec<(u32, Time)>,
    closing: bool,
    entered: bool,
    close: bool,
    output: Option<(Printer, Printer)>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}

impl StandIn {
    fn new(seed: u64, plans: &[Plan]) -> Self {
        Self {
            seed,
            plans: plans.iter().copied().collect(),
            jobs: VecDeque::new(),
            reports: Vec::new(),
            answered: Vec::new(),
            closing: false,
            entered: false,
            close: false,
            output: None,
            completions: Queue::with_capacity(1),
            submissions: Queue::with_capacity(1),
        }
    }

    fn enter(&mut self, input: Input) {
        self.entered = true;
        match input {
            Input::Exchange(exchange) => {
                let plan =
                    self.plans.pop_front().unwrap_or_else(|| unreachable!("seed {}: one plan per input", self.seed));
                self.jobs.push_back(Job { exchange: exchange.id, left: plan.steps, delay: plan.delay, due: None });
            }
            Input::Abandon { .. } => self.close = true,
            Input::ReadOn | Input::Stream(_) => {
                unreachable!("seed {}: the script accepts exchanges and a closing abandon", self.seed)
            }
        }
    }
}

impl Host for StandIn {
    fn state_digest(&self) -> Option<[u8; 32]> {
        Some(skein_lib::state_digest(&(&self.jobs, &self.answered, self.closing, self.entered, self.output.is_some())))
    }

    fn iterate(&mut self, now: Time, _wall: Wall) {
        self.entered = false;
        if self.close {
            self.closing = true;
        }
        if let Some(job) = self.jobs.front_mut() {
            if job.due.is_none() && job.delay != Duration::ZERO {
                job.due = Some(now.saturating_add(job.delay));
            }
            if job.due.is_some_and(|due| due > now) {
                return;
            }
            job.left = job.left.saturating_sub(1);
            if job.left == 0 {
                let job = self.jobs.pop_front().expect("the front job");
                self.answered.push((job.exchange, now));
                self.reports.push(fetch::Report::Answered {
                    exchange: job.exchange,
                    status: 200,
                    body: Box::from(&b"answer"[..]),
                    headers: Vec::new(),
                });
            }
        }
    }

    fn drain(&mut self) {
        if let Some((mut output, mut error)) = self.output.take() {
            output.write_all(b"one\ntwo\n").expect("seeded printer has room");
            error.write_all(b"error\n").expect("seeded printer has room");
        }
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }

    fn work_pending(&self, now: Time) -> bool {
        self.entered
            || self.output.is_some()
            || self.jobs.front().is_some_and(|job| job.due.is_none_or(|due| due <= now))
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.jobs.front().and_then(|job| job.due)
    }

    fn is_empty(&self) -> bool {
        self.closing && self.jobs.is_empty() && self.output.is_none()
    }

    fn operations(&self) -> u32 {
        1
    }
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Visitor(host) | Self::Printing(host) => host,
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Visitor(host) | Self::Printing(host) => host,
        }
    }
}

/// A calm world, visitor first, trace whole, optionally a printing process.
#[must_use]
pub fn world(seed: u64, plans: &[Plan], printing: bool) -> (World<Process>, Reach<Process>) {
    let mut world = World::new(seed, Config { trace: Trace::Whole, ..Config::calm() });
    world.digests(Digests::Traced);
    let at = world.spawn(|| Process::Visitor(StandIn::new(seed, plans)));
    if printing {
        let output = world.printer(b"script", Stream::Output);
        let error = world.printer(b"script", Stream::Error);
        world.spawn(|| {
            let mut host = StandIn::new(seed, &[]);
            host.closing = true;
            host.output = Some((output, error));
            Process::Printing(host)
        });
    }
    (world, Reach { at, enter, take })
}

/// The stand-in's visitor input seam.
pub fn enter(process: &mut Process, input: Input) {
    match process {
        Process::Visitor(host) => host.enter(input),
        Process::Printing(_) => unreachable!("Reach names the visitor"),
    }
}

/// The stand-in's reports in their order.
pub fn take(process: &mut Process, reports: &mut Vec<fetch::Report>) {
    match process {
        Process::Visitor(host) => reports.append(&mut host.reports),
        Process::Printing(_) => unreachable!("Reach names the visitor"),
    }
}

/// A fetch input identified by its script's exchange number.
#[must_use]
pub fn input(id: u32) -> Input {
    Input::Exchange(skein_ui_web::bridge::Exchange {
        id,
        path: Box::from(&b"/"[..]),
        method: String::from("GET"),
        headers: Vec::new(),
        body: Box::from([]),
        bound: 16,
        cache: fetch::Cache::Browser,
        keep: Vec::new(),
    })
}
