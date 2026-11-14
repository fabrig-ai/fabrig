//! Cuts the actual private keeper's refresh through shared-world restart
//! (oauth/accounts.md, section 4; testing/simulator.md, section 3.3). Recovery only loads and
//! lends the kept old or new record; it never runs another issuer exchange.

use crate::private::FileSystem;
use crate::world::{self, Client, Process as Inner, Story};
use skein_fake_peers::Transport;
use skein_lib::{Duration, Time, Wall, bytes};
use skein_world::{Host, Inherited, Memory, Referee, StartupRoot, World};
use std::net::Ipv4Addr;

/// One restartable process, retaining its inherited signal until its owner settles.
pub type Process = crate::restart::Restartable<Inner>;

fn issuer(inherited: &Inherited) -> Process {
    Process::new(
        world::issuer(Transport::Plaintext, Story::NotKept, (Ipv4Addr::LOCALHOST, 31000).into()),
        inherited.signal,
        false,
        |_| {},
        |_, _| {},
    )
}
fn owner(inherited: &Inherited) -> Process {
    let root = inherited.roots.first().expect("private startup root").1;
    Process::new(Inner::Client(Box::new(Client::recovery(root))), inherited.signal, false, |_| {}, |_, _| {})
}

/// Only the original owner refreshes; fresh incarnations only recover.
#[derive(Debug)]
pub struct Judge {
    started: bool,
    shutdown: bool,
    wake: Option<Time>,
    passed: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let address = procs[0].inner.issuer().expect("issuer process").address();
        let client = procs[1].inner.client_mut().expect("client process");
        if let Some(address) = address {
            client.address = Some(address);
            if !self.started && client.unstarted() {
                client.refresh_at_start();
                self.started = true;
            }
        }
        if procs[1].is_empty() && !self.shutdown {
            self.shutdown = true;
            procs[0].inner.issuer_mut().expect("issuer").shutdown();
        }
    }
    fn watch(&mut self, now: Time, procs: &[Process]) -> Result<(), String> {
        let address_ready = procs[0].inner.issuer().is_some_and(|peer| peer.address().is_some());
        let client = procs[1].inner.client().expect("client process");
        self.wake = ((client.unstarted() && address_ready) || (procs[1].is_empty() && !self.shutdown)).then_some(now);
        self.passed = self.shutdown && procs.iter().all(Host::is_empty);
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake.or(Some(Time::from_nanos(Duration::from_secs(180).as_nanos())))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= Time::from_nanos(Duration::from_secs(180).as_nanos()) && !self.passed)
            .then(|| "private recovery did not settle".to_owned())
    }
    fn passed(&self) -> bool {
        self.passed
    }
}

/// Constructs a restartable keeper world; the caller chooses shared-world cut points.
#[must_use]
pub fn world(seed: u64) -> World<Process, Judge, FileSystem> {
    let mut config = skein_sim::Config::calm();
    config.wall = Wall::EPOCH;
    let mut world =
        World::new(seed, config, Judge { started: false, shutdown: false, wake: None, passed: false }, Memory::Checked)
            .with_machine(FileSystem::for_cuts());
    world.spawn_restartable(Vec::new(), issuer);
    world.spawn_restartable(vec![StartupRoot { name: bytes::copy_of(b"root"), path: bytes::copy_of(b"root") }], owner);
    world
}

/// The kept generation loaded by the fresh owner after a cut.
#[must_use]
pub fn recovered(procs: &[Process]) -> u64 {
    procs[1].inner.client().expect("client process").recovered.expect("recovery loaded a record")
}
