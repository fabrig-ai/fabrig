//! Store-owned settling over both fixture owners (protocol/store/jobs.md, 6).
use super::*;
use crate::Job;
use jig_test_patches::layouts::{SETTLED, SETTLING_CONVERTING, SETTLING_DROPPING, SETTLING_KEEPING};

pub(super) const JOB: Job = Job { name: "settle-fixtures", keys: &[], converts: None, settles: true, counts: &[] };
pub(super) fn family(owner: Owner) -> KeptFamily {
    if owner == Owner::Application {
        return KeptFamily { current: SETTLED, ..FAMILY };
    }
    KeptFamily {
        current: SETTLED,
        keys: &[
            Keys {
                from: jig_test_patches_jig::keys::NOTES_FROM,
                to: jig_test_patches_jig::keys::NOTES_TO,
                root: "Growing",
            },
            Keys { from: b"\x02s", to: b"\x02t", root: "Stable" },
        ],
        types: &[
            ValueType { number: 0, root: "Growing", variant: None, keys: &[KeyRange { from: b"\x02n", to: b"\x02o" }] },
            ValueType { number: 1, root: "Stable", variant: None, keys: &[KeyRange { from: b"\x02s", to: b"\x02t" }] },
        ],
        ..FAMILY
    }
}
pub(super) fn set(owner: Owner, family: KeptFamily, jobs: &'static [Job]) -> PatchSet {
    PatchSet { owner, name: "fixture-layouts", families: Box::leak(Box::new([family])), jobs, ..SET }
}
pub(super) fn key(owner: Owner, ty: usize, suffix: u8) -> Box<[u8]> {
    let from = family(owner).types[ty].keys[0].from;
    let key = [from, b"/", &[suffix]].concat();
    if owner == Owner::Application { crate::keys::application(&key, 1024).unwrap() } else { key.into() }
}
pub(super) fn layout(text: &str, number: u16, id: u16) -> Vec<skein_kv::Op> {
    let schema = skein_codec::schema::parse(text.as_bytes()).unwrap();
    let root = if number == 0 { b"Growing".as_slice() } else { b"Stable" };
    let fingerprint = schema.layout(schema.root(root, None).unwrap(), &[]).unwrap().digest();
    let layout = crate::registry_v1::Layout::new(
        &crate::registry_v1::CEILINGS,
        crate::registry_v1::LayoutParts {
            root: root.into(),
            variant: None,
            layout: fingerprint,
            text: hash(text),
            held: skein_lib::List::with_capacity(0),
        },
    )
    .unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(layout.measure()).unwrap());
    layout.encode(&mut writer).unwrap();
    let text_row = crate::registry_v1::Text::new(
        &crate::registry_v1::CEILINGS,
        crate::registry_v1::TextParts { text: text.as_bytes().into() },
    )
    .unwrap();
    let mut text_writer = skein_lib::Writer::new(usize::try_from(text_row.measure()).unwrap());
    text_row.encode(&mut text_writer).unwrap();
    vec![
        skein_kv::Op::Put {
            key: crate::keys::encode_own(&crate::keys::Own::Layout { family: FAMILY.name.into(), number, id }, 1024)
                .unwrap(),
            value: writer.finish(),
        },
        skein_kv::Op::Put {
            key: crate::keys::encode_own(&crate::keys::Own::Text(hash(text)), 1024).unwrap(),
            value: text_writer.finish(),
        },
    ]
}
pub(super) fn baseline(owner: Owner, values: &[&[u8]]) -> Vec<Item> {
    build(owner, values, false, false)
}
fn build(owner: Owner, values: &[&[u8]], done: bool, stable: bool) -> Vec<Item> {
    let family = KeptFamily { types: &[], ..family(owner) };
    let mut disk = Disk::new(&[]);
    let mut store = fresh(&mut disk, set(owner, family, &[]));
    let mut ops = Vec::new();
    for (index, bytes) in values.iter().enumerate() {
        ops.push(skein_kv::Op::Put { key: key(owner, 0, u8::try_from(index).unwrap()), value: Box::from(*bytes) });
    }
    for (text, id) in [(SETTLING_KEEPING, 1), (SETTLING_CONVERTING, 2), (SETTLING_DROPPING, 3), (SETTLED, 4)] {
        ops.extend(layout(text, 0, id));
    }
    ops.extend(layout(SETTLED, 1, 2));
    if stable {
        ops.extend(layout(SETTLING_KEEPING, 1, 1));
        ops.push(skein_kv::Op::Put { key: key(owner, 1, 0), value: Box::from([0, 1, 1]) });
    }
    for chunk in ops.chunks(8) {
        raw(&mut disk, &mut store, chunk.to_vec());
    }
    if done {
        let done = crate::jobs::kept::operation(&store, owner, &JOB, store.stores.store.last() + 1).unwrap();
        raw(&mut disk, &mut store, vec![done]);
    }
    disk.items()
}
#[test]
fn settling_reheads_keeps_converts_and_drops_only_held_by_no_value_then_retires_by_proof() {
    for owner in [Owner::Application, Owner::Jig] {
        let mut disk = Disk::new(&build(
            owner,
            &[&[0, 1, 0, 42, 0, 7], &[0, 2, 0, 42, 0], &[0, 3, 0, 42, 0, 7], &[0, 4, 0, 42, 0, 7]],
            false,
            true,
        ));
        let before = snapshot(&mut disk);
        let set = set(owner, family(owner), &[JOB]);
        let mut store = store(&disk, set);
        // One put per commit, while the raw page holds several values.
        store.limits.commits.writes = 2;
        let events = disk.open(&mut store);
        let [Event::Ran { of, job, rows: 5, counts, commits, .. }, Event::Opened { pending: true, .. }, ..] =
            events.as_ref()
        else {
            panic!("{events:?}")
        };
        assert_eq!(*of, owner);
        assert_eq!(job.as_ref(), JOB.name);
        let mut expected = vec![
            crate::JobCount { name: "put".into(), count: 4 },
            crate::JobCount { name: "retired".into(), count: 4 },
        ];
        if owner == Owner::Jig {
            for name in ["turns-6", "turns-7", "turns-other", "turns-unread"] {
                expected.push(crate::JobCount { name: name.into(), count: 0 });
            }
        }
        assert_eq!(counts.as_ref(), expected);
        assert!(*commits >= 5, "puts and retirement split within the commit edge");
        assert_eq!(snapshot(&mut disk), before);
        for i in 0..4 {
            assert_eq!(fixtures::get_kv(&mut store, &key(owner, 0, i)).unwrap().as_ref(), &[0, 4, 0, 42, 0, 7]);
        }
        for id in 1..4 {
            assert!(row(&mut store, 0, id).is_none());
        }
        assert!(row(&mut store, 0, 4).is_some());
        assert!(row(&mut store, 1, 2).is_some());
        assert!(row(&mut store, 1, 1).is_none());
        assert_eq!(fixtures::get_kv(&mut store, &key(owner, 1, 0)).unwrap().as_ref(), &[0, 2, 1]);
        for text in [SETTLING_KEEPING, SETTLING_CONVERTING, SETTLING_DROPPING] {
            assert!(super::text(&mut store, hash(text)).is_none());
        }
        assert!(text(&mut store, hash(SETTLED)).is_some());
        assert!(jobs::done(&mut store, owner, JOB.name).is_some());
        switch(&mut disk, &mut store);
        let mut again = super::store(&disk, set);
        let events = disk.open(&mut again);
        assert!(
            { if let [Event::Opened { pending: false, .. }, ..] = events.as_ref() { true } else { false } },
            "{events:?}"
        );
        // Recovery may create an empty segment, but no commit follows.
        assert_eq!(again.stores.store.last(), store.stores.store.last());
        assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
        let space = again
            .stores
            .store
            .census()
            .keyspaces()
            .iter()
            .find(|space| {
                space.prefix() == family(owner).types[0].keys[0].from
                    || (owner == Owner::Application && space.prefix() == b"\x02a")
            })
            .unwrap();
        assert_eq!(space.heads().len(), 1);
        assert_eq!((space.heads()[0].head, space.heads()[0].rows), (Some(4), 4));
        let stable = again
            .stores
            .store
            .census()
            .keyspaces()
            .iter()
            .find(|space| {
                space.prefix() == family(owner).types[1].keys[0].from
                    || (owner == Owner::Application && space.prefix() == b"\x02c")
            })
            .unwrap();
        assert_eq!(stable.heads().len(), 1);
        assert_eq!((stable.heads()[0].head, stable.heads()[0].rows), (Some(2), 1));
    }
}
#[test]
fn settling_refuses_dropped_values_and_unknown_heads_and_retries_from_the_unchanged_active() {
    for owner in [Owner::Application, Owner::Jig] {
        for (value, expected) in [
            (&[0, 3, 0, 42, 1, 7][..], "holds the variant `gone` of `Choice`, which this build no longer has"),
            (&[0, 99, 0, 42, 0, 7][..], "is at layout `99`, which the registry does not hold"),
            (&[0][..], "is at layout `0`, which the registry does not hold"),
            (&[0, 2][..], "does not convert from layout `2`: `Short`"),
        ] {
            let mut disk = Disk::new(&baseline(owner, &[value]));
            let before = snapshot(&mut disk);
            for _ in 0_u32..2 {
                let mut store = store(&disk, set(owner, family(owner), &[JOB]));
                let events = disk.open(&mut store);
                let [Event::OpenRefused { why: OpenRefusal::Job { owner: of, job, key: Some(found), why }, .. }] =
                    events.as_ref()
                else {
                    panic!("{events:?}")
                };
                assert_eq!(*of, owner);
                assert_eq!(job.as_ref(), JOB.name);
                let whole = key(owner, 0, 0);
                let expected_key =
                    if owner == Owner::Application { crate::keys::of_application(&whole).unwrap() } else { &whole };
                assert_eq!(found.as_ref(), expected_key);
                assert_eq!(why.as_ref(), expected);
                assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
                assert_eq!(snapshot(&mut disk), before);
            }
        }
    }
}
#[test]
fn a_done_settling_job_cannot_admit_a_new_drop() {
    let owner = Owner::Application;
    let mut disk = Disk::new(&build(owner, &[&[0, 3, 0, 42, 0, 7]], true, false));
    let before = snapshot(&mut disk);
    let mut store = store(&disk, set(owner, family(owner), &[JOB]));
    let events = disk.open(&mut store);
    assert!(
        {
            if let [Event::OpenRefused { why: OpenRefusal::Layout { id: 3, why: Unplanned::Underived(why), .. }, .. }] =
                events.as_ref()
            {
                why.why == skein_codec::layout::Why::VariantDropped
            } else {
                false
            }
        },
        "{events:?}"
    );
    assert_eq!(snapshot(&mut disk), before);
    assert!(!disk.tree().iter().any(|(p, ..)| p.as_ref() == b"generations/2"));
}
#[test]
fn an_empty_first_commit_marks_settling_done_without_running_it() {
    for owner in [Owner::Application, Owner::Jig] {
        let set = set(owner, family(owner), &[JOB]);
        let mut disk = Disk::new(&[]);
        let mut store = fresh(&mut disk, set);
        assert_eq!(jobs::done(&mut store, owner, JOB.name), Some(1));
        let mut again = super::store(&disk, set);
        let events = disk.open(&mut again);
        assert!(
            { if let [Event::Opened { pending: false, last: 1, .. }, ..] = events.as_ref() { true } else { false } },
            "{events:?}"
        );
    }
}
#[test]
fn settling_admission_requires_an_unparameterized_single_job_with_registered_types() {
    for owner in [Owner::Application, Owner::Jig] {
        let family = family(owner);
        for (jobs, registered, valid) in [
            (vec![JOB], true, true),
            (vec![Job { keys: family.types[0].keys, ..JOB }], true, false),
            (
                vec![Job {
                    converts: Some(crate::Converts { family: family.name, value_type: 0, from: [0; 32] }),
                    ..JOB
                }],
                true,
                false,
            ),
            (vec![Job { counts: &["put"], ..JOB }], true, false),
            (vec![JOB, Job { name: "another", ..JOB }], true, false),
            (vec![JOB], false, false),
        ] {
            let set = set(
                owner,
                if registered { family } else { KeptFamily { types: &[], ..family } },
                Box::leak(jobs.into_boxed_slice()),
            );
            let mut disk = Disk::new(&[]);
            let mut store = store(&disk, set);
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
                assert!(!disk.tree().iter().any(|(path, ..)| path.as_ref() == b"active"));
            }
        }
    }
}
