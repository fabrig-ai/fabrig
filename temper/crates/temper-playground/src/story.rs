//! A world's inputs and calm start (temper/docs/design/playground.md, 4).
use crate::{
    documents, fakes,
    machine::{self, Machine},
    programs::{self, Harness, Process},
};
use skein_lib::Wall;
use skein_sim_world::{Bare, Inherited, Meter, Stream, World};

/// Inputs read as a deployment, with one seed and a supplied wall clock.
pub struct Start {
    pub seed: u64,
    pub wall: Wall,
    pub deployment: Box<[u8]>,
    pub fakes: Fakes,
    pub links: Links,
}
/// Built neighbours; the HTTP peer and git transport share one forge.
pub struct Fakes {
    pub llm: skein_fake_peers::llm::Peer,
    pub forge: temper_fake_git::Typed,
    pub peer: skein_fake_peers::forge::Peer,
}
impl Fakes {
    /// Build the composition's default Forgejo peer beside its git transport.
    #[must_use]
    pub fn new(llm: skein_fake_peers::llm::Peer, forge: temper_fake_git::Typed, seed: u64) -> Self {
        let peer = fakes::forge_peer(&forge, seed);
        Self { llm, forge, peer }
    }
}
/// Who consumes the service's sign-in links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Links {
    Keep,
    Print,
}

/// Make a harness-free world. The service is process zero.
#[must_use]
pub fn world(start: Start) -> World<Process<()>, Machine> {
    world_metered(start, Bare)
}

/// Make a harness-free world with a meter installed before admitting any process.
#[must_use]
pub fn world_metered<N: Meter>(start: Start, meter: N) -> World<Process<()>, Machine, N> {
    let Start { seed, wall, deployment, fakes, links } = start;
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    let document: serde_json::Value = serde_json::from_slice(&deployment).expect("deployment JSON");
    if let Some(key) = document.pointer("/application/demo/llm/key").and_then(serde_json::Value::as_str) {
        machine::lay_handover(&mut disk, key.as_bytes(), documents::KEY);
    }
    if let Some(setup) = document["setup"].as_array() {
        for entry in setup {
            if let Some(directory) = entry["directory"].as_str() {
                machine::lay_handover(&mut disk, format!("{directory}/admin_token").as_bytes(), fakes::ADMIN_TOKEN);
            }
        }
    }
    let machine = Machine::new(disk);
    let parameters = programs::Parameters { deployment, seed, links, forge: fakes.forge, disk: machine.0.clone() };
    let mut config = skein_sim::Config::calm();
    config.wall = wall;
    config.trace = skein_sim::Trace::Off;
    let mut world = World::new(seed, config).with_machine(machine).with_meter(meter);
    for (program, roots) in programs::programs::<()>() {
        world.host_roots(program, roots);
    }
    programs::with(parameters, || {
        world.spawn_restartable(programs::service_roots(), programs::service::<()>);
    });
    let printer = world.printer(b"temper serve", Stream::Output);
    let (_, _, processes) = world.parts();
    if let Process::Service(service) = &mut processes[0] {
        service.print_to(printer);
    }
    world.spawn_signals(|signal| {
        programs::llm_peer(fakes.llm, &Inherited { signal, pipes: vec![], roots: vec![], appends: vec![] })
    });
    world.spawn_signals(|signal| {
        programs::fake_forge(fakes.peer, &Inherited { signal, pipes: vec![], roots: vec![], appends: vec![] })
    });
    world
}

/// Step with this world's factory parameters installed.
/// The scope ends before returning to the caller or another world's loop.
pub fn step<H: Harness, N: Meter>(world: &mut World<Process<H>, Machine, N>) -> skein_sim_world::Step {
    let parameters = world
        .processes()
        .iter()
        .find_map(
            |process| {
                if let Process::Service(service) = process { Some(service.parameters.clone()) } else { None }
            },
        )
        .expect("a composition has its service");
    programs::with(parameters, || world.step())
}

#[cfg(test)]
#[path = "tests/world.rs"]
mod tests;
