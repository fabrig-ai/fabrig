//! A world's inputs and calm start (temper/docs/design/playground.md, 4).
use crate::{
    documents, fakes,
    machine::{self, Machine},
    programs::{self, Harness, Process},
};
use skein_lib::{Time, Wall};
use skein_playground::{
    slices::{Reach, Slices},
    visitor::Visitor,
};
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
    if !document["application"]["demo"].is_null() {
        use temper_forge_setup::{
            machine::BoxKind,
            place::{File, Place},
        };
        let directories = jig_protocol_config::Directories::working();
        let named = temper_service::startup::named_with(&deployment, &directories, &documents::given(&document))
            .expect("deployment's machine");
        let last = document["setup"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry["forge"].as_u64())
            .max()
            .unwrap_or(0);
        let number = u16::try_from(last + 1).expect("playground forge number");
        let entry = temper_forge_setup::machine::entry(&named.machine, number, BoxKind::Fake, &directories)
            .expect("playground's fake machine");
        let path = Place { directory: entry.directory, user: Box::new([]) }.path(File::AdminToken);
        machine::lay_handover(&mut disk, &path, fakes::ADMIN_TOKEN);
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
    let at = world.spawn(|| Process::Visitor(Box::new(Visitor::new(&documents::visitor(&document)))));
    assert_eq!(at, VISITOR, "the visitor follows the service and its two fake peers");
    world
}

/// The visitor follows the service and its two fake peers.
pub const VISITOR: usize = 3;

/// Reach the composition's visitor between steps.
#[must_use]
pub fn reach<H: Harness>() -> Reach<Process<H>> {
    Reach {
        at: VISITOR,
        enter: |process, input| {
            let Process::Visitor(visitor) = process else { unreachable!("Reach names the visitor") };
            visitor.enter(input);
        },
        take: |process, reports| {
            let Process::Visitor(visitor) = process else { unreachable!("Reach names the visitor") };
            visitor.take(reports);
        },
    }
}

/// Step with this world's factory parameters installed.
/// The scope ends before returning to the caller or another world's loop.
pub fn step<H: Harness, N: Meter>(world: &mut World<Process<H>, Machine, N>) -> skein_sim_world::Step {
    programs::with(parameters(world), || world.step())
}

/// The world's factory parameters, read by its hosted programs while it steps.
#[must_use]
pub fn parameters<H: Harness, N: Meter>(world: &World<Process<H>, Machine, N>) -> programs::Parameters {
    world
        .processes()
        .iter()
        .find_map(
            |process| {
                if let Process::Service(service) = process { Some(service.parameters.clone()) } else { None }
            },
        )
        .expect("a composition has its service")
}

/// A slice's budget in steps (skein/docs/design/playground.md, 3).
pub const SLICE_STEPS: u32 = 256;

/// A story's world in skein's slices, its visitor reached, at 1× from `browser`.
#[must_use]
pub fn slices(start: Start, budget: u32, browser: Time) -> Slices<Process<()>, Machine> {
    Slices::new(world(start), reach(), budget, browser)
}

/// Make slices from a worker start message; an unknown story has no world.
#[must_use]
pub fn start_slices(start: &skein_ui_web::bridge::Start, browser: Time) -> Option<Slices<Process<()>, Machine>> {
    let story = crate::stories::named(&start.story)?;
    Some(slices(story.start(start.seed, start.start), SLICE_STEPS, browser))
}

#[cfg(test)]
#[path = "tests/world.rs"]
mod tests;
