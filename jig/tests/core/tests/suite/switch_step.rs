//! The start's switch and mark follow durable restores (domain/engine/restart.md).

use jig_core_world::fixtures::PROJECT;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_protocol_store::core_v6::Record as _;
use jig_protocol_store::{Wire, component_v3};
use jig_test_domain as root;
use jig_test_store_files::Fault;
use skein_lib::Reader;

use crate::closed_proofs;
use closed_proofs as f;

const WIRES: &[Wire] = &[Wire { name: "channel", version: 3 }, Wire { name: "fixture", version: 7 }];
const RESTORES: &[core::start::Phase] = &[
    core::start::Phase::LoadCore,
    core::start::Phase::LoadInbox,
    core::start::Phase::RestoreConnector { connector: 1 },
    core::start::Phase::RestoreConnector { connector: 2 },
    core::start::Phase::Sweep,
    core::start::Phase::Switch,
];

fn before() -> World {
    let mut world = World::new(Setup { seed: f::SEED, owner_sets: f::SETS, ..Setup::default() });
    // Establish the recovered source bytes before comparing generations: a
    // normal open may trim a log's unused tail during recovery (protocol/store/opening.md, 1).
    world.store.crash(f::SEED).expect("crash readback");
    world
}

fn at_switch() -> World {
    let mut world = before();
    world.store.wires(WIRES);
    world.hold_switch = true;
    world.restart(f::SEED, false);
    world
}

fn mark(world: &World, since: usize) -> &[root::Write] {
    let marks: Vec<_> = world.commits[since..].iter().filter(|writes| writes.contains(&root::Write::Wires)).collect();
    assert_eq!(marks.len(), 1, "one start, one mark");
    marks[0]
}

fn wires(world: &mut World) -> Box<[u8]> {
    world
        .store
        .retained()
        .expect("complete retained inventory")
        .into_iter()
        .find_map(|(key, value)| {
            (jig_protocol_store::keys::decode_own(&key) == Some(jig_protocol_store::keys::Own::Wires)).then_some(value)
        })
        .expect("component wires row")
}

fn durable_mark(world: World, since: usize, last: u64) {
    assert_eq!(world.store.applied, last + 1, "the mark is the next numbered commit");
    assert_eq!(mark(&world, since).len(), 2, "wires and deployment header alone");
    assert_eq!(world.domain.start_phase(), Some(core::start::Phase::Wires), "no adoption before mark acknowledgement");
    let mut world = world;
    world.release_commits();
    assert!(world.domain.ready());
    let bytes = wires(&mut world);
    let row = component_v3::Wires::decode(&component_v3::CEILINGS, &mut Reader::new(&bytes)).expect("component wires");
    let wires: Vec<_> = row.wires.iter().map(|wire| (wire.name.as_ref().to_vec(), wire.version)).collect();
    assert_eq!(wires, [(b"channel".to_vec(), 3), (b"fixture".to_vec(), 7)]);
}

#[test]
fn a_start_with_nothing_pending_takes_the_switch_step_and_commits_its_wires() {
    let mut world = at_switch();
    assert_eq!(world.restart_steps, RESTORES);
    assert!(!world.store.generation_pending);
    let last = world.store.applied;
    let since = world.commits.len();
    world.store.fault(Fault::Hold { commits: 1 });
    world.finish_switch();
    assert_eq!(world.store.switch_requests, 0);
    durable_mark(world, since, last);
}

#[test]
fn a_first_start_takes_the_switch_step() {
    let mut world = World::new(Setup {
        configuration: Some(jig_test_profile::config(f::SEED)),
        limits: jig_test_profile::limits(),
        boot: jig_core_world::world::Boot::Empty,
        rows: Some(Box::new([])),
        ..Setup::default()
    });
    let restores = &world.restart_steps[..RESTORES.len()];
    assert_eq!(restores, RESTORES);
    assert_eq!(world.store.switch_requests, 0);
    assert_eq!(mark(&world, 0).len(), 2);
    assert!(world.domain.ready());
    let bytes = wires(&mut world);
    let row = component_v3::Wires::decode(&component_v3::CEILINGS, &mut Reader::new(&bytes)).expect("empty wires row");
    assert!(row.wires.is_empty(), "an empty shell wire list is still committed");
}

