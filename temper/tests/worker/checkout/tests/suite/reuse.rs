//! Workspace reuse is opt-in, and eviction keeps the workspace's path.
//! Dirty holds follow temper/docs/design/domain/worker/checkouts.md and
//! temper/docs/design/protocol/git.md, 4.5.

use std::collections::BTreeSet;

use skein_lib::{Duration, Time, Token};
use temper_fake_checkout::{
    git::{Remote, Tree},
    in_git,
};
use temper_worker_checkout_world::client::{Pick, Plan};
use temper_worker_checkout_world::{LIMITS, Settings, World, noisy, submit_noisily};
use temper_worker_checkout_world::{hosted, translate};
use temper_worker_domain_checkout::{Failure, Landing, Limits, Prepared};
use temper_worker_protocol_git::invocation::Command;

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

fn dirty_files(tracked: &[u8]) -> Tree {
    Tree::from([
        (b"NEW".to_vec(), b"staged new file\n".to_vec()),
        (b".git/temper-index/NEW".to_vec(), b"staged new file\n".to_vec()),
        (tracked.to_vec(), b"unstaged edit\n".to_vec()),
        (b".git/MERGE_HEAD".to_vec(), 1_u64.to_le_bytes().to_vec()),
        (b".git/temper-conflicts/0".to_vec(), tracked.to_vec()),
        (b".gitignore".to_vec(), b"target/\n".to_vec()),
        (b"target/o".to_vec(), b"ignored build output\n".to_vec()),
    ])
}

fn dirty(world: &mut World, at: &[u8]) {
    for (path, content) in world.disk_tree(at).into_iter().filter(|(path, _)| !in_git(path)) {
        world.write_disk(&[at, b"/.git/temper-index/", &path].concat(), &content);
    }
    for (path, content) in dirty_files(b"README") {
        world.write_disk(&[at, b"/", &path].concat(), &content);
    }
}

fn tree_without_merge(mut disk: Tree) -> Tree {
    assert!(!disk.contains_key(b".git/MERGE_HEAD".as_slice()), "no merge in progress");
    assert!(!disk.keys().any(|path| path.starts_with(b".git/temper-conflicts/")), "no conflict state");
    disk.retain(|path, _| !in_git(path));
    disk
}

fn clean_at(world: &World, client: u64, at: &[u8]) {
    ready(world, client);
    assert_eq!(world.checkouts(client), [(b"r0".to_vec(), true)], "the hold requests cleaning");
    let commit = world.forge().branch(world.repository(0), b"main").expect("main exists");
    assert_eq!(tree_without_merge(world.disk_tree(at)), world.forge().tree(commit));
}

#[test]
fn a_kept_workspace_from_another_workstream_starts_from_exactly_its_commit() {
    let mut world = World::new(settings(true, 1));
    let first = world.submit(Time::ZERO, plan(0));
    world.run(ITERATIONS);
    ready(&world, first);
    dirty(&mut world, b"ws/0/r0");
    let before = world.told().0;
    let next = world.submit(world.now(), plan(1));
    world.run(ITERATIONS);
    clean_at(&world, next, b"ws/0/r0");
    assert_eq!(world.client(first).workspace, world.client(next).workspace);
    let (after, lost) = world.told();
    assert_eq!(lost, 0);
    assert_eq!(after.kept, before.kept + 1, "Cached::Kept");
    assert_eq!((after.makes, after.clones), (before.makes, before.clones));
}

#[test]
fn the_same_workstreams_next_hold_starts_from_exactly_its_commit() {
    for reuse in [true, false] {
        let mut world = World::new(settings(reuse, 1));
        let first = world.submit(Time::ZERO, plan(0));
        world.run(ITERATIONS);
        ready(&world, first);
        dirty(&mut world, b"ws/0/r0");
        let before = world.told().0;
        let next = world.submit(world.now(), plan(0));
        world.run(ITERATIONS);
        clean_at(&world, next, b"ws/0/r0");
        assert_eq!(world.client(first).workspace, world.client(next).workspace);
        let (after, lost) = world.told();
        assert_eq!(lost, 0);
        assert_eq!(after.reused, before.reused + 1, "Cached::Reused");
        assert_eq!((after.makes, after.clones), (before.makes, before.clones));
    }
}

