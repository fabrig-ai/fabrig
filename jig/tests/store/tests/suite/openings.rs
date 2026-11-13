//! Openings belong to the last start and are reported only by rehearsal (protocol/store/records.md 3).
use jig_protocol_store::{Event, OpenedTerm, OpenedWire, PatchSet, PatchSets, Request, WorkerOpening, Write};
use jig_store_world::{World, root};
use skein_lib::{Token, Wall};

const SEED: u64 = 0x4f50_454e;
const SETS: PatchSets = PatchSets {
    sets: &[
        jig_protocol_store::SKEIN_PATCH_SET,
        jig_protocol_store::SMITH_PATCH_SET,
        jig_protocol_accounts::JIG_PATCH_SET,
    ],
};

fn opening(worker: &str, at: u64) -> WorkerOpening {
    WorkerOpening {
        worker: worker.into(),
        wires: Box::new([
            OpenedWire { name: "channel".into(), lowest: 2, layouts: Box::new([[1; 32], [2; 32]]) },
            OpenedWire { name: "turns".into(), lowest: 7, layouts: Box::new([[3; 32]]) },
        ]),
        output: 4096,
        terms: Box::new([OpenedTerm { kind: 3, largest: 81 }, OpenedTerm { kind: 9, largest: 2048 }]),
        at: Wall::from_nanos(at),
    }
}
fn started() -> World {
    let mut world = World::unopened(SEED);
    world.patch_sets(SETS);
    assert_eq!(world.open(), Some(0));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(1)), Write::Wires])), Some(1));
    world
}
fn rehearse(world: &mut World, expected: &[WorkerOpening]) {
    world.crash(SEED);
    let before = world.contents();
    let events = world.rehearse();
    let Some(Event::Rehearsing { openings, .. }) = events.first() else { panic!("{events:?}") };
    assert_eq!(openings.as_ref(), expected);
    assert!(
        matches!(events.get(1), Some(Event::Decoded { records, .. }) if *records == 1 + u64::try_from(expected.len()).expect("opening count")),
        "every opening is walked by its kept text: {events:?}"
    );
    assert!(matches!(events.last(), Some(Event::Opened { pending: false, .. })), "{events:?}");
    assert_eq!(world.contents(), before);
    let events = world.request(Request::Close { owner: Token::new(900) });
    assert!(matches!(events.as_slice(), [Event::Closed { removed: true, .. }]), "{events:?}");
}

#[test]
fn rehearsal_keeps_whole_openings_in_workers_name_order_without_loading_them_at_start() {
    let mut world = started();
    let w1 = opening("w1", 1_700_000_000_123_456_789);
    let w2 = opening("w2", 1_700_000_001_987_654_321);
    assert_eq!(
        world.commit(Box::new([Write::Save(root::header(2)), Write::Opening(w2.clone()), Write::Opening(w1.clone())])),
        Some(2)
    );
    world.crash(SEED);
    let restored = world.reopen();
    assert_eq!(restored.last, 2);
    assert_eq!(restored.rows.values().cloned().collect::<Vec<_>>(), vec![root::header(2)], "a start loads no opening");
    rehearse(&mut world, &[w1, w2]);
}

#[test]
fn a_later_opening_replaces_only_that_worker() {
    let mut world = started();
    let old = opening("w1", 1);
    let other = opening("w2", 2);
    assert_eq!(
        world.commit(Box::new([Write::Save(root::header(2)), Write::Opening(old), Write::Opening(other.clone())])),
        Some(2)
    );
    let mut later = opening("w1", 3);
    later.output = 1;
    later.wires = Box::new([]);
    later.terms = Box::new([OpenedTerm { kind: 5, largest: u32::MAX }]);
    assert_eq!(world.commit(Box::new([Write::Save(root::header(3)), Write::Opening(later.clone())])), Some(3));
    rehearse(&mut world, &[later, other]);
}

#[test]
fn wires_clears_all_openings_in_its_commit_and_later_openings_survive() {
    let mut world = started();
    assert_eq!(
        world.commit(Box::new([
            Write::Save(root::header(2)),
            Write::Opening(opening("w2", 1)),
            Write::Opening(opening("w1", 2))
        ])),
        Some(2)
    );
    world.crash(SEED);
    assert_eq!(world.open(), Some(2));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(3)), Write::Wires])), Some(3));
    rehearse(&mut world, &[]);
    world.crash(SEED);
    assert_eq!(world.open(), Some(3));
    let later = opening("w1", 4);
    assert_eq!(world.commit(Box::new([Write::Save(root::header(4)), Write::Opening(later.clone())])), Some(4));
    rehearse(&mut world, &[later]);
}

#[test]
fn wires_clears_accepted_openings_even_before_they_are_durable() {
    let mut world = started();
    world.enqueue(Request::Commit {
        number: 2,
        writes: Box::new([Write::Save(root::header(2)), Write::Opening(opening("w1", 2))]),
    });
    world.enqueue(Request::Commit {
        number: 3,
        writes: Box::new([
            Write::Save(root::header(3)),
            Write::Opening(opening("w2", 3)),
            Write::Wires,
            Write::Opening(opening("w3", 4)),
        ]),
    });
    let events = world.settle();
    assert!(
        matches!(events.as_slice(), [Event::Committed { number: 2, .. }, Event::Committed { number: 3, .. }]),
        "{events:?}"
    );
    rehearse(&mut world, &[opening("w3", 4)]);
}

#[test]
fn a_store_from_before_the_family_opens_and_rehearses_without_openings() {
    let jig = jig_protocol_accounts::JIG_PATCH_SET;
    let families =
        jig.families.iter().filter(|family| family.name != "jig-store-openings").copied().collect::<Vec<_>>();
    let old = PatchSets {
        sets: Box::leak(Box::new([
            SETS.sets[0],
            SETS.sets[1],
            PatchSet { families: Box::leak(families.into_boxed_slice()), ..jig },
        ])),
    };
    let mut world = World::unopened(SEED);
    world.patch_sets(old);
    assert_eq!(world.open(), Some(0));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(1)), Write::Wires])), Some(1));
    world.crash(SEED);
    world.patch_sets(SETS);
    assert_eq!(world.open(), Some(1), "the layouts row has no openings field, and needs none");
    assert!(world.patched().is_empty(), "introducing the family rewrites no row");
    rehearse(&mut world, &[]);
}
