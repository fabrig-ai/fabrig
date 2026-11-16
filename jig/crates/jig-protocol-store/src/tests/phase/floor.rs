//! Floor writes and explicit owner levels (protocol/store/records.md, 3; opening.md, 3).
use super::*;
use crate::{Owner, PatchSet, PatchSets, component_v3};
use skein_lib::{Reader, Writer};

fn store(disk: &Disk, sets: PatchSets) -> Store {
    let mut store = disk.store_with_chunk(CHUNK);
    store.application.version = 0;
    store.patch_sets(sets);
    store
}
fn key() -> Box<[u8]> {
    let key = crate::keys::encode_own(&crate::keys::Own::Floor, fixtures::KEY_LIMIT).unwrap();
    assert_eq!(key.as_ref(), &[0, 13]);
    assert_eq!(crate::keys::decode_own(&key), Some(crate::keys::Own::Floor));
    key
}
fn floor(store: &mut Store) -> Option<(u32, Box<[u8]>)> {
    let bytes = fixtures::get_kv(store, &key())?;
    let row = component_v3::Floor::decode(&component_v3::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    Some((row.number(), row.build().into()))
}
fn commit(disk: &mut Disk, store: &mut Store) {
    assert!({
        if let [Event::Committed { number: 1, .. }] =
            disk.request(store, fixtures::commit(1, Box::new([fixtures::header(1)]))).as_ref()
        {
            true
        } else {
            false
        }
    });
}
fn floor_op(number: Option<u32>) -> skein_kv::Op {
    let Some(number) = number else { return skein_kv::Op::Erase { key: key() } };
    let row = component_v3::Floor::new(
        &component_v3::CEILINGS,
        component_v3::FloorParts { number, build: b"older-build".as_slice().into() },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    skein_kv::Op::Put { key: key(), value: writer.finish() }
}

#[test]
fn fresh_floor_keeps_the_bounded_build_and_reopen_writes_nothing() {
    const SIXTY_THREE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let crossing: &'static str = Box::leak(format!("{SIXTY_THREE}é").into_boxed_str());
    let boundary: &'static str = Box::leak(format!("{}é", &SIXTY_THREE[..62]).into_boxed_str());
    for (name, expected) in
        [(Some("test-build"), "test-build"), (None, ""), (Some(crossing), SIXTY_THREE), (Some(boundary), boundary)]
    {
        let mut disk = Disk::new(&[]);
        let mut first = store(&disk, BASE);
        if let Some(name) = name {
            first.build_name(name);
        }
        assert_opened(&disk.open(&mut first), 0);
        assert_eq!(floor(&mut first), None);
        commit(&mut disk, &mut first);
        assert_eq!(floor(&mut first), Some((crate::FLOOR, expected.as_bytes().into())));
        let mut disk = Disk::new(&disk.items());
        let mut reopened = store(&disk, BASE);
        reopened.build_name("new-build");
        assert_opened(&disk.open(&mut reopened), 1);
        assert_eq!(floor(&mut reopened), Some((crate::FLOOR, expected.as_bytes().into())));
        assert!(
            !disk
                .requests()
                .iter()
                .any(|(_, seen)| { if let Seen::Write { .. } | Seen::Store { .. } = seen { true } else { false } })
        );
    }
}

type Contents = Vec<(Box<[u8]>, Kind, u32, Box<[u8]>)>;
fn contents(disk: &mut Disk) -> Contents {
    disk.contents().into_iter().filter(|(_, kind, _, bytes)| *kind != Kind::File || !bytes.is_empty()).collect()
}

#[test]
fn missing_or_old_floor_refuses_before_plans_jobs_versions_and_registry() {
    for old in [None, Some(0)] {
        let mut disk = Disk::new(&[]);
        let mut first = store(&disk, BASE);
        assert_opened(&disk.open(&mut first), 0);
        commit(&mut disk, &mut first);
        let format_key = crate::keys::encode_own(&crate::keys::Own::Format, fixtures::KEY_LIMIT).unwrap();
        let mut format =
            crate::codec::component::Format::decode(&fixtures::get_kv(&mut first, &format_key).unwrap()).unwrap();
        format.tasks = 0;
        registry::raw(
            &mut disk,
            &mut first,
            vec![
                floor_op(old),
                skein_kv::Op::Put { key: format_key, value: format.encode().unwrap() },
                skein_kv::Op::Put {
                    key: crate::keys::encode_own(
                        &crate::keys::Own::Layout { family: "broken".into(), number: 0, id: 1 },
                        fixtures::KEY_LIMIT,
                    )
                    .unwrap(),
                    value: Box::new([255]),
                },
            ],
        );
        let mut disk = Disk::new(&disk.items());
        let before = contents(&mut disk);
        let mut reopened = store(&disk, SETS);
        let events = disk.open(&mut reopened);
        assert!(
            {
                if let [Event::OpenRefused { why: OpenRefusal::OlderBuild { why: crate::Older::Floor }, .. }] =
                    events.as_ref()
                {
                    true
                } else {
                    false
                }
            },
            "{events:?}"
        );
        assert_eq!(contents(&mut disk), before);
        assert!(reopened.directory.patch.is_none());
        assert!(reopened.registry.batch(&reopened.limits).unwrap().is_empty());
        assert_eq!(reopened.stores.store.last(), 2);
    }
}

#[test]
fn current_or_higher_floor_opens_without_changing_its_row() {
    for number in [crate::FLOOR, crate::FLOOR + 1] {
        let mut disk = Disk::new(&[]);
        let mut first = store(&disk, BASE);
        assert_opened(&disk.open(&mut first), 0);
        commit(&mut disk, &mut first);
        registry::raw(&mut disk, &mut first, vec![floor_op(Some(number))]);
        let mut disk = Disk::new(&disk.items());
        let mut reopened = store(&disk, BASE);
        assert_opened(&disk.open(&mut reopened), 2);
        assert_eq!(floor(&mut reopened), Some((number, b"older-build".as_slice().into())));
        assert!(
            !disk
                .requests()
                .iter()
                .any(|(_, seen)| { if let Seen::Write { .. } | Seen::Store { .. } = seen { true } else { false } })
        );
    }
}

#[test]
fn ahead_and_unleveled_refuse_before_a_missing_floor() {
    for (jig, why) in [
        (Some(3), OpenRefusal::Ahead { owner: Owner::Jig, found: Some(3), wanted: Some(2) }),
        (None, OpenRefusal::OlderBuild { why: crate::Older::Unleveled }),
    ] {
        let mut disk = Disk::new(&[]);
        let mut first = store(&disk, BASE);
        assert_opened(&disk.open(&mut first), 0);
        commit(&mut disk, &mut first);
        let key = crate::keys::encode_own(&crate::keys::Own::Format, fixtures::KEY_LIMIT).unwrap();
        let mut format = crate::codec::component::Format::decode(&fixtures::get_kv(&mut first, &key).unwrap()).unwrap();
        format.levels.jig = jig;
        registry::raw(
            &mut disk,
            &mut first,
            vec![floor_op(None), skein_kv::Op::Put { key, value: format.encode().unwrap() }],
        );
        let mut disk = Disk::new(&disk.items());
        let before = contents(&mut disk);
        let mut reopened = store(&disk, SETS);
        let events = disk.open(&mut reopened);
        let [Event::OpenRefused { why: found, .. }] = events.as_ref() else { panic!("{events:?}") };
        assert_eq!(*found, why);
        assert_eq!(contents(&mut disk), before);
        assert!(reopened.directory.patch.is_none());
    }
}

#[test]
fn floorless_rehearsal_refuses_before_decoding_and_announces_no_fresh_plan() {
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, BASE);
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    registry::raw(&mut disk, &mut first, vec![floor_op(None)]);
    let mut disk = Disk::new(&disk.items());
    let before = contents(&mut disk);
    let app = SETS.sets[0];
    let marked = Box::leak(app.patches[0].replacen("\n\n", "\nmark fixture-fresh\n\n", 1).into_boxed_str());
    let sets = PatchSets {
        sets: Box::leak(Box::new([
            PatchSet { patches: Box::leak(Box::new([marked, app.patches[1]])), fresh: Some("fixture-fresh"), ..app },
            SETS.sets[1],
        ])),
    };
    let mut reopened = store(&disk, sets);
    let directory = disk.root();
    let scratch = disk.scratch_root();
    let events = disk.request(
        &mut reopened,
        Request::Rehearse { owner: Token::new(1), directory, scratch, installation: fixtures::INSTALLATION },
    );
    assert!(
        {
            if let [
                Event::Rehearsing { fresh: false, .. },
                Event::OpenRefused { why: OpenRefusal::OlderBuild { why: crate::Older::Floor }, .. },
            ] = events.as_ref()
            {
                true
            } else {
                false
            }
        },
        "{events:?}"
    );
    assert!(reopened.directory.patch.is_none());
    assert_eq!(contents(&mut disk), before);
}

#[test]
fn nonempty_store_without_registry_refuses_instead_of_seeding() {
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, BASE);
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    let mut disk = Disk::new(&disk.items());
    let before = contents(&mut disk);
    let family = crate::KeptFamily {
        current: jig_test_patches::layouts::KEPT,
        texts: &[jig_test_patches::layouts::KEPT],
        name: "fixture-layouts",
        map: crate::KeptMap::Store,
        frames: None,
        flights: &[],
        keys: &[crate::Keys { from: b"a", to: b"b", root: "Growing" }],
        retired: &[],
        types: &[crate::ValueType {
            number: 0,
            root: "Growing",
            variant: None,
            keys: &[crate::KeyRange { from: b"a", to: b"b" }],
        }],
    };
    let sets = PatchSets {
        sets: Box::leak(Box::new([PatchSet { families: Box::leak(Box::new([family])), ..BASE.sets[0] }, BASE.sets[1]])),
    };
    let mut reopened = store(&disk, sets);
    let events = disk.open(&mut reopened);
    assert!(
        {
            if let [Event::OpenRefused { why: OpenRefusal::OlderBuild { why: crate::Older::Unregistered }, .. }] =
                events.as_ref()
            {
                true
            } else {
                false
            }
        },
        "{events:?}"
    );
    assert_eq!(contents(&mut disk), before);
    assert!(reopened.registry.batch(&reopened.limits).unwrap().is_empty());
}