#[test]
fn a_restart_with_a_dirty_clone_starts_from_exactly_its_commit() {
    let mut world = World::new(settings(true, 1));
    let first = world.submit(Time::ZERO, plan(0));
    world.run(ITERATIONS);
    ready(&world, first);
    dirty(&mut world, b"ws/0/r0");
    world.restart();
    let before = world.told().0;
    let next = world.submit(world.now(), plan(0));
    world.run(ITERATIONS);
    clean_at(&world, next, b"ws/0/r0");
    assert_eq!(world.client(first).workspace, world.client(next).workspace);
    let (after, lost) = world.told();
    assert_eq!(lost, 0);
    assert_eq!(after.looking, before.looking + 1, "Cached::Looking");
    assert_eq!((after.makes, after.clones), (before.makes, before.clones));
}

#[test]
fn a_saved_start_on_a_dirty_workspace_is_made_again() {
    let mut world = World::new(settings(true, 1));
    let saver = world.submit(Time::ZERO, Plan { save: true, ..plan(0) });
    world.run(ITERATIONS);
    ready(&world, saver);
    let expected = world.client(saver).left[0].clone();
    assert_ne!(world.client(saver).left, world.client(saver).start, "the saved tree has an edit");
    dirty(&mut world, b"ws/0/r0");
    let before = world.told().0;
    let resumer = world.submit(world.now(), Plan { start: Some(Pick::Saved), ..plan(0) });
    world.run(ITERATIONS);
    ready(&world, resumer);
    assert_eq!(world.client(resumer).start.as_slice(), std::slice::from_ref(&expected));
    assert_eq!(tree_without_merge(world.disk_tree(b"ws/0/r0")), expected);
    let (after, lost) = world.told();
    assert_eq!(lost, 0);
    assert_eq!(after.rebuilt, before.rebuilt + 1, "a saved start rebuilds its cached workspace");
    assert_eq!((after.makes, after.clones), (before.makes + 1, before.clones + 1));
    assert!(world.disk_tree(b"ws/0/r0/.git/temper-index").is_empty(), "the dirty index went with the workspace");
}

#[test]
fn the_two_commands_leave_the_commits_tree_and_nothing_staged() {
    let mut scenario = hosted::scenario();
    let expected = scenario.transport.tree(scenario.base);
    for (path, content) in dirty_files(b"conflict.txt") {
        scenario.checkout.write(&[scenario.at.as_slice(), b"/", &path].concat(), &content);
    }
    let commit = translate::commit(scenario.base);
    for command in [Command::CheckOut { commit }, Command::Clean] {
        let answer = hosted::apply(&mut scenario, &command);
        assert_eq!(answer.exit, 0, "{command:?}: {answer:?}");
    }
    assert_eq!(tree_without_merge(scenario.checkout.tree(&scenario.at)), expected);
    assert_eq!(scenario.checkout.tree(&[scenario.at.as_slice(), b"/.git/temper-index"].concat()), expected);
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
            assert!(matches!(world.client(first).prepared, Some(Prepared::Failed { failure: Failure::Transient, .. })));
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
            World::new(Settings { checkout: Limits { workspaces: 2, facts: 2, ..LIMITS }, ..settings(reuse, 1) });
        world.submit(Time::ZERO, plan(0));
        world.submit(Time::ZERO, plan(1));
        world.run(ITERATIONS);
        let before = world.told().1;
        world.restart();
        let after = world.told().1;
        let client = world.submit(world.now(), plan(0));
        world.run(ITERATIONS);
        ready(&world, client);
        assert!(before > 0, "simultaneous holds exceed the two-fact queue");
        assert_eq!(after, before, "the cumulative fact counters retain their lost count");
    }
}
