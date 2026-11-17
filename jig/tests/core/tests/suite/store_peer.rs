use jig_core_world::Fault;
use jig_core_world::Store;
use jig_core_world::client_peer::ClientPeer;
use jig_core_world::fixtures::sign_in as signin;
use jig_core_world::store_peer::StorePeer;
use jig_core_world::world::Cut;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_test_domain::{self as root, Write};
use skein_lib::{Duration, Time, Token};

fn sign_in(number: u8) -> root::Event {
    signin(Token::new(7000 + u64::from(number)), 0, &[number, 255], &[number], b"snapshot party")
}

fn observed_world(seed: u64) -> World {
    let mut world = World::new(Setup { seed, ..Setup::default() });
    assert!(world.store_peer.is_none(), "default bootstrap has no capturing peer");
    let mut peer = StorePeer::after(world.store.applied);
    peer.capture(Time::ZERO, &world.store);
    assert!(peer.take().is_empty(), "installing after bootstrap does not reclassify it");
    world.store_peer = Some(peer);
    world.client_peer = Some(ClientPeer::default());
    world
}

#[test]
fn an_applied_world_transaction_is_captured_before_its_held_client_answer() {
    let mut world = observed_world(641);
    let before = world.store.applied;
    world.cut = Some(Cut::Durable(before + 1));
    world.store.fault(Fault::Hold { commits: 1 });
    world.send(sign_in(11));
    assert_eq!(world.store.applied, before + 1);
    assert!(world.client_peer.as_ref().expect("client").delivered.is_empty());
    assert_eq!(world.store.held.front(), Some(&world.store.applied));
    let snapshots = world.store_peer.as_mut().expect("store peer").take();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].commit, world.store.applied, "the actual numbered durable transaction");
    assert_eq!(snapshots[0].rows, world.store.rows, "the complete externally applied store");
    assert!(snapshots[0].at > Time::ZERO);
    assert!(snapshots[0].at.as_nanos() <= world.wall_time().as_nanos());
    assert_eq!(world.reached_cut, Some(Cut::Durable(world.store.applied)));
    assert_eq!(snapshots[0].at, Time::from_nanos(world.wall_time().as_nanos()), "the exact public applied tick");
}

#[test]
fn repeated_drains_and_released_acknowledgements_do_not_duplicate_a_snapshot() {
    let mut world = observed_world(642);
    world.store.fault(Fault::Hold { commits: 1 });
    world.send(sign_in(12));
    let applied = world.store.applied;
    assert_eq!(world.store_peer.as_mut().expect("store peer").take().len(), 1);
    for _ in 0..3 {
        world.drain();
    }
    assert_eq!(world.store.applied, applied);
    assert_eq!(world.store_peer.as_mut().expect("store peer").take().len(), 0);
    world.release_commits();
    assert_eq!(world.store.applied, applied);
    assert_eq!(world.client_peer.as_ref().expect("client").delivered.len(), 1);
    assert_eq!(world.store_peer.as_mut().expect("store peer").take().len(), 0);
    let mut empty = StorePeer::new();
    empty.capture(Time::from_nanos(1), &Store::new(&jig_test_profile::limits(), &[]));
    assert!(empty.take().is_empty(), "no invented Durable0");
}

#[test]
fn a_single_drain_preserves_every_intermediate_applied_number_and_whole_snapshot() {
    let mut world = observed_world(643);
    let before = world.store.applied;
    world.store.fault(Fault::Hold { commits: 3 });
    world.send_all(vec![sign_in(13), sign_in(14), sign_in(15)]);
    let snapshots = world.store_peer.as_mut().expect("store peer").take();
    assert_eq!(snapshots.len(), 3, "three real sign-in transactions in one drain");
    let actual: Vec<_> = world.store.held.iter().copied().collect();
    assert_eq!(snapshots.iter().map(|row| row.commit).collect::<Vec<_>>(), actual);
    assert_eq!(actual, vec![before + 1, before + 2, before + 3]);
    assert!(snapshots.windows(2).all(|pair| pair[0].at < pair[1].at));
    assert!(snapshots.windows(2).all(|pair| pair[0].rows != pair[1].rows));
    assert_eq!(snapshots.last().expect("last").rows, world.store.rows);
    assert!(world.client_peer.as_ref().expect("client").delivered.is_empty());
}

#[test]
fn supplied_monotonic_time_is_exact_and_independent_of_wall_corrections() {
    let mut store = Store::new(&jig_test_profile::limits(), &[]);
    let mut peer = StorePeer::new();
    let row = opaque_turn(b"independent wall timestamp");
    store.submit(1, writes(1, Write::Save(row)), 0);
    assert_eq!(store.tick(false), Ok(Some(1)));
    peer.capture(Time::from_nanos(7), &store);
    assert_eq!(peer.take()[0].at, Time::from_nanos(7), "no conversion from the row's unrelated Wall");
    let mut ordinary = observed_world(644);
    let mut corrected = observed_world(644);
    corrected.correct_wall_backwards(Duration::from_secs(60));
    ordinary.send(sign_in(16));
    corrected.send(sign_in(16));
    let first = ordinary.store_peer.as_mut().expect("store peer").take();
    let second = corrected.store_peer.as_mut().expect("store peer").take();
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(first[0].at, second[0].at, "Wall corrections never alter the supplied monotonic tick");
    assert_eq!(first[0].commit, second[0].commit);
}

use crate::store_peer_support::{opaque_turn, writes};