fn traced_request(
    disk: &mut Disk,
    store: &mut Store,
    request: Request,
    initial: Option<(u32, Box<[u8]>)>,
) -> Box<[Event]> {
    let mut changed_at = None;
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    let env = fixtures::env();
    crate::down(store, &env, request, &mut above, &mut below);
    for _ in 0_u32..100 {
        crate::fire(store, &env, &mut above, &mut below);
        disk.drive(&mut below, |io, below| {
            crate::up(store, &env, io, &mut above, below);
            crate::fire(store, &env, &mut above, below);
            if ({
                if let Stage::Walking
                | Stage::Windows
                | Stage::Sealing
                | Stage::Checking
                | Stage::Manifest
                | Stage::Switching = stage(store)
                {
                    true
                } else {
                    false
                }
            } || { if let Phase::Ready { .. } = store.phase { true } else { false } })
                && floor(store) != initial
                && changed_at.is_none()
            {
                changed_at = Some(store.stores.store.last());
            }
        });
        if !crate::patching::phase::ready(store) {
            break;
        }
    }
    let events = fixtures::events(&mut above);
    assert!(
        events.iter().any(|event| { if let Event::Opened { pending: true, .. } = event { true } else { false } }),
        "{events:?}"
    );
    assert_eq!(changed_at, None, "the upgrade leaves the first commit's floor unchanged");
    events
}

