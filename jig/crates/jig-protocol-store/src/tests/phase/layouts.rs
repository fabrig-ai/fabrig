//! An older build's layouts row is opaque and preserved (protocol/store/records.md, 3).
use super::*;

#[test]
fn an_old_layouts_row_survives_open_rehearsal_upgrade_and_census_unnamed() {
    // The old record was version 3 followed by ten fixed 32-byte fingerprints.
    let mut writer = skein_lib::Writer::new(322);
    writer.put(&3_u16.to_be_bytes()).unwrap();
    for family in 0..10_u8 {
        writer.put(&[family; 32]).unwrap();
    }
    let old = writer.finish();
    let mut expected = Vec::new();
    for keep in [false, true] {
        let (mut source, mut first) = source_with_sets(false, upgrade_sets(0));
        let ops = if keep { vec![skein_kv::Op::Put { key: Box::new([0, 3]), value: old.clone() }] } else { vec![] };
        registry::raw(&mut source, &mut first, ops);
        let items = source.items();
        for stage in 0..3 {
            let mut disk = Disk::new(&items);
            let mut store = disk.store_with_chunk(CHUNK);
            store.owner_sets(upgrade_sets(usize::from(stage != 0)));
            let events = if stage == 1 {
                let directory = disk.root();
                let scratch = disk.scratch_root();
                disk.request(
                    &mut store,
                    Request::Rehearse {
                        owner: Token::new(1),
                        directory,
                        scratch,
                        installation: fixtures::INSTALLATION,
                        reach: crate::Reach::Whole,
                    },
                )
            } else {
                disk.open(&mut store)
            };
            let events = finish_empty_jobs(&mut disk, &mut store, events);
            let [.., Event::Opened { last, .. }, Event::Census { census }] = events.as_slice() else {
                panic!("opened and censused: {events:?}")
            };
            assert_eq!(*last, if stage == 0 { 2 } else { 4 });
            let named: Vec<_> = census
                .keyspaces
                .iter()
                .filter_map(|space| {
                    if space.prefix.as_ref() == [0, 3] {
                        assert_eq!(space.name.as_ref(), "");
                        assert_eq!(space.rows, 1);
                        None
                    } else {
                        Some((space.prefix.clone(), space.name.clone(), space.rows, space.keys, space.values))
                    }
                })
                .collect();
            if keep {
                assert_eq!(named, expected[stage]);
            } else {
                expected.push(named);
            }
            assert_eq!(fixtures::get_kv(&mut store, &[0, 3]), keep.then(|| old.clone()));
            read_row(&mut disk, &mut store);
            if stage == 2 {
                let events = disk.request(&mut store, Request::Switch { owner: Token::new(1) });
                let [Event::Switched { .. }] = events.as_ref() else { panic!("switched: {events:?}") };
                assert_eq!(fixtures::get_kv(&mut store, &[0, 3]), keep.then(|| old.clone()));
            }
        }
    }
}
