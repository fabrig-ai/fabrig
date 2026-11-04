//! The site's processes composed as a product world (skein/docs/design/playground.md, 8).

#[path = "../../../src/site/server.rs"]
mod server;

use skein_io::kernel::{Complete, Submit};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_playground::{
    slices::Reach,
    visitor::{Config, Visitor},
};
use skein_playground_world::site::{self, Body, Route};
use skein_shell::{Delegate, Host};
use skein_sim_world::{Printer, Stream, World};
use skein_ui_web::{bridge::Input, fetch};
use std::io::Write;

pub enum Process {
    Visitor(Box<PrintingVisitor>),
    Site(Box<server::Server>),
}

pub struct PrintingVisitor {
    visitor: Visitor,
    prints: Option<(Printer, Printer)>,
}

impl Host for PrintingVisitor {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.visitor.iterate(now, wall);
    }
    fn drain(&mut self) {
        if let Some((mut output, mut error)) = self.prints.take() {
            output.write_all(b"visiting\nready\n").expect("short lines fit the printer");
            error.write_all(b"diagnostic\n").expect("short line fits the printer");
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.visitor.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.visitor.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.prints.is_some() || self.visitor.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.visitor.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.visitor.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.prints.is_none() && self.visitor.is_empty()
    }
    fn operations(&self) -> u32 {
        self.visitor.operations()
    }
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Visitor(visitor) => visitor.as_ref(),
            Self::Site(server) => server.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Visitor(visitor) => visitor.as_mut(),
            Self::Site(server) => server.as_mut(),
        }
    }
}

pub fn world(seed: u64, start: Wall) -> (World<Process>, Reach<Process>) {
    let config = site::config();
    let mut world = World::new(seed, skein_sim::Config { wall: start, ..skein_sim::Config::calm() });
    world.spawn(|| {
        Process::Site(Box::new(server::Server::new(
            &config,
            vec![Route::new(
                b"/",
                200,
                Body::Pieces {
                    pieces: vec![(Duration::from_nanos(200 + seed % 101), b"answer".as_slice().into())],
                    finish: true,
                },
            )],
        )))
    });
    let output = world.printer(b"visitor", Stream::Output);
    let error = world.printer(b"visitor", Stream::Error);
    let at = world.spawn(|| {
        Process::Visitor(Box::new(PrintingVisitor { visitor: Visitor::new(&config), prints: Some((output, error)) }))
    });
    (world, Reach { at, enter, take })
}

fn enter(process: &mut Process, input: Input) {
    match process {
        Process::Visitor(visitor) => visitor.visitor.enter(input),
        Process::Site(_) => unreachable!("Reach names the visitor"),
    }
}
fn take(process: &mut Process, reports: &mut Vec<fetch::Report>) {
    match process {
        Process::Visitor(visitor) => visitor.visitor.take(reports),
        Process::Site(_) => unreachable!("Reach names the visitor"),
    }
}

pub fn listening(world: &World<Process>) -> bool {
    world.processes().iter().any(|process| match process {
        Process::Site(server) => server.listening(),
        Process::Visitor(_) => false,
    })
}