#[test]
fn patch_upgrade_leaves_the_first_commits_floor_unchanged() {
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, BASE);
    first.build_name("first-build");
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    let initial = floor(&mut first);
    let mut disk = Disk::new(&disk.items());
    let mut upgraded = store(&disk, SETS);
    upgraded.build_name("test-build");
    let request = Request::Open { owner: Token::new(1), root: disk.root(), installation: fixtures::INSTALLATION };
    traced_request(&mut disk, &mut upgraded, request, initial.clone());
    assert_eq!(floor(&mut upgraded), initial);
}

#[test]
fn level_admission_refuses_count_and_family_disagreements_at_patch_zero() {
    let app = SETS.sets[0];
    for set in [
        PatchSet { level: Some(1), ..app },
        PatchSet { level: None, ..app },
        PatchSet { families: &[], patches: &[], level: Some(0), ..app },
    ] {
        let mut fixture = Disk::new(&[]);
        let mut first = store(&fixture, BASE);
        assert_opened(&fixture.open(&mut first), 0);
        commit(&mut fixture, &mut first);
        for items in [Vec::new(), fixture.items()] {
            let mut disk = Disk::new(&items);
            let sets = PatchSets { sets: Box::leak(Box::new([set])) };
            let mut store = store(&disk, sets);
            let last = u64::from(!items.is_empty());
            let events = disk.open(&mut store);
            assert!(
                {
                    if let [Event::OpenRefused { why: OpenRefusal::Set { owner: Owner::Application, patch: 0 }, .. }] =
                        events.as_ref()
                    {
                        true
                    } else {
                        false
                    }
                },
                "{events:?}"
            );
            assert_eq!(store.stores.store.last(), last);
        }
    }
}

