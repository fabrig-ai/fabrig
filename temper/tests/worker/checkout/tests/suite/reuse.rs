//! Workspace reuse is opt-in, and eviction keeps the workspace's path.

use std::collections::BTreeSet;

use skein_lib::{Duration, Time};
use temper_worker_checkout_world::client::{Pick, Plan};
use temper_worker_checkout_world::{LIMITS, Settings, World, noisy, submit_noisily};
use temper_worker_domain_checkout::{Landing, Limits, Prepared};

const ITERATIONS: u32 = 100_000;

fn at(secs: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_secs(secs))
}

fn settings(reuse: bool, seed: u64) -> Settings {
    Settings {
        checkout: Limits { workspaces: 1, ..LIMITS },
        repositories: 1,
        workstreams: 2,
        reuse,
        ..Settings::calm(seed)
    }
}

fn plan(workstream: u32) -> Plan {
    Plan { start: Some(Pick::Branch), pushes: 0, ..Plan::simple(workstream) }
}

fn ready(world: &World, client: u64) {
    assert!(matches!(world.client(client).prepared, Some(Prepared::Ready { .. })));
}

#[test]
fn a_new_workstream_reuses_the_evicted_clone_and_a_changed_file_is_the_only_one_touched() {
    for reuse in [true, false] {
        changed_file(reuse);
    }
}

fn changed_file(reuse: bool) {
    let mut world = World::new(settings(reuse, 1));
    assert_eq!(world.workstream(0), [0]);
    assert_eq!(world.workstream(1), [0]);
    let first = world.submit(Time::ZERO, plan(0));
    world.advance_at(at(100), 0, b"main");
    let second = world.submit(at(200), plan(1));
    world.run(ITERATIONS);
    ready(&world, first);
    ready(&world, second);
    assert_eq!(world.client(first).workspace, world.client(second).workspace, "eviction retains the path");
    let (told, lost) = world.told();
    assert_eq!(lost, 0);
    let expected = if reuse { (1, 1, 0, 1, 1) } else { (1, 0, 1, 2, 2) };
    assert_eq!((told.new, told.kept, told.evicted, told.makes, told.clones), expected);
    let paths: &[&[u8]] = if reuse { &[b"THEIRS"] } else { &[b"README", b"src/lib.rs", b"THEIRS"] };
    let expected: BTreeSet<Vec<u8>> = paths.iter().map(|path| path.to_vec()).collect();
    assert_eq!(world.touched(second, b"r0"), expected, "only changed contents count");
}

#[test]
fn other_remotes_evict_as_today() {
    for reuse in [true, false] {
        other_remotes(reuse);
    }
}

fn other_remotes(reuse: bool) {
    let mut world = World::new(Settings { repositories: 2, ..settings(reuse, 1) });
    assert_ne!(world.workstream(0), world.workstream(1), "different repositories force eviction");
    let first = world.submit(Time::ZERO, plan(0));
    let second = world.submit(at(200), plan(1));
    world.run(ITERATIONS);
    ready(&world, first);
    ready(&world, second);
    let (told, lost) = world.told();
    assert_eq!(lost, 0);
    assert_eq!((told.kept, told.evicted), (0, 1));
}

#[test]
fn a_saved_start_evicts_as_today() {
    for reuse in [true, false] {
        saved_start(reuse);
    }
}

fn saved_start(reuse: bool) {
    let mut world = World::new(settings(reuse, 1));
    let saver = world.submit(Time::ZERO, Plan { save: true, ..plan(0) });
    let other = world.submit(at(200), plan(1));
    world.run(ITERATIONS);
    ready(&world, saver);
    ready(&world, other);
    assert!(matches!(world.client(saver).landings[0].1[0], Landing::Landed { .. }), "the first client saves its edit");
    assert_ne!(world.client(saver).left, world.client(saver).start);
    let before = world.told().0;
    assert_eq!((before.kept, before.evicted), if reuse { (1, 0) } else { (0, 1) });
    let resumer = world.submit(at(400), Plan { start: Some(Pick::Saved), ..plan(0) });
    world.run(ITERATIONS);
    ready(&world, resumer);
    assert_eq!(world.client(resumer).start, world.client(saver).left, "saved work is restored");
    let (after, lost) = world.told();
    assert_eq!(lost, 0);
    assert_eq!(after.evicted, before.evicted + 1, "a saved start always makes an empty workspace");
    assert_eq!(after.kept, before.kept);
    let expected = if reuse { (1, 1, 1, 2, 2) } else { (1, 0, 2, 3, 3) };
    assert_eq!((after.new, after.kept, after.evicted, after.makes, after.clones), expected);
}

#[test]
fn a_noisy_run_holds_the_referees() {
    for reuse in [true, false] {
        for seed in 0..10 {
            let settings = Settings { reuse, ..noisy(seed) };
            let mut world = World::new(settings);
            submit_noisily(&mut world, &settings, seed);
            world.run(ITERATIONS);
        }
    }
}