#[test]
fn a_live_set_past_lowered_limits_leaves_the_old_generation_active() {
    let mut world = before();
    let source = world.store.generation_contents(b"generations/1");
    let rows = world.store.rows.clone();
    let last = world.store.applied;
    let mut limits = jig_test_profile::limits();
    limits.core.authority.projects = 0;
    let mut config = jig_core_world::world::fixture(f::SEED, false).0;
    // Projects already exist in the store; this build seeds no new policies.
    config.core.seed_projects = Box::new([]);
    config.core.defaults.policies = skein_lib::Map::with_capacity(0);
    world.restart_with_limits(config, limits);
    assert!(world.stopped);
    assert_eq!(
        world.domain.start_failure(),
        Some((
            Some(core::start::Phase::LoadCore),
            core::start::Failure::LiveSet {
                set: core::start::LiveSet::Projects,
                key: core::Key::People(jig_domain_people::Key::Policy(1)),
                held: 1,
                limit: 0,
            }
        ))
    );
    assert!(!world.restart_steps.contains(&core::start::Phase::Switch));
    assert_eq!(world.store.switch_requests, 0);
    assert_eq!(world.store.file_bytes(b"active"), Some(b"1\n".to_vec()));
    assert_eq!(world.store.generation_contents(b"generations/1"), source);
    world.store.crash(f::SEED).expect("crash readback");
    assert_eq!(world.store.applied, last);
    assert_eq!(world.store.rows, rows);
    assert!(!world.store.generation_pending);
    // The lowered policy limit deliberately refuses the restart.
    world.broken();
}

#[test]
fn a_refused_store_switch_stops_the_start() {
    let mut world = at_switch();
    let since = world.commits.len();
    // A stale shell pending flag asks a real nonpending store to switch;
    // its OpenRefused terminal must stop the root through the normal route.
    world.store.generation_pending = true;
    world.finish_switch();
    assert!(world.stopped);
    assert!(!world.domain.ready());
    assert!(world.commits[since..].is_empty(), "no mark after refusal");
    // The deliberately stale pending flag leaves the start stopped after Switch refusal.
    world.broken();
}

#[test]
fn a_failed_mark_never_adopts_the_runs() {
    let mut world = at_switch();
    let next = world.store.applied + 1;
    world.store.fault(Fault::Fail { commit: next });
    world.finish_switch();
    assert!(world.stopped);
    assert_eq!(world.domain.start_phase(), Some(core::start::Phase::Wires));
    assert_eq!(world.restart_steps, RESTORES);
    // The Wires commit is deliberately failed and the store remains failed.
    world.broken();
}

#[test]
fn stopping_before_the_mark_does_not_commit_it() {
    let mut world = at_switch();
    let before = world.store.applied;
    world.send(root::Event::Stop);
    world.finish_switch();
    assert!(world.domain.stopping());
    assert_eq!(world.store.applied, before);
    assert_eq!(world.domain.start_phase(), Some(core::start::Phase::Switch));
    world.restart(f::SEED, false);
    assert!(world.domain.ready());
}

#[test]
fn stopping_with_the_mark_held_does_not_adopt_runs() {
    let mut world = at_switch();
    world.store.fault(Fault::Hold { commits: 1 });
    world.finish_switch();
    let mark = world.store.applied;
    world.send(root::Event::Stop);
    world.release_commits();
    assert!(world.domain.stopping());
    assert_eq!(world.store.applied, mark);
    assert_eq!(world.domain.start_phase(), Some(core::start::Phase::Wires));
    assert!(!world.domain.ready());
    assert!(!world.restart_steps.contains(&core::start::Phase::AdoptRuns));
    world.restart(f::SEED, false);
    assert!(world.domain.ready());
}

