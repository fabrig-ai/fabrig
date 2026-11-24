//! Settled puts across power loss (jig/docs/design/protocol/store/settling.md, 6).
use jig_protocol_store::{KeptFamily, KeptMap, KeyRange, Keys, Owner, OwnerSet, OwnerSets, ValueType, Write};
use jig_store_world::{World, referee::Referee, root};
use jig_test_layouts::layouts::{CURRENT, KEPT};

const SEED: u64 = 91_481;
const FAMILY: KeptFamily = KeptFamily {
    name: "fixture-layouts",
    current: KEPT,
    map: KeptMap::Store,
    keys: &[Keys { from: b"a", to: b"b", root: "Growing" }],
    types: &[ValueType { number: 0, root: "Growing", variant: None, keys: &[KeyRange { from: b"a", to: b"b" }] }],
    retired: &[],
    frames: None,
};
const SOURCE: OwnerSet =
    OwnerSet { owner: Owner::Application, name: "fixture-layouts", families: &[FAMILY], jobs: &[], level: Some(0) };
const BINARY: OwnerSet = OwnerSet { families: &[KeptFamily { current: CURRENT, ..FAMILY }], ..SOURCE };
const KEY: &[u8] = b"a/held";
const READ: &[u8] = &[0, 2, 0, 39, 7];
const STORED: &[u8] = &[0, 3, 0, 39, 7]; // id 3 in the registry, version 2 at the read edge
const DECISION: &[u8] = b"z/decision";

fn put(key: &[u8], value: &[u8]) -> Write {
    Write::Put { key: Box::from(key), value: Box::from(value) }
}

fn serving() -> (World, u64) {
    let mut world = World::unopened(SEED);
    world.owner_sets(OwnerSets { sets: &[SOURCE] });
    assert_eq!(world.open(), Some(0));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(1)), put(KEY, &[0, 2, 0, 39])])), Some(1));
    world.crash(SEED);
    world.owner_sets(OwnerSets { sets: &[BINARY] });
    let last = world.open_switched().expect("new binary serves");
    // The upgrade wrote a new header. The fixture's logical value, including
    // the added default, is known independently of what the walk puts.
    let expected = world.model.prefixes.get_mut(&last).expect("upgraded prefix");
    expected.rows.insert(root::header(last).key(), root::header(last));
    expected.application.insert(Box::from(KEY), Box::from(READ));
    assert_eq!(world.mark(), Some(last + 1));
    (world, last + 2)
}

fn decision(number: u64) -> Box<[Write]> {
    Box::new([Write::Save(root::header(number)), put(DECISION, b"kept whole")])
}

#[test]
fn crashes_at_every_riding_commits_operation_keep_the_decoded_prefix() {
    let (mut probe, number) = serving();
    probe.trace_writes();
    assert_eq!(probe.commit(decision(number)), Some(number));
    let settled_key = jig_protocol_store::keys::application(KEY, 32).expect("fixture key");
    assert!(
        probe.written().iter().any(|(_, bytes)| {
            super::upgrades::commits::commit_keys(bytes).is_some_and(|puts| {
                puts.iter().any(|(key, value)| *key == settled_key && value.as_ref() == STORED)
                    && puts.iter().any(|(key, _)| key.as_ref() == b"\x02z/decision")
            })
        }),
        "the cut commit contains the root's put and the settled put"
    );
    for cut in 0..=probe.operation_count() {
        let (mut world, number) = serving();
        world.cut_at(cut);
        let acknowledged = world.commit(decision(number));
        world.crash(SEED);
        let restored = world.reopen();
        Referee::check(&world.model, &restored).unwrap_or_else(|why| panic!("cut {cut}: {why:?}, {restored:?}"));
        assert!(acknowledged.is_none() || restored.last == number, "acknowledged rider at cut {cut}");
        assert_eq!(restored.application.get(KEY).map(Box::as_ref), Some(READ), "decoded held value at cut {cut}");
    }
}
