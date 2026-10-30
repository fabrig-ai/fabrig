//! Seeded pages against independently committed inputs (store.md, 7.1 and 14).
use jig_protocol_store::{Restore, Row, Write, keys, restore::Item};
use jig_store_world::{World, root};
use skein_lib::Rng;
use std::collections::BTreeMap;

// Each page must hold the two-byte application key and largest 32-byte value.
const APPLICATION_BYTES: u64 = 34;
const INVENTORY_BYTES: u64 = 65_536;
const SEEDS: u64 = 32;
const RECORDS: u64 = 20;

#[test]
fn seeded_pages_match_committed_application_rows() {
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed);
        let mut world = World::new(seed);
        world.commit(root::first()).expect("first commit");
        let expected: BTreeMap<Box<[u8]>, Box<[u8]>> = (0..RECORDS)
            .map(|number| {
                let key = Box::from([u8::try_from(number).expect("twenty rows"), 0]);
                let value =
                    vec![u8::try_from(number).expect("twenty rows"); usize::try_from(rng.below(33)).expect("value")]
                        .into_boxed_slice();
                (key, value)
            })
            .collect();
        let mut writes = vec![Write::Save(root::header(2))];
        writes.extend(expected.iter().map(|(key, value)| Write::Put { key: key.clone(), value: value.clone() }));
        world.commit(writes.into_boxed_slice()).expect("source commits");
        let mut found = BTreeMap::new();
        let mut after = None;
        loop {
            let rows = u32::try_from(1 + rng.below(4)).expect("row budget");
            let page = world
                .restore_page(
                    Restore::Application(Box::new([skein_kv::Range::prefix(b"")])),
                    after.as_ref(),
                    rows,
                    APPLICATION_BYTES,
                )
                .expect("typed page");
            assert!(page.items.len() <= usize::try_from(rows).expect("rows"));
            let mut bytes = 0;
            for item in page.items {
                let Item::Row(Row::Application { key, value }) = item else { panic!("application row") };
                bytes += key.len() + value.len();
                assert!(found.insert(key, value).is_none(), "exactly once, seed {seed}");
            }
            assert!(bytes <= usize::try_from(APPLICATION_BYTES).expect("bytes"));
            after = page.next;
            if after.is_none() {
                break;
            }
        }
        assert_eq!(found, expected, "typed source readback seed {seed}");
        let expected: BTreeMap<_, _> = expected
            .into_iter()
            .map(|(key, value)| (keys::application(&key, root::limits().store.key).expect("raw key"), value))
            .collect();
        let mut found = BTreeMap::new();
        let mut after = None;
        loop {
            let rows = u32::try_from(1 + rng.below(4)).expect("row budget");
            let page = world.retained_page(after.as_deref(), rows, INVENTORY_BYTES).expect("inventory page");
            assert!(page.rows.len() <= usize::try_from(rows.min(root::limits().store.page.rows)).expect("rows"));
            for row in page.rows {
                assert!(after.as_deref().is_none_or(|after| row.key.as_ref() > after));
                if row.key.first() == Some(&keys::APPLICATION) {
                    assert!(found.insert(row.key, row.value).is_none(), "exactly once, seed {seed}");
                }
            }
            after = page.next;
            if after.is_none() {
                break;
            }
        }
        assert_eq!(found, expected, "raw source readback seed {seed}");
    }
}
