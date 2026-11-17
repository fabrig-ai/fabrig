//! Publication's retention and named reader locks (checkpoints.md, 4.7 and 6).
use super::*;
use crate::{OpenRefusal, Purpose, Store, Unready};
use fixtures::disk::Disk;
use skein_fake_machine::Item;

fn at(disk: &mut Disk, store: &mut Store, request: Request, written: u64) -> Box<[Event]> {
    let mut env = fixtures::env();
    env.wall = skein_lib::Wall::from_nanos(written);
    let mut above = skein_lib::Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(store, &env, request, &mut above, &mut below);
    for _ in 0_u32..1000 {
        crate::fire(store, &env, &mut above, &mut below);
        disk.drive(&mut below, |io, below| crate::up(store, &env, io, &mut above, below));
        if !store.work_pending() {
            break;
        }
    }
    fixtures::events(&mut above)
}
fn take(disk: &mut Disk, store: &mut Store, suffix: u8, written: u64) -> crate::Checkpointed {
    let events = at(disk, store, Request::Checkpoint { owner: Token::new(80), draw: [suffix; 16] }, written);
    let [Event::Checkpointed { checkpoint, .. }] = &*events else { panic!("checkpoint: {events:?}") };
    *checkpoint
}
fn path(name: CheckpointName) -> Vec<u8> {
    [b"checkpoints/".as_slice(), &name.bytes()].concat()
}
fn names(disk: &mut Disk) -> Vec<CheckpointName> {
    disk.tree()
        .iter()
        .filter_map(|(path, _, _)| path.strip_prefix(b"checkpoints/").and_then(CheckpointName::parse))
        .collect()
}
fn older(written: [u64; 3]) -> (Vec<Item>, [CheckpointName; 3]) {
    // The commit in a name is not the retention clock.
    let names = [
        CheckpointName { store: 90, suffix: 0 },
        CheckpointName { store: 80, suffix: 0 },
        CheckpointName { store: 70, suffix: 0 },
    ];
    let mut items = vec![Item::directory(b"checkpoints")];
    for (name, written) in names.into_iter().zip(written) {
        let manifest = Manifest {
            generation: 1,
            from: None,
            before: crate::format::NONE,
            after: crate::format::NONE,
            counts: List::with_capacity(0),
            store: name.store,
            secrets: Some(0),
            written,
        }
        .encode();
        let base = path(name);
        for suffix in [
            b"".as_slice(),
            b"/generations",
            b"/generations/1",
            b"/generations/1/store",
            b"/generations/1/store/logs",
            b"/generations/1/store/logs/shard",
        ] {
            items.push(Item::directory(&[base.as_slice(), suffix].concat()));
        }
        items.push(Item::file(&[base.as_slice(), b"/generations/1/manifest"].concat(), &manifest));
        items.push(Item::file(
            &[base.as_slice(), b"/generations/1/store/logs/shard/file"].concat(),
            b"retained elsewhere",
        ));
    }
    (items, names)
}
#[test]
fn retention_orders_by_written_then_name_and_always_keeps_the_published_checkpoint() {
    for (written, made, survivor) in [([10, 20, 30], 40, 2), ([30, 30, 30], 40, 0), ([10, 20, 30], 0, 2)] {
        let (items, old) = older(written);
        let mut seed = Disk::new(&[]);
        let mut store = seed.store();
        seed.open(&mut store);
        seed.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)])));
        let mut deployment = seed.items();
        deployment.extend(items);
        let mut disk = Disk::new(&deployment);
        let mut store = disk.store();
        disk.open(&mut store);
        let checkpoint = take(&mut disk, &mut store, 1, made);
        assert_eq!(checkpoint.removed, 2);
        assert_eq!(checkpoint.skipped, 0);
        let mut expected = vec![old[survivor], checkpoint.name];
        expected.sort();
        assert_eq!(names(&mut disk), expected);
    }
}
#[test]
fn retention_skips_unreadable_manifests_without_removing_their_trees() {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store();
    disk.open(&mut store);
    disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let old = take(&mut disk, &mut store, 1, 10).name;
    let manifest = [path(old), b"/generations/1/manifest".to_vec()].concat();
    let mut below = fixtures::below();
    below.checkpoints.push(skein_io::file::Request::Store {
        owner: Token::new(81),
        root: disk.root().token,
        path: manifest.into_boxed_slice(),
        bytes: Box::from(b"unreadable".as_slice()),
        expected: skein_io::file::Expect::Any,
        no_follow: true,
    });
    disk.drive(&mut below, |_, _| {});
    store.limits.checkpoints.keep = 1;
    let new = take(&mut disk, &mut store, 2, 20);
    assert_eq!((new.removed, new.skipped), (0, 1));
    assert!(names(&mut disk).contains(&old));
}
#[test]
fn named_holds_block_other_readers_removal_and_retention_until_close() {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store();
    disk.open(&mut store);
    disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let old = take(&mut disk, &mut store, 1, 10).name;
    let root = disk.root();
    let owner = Token::new(91);
    let events = disk.request(&mut store, Request::HoldCheckpoint { owner, root, name: old });
    let [Event::CheckpointHeld { owner: found }] = &*events else { panic!("held: {events:?}") };
    assert_eq!(*found, owner);
    let mut reader = disk.store();
    for request in [
        Request::HoldCheckpoint { owner: Token::new(92), root, name: old },
        Request::RemoveCheckpoint { owner: Token::new(92), root, name: old },
    ] {
        let events = disk.request(&mut reader, request);
        let [Event::CheckpointRefused { why: CheckpointRefusal::InUse, .. }] = &*events else {
            panic!("in use: {events:?}")
        };
    }
    store.limits.checkpoints.keep = 1;
    let second = take(&mut disk, &mut store, 2, 20);
    assert_eq!((second.removed, second.skipped), (0, 1));
    let events = disk.request(&mut store, Request::Close { owner });
    let [Event::Closed { owner: found, removed: false }] = &*events else { panic!("reader close: {events:?}") };
    assert_eq!(*found, owner);
    let third = take(&mut disk, &mut store, 3, 30);
    assert_eq!((third.removed, third.skipped), (2, 0));
    assert_eq!(names(&mut disk), vec![third.name]);
    // Named removal needs no deployment open, even while the serving store owns its lock.
    let events = disk.request(&mut reader, Request::RemoveCheckpoint { owner, root, name: third.name });
    let [Event::CheckpointRemoved { .. }] = &*events else { panic!("removed: {events:?}") };
    assert_eq!(names(&mut disk), Vec::<CheckpointName>::new());
    for request in
        [Request::HoldCheckpoint { owner, root, name: old }, Request::RemoveCheckpoint { owner, root, name: old }]
    {
        let events = disk.request(&mut reader, request);
        let [Event::CheckpointRefused { why: CheckpointRefusal::Missing, .. }] = &*events else {
            panic!("missing: {events:?}")
        };
    }
    let events = disk.request(&mut store, fixtures::commit(2, Box::new([fixtures::header(2)])));
    let [Event::Committed { number: 2, .. }] = &*events else { panic!("still serving: {events:?}") };
}
#[test]
fn checkpoint_open_preserves_transcript_tails_and_strays_instead_of_running_startup_work() {
    let mut seed = Disk::new(&[]);
    let mut store = seed.store();
    seed.open(&mut store);
    seed.request(
        &mut store,
        fixtures::commit(1, Box::new([fixtures::header(1), crate::Write::Save(fixtures::run_proof(7))])),
    );
    let mut items = seed.items();
    items.extend([
        Item::directory(b"generations/9"),
        Item::file(b"generations/9/stray", b"untouched"),
        Item::directory(b"transcripts/0000000000000000"),
        Item::file(b"transcripts/0000000000000000/0000000000000007", b"uncommitted tail"),
    ]);
    let mut disk = Disk::new(&items);
    let before = disk.contents();
    let root = disk.root();
    let mut store = disk.store();
    let events = disk.request(
        &mut store,
        Request::Open {
            owner: Token::new(94),
            root,
            installation: fixtures::INSTALLATION,
            purpose: Purpose::Checkpoint,
        },
    );
    let Some(Event::Opened { last: 1, pending: false, .. }) = events.first() else {
        panic!("checkpoint open: {events:?}")
    };
    let after = disk.contents();
    for file in before {
        assert!(after.contains(&file), "checkpoint open changed {:?}", file.0);
    }
}

