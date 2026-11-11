//! Transfer accounting (temper/docs/design/domain/worker/checkouts.md).
use skein_lib::{Duration, Time};
use std::collections::BTreeSet;
use temper_fake_checkout::{git::Remote, in_git};
use temper_worker_checkout_world::{
    LIMITS, Settings, World,
    client::{Pick, Plan},
};
use temper_worker_domain_checkout::{Landing, Limits, Prepared, git::Kind};
const ITERATIONS: u32 = 100_000;
fn plan(workstream: u32) -> Plan {
    Plan { start: Some(Pick::Branch), pushes: 0, ..Plan::simple(workstream) }
}
fn world(workspaces: u32) -> World {
    let mut world = World::new(Settings {
        checkout: Limits { workspaces, ..LIMITS },
        repositories: 1,
        workstreams: 2,
        ..Settings::calm(1)
    });
    for second in 1..=3 {
        world.advance_at(Time::ZERO.saturating_add(Duration::from_secs(second)), 0, b"main");
    }
    world.run(ITERATIONS);
    world
}
fn prepare(world: &mut World, workstream: u32) -> u64 {
    let client = world.submit(world.now(), plan(workstream));
    world.run(ITERATIONS);
    ready(world, client);
    client
}
fn ready(world: &World, client: u64) {
    assert!(matches!(world.client(client).prepared, Some(Prepared::Ready { .. })));
}
fn advance(world: &mut World) -> u64 {
    world.advance_at(world.now(), 0, b"main");
    world.run(ITERATIONS);
    world.forge().branch(world.repository(0), b"main").expect("main exists")
}
fn imported(world: &World, client: u64) -> BTreeSet<u64> {
    let transfers = world.transfers(client, b"r0");
    assert!(!transfers.is_empty(), "the ledger observed clone or fetch operations");
    transfers.iter().flat_map(|transfer| transfer.commits.iter().copied()).collect()
}
fn exact_tree(world: &World, client: u64, commit: u64) {
    let workspace = world.client(client).workspace.expect("ready workspace");
    let path = [temper_worker_checkout_world::translate::dir(workspace).as_slice(), b"/r0"].concat();
    let mut tree = world.disk_tree(&path);
    tree.retain(|path, _| !in_git(path));
    assert_eq!(tree, world.forge().tree(commit));
}
#[test]
fn another_workspace_fetches_only_the_new_commit() {
    let mut world = world(2);
    let first = prepare(&mut world, 0);
    assert_eq!(imported(&world, first).len(), 4);
    let transfers = world.transfers(first, b"r0");
    assert_eq!(transfers[0].kind, Kind::Store);
    assert_eq!(transfers[0].commits.len(), 4, "the Store brings main's history");
    assert!(
        transfers.iter().skip(1).all(|transfer| transfer.commits.is_empty()),
        "Clone and main's fetch bring nothing more"
    );
    let tip = advance(&mut world);
    let next = prepare(&mut world, 1);
    assert_ne!(world.client(first).workspace, world.client(next).workspace);
    exact_tree(&world, next, tip);
    assert_eq!(imported(&world, next), BTreeSet::from([tip]), "B brings one commit, not the history");
}
#[test]
fn an_evicted_workspace_fetches_only_the_new_commit() {
    let mut world = world(1);
    let first = prepare(&mut world, 0);
    let tip = advance(&mut world);
    let next = prepare(&mut world, 1);
    assert_eq!(world.client(first).workspace, world.client(next).workspace);
    exact_tree(&world, next, tip);
    assert_eq!(world.told().0.evicted, 1);
    assert_eq!(imported(&world, next), BTreeSet::from([tip]), "eviction brings one commit, not the history");
}
#[test]
fn a_saved_start_fetches_only_the_saved_commit_the_store_lacked() {
    let mut world = world(1);
    let saver = world.submit(world.now(), Plan { save: true, ..plan(0) });
    world.run(ITERATIONS);
    ready(&world, saver);
    let Landing::Landed { commit, .. } = world.client(saver).landings[0].1[0] else { panic!("saved edit") };
    let tip = temper_worker_checkout_world::translate::fake(commit);
    let makes = world.told().0.makes;
    world.write_disk(b"ws/0/r0/UNTRACKED", b"discard me");
    let resumer = world.submit(world.now(), Plan { start: Some(Pick::Saved), ..plan(0) });
    world.run(ITERATIONS);
    ready(&world, resumer);
    assert_eq!(world.told().0.makes, makes + 1);
    assert_eq!(world.client(resumer).start, world.client(saver).left);
    exact_tree(&world, resumer, tip);
    assert_eq!(imported(&world, resumer), BTreeSet::from([tip]), "only the saved commit was not fetched before");
}
#[test]
fn a_reused_workspace_fetches_only_new_commits_and_nothing_when_unchanged() {
    let mut world = world(1);
    prepare(&mut world, 0);
    let tip = advance(&mut world);
    let next = prepare(&mut world, 0);
    exact_tree(&world, next, tip);
    assert_eq!(imported(&world, next), BTreeSet::from([tip]));
    let unchanged = prepare(&mut world, 0);
    assert!(imported(&world, unchanged).is_empty());
    assert_eq!(world.told().0.reused, 2);
}
#[test]
fn a_clone_transfers_no_objects() {
    let mut world = world(1);
    let first = prepare(&mut world, 0);
    let clones: Vec<_> = world.transfers(first, b"r0").iter().filter(|transfer| transfer.kind == Kind::Clone).collect();
    assert_eq!(clones.len(), 1);
    assert!(clones[0].commits.is_empty(), "clone transferred {:?}", clones[0].commits);
}
