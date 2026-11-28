//! A child refuses its pending restore before activation (protocol/store/opening.md 2, step 7).
//! The store checks encodings and counts; the modelled child's stricter rule
//! runs over the copy. Ending that start leaves the old generation active.
use jig_store_world::upgrade_fixtures::{self, BASE, Records, SETS};
use jig_test_layouts::records::SHELF;

/// A referee rule standing in for a child's restore rule (domain/engine/restart.md);
/// the store world has no children, and jig's store has no such rule.
/// This child accepts shelves with at most two books.
fn a_child_refuses(key: &[u8], bytes: &[u8]) -> bool {
    if !key.starts_with(b"s/") {
        return false;
    }
    let schema = skein_codec::schema::parse(SHELF.as_bytes()).expect("kept shelf schema");
    let root = schema.find(b"Shelf").expect("shelf root");
    let mut value = skein_codec::value::Value::with_room(16);
    let shelf = value.decode(&schema, root, bytes.into()).expect("upgraded shelf decodes");
    let books = value.field(shelf, skein_codec::schema::FieldIx::new(1)).expect("shelf books field");
    value.uint(books).expect("weight is a u32") > 2
}

#[test]
fn a_record_a_child_refuses_before_switch_keeps_the_old_generation_active() {
    const SEED: u64 = 911;
    let records = Records::story();
    let mut world = records.committed(SEED, BASE);
    let original = upgrade_fixtures::source(&mut world);
    world.owner_sets(SETS);
    let last = world.open().expect("pending upgraded open");
    assert!(world.pending);
    assert_eq!(world.active(), b"1\n");
    records.check_public(&mut world, &"pending restore");

    let key = b"s/2".as_slice();
    let expected = records.rows().get(key).expect("story's heaviest shelf").clone();
    // The modelled child ends this start before asking for Switch.
    let refused: Vec<_> =
        world.application(false).into_iter().filter(|(key, bytes)| a_child_refuses(key, bytes)).collect();
    assert_eq!(refused, [(key.into(), expected.clone())]);
    world.crash(SEED);

    for start in ["first further start", "second further start"] {
        assert_eq!(world.open(), Some(last), "{start}");
        assert!(!world.ran().is_empty(), "{start} runs the job over a new copy");
        assert!(world.pending);
        assert_eq!(world.active(), b"1\n", "{start}");
        let refused: Vec<_> =
            world.application(false).into_iter().filter(|(key, bytes)| a_child_refuses(key, bytes)).collect();
        assert_eq!(refused, [(key.into(), expected.clone())], "{start}");
        upgrade_fixtures::kept(&original, &upgrade_fixtures::source(&mut world), &start);
        world.crash(SEED);
    }

    // The old binary starts against A without any operator rewriting active.
    world.owner_sets(BASE);
    assert_eq!(world.open_switched(), Some(1));
    world.accept_recovery(1);
    assert_eq!(world.ran(), []);
    assert_eq!(world.active(), b"1\n");
    upgrade_fixtures::kept(&original, &upgrade_fixtures::source(&mut world), &"operator's way back");
    records.check(&mut world, &"operator's way back");
    assert!(!world.paths().iter().any(|path| path == b"generations/2" || path.starts_with(b"generations/2/")));
}
