//! Fault models through the checkout, not just the fake (testing/worker.md, 3.1 and 3.2).

use skein_lib::{Duration, Time};
use skein_world::faults::{Counter, FaultLedger, Kind as FaultKind, Struck};
use skein_world::streams::{Stream, Streams};
use temper_worker_checkout_world::client::{Pick, Plan};
use temper_worker_checkout_world::{LIMITS, Settings, World, translate};
use temper_worker_domain_checkout::git::Kind;
use temper_worker_domain_checkout::{Failure, Limits, Prepared};
use temper_worker_inspect::faults::{DISK_FULL, FETCH_CUT};

const ITERATIONS: u32 = 10_000;

fn world(streams: Streams) -> World {
    let mut world = World::new(Settings {
        seed: streams.seed_of(Stream::Workload, "worker"),
        checkout: Limits { workspaces: 2, preparations: 2, ..LIMITS },
        repositories: 1,
        workstreams: 2,
        ..Settings::calm(streams.seed())
    });
    world.set_repositories(0, &[0]);
    world.set_repositories(1, &[0]);
    world
}

fn prepare(world: &mut World, workstream: u32) -> u64 {
    let client = world.submit(world.now(), Plan { start: Some(Pick::Branch), pushes: 0, ..Plan::simple(workstream) });
    world.run(ITERATIONS);
    client
}

fn ready(world: &World, client: u64) {
    assert!(matches!(world.client(client).prepared, Some(Prepared::Ready { .. })), "{:?}", world.client(client));
}

fn failed<'a>(world: &'a World, client: u64, stage: &str, fault: &str) -> &'a [u8] {
    let Some(Prepared::Failed { failure: Failure::Transient, detail }) = &world.client(client).prepared else {
        panic!("expected transient prepare failure: {:?}", world.client(client));
    };
    let text = std::str::from_utf8(detail).expect("text detail");
    assert!(text.starts_with(&format!("{stage}: {fault} (")), "{text}");
    detail
}

fn ledger(ledger: &FaultLedger, kind: FaultKind, world: &World, client: u64, operation: &str) {
    let [entry] = ledger.entries() else { panic!("one fault: {ledger:?}") };
    assert_eq!(entry.kind, kind);
    assert_eq!(
        entry.struck,
        Struck {
            request: Some(format!("hold {} {operation}", world.client(client).hold.expect("admitted").raw())),
            ..Struck::default()
        }
    );
    for counter in [
        Counter::ProviderRetries,
        Counter::SessionRetries,
        Counter::TaskTries,
        Counter::GateRetries,
        Counter::Repairs,
        Counter::Asks,
    ] {
        assert_eq!(entry.count(counter), 0, "worker faults spend no tries");
    }
}

#[test]
fn a_fetch_cut_at_its_deadline_is_made_again() {
    cut_is_made_again(Kind::Fetch, "fetch", "fetch");
}

#[test]
fn a_checkout_cut_at_its_deadline_is_made_again() {
    cut_is_made_again(Kind::CheckOut, "check out", "checkout");
}

