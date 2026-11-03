//! The site's processes composed for a native client (skein/docs/design/playground.md, 5, 8).

// The same scripted HTTP server runs under both world loops.
#[expect(clippy::duplicate_mod, reason = "the site's private server is shared by the native composition")]
#[path = "site/server.rs"]
mod server;

use crate::site::{Body, Route, config};
use server::Server;
use skein_playground::{
    slices::Reach,
    visitor::{Config, Visitor},
};
use skein_shell::{Delegate, Host};
use skein_sim_world::World;
use skein_ui_web::{bridge, fetch};

/// The visitor and the site's independent HTTP server.
pub enum Process {
    Visitor(Box<Visitor>),
    Server(Box<Server>),
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Visitor(visitor) => visitor.as_ref(),
            Self::Server(server) => server.as_ref(),
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Visitor(visitor) => visitor.as_mut(),
            Self::Server(server) => server.as_mut(),
        }
    }
}

/// A calm site world, with the listener ready before the client sends its first duty.
#[must_use]
pub fn world(start: &bridge::Start, routes: Vec<Route>) -> (World<Process>, Reach<Process>) {
    let config = config();
    let mut world = World::new(start.seed, skein_sim::Config { wall: start.start, ..skein_sim::Config::calm() });
    world.spawn(|| Process::Server(Box::new(Server::new(&config, routes))));
    let at = world.spawn(|| Process::Visitor(Box::new(Visitor::new(&config))));
    while world.busy() {
        world.step();
    }
    assert!(
        world.processes().iter().any(|process| match process {
            Process::Server(server) => server.listening(),
            Process::Visitor(_) => false,
        }),
        "seed {}: the site listener is ready",
        start.seed
    );
    (world, Reach { at, enter, take })
}

fn enter(process: &mut Process, input: bridge::Input) {
    match process {
        Process::Visitor(visitor) => visitor.enter(input),
        Process::Server(_) => unreachable!("Reach names the visitor"),
    }
}

fn take(process: &mut Process, reports: &mut Vec<fetch::Report>) {
    match process {
        Process::Visitor(visitor) => visitor.take(reports),
        Process::Server(_) => unreachable!("Reach names the visitor"),
    }
}
