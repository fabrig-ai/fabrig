//! The run's view of main's counts across holds and deliveries.

use super::{World, design_spec, design_world, prepare_single, release_heads};
use crate::translate;
use skein_lib::Token;
use temper_worker_domain_checkout::Landing;

const REMOTE: &[u8] = b"forge/temper";
const FIRST: &[u8] = b"{\"checks\":1}\n";
const SECOND: &[u8] = b"{\"checks\":2}\n";
const THIRD: &[u8] = b"{\"checks\":3}\n";

fn prepare_goal(world: &mut World) -> Vec<u8> {
    let tip = world.forge.branch(REMOTE, b"g").expect("the goal's branch");
    let (_, at) = prepare_single(world, design_spec(tip));
    at
}

pub(super) fn assert_ref(world: &World, at: &[u8], expected: Option<u64>) {
    let reference = [at, b"/.git/refs/temper/counts"].concat();
    assert_eq!(world.disk.content(&reference), expected.map(u64::to_le_bytes).as_ref().map(<[u8; 8]>::as_slice));
}

pub fn present() {
    let mut world = design_world();
    let counts = world.forge.publish_counts(REMOTE, FIRST);
    let at = prepare_goal(&mut world);
    assert_ref(&world, &at, Some(counts));

    let digest = skein_lib::sha256::digest(REMOTE);
    let store = Token::new(u64::from_be_bytes(*digest.first_chunk::<8>().expect("store token")));
    let store = translate::store_path(store);
    let object = format!("/objects/trees/{counts}/counts.json");
    assert_eq!(world.disk.content(&[store.as_slice(), object.as_bytes()].concat()), Some(FIRST));
    assert!(
        !world.disk.exists(&[at.as_slice(), b"/.git", object.as_bytes()].concat()),
        "the store has the blob without a content fetch into the checkout"
    );
    release_heads(&mut world);
}

pub fn absent() {
    let mut world = design_world();
    let at = prepare_goal(&mut world);
    assert_ref(&world, &at, None);
    release_heads(&mut world);
}

fn prepare_cached_goal(world: &mut World) -> Vec<u8> {
    world.trace.clear();
    let at = prepare_goal(world);
    assert!(world.trace.iter().all(|line| !line.contains("op: Make {")), "no new workspace over the cached one");
    at
}

fn refreshed_goal() -> (World, Vec<u8>, u64) {
    let mut world = design_world();
    let first = world.forge.publish_counts(REMOTE, FIRST);
    let at = prepare_goal(&mut world);
    assert_ref(&world, &at, Some(first));
    release_heads(&mut world);

    let second = world.forge.publish_counts(REMOTE, SECOND);
    let at = prepare_cached_goal(&mut world);
    (world, at, second)
}

pub fn refreshed_and_removed() {
    let (mut world, at, second) = refreshed_goal();
    assert_ref(&world, &at, Some(second));
    release_heads(&mut world);
    world.forge.remove_counts(REMOTE);
    let at = prepare_cached_goal(&mut world);
    assert_ref(&world, &at, None);
    release_heads(&mut world);
}

pub fn frozen_through_delivery() {
    let (mut world, at, second) = refreshed_goal();
    world.forge.publish_counts(REMOTE, THIRD);
    world.disk.write(&[at.as_slice(), b"/design.md"].concat(), b"Design\n");
    let landings = world.push(false, false);
    assert!(matches!(landings.as_ref(), [Landing::Landed { .. }]));
    assert_ref(&world, &at, Some(second));
    release_heads(&mut world);
}
