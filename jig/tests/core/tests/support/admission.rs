//! Admission fixtures using the core world's controlled store completions.
use jig_core_world::{
    effects::{Setup, World},
    fixtures, world,
};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_test_domain as root;
use skein_lib::{Duration, Map, Token};
const WORKER: Token = Token::new(7);
/// A signed-in party's chat, assigned to the walking world's worker.
#[must_use]
pub fn chat(seed: u64) -> World {
    configured_chat(world::config(seed), &world::limits())
}

/// A live chat whose worker can reconnect listing its run and one stray.
#[must_use]
pub fn reconnecting_chat(seed: u64) -> World {
    // The current run and the stale listing each occupy one host slot.
    const HOSTED: u32 = 2;
    let mut limits = world::limits();
    limits.core.fleet.slots = HOSTED;
    configured_chat(world::config(seed), &limits)
}

/// A chat fixture with room and a configured policy for two more projects.
#[must_use]
pub fn project_maker(seed: u64) -> World {
    // The seed plus two makes; the people's rosters also include deployment roles.
    const PROJECTS: u32 = 3;
    let mut limits = world::limits();
    limits.core.authority.projects = PROJECTS;
    limits.core.people.projects = PROJECTS + 1;
    let mut configuration = world::config(seed);
    configuration.core.defaults.rules =
        authority::Domain::new(configuration.core.defaults.rules.clone(), limits.core.authority)
            .expect("project room")
            .rules()
            .clone();
    let seed = &configuration.core.seed_projects[0].policy;
    configuration.core.defaults.policies = Map::with_capacity(1);
    configuration
        .core
        .defaults
        .policies
        .insert(
            1,
            core::ConfiguredPolicy {
                policy: seed.policy.clone(),
                permissions: Box::new([]),
                templates: seed.templates.clone(),
            },
        )
        .expect("one configured policy");
    configured_chat(configuration, &limits)
}

/// A live chat named as owner of the first connector's first resource.
#[must_use]
pub fn resource_owner(seed: u64) -> World {
    let mut driver = chat(seed);
    let task = driver.assignment().task;
    let resource = jig_test_connector_world::path(1, 1);
    driver.offer(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::Adopt {
            project: 1,
            resource: resource.clone(),
            role: jig_test_connector::ResourceRole::Owned,
        },
    });
    driver.drain();
    driver.offer(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::Names { task, project: 1, resources: Box::new([resource.clone()]) },
    });
    driver.drain();
    driver.offer(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::System(jig_test_connector::SystemEvent::Fact {
            resource,
            fact: jig_test_connector::Fact { state: Some(10), observed: driver.env().wall, pending: false },
            origin: jig_test_connector::Origin::Own,
        }),
    });
    driver.drain();
    driver
}

/// A real resource-bearing run lost while its connector's fresh read is withheld.
#[must_use]
pub fn lost_resource_writer(seed: u64) -> (World, u64) {
    let (configuration, limits) = jig_core_world::faults::plan_fixture(seed);
    let mut world = World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() });
    fixtures::configure_resources(&mut world);
    let parent = world.assignments[0];
    let spec = fixtures::resource_member(&world, 1, b"writer that disappears");
    let child = fixtures::delegate(&mut world, parent, 1, spec);
    world.send(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::Names {
            task: child,
            project: 1,
            resources: Box::new([jig_test_connector_world::path(1, 1)]),
        },
    });
    let assignment = world.assigned.take().expect("the resource's assigned writer");
    let resource = jig_test_connector_world::path(1, 1);
    world.send(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::System(jig_test_connector::SystemEvent::Fact {
            resource: resource.clone(),
            fact: jig_test_connector::Fact { state: Some(10), observed: world.wall_time(), pending: false },
            origin: jig_test_connector::Origin::Own,
        }),
    });
    world.hold_lost_reads = true;
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: WORKER })));
    world.advance(Duration::from_secs(6));
    // Fleet fires one expired attempt at a time: the caller, then its writer.
    world.send(root::Event::Timer(core::Timer::Fleet));
    let writer = world
        .store
        .rows
        .values()
        .find_map(|row| {
            if let root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Writer(slot))) = row
                && slot.lost
                && slot.resource.path.as_ref() == resource.segments()
            {
                Some(slot.number)
            } else {
                None
            }
        })
        .expect("the lost writer's read is still owed");
    world.assigned = Some(assignment);
    let (now, wall) = world.clocks();
    world.set_clock(now, wall);
    world.manual = true;
    world.commits.clear();
    (world, writer)
}

fn configured_chat(configuration: root::Config, limits: &root::Limits) -> World {
    let mut world = World::new(Setup { configuration: Some(configuration), limits: *limits, ..Setup::default() });
    let (now, wall) = world.clocks();
    world.set_clock(now, wall);
    world.manual = true;
    world.commits.clear();
    world
}
