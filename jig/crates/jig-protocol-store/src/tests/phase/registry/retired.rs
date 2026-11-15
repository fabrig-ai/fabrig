//! Retired ranges are probed even without registry rows (protocol/store/layouts.md, 8).
use super::settling::{JOB, key, layout, set};
use super::*;
use crate::Retired;

fn family(owner: Owner) -> KeptFamily {
    let base = settling::family(owner);
    KeptFamily {
        current: CURRENT,
        types: Box::leak(Box::new([base.types[0]])),
        retired: Box::leak(Box::new([Retired { number: 1, keys: base.types[1].keys }])),
        ..base
    }
}
fn baseline(owner: Owner, registry_row: bool, value: bool) -> Vec<Item> {
    let mut disk = Disk::new(&[]);
    let base = KeptFamily { types: &[], retired: &[], ..family(owner) };
    let mut store = fresh(&mut disk, set(owner, base, &[]));
    let mut ops = layout(CURRENT, 0, 2);
    if registry_row {
        ops.extend(layout(WRITTEN, 1, 9));
    }
    if value {
        ops.push(skein_kv::Op::Put { key: key(owner, 1, 0), value: Box::from([0, 9, 1]) });
    }
    raw(&mut disk, &mut store, ops);
    disk.items()
}
#[test]
fn retired_ranges_refuse_values_before_writing_with_or_without_a_registry_row() {
    for owner in [Owner::Application, Owner::Jig] {
        for registry in [false, true] {
            let mut disk = Disk::new(&baseline(owner, registry, true));
            let before = snapshot(&mut disk);
            let mut store = store(&disk, set(owner, family(owner), &[JOB]));
            let events = disk.open(&mut store);
            assert!(
                {
                    if let [
                        Event::OpenRefused {
                            why:
                                OpenRefusal::Layout { family, value_type: 1, id: 9, values: None, why: Unplanned::Retired },
                            ..
                        },
                    ] = events.as_ref()
                    {
                        family.as_ref() == FAMILY.name
                    } else {
                        false
                    }
                },
                "{events:?}"
            );
            assert_eq!(snapshot(&mut disk), before);
            assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
            assert!(!disk.tree().iter().any(|(p, ..)| p.as_ref() == b"generations/2"));
        }
    }
}
#[test]
fn retired_rows_and_orphan_texts_wait_for_a_writing_open() {
    for owner in [Owner::Application, Owner::Jig] {
        for writing in [false, true] {
            let mut disk = Disk::new(&baseline(owner, true, false));
            let mut store = store(&disk, set(owner, family(owner), if writing { &[JOB] } else { &[] }));
            let events = disk.open(&mut store);
            assert!(
                {
                    if let Some(Event::Opened { pending, .. }) =
                        events.iter().find(|event| if let Event::Opened { .. } = event { true } else { false })
                    {
                        *pending == writing
                    } else {
                        false
                    }
                },
                "{events:?}"
            );
            assert_eq!(row(&mut store, 1, 9).is_none(), writing);
            assert_eq!(text(&mut store, hash(WRITTEN)).is_none(), writing);
            assert!(row(&mut store, 0, 2).is_some());
            assert!(text(&mut store, hash(CURRENT)).is_some(), "texts still named survive");
        }
    }
}
#[test]
fn putting_into_a_retired_range_fails_the_commit() {
    for owner in [Owner::Application, Owner::Jig] {
        let mut disk = Disk::new(&baseline(owner, false, false));
        let mut store = store(&disk, set(owner, family(owner), &[]));
        let opened = disk.open(&mut store);
        assert!(
            { if let [Event::Opened { pending: false, .. }, ..] = opened.as_ref() { true } else { false } },
            "{opened:?}"
        );
        let number = store.stores.store.last() + 1;
        let key = key(owner, 1, 0);
        let relative = if owner == Owner::Application { crate::keys::of_application(&key).unwrap() } else { &key };
        // Root writes are application-relative; commits::submit also covers upstream puts.
        let value = Box::from([0, 2, 1]);
        let events = if owner == Owner::Application {
            disk.request(
                &mut store,
                fixtures::commit(
                    number,
                    Box::new([fixtures::header(number), Write::Put { key: relative.into(), value }]),
                ),
            )
        } else {
            let mut above = Queue::uncapped();
            let mut below =
                crate::Below { store: Queue::uncapped(), secrets: Queue::uncapped(), transcripts: Queue::uncapped() };
            let env = fixtures::env();
            crate::commits::submit(
                &mut store,
                &env,
                number,
                Box::new([skein_kv::Op::Put { key, value }]),
                &mut above,
                &mut below,
            );
            {
                let mut events = Vec::new();
                while let Some(event) = above.pop() {
                    events.push(event);
                }
                events.into_boxed_slice()
            }
        };
        assert!({ if let [Event::Failed { .. }] = events.as_ref() { true } else { false } }, "{events:?}");
        assert_eq!(store.stores.store.last(), number - 1);
    }
}
#[test]
fn retired_admission_checks_numbers_and_the_ordered_disjoint_exact_union_of_ranges() {
    for owner in [Owner::Application, Owner::Jig] {
        let base = family(owner);
        let retired = base.retired[0];
        let range = retired.keys[0];
        let mid = Box::leak([range.from, b"/"].concat().into_boxed_slice());
        for (retired, valid) in [
            (vec![retired], true),
            (vec![], false),
            (vec![Retired { number: 0, ..retired }], false),
            (vec![retired, retired], false),
            (vec![Retired { keys: &[], ..retired }], false),
            (vec![Retired { keys: Box::leak(Box::new([KeyRange { from: range.from, to: mid }])), ..retired }], false),
            (vec![Retired { keys: base.types[0].keys, ..retired }], false),
            (
                vec![Retired { keys: Box::leak(Box::new([KeyRange { from: range.to, to: range.from }])), ..retired }],
                false,
            ),
            (
                vec![Retired {
                    keys: Box::leak(Box::new([
                        KeyRange { from: mid, to: range.to },
                        KeyRange { from: range.from, to: mid },
                    ])),
                    ..retired
                }],
                false,
            ),
        ] {
            let family = KeptFamily { retired: Box::leak(retired.into_boxed_slice()), ..base };
            let mut disk = Disk::new(&[]);
            let mut store = store(&disk, set(owner, family, &[]));
            let events = disk.open(&mut store);
            if valid {
                assert_opened(&events, 0);
            } else {
                assert!(
                    {
                        if let [Event::OpenRefused { why: OpenRefusal::Set { owner: of, patch: 0 }, .. }] =
                            events.as_ref()
                        {
                            *of == owner
                        } else {
                            false
                        }
                    },
                    "{events:?}"
                );
            }
        }
    }
}