#[test]
fn a_level_survives_an_empty_patch_array_in_first_commit_reopen_and_ahead() {
    let set = PatchSet { patches: &[], level: Some(7), ..BASE.sets[0] };
    let sets = PatchSets { sets: Box::leak(Box::new([set])) };
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, sets);
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    let format_key = crate::keys::encode_own(&crate::keys::Own::Format, fixtures::KEY_LIMIT).unwrap();
    let bytes = fixtures::get_kv(&mut first, &format_key).unwrap();
    assert_eq!(crate::codec::component::Format::decode(&bytes).unwrap().levels.application, Some(7));
    let items = disk.items();
    let mut disk = Disk::new(&items);
    let mut reopened = store(&disk, sets);
    assert_opened(&disk.open(&mut reopened), 1);
    assert!(reopened.directory.patch.is_none());
    let mut disk = Disk::new(&items);
    let older = PatchSets { sets: Box::leak(Box::new([PatchSet { level: Some(6), ..set }])) };
    let mut reopened = store(&disk, older);
    assert!({
        if let [
            Event::OpenRefused {
                why: OpenRefusal::Ahead { owner: Owner::Application, found: Some(7), wanted: Some(6) },
                ..
            },
        ] = disk.open(&mut reopened).as_ref()
        {
            true
        } else {
            false
        }
    });
}

#[test]
fn a_job_only_upgrade_leaves_the_first_commits_floor_unchanged() {
    const JOB: crate::Job = crate::Job {
        name: "floor-fixture",
        keys: &[crate::KeyRange { from: b"l", to: b"m" }],
        converts: None,
        counts: &[],
    };
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, BASE);
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    let mut disk = Disk::new(&disk.items());
    let sets = PatchSets { sets: Box::leak(Box::new([PatchSet { jobs: &[JOB], ..BASE.sets[0] }, BASE.sets[1]])) };
    let mut upgraded = store(&disk, sets);
    upgraded.build_name("job-build");
    let events = disk.open(&mut upgraded);
    let [Event::Job { owner, page, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert!(page.end && page.rows.is_empty());
    let initial = floor(&mut upgraded);
    assert_eq!(initial, Some((crate::FLOOR, Box::from(b"".as_slice()))));
    let request = Request::Job {
        owner: *owner,
        answer: Ok(crate::JobAnswer { counts: Box::new([]), writes: Box::new([]), through: None }),
    };
    traced_request(&mut disk, &mut upgraded, request, initial.clone());
    assert_eq!(floor(&mut upgraded), initial);
}