fn cut_is_made_again(kind: Kind, stage: &str, operation: &str) {
    let streams = Streams::new(1);
    let mut world = world(streams);
    let other = prepare(&mut world, 1);
    let first = prepare(&mut world, 0);
    ready(&world, other);
    ready(&world, first);
    let other_path = translate::dir(world.client(other).workspace.expect("ready"));
    let before_other = world.disk_tree(&other_path);
    let workspace = translate::dir(world.client(first).workspace.expect("ready"));
    let later = world.now().saturating_add(Duration::from_secs(1));
    world.advance_at(later, 0, b"main");
    world.run(ITERATIONS);
    let head = world.forge().branch(world.repository(0), b"main").expect("main");
    world.cut_next(kind);
    let client = prepare(&mut world, 0);
    let detail = failed(&world, client, stage, "TimedOut");
    assert!(std::str::from_utf8(detail).expect("detail").contains("operation 60000 ms of 60000 ms"));
    ledger(world.faults(), FETCH_CUT, &world, client, operation);
    let entry = &world.faults().entries()[0];
    assert_eq!(entry.until, Some(entry.from), "a cut is an instant");
    assert_eq!(entry.from, failure_time(&world, client), "the fault falls at the timed-out terminal");
    let before_heal = world.disk_tree(b"stores");
    let object = format!("/objects/{head}").into_bytes();
    let imported: Vec<_> = before_heal.iter().filter(|(path, _)| path.ends_with(&object)).collect();
    assert!(!imported.is_empty(), "the store imported the new head");
    match kind {
        Kind::Fetch => assert!(before_heal.keys().any(|path| path.ends_with(b"/fetched.lock"))),
        Kind::CheckOut => assert!(world.disk_tree(&workspace).contains_key(b"r0/.git/index.lock".as_slice())),
        _ => unreachable!("the focused cut cases"),
    }
    assert_eq!(world.disk_tree(&other_path), before_other, "the cut touches no other workspace");
    let makes = world.told().0.makes;
    let next = prepare(&mut world, 0);
    ready(&world, next);
    assert_eq!(world.told().0.makes, makes + 1, "the untrusted workspace is made again");
    for tree in [world.disk_tree(b"stores"), world.disk_tree(b"ws")] {
        assert!(!tree.keys().any(|path| path.ends_with(b".lock")), "no trusted lock remains");
    }
    let after_heal = world.disk_tree(b"stores");
    for (path, content) in imported {
        assert_eq!(after_heal.get(path), Some(content), "Make keeps imported store objects");
    }
    assert_eq!(world.disk_tree(&other_path), before_other, "recovery touches no other workspace");
}

#[test]
fn a_full_disk_fails_releases_and_recovers_after_make() {
    let streams = Streams::new(2);
    let mut world = world(streams);
    let first = prepare(&mut world, 0);
    ready(&world, first);
    let workspace = translate::dir(world.client(first).workspace.expect("ready"));
    world.write_disk(&[workspace.as_slice(), b"/discarded"].concat(), &[b'x'; 4096]);
    let mut faults = streams.rng(Stream::Faults, "worker");
    let capacity = world.disk_used() + faults.between(1, 4);
    world.limit_disk(capacity - world.disk_used());
    // Fetch adds a pack, taking usage past capacity; the fetch of the push branch is refused.
    let client = prepare(&mut world, 0);
    let detail = failed(&world, client, "fetch push", "Broken");
    assert!(std::str::from_utf8(detail).expect("detail").ends_with(": No space left on device"));
    ledger(world.faults(), DISK_FULL, &world, client, "fetch");
    assert_eq!(world.faults().entries()[0].until, None, "no success after freeing yet");
    assert!(world.disk_used() >= capacity);
    assert_eq!(world.faults().entries()[0].from, failure_time(&world, client));
    let makes = world.told().0.makes;
    let trace_start = world.trace().len();
    let next = prepare(&mut world, 0);
    ready(&world, next);
    assert_eq!(world.told().0.makes, makes + 1);
    assert!(world.disk_used() < capacity, "Make frees the disposable bytes");
    assert!(!world.disk_tree(&workspace).contains_key(b"discarded".as_slice()));
    let entry = &world.faults().entries()[0];
    let until = entry.until.expect("the first passing operation after Make closes the interval");
    assert!(until > entry.from);
    let terminals: Vec<_> =
        world.trace()[trace_start..].iter().filter(|line| line.contains("checkout <- done ")).collect();
    assert!(terminals[0].ends_with("Succeeded"), "Make freed space");
    assert!(terminals[1].ends_with("Succeeded"), "the next operation (Clone) passed");
    assert_eq!(until, time(terminals[1]), "the first success after freeing stops the fault");
}

fn time(line: &str) -> Time {
    Time::from_nanos(line.split_whitespace().next().expect("trace time").parse().expect("nanoseconds"))
}

fn failure_time(world: &World, client: u64) -> Time {
    let prefix = format!("checkout <- done {} Failed", world.client(client).hold.expect("admitted").raw());
    time(world.trace().iter().rfind(|line| line.contains(&prefix)).expect("the failed operation's terminal"))
}
