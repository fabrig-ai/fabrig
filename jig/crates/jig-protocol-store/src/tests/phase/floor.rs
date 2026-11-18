//! Floor writes and explicit owner levels (protocol/store/records.md, 3; opening.md, 3).
use super::*;
use crate::tests::owners::SETS;
use crate::{Owner, OwnerSet, OwnerSets, component_v3};
use skein_lib::{Reader, Writer};

fn store(disk: &Disk, sets: OwnerSets) -> Store {
    let mut store = disk.store_with_chunk(CHUNK);
    store.application.version = 0;
    store.owner_sets(sets);
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
        let mut first = store(&disk, upgrade_sets(0));
        if let Some(name) = name {
            first.build_name(name);
        }
        assert_opened(&disk.open(&mut first), 0);
        assert_eq!(floor(&mut first), None);
        commit(&mut disk, &mut first);
        assert_eq!(floor(&mut first), Some((crate::FLOOR, expected.as_bytes().into())));
        let mut disk = Disk::new(&disk.items());
        let mut reopened = store(&disk, upgrade_sets(0));
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
    let mut first = store(&disk, upgrade_sets(0));
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    registry::raw(&mut disk, &mut first, vec![floor_op(None)]);
    let mut disk = Disk::new(&disk.items());
    let before = contents(&mut disk);
    let mut reopened = store(&disk, upgrade_sets(1));
    let directory = disk.root();
    let scratch = disk.scratch_root();
    let events = disk.request(
        &mut reopened,
        Request::Rehearse {
            owner: Token::new(1),
            directory,
            scratch,
            installation: fixtures::INSTALLATION,
            reach: crate::Reach::Whole,
        },
    );
    assert!(
        {
            if let [
                Event::Rehearsing { .. },
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
fn openings_reads_rows_even_when_whole_refuses_ahead_or_below_floor() {
    for ahead in [false, true] {
        let mut disk = Disk::new(&[]);
        let mut first = store(&disk, BASE);
        assert!(first.wires(&[crate::Wire { name: "fixture-wire", version: 7 }]));
        assert_opened(&disk.open(&mut first), 0);
        let events = disk.request(&mut first, fixtures::commit(1, Box::new([fixtures::header(1), Write::Wires])));
        assert!({ if let [Event::Committed { .. }] = events.as_ref() { true } else { false } });
        let key = crate::keys::encode_own(&crate::keys::Own::Format, fixtures::KEY_LIMIT).unwrap();
        let mut format = crate::codec::component::Format::decode(&fixtures::get_kv(&mut first, &key).unwrap()).unwrap();
        if ahead {
            format.levels.jig = Some(3);
        }
        registry::raw(
            &mut disk,
            &mut first,
            vec![floor_op(Some(0)), skein_kv::Op::Put { key, value: format.encode().unwrap() }],
        );
        let items = disk.items();
        for reach in [crate::Reach::Whole, crate::Reach::Openings] {
            let mut disk = Disk::new(&items);
            let before = disk.contents();
            let mut reopened = store(&disk, SETS);
            let directory = disk.root();
            let scratch = disk.scratch_root();
            let events = disk.request(
                &mut reopened,
                Request::Rehearse {
                    owner: Token::new(1),
                    directory,
                    scratch,
                    installation: fixtures::INSTALLATION,
                    reach,
                },
            );
            let [Event::Rehearsing { wires, openings, .. }, terminal] = events.as_ref() else { panic!("{events:?}") };
            assert_eq!(
                wires.as_deref(),
                Some([crate::KeptWire { name: "fixture-wire".into(), version: 7 }].as_slice())
            );
            assert!(openings.is_empty());
            match reach {
                crate::Reach::Whole => {
                    let why = if ahead {
                        OpenRefusal::Ahead { owner: Owner::Jig, found: Some(3), wanted: Some(2) }
                    } else {
                        OpenRefusal::OlderBuild { why: crate::Older::Floor }
                    };
                    let Event::OpenRefused { why: found, .. } = terminal else { panic!("{terminal:?}") };
                    assert_eq!(*found, why);
                }
                crate::Reach::Openings => assert!(
                    { if let Event::Closed { removed: true, .. } = terminal { true } else { false } },
                    "{terminal:?}"
                ),
            }
            assert_eq!(disk.contents(), before);
        }
    }
}

#[test]
fn nonempty_store_without_registry_refuses_instead_of_seeding() {
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, crate::tests::owners::BASE);
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    let mut disk = Disk::new(&disk.items());
    let before = contents(&mut disk);
    let family = crate::KeptFamily {
        current: jig_test_layouts::layouts::KEPT,
        name: "fixture-layouts",
        map: crate::KeptMap::Store,
        frames: None,
        keys: &[crate::Keys { from: b"a", to: b"b", root: "Growing" }],
        retired: &[],
        types: &[crate::ValueType {
            number: 0,
            root: "Growing",
            variant: None,
            keys: &[crate::KeyRange { from: b"a", to: b"b" }],
        }],
    };
    let sets = OwnerSets {
        sets: Box::leak(Box::new([OwnerSet { families: Box::leak(Box::new([family])), ..BASE.sets[0] }, BASE.sets[1]])),
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
                if let Stage::Jobs | Stage::Sealing | Stage::Checking | Stage::Manifest | Stage::Switching =
                    stage(store)
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
        let mut retained = Vec::new();
        while let Some(event) = above.pop() {
            if let Event::Job { owner, page, .. } = event {
                assert!(page.end && page.rows.is_empty());
                crate::down(
                    store,
                    &env,
                    Request::Job {
                        owner,
                        answer: Ok(crate::JobAnswer { writes: Box::new([]), through: None, counts: Box::new([]) }),
                    },
                    &mut above,
                    &mut below,
                );
            } else {
                retained.push(event);
            }
        }
        for event in retained {
            above.push(event);
        }
        if !crate::upgrade::phase::ready(store)
            && below.store.is_empty()
            && below.secrets.is_empty()
            && below.transcripts.is_empty()
        {
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
fn job_upgrade_leaves_the_first_commits_floor_unchanged() {
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, upgrade_sets(0));
    first.build_name("first-build");
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    let initial = floor(&mut first);
    let mut disk = Disk::new(&disk.items());
    let mut upgraded = store(&disk, upgrade_sets(1));
    upgraded.build_name("test-build");
    let request = Request::Open {
        purpose: crate::Purpose::Serve,
        owner: Token::new(1),
        root: disk.root(),
        installation: fixtures::INSTALLATION,
    };
    traced_request(&mut disk, &mut upgraded, request, initial.clone());
    assert_eq!(floor(&mut upgraded), initial);
}

#[test]
fn a_level_survives_an_empty_patch_array_in_first_commit_reopen_and_ahead() {
    let set = OwnerSet { level: Some(7), ..BASE.sets[0] };
    let sets = OwnerSets { sets: Box::leak(Box::new([set])) };
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
    let older = OwnerSets { sets: Box::leak(Box::new([OwnerSet { level: Some(6), ..set }])) };
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
    let sets = OwnerSets { sets: Box::leak(Box::new([OwnerSet { jobs: &[JOB], ..BASE.sets[0] }, BASE.sets[1]])) };
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