// A link refusal stops the start before switching or adopting runs.
fn refuses_before_switch(mut world: World) {
    // Each caller deliberately seeds an unmade link.
    world.broken();
    world.store.crash(f::SEED).expect("crash readback");
    let source = world.store.generation_contents(b"generations/1");
    let last = world.store.applied;
    let assignments = world.assignments.clone();
    world.restart(f::SEED, false);
    assert_eq!(
        world.domain.start_failure(),
        Some((
            Some(core::start::Phase::Link),
            core::start::Failure::Link {
                record: core::Key::Tasks(jig_domain_tasks::Key::Live(1)),
                linked: core::Linked::Project(PROJECT),
            }
        ))
    );
    assert!(!world.domain.ready());
    assert_eq!(world.domain.start_phase(), Some(core::start::Phase::Link));
    assert_eq!(world.store.switch_requests, 0);
    assert_eq!(world.store.file_bytes(b"active"), Some(b"1\n".to_vec()));
    assert_eq!(world.store.generation_contents(b"generations/1"), source);
    assert_eq!(world.store.file_bytes(b"generations/2/manifest"), None);
    assert_eq!(world.assignments, assignments);
    assert!(!world.restart_steps.iter().any(|step| matches!(
        step,
        core::start::Phase::Switch
            | core::start::Phase::AdoptRuns
            | core::start::Phase::ReadAfresh { .. }
            | core::start::Phase::SettleOutbox { .. }
    )));
    world.store.crash(f::SEED).expect("crash readback");
    assert_eq!(world.store.applied, last);
    // Compare the recovered files, including every map, at the same sets.
    assert_eq!(world.store.generation_contents(b"generations/1"), source);
    assert!(!world.store.generation_pending);
}

const ORPHAN_SOURCE: u32 = 97;

fn orphan_source() -> root::Record {
    root::Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Source {
        account: 98,
        number: ORPHAN_SOURCE,
        place: 0,
        kind: jig_domain_accounts::Kind::Key,
        name: Box::new([]),
        held: None,
    }))
}

#[test]
fn an_account_source_without_its_account_refuses_at_link_on_the_active_generation() {
    let mut world = before();
    world.seed(Box::new([orphan_source()]));
    world.broken();
    world.restart(f::SEED, false);
    assert_eq!(
        world.domain.start_failure(),
        Some((
            Some(core::start::Phase::Link),
            core::start::Failure::Link {
                record: core::Key::Accounts(jig_domain_accounts::Key::Source { account: 98, number: ORPHAN_SOURCE }),
                linked: core::Linked::Record(core::Key::Accounts(jig_domain_accounts::Key::Account { number: 98 })),
            }
        ))
    );
    assert!(!world.domain.ready());
    assert_eq!(world.domain.start_phase(), Some(core::start::Phase::Link));
    assert_eq!(world.domain.accounts().restore_limit(), None);
    assert!(world.domain.accounts().source(ORPHAN_SOURCE).is_some(), "the source is retained until its link is made");
    assert_eq!(world.store.file_bytes(b"active"), Some(b"1\n".to_vec()));
    assert_eq!(world.store.switch_requests, 0);
    // The seeded source deliberately names an absent account and leaves Link unfinished.
    world.broken();
}

#[test]
fn a_task_naming_a_missing_project_refuses_before_the_switch() {
    let mut world = before();
    let row = jig_core_world::fixtures::live_row(&world, world.task()).clone();
    assert_eq!(row.number, 1);
    // Leave the task and its funding internally consistent, but without its project.
    world.erase(root::Key::Core(core::Key::People(jig_domain_people::Key::Policy(PROJECT))));
    world.seed(Box::new([root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(row))))]));
    refuses_before_switch(world);
}