#[test]
fn checkpoint_open_refuses_a_first_start_and_a_pending_job_before_writing() {
    const BASE: crate::PatchSet = crate::PatchSet {
        owner: crate::Owner::Application,
        name: "fixture",
        level: Some(0),
        families: &[crate::STORE_FAMILIES[0]],
        patches: &[],
        fresh: None,
        jobs: &[],
    };
    let mut disk = Disk::new(&[]);
    let mut empty = disk.store();
    let root = disk.root();
    let open = || Request::Open {
        owner: Token::new(93),
        root,
        installation: fixtures::INSTALLATION,
        purpose: Purpose::Checkpoint,
    };
    let before = disk.contents();
    let events = disk.request(&mut empty, open());
    let [Event::OpenRefused { why: OpenRefusal::Unready { why: Unready::Empty }, .. }] = &*events else {
        panic!("empty: {events:?}")
    };
    assert_eq!(disk.contents(), before);
    disk.crash(1);
    let mut store = disk.store();
    store.patch_sets(crate::PatchSets { sets: &[BASE] });
    disk.open(&mut store);
    disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)])));
    disk.crash(2);
    let before = disk.contents();
    let root = disk.root();
    let mut stopped = disk.store();
    stopped.patch_sets(crate::PatchSets {
        sets: &[crate::PatchSet {
            jobs: &[crate::Job {
                name: "pending",
                keys: &[crate::KeyRange { from: &[0, 1], to: &[0, 2] }],
                converts: None,
                settles: false,
                counts: &[],
            }],
            ..BASE
        }],
    });
    let events = disk.request(
        &mut stopped,
        Request::Open {
            owner: Token::new(93),
            root,
            installation: fixtures::INSTALLATION,
            purpose: Purpose::Checkpoint,
        },
    );
    let [Event::OpenRefused { why: OpenRefusal::Unready { why: Unready::Upgrade }, .. }] = &*events else {
        panic!("upgrade: {events:?}")
    };
    // Recovery may add its empty segment; every byte that stood is unchanged.
    let after = disk.contents();
    for file in before {
        assert!(after.contains(&file), "existing file changed: {:?}", file.0);
    }
    for (path, kind, _, bytes) in after {
        if kind == skein_io::kernel::Kind::File && path.windows(5).any(|part| part == b"/log-") {
            assert!(bytes.is_empty() || path.ends_with(b"00000000000000000001"));
        }
    }
}
