//! Workspace reuse is opt-in, and eviction keeps the workspace's path.

use std::collections::BTreeSet;

use skein_lib::{Duration, Time, Token};
use temper_worker_checkout_world::client::{Pick, Plan};
use temper_worker_checkout_world::{LIMITS, Settings, World, noisy, submit_noisily};
use temper_worker_domain_checkout::{Failure, Landing, Limits, Prepared};

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
    assert_eq!(
        (if reuse { told.looking } else { told.new }, told.kept, told.evicted, told.makes, told.clones),
        expected
    );
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
    assert_eq!(
        (if reuse { after.looking } else { after.new }, after.kept, after.evicted, after.makes, after.clones),
        expected
    );
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

#[test]
fn a_restart_reuses_the_clone_and_a_changed_file_is_the_only_one_touched() {
    for reuse in [true, false] {
        let mut world = World::new(settings(reuse, 1));
        let first = world.submit(Time::ZERO, plan(0));
        world.run(ITERATIONS);
        world.restart();
        let now = world.now();
        world.advance_at(now, 0, b"main");
        let second = world.submit(now.saturating_add(Duration::from_secs(1)), plan(0));
        world.run(ITERATIONS);
        ready(&world, first);
        ready(&world, second);
        assert_eq!(world.client(first).workspace, Some(Token::new(0)));
        assert_eq!(world.client(second).workspace, Some(Token::new(0)));
        let (told, lost) = world.told();
        assert_eq!(lost, 0);
        assert_eq!((told.looking, told.new, told.makes, told.clones), if reuse { (2, 0, 1, 1) } else { (0, 2, 2, 2) });
        let paths: &[&[u8]] = if reuse { &[b"THEIRS"] } else { &[b"README", b"src/lib.rs", b"THEIRS"] };
        assert_eq!(world.touched(second, b"r0"), paths.iter().map(|path| path.to_vec()).collect());
    }
}

#[test]
fn a_restart_with_another_remote_makes_it_again() {
    for reuse in [true, false] {
        let mut world = World::new(Settings {
            repositories: 2,
            checkout: Limits { repositories: 1, workspaces: 1, ..LIMITS },
            ..settings(reuse, 1)
        });
        assert_ne!(world.workstream(0), world.workstream(1));
        let first = world.submit(Time::ZERO, plan(0));
        world.run(ITERATIONS);
        world.restart();
        let second = world.submit(world.now(), plan(1));
        world.run(ITERATIONS);
        ready(&world, first);
        ready(&world, second);
        let (told, lost) = world.told();
        assert_eq!(lost, 0);
        assert_eq!((told.looking, told.new, told.makes, told.clones), if reuse { (2, 0, 2, 2) } else { (0, 2, 2, 2) });
    }
}

#[test]
fn a_damaged_clone_heals() {
    for reuse in [true, false] {
        let mut world = World::new(settings(reuse, 1));
        world.submit(Time::ZERO, plan(0));
        world.run(ITERATIONS);
        world.restart();
        world.write_disk(b"ws/0/r0/.git/index.lock", b"");
        let first = world.submit(world.now(), plan(0));
        world.run(ITERATIONS);
        if reuse {
            assert_eq!(world.client(first).prepared, Some(Prepared::Failed { failure: Failure::Transient }));
        } else {
            ready(&world, first);
        }
        let second = world.submit(world.now(), plan(0));
        world.run(ITERATIONS);
        // Successful check-out proves the lock went with the spoiled workspace.
        ready(&world, second);
        let (told, lost) = world.told();
        assert_eq!(lost, 0);
        assert_eq!(
            (told.looking, told.new, told.rebuilt, told.reused, told.makes, told.clones),
            if reuse { (2, 0, 1, 0, 2, 2) } else { (0, 2, 0, 1, 2, 2) }
        );
    }
}

#[test]
fn what_an_earlier_worker_left_is_made_again() {
    for reuse in [true, false] {
        let mut world = World::new(Settings { leftovers: 1000, ..settings(reuse, 1) });
        let client = world.submit(Time::ZERO, plan(0));
        world.run(ITERATIONS);
        ready(&world, client);
        assert_eq!(world.stats().made_over, 1);
        let (told, lost) = world.told();
        assert_eq!(lost, 0);
        assert_eq!((told.looking, told.new, told.makes, told.clones), if reuse { (1, 0, 1, 1) } else { (0, 1, 1, 1) });
    }
}

#[test]
fn a_restart_keeps_the_paths() {
    for reuse in [true, false] {
        for order in [[0, 1], [1, 0]] {
            let mut world = World::new(Settings { checkout: Limits { workspaces: 2, ..LIMITS }, ..settings(reuse, 1) });
            let first = world.submit(Time::ZERO, plan(order[0]));
            let second = world.submit(at(100), plan(order[1]));
            world.run(ITERATIONS);
            world.restart();
            let now = world.now();
            let third = world.submit(now, plan(order[1]));
            let fourth = world.submit(now.saturating_add(Duration::from_secs(100)), plan(order[0]));
            world.run(ITERATIONS);
            for (client, token) in [(first, 0), (second, 1 << 32), (third, 0), (fourth, 1 << 32)] {
                ready(&world, client);
                assert_eq!(world.client(client).workspace, Some(Token::new(token)));
            }
            let (told, lost) = world.told();
            assert_eq!(lost, 0);
            assert_eq!(
                (told.looking, told.new, told.kept, told.makes, told.clones),
                if reuse { (4, 0, 0, 2, 2) } else { (0, 4, 0, 4, 4) }
            );
        }
    }
}

#[test]
fn a_noisy_run_across_restarts_holds_the_referees() {
    for reuse in [true, false] {
        for seed in 0..10 {
            let settings = Settings { reuse, ..noisy(seed) };
            let mut world = World::new(settings);
            submit_noisily(&mut world, &settings, seed);
            world.run(ITERATIONS);
            world.restart();
            submit_noisily(&mut world, &settings, seed);
            world.run(ITERATIONS);
        }
    }
}

#[test]
fn a_restart_keeps_the_count_of_facts_dropped_before_it() {
    for reuse in [true, false] {
        let mut world =
            World::new(Settings { checkout: Limits { workspaces: 2, facts: 3, ..LIMITS }, ..settings(reuse, 1) });
        world.submit(Time::ZERO, plan(0));
        world.submit(Time::ZERO, plan(1));
        world.run(ITERATIONS);
        let before = world.told().1;
        world.restart();
        let after = world.told().1;
        let client = world.submit(world.now(), plan(0));
        world.run(ITERATIONS);
        ready(&world, client);
        assert!(before > 0, "simultaneous holds exceed the three-fact queue");
        assert_eq!(after, before, "the cumulative fact counters retain their lost count");
    }
}
