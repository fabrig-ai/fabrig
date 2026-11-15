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

#[test]
fn an_idle_open_does_not_write_a_missing_floor() {
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, BASE);
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    registry::raw(&mut disk, &mut first, vec![floor_op(None)]);
    let mut disk = Disk::new(&disk.items());
    let mut reopened = store(&disk, BASE);
    assert_opened(&disk.open(&mut reopened), 2);
    assert_eq!(floor(&mut reopened), None);
    assert!(
        !disk
            .requests()
            .iter()
            .any(|(_, seen)| { if let Seen::Write { .. } | Seen::Store { .. } = seen { true } else { false } })
    );
}

fn traced_request(
    disk: &mut Disk,
    store: &mut Store,
    request: Request,
    initial: Option<(u32, Box<[u8]>)>,
    change: bool,
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
    assert_eq!(
        changed_at,
        if change { Some(store.stores.store.last()) } else { None },
        "floor changes only in the last commit"
    );
    events
}

#[test]
fn patch_upgrade_writes_floor_only_in_the_seal_and_keeps_equal_or_higher_rows() {
    for old in [None, Some(0), Some(crate::FLOOR), Some(crate::FLOOR + 1)] {
        let mut disk = Disk::new(&[]);
        let mut first = store(&disk, BASE);
        assert_opened(&disk.open(&mut first), 0);
        commit(&mut disk, &mut first);
        registry::raw(&mut disk, &mut first, vec![floor_op(old)]);
        let mut disk = Disk::new(&disk.items());
        let mut upgraded = store(&disk, SETS);
        upgraded.build_name("test-build");
        let changes = old.is_none_or(|n| n < crate::FLOOR);
        let expected =
            if changes { (crate::FLOOR, b"test-build".as_slice()) } else { (old.unwrap(), b"older-build".as_slice()) };
        let request = Request::Open { owner: Token::new(1), root: disk.root(), installation: fixtures::INSTALLATION };
        traced_request(&mut disk, &mut upgraded, request, old.map(|n| (n, b"older-build".as_slice().into())), changes);
        assert_eq!(floor(&mut upgraded), Some((expected.0, expected.1.into())));
    }
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
fn a_job_only_upgrade_adds_the_floor_in_its_last_commit() {
    const JOB: crate::Job = crate::Job {
        name: "floor-fixture",
        settles: false,
        keys: &[crate::KeyRange { from: b"l", to: b"m" }],
        converts: None,
        counts: &[],
    };
    let mut disk = Disk::new(&[]);
    let mut first = store(&disk, BASE);
    assert_opened(&disk.open(&mut first), 0);
    commit(&mut disk, &mut first);
    registry::raw(&mut disk, &mut first, vec![floor_op(None)]);
    let mut disk = Disk::new(&disk.items());
    let sets = PatchSets { sets: Box::leak(Box::new([PatchSet { jobs: &[JOB], ..BASE.sets[0] }, BASE.sets[1]])) };
    let mut upgraded = store(&disk, sets);
    upgraded.build_name("job-build");
    let events = disk.open(&mut upgraded);
    let [Event::Job { owner, page, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert!(page.end && page.rows.is_empty());
    assert_eq!(floor(&mut upgraded), None);
    let request = Request::Job {
        owner: *owner,
        answer: Ok(crate::JobAnswer { counts: Box::new([]), writes: Box::new([]), through: None }),
    };
    traced_request(&mut disk, &mut upgraded, request, None, true);
    assert_eq!(floor(&mut upgraded), Some((crate::FLOOR, b"job-build".as_slice().into())));
}
