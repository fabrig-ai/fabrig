//! Restore intent matching and record-driven staging (restoring.md, 4–5 and 8).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust simulated-disk fixtures"
)]
use super::fixtures::{self, disk::Disk};
use crate::{
    CheckpointName, RestoreRefusal,
    codec::component::{Checkpoint, CheckpointFile, CheckpointFileParts, Manifest},
    restoring::{
        roll::{Intent, Progress, Rolling},
        staging::Staging,
    },
};
use skein_fake_machine::Item;
use skein_lib::{List, Reader, Token, Writer};
const OWNER: Token = Token::new(73);
fn name(suffix: u64) -> CheckpointName {
    CheckpointName { store: 1, suffix }
}
fn manifest() -> Manifest {
    Manifest {
        generation: 2,
        from: None,
        before: crate::format::NONE,
        after: crate::format::NONE,
        counts: List::with_capacity(0),
        store: 1,
        secrets: Some(0),
        written: 1,
    }
}
fn intent() -> Intent {
    Intent { generation: 2, active: skein_io::digest::digest(b"1\n"), source: name(1), safety: name(2), made: 3 }
}
fn roll(disk: &mut Disk) -> Progress {
    let mut below = fixtures::below();
    let mut rolling = Rolling::begin(disk.root(), OWNER, &mut below.checkpoints);
    let mut progress = Progress::Pending;
    disk.drive(&mut below, |io, below| {
        let crate::Io::Checkpoints(arrived) = io else { panic!("roll lane") };
        progress = rolling.arrived(arrived, &mut below.checkpoints);
    });
    progress
}
fn rolling_items(trees: u8, active: &[u8], bytes: &[u8]) -> Vec<Item> {
    let mut items = vec![
        Item::file(b"active", active),
        Item::file(b"restore", bytes),
        Item::directory(b"generations"),
        Item::directory(b"generations/2"),
        Item::file(b"generations/2/manifest", &manifest().encode()),
    ];
    for (bit, path, content) in [
        (1, b"transcripts.staged".as_slice(), b"restored".as_slice()),
        (2, b"transcripts", if trees == 3 { b"old".as_slice() } else { b"restored" }),
        (4, b"transcripts.aside", b"old"),
    ] {
        if trees & bit != 0 {
            items.push(Item::directory(path));
            items.push(Item::file(&[path, b"/task"].concat(), content));
        }
    }
    items
}
#[test]
fn the_four_matching_intents_roll_forward_and_every_other_state_is_untouched() {
    for (trees, active) in [(3, b"1\n".as_slice()), (5, b"1\n"), (6, b"1\n"), (6, b"2\n")] {
        let mut disk = Disk::new(&rolling_items(trees, active, &intent().encode()));
        assert_eq!(roll(&mut disk), Progress::Done);
        assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
        assert_eq!(disk.bytes(b"transcripts/task").as_ref(), b"restored");
        assert!(!disk.tree().iter().any(|(path, ..)| path.as_ref() == b"restore"
            || path.starts_with(b"transcripts.aside")
            || path.starts_with(b"transcripts.staged")));
    }
    for (trees, active, bytes) in [
        (7, b"1\n".as_slice(), intent().encode()),
        (3, b"9\n", intent().encode()),
        (3, b"1\n", Box::from(b"undecodable".as_slice())),
    ] {
        let mut disk = Disk::new(&rolling_items(trees, active, &bytes));
        let before = disk.contents();
        assert_eq!(roll(&mut disk), Progress::Unmatched);
        assert_eq!(disk.contents(), before);
    }
    let mut items = rolling_items(6, b"1\n", &intent().encode());
    let wrong = Manifest { generation: 3, ..manifest() }.encode();
    *items.iter_mut().find(|item| item.path == b"generations/2/manifest").unwrap() =
        Item::file(b"generations/2/manifest", &wrong);
    let mut disk = Disk::new(&items);
    let before = disk.contents();
    assert_eq!(roll(&mut disk), Progress::Unmatched);
    assert_eq!(disk.contents(), before);
}
#[test]
fn malformed_intents_refuse_every_open_and_back_and_rehearsal_never_copies() {
    use crate::{Event, Generations, OpenRefusal, Purpose, Request};
    let mut base = Disk::new(&[]);
    let mut store = base.store();
    base.open(&mut store);
    base.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1), crate::Write::Wires])));
    let mut items = base.items();
    items.push(Item::file(b"restore", b"undecodable"));
    items.push(Item::directory(b"scratch"));
    items.push(Item::file(b"scratch/sentinel", b"must not copy"));
    for kind in 0_u8..4 {
        let mut disk = Disk::new(&items);
        let mut store = disk.store();
        let request = match kind {
            0 | 1 => Request::Open {
                owner: OWNER,
                root: disk.root(),
                installation: fixtures::INSTALLATION,
                purpose: if kind == 0 { Purpose::Serve } else { Purpose::Checkpoint },
            },
            2 => Request::Back { owner: OWNER, root: disk.root() },
            3 => {
                let arrived = disk.file(
                    0,
                    skein_io::file::Request::OpenDirectory {
                        owner: OWNER,
                        root: disk.root().fd,
                        name: Box::from(b"scratch".as_slice()),
                        no_follow: true,
                    },
                );
                Request::Rehearse {
                    owner: OWNER,
                    directory: disk.root(),
                    scratch: crate::directory::opened(arrived).unwrap(),
                    reach: crate::Reach::Whole,
                    installation: fixtures::INSTALLATION,
                }
            }
            _ => unreachable!(),
        };
        let before = disk.contents();
        let events = disk.request(&mut store, request);
        let Some(Event::OpenRefused { why: OpenRefusal::Generations { why }, .. }) = events.last() else {
            panic!("open {kind}: {events:?}")
        };
        assert_eq!(*why, if kind == 3 { Generations::Unfinished } else { Generations::Unmatched });
        assert_eq!(disk.contents(), before);
    }
}
#[test]
fn with_no_intent_an_open_sweeps_only_the_restore_strays() {
    let mut base = Disk::new(&[]);
    let mut store = base.store();
    base.open(&mut store);
    base.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1), crate::Write::Wires])));
    let mut items = base.items();
    items.extend([
        Item::file(b"restore.skein-0123456789abcdef", b"temporary"),
        Item::directory(b"transcripts.staged"),
        Item::directory(b"transcripts.staged/shard"),
        Item::file(b"transcripts.staged/shard/task", b"partial"),
        Item::directory(b"transcripts.aside"),
        Item::file(b"transcripts.aside/task", b"old"),
        Item::file(b"restore.skein-not-a-temporary", b"unrelated"),
    ]);
    for purpose in [crate::Purpose::Serve, crate::Purpose::Checkpoint] {
        let mut disk = Disk::new(&items);
        let mut store = disk.store();
        let events = disk.request(
            &mut store,
            crate::Request::Open { owner: OWNER, root: disk.root(), installation: fixtures::INSTALLATION, purpose },
        );
        let [crate::Event::Opened { .. }, ..] = events.as_ref() else { panic!("{events:?}") };
        assert!(!disk.tree().iter().any(|(path, ..)| path.starts_with(b"transcripts.staged")
            || path.starts_with(b"transcripts.aside")
            || path.as_ref() == b"restore.skein-0123456789abcdef"));
        assert_eq!(disk.bytes(b"restore.skein-not-a-temporary").as_ref(), b"unrelated");
    }
}
#[test]
fn a_corrupt_safety_open_has_one_restore_refusal_and_releases_its_lock() {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store();
    disk.open(&mut store);
    disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1), crate::Write::Wires])));
    let events = disk.request(&mut store, crate::Request::Checkpoint { owner: OWNER, draw: [1; 16] });
    let [crate::Event::Checkpointed { checkpoint, .. }] = events.as_ref() else { panic!("source: {events:?}") };
    let source = checkpoint.name;
    let crate::Write::Save(header) = fixtures::header(1) else { unreachable!() };
    let key = crate::keys::encode(&header.key(), fixtures::KEY_LIMIT).unwrap();
    let env = fixtures::env();
    let mut replies = skein_lib::Queue::uncapped();
    let mut below = fixtures::below();
    skein_kv::down(
        &mut store.stores.store,
        &env.with(fixtures::store_limits().store),
        skein_kv::Request::Commit {
            owner: OWNER,
            ops: Box::new([skein_kv::Op::Put { key, value: Box::from(b"corrupt".as_slice()) }]),
        },
        &mut replies,
        &mut below.store,
    );
    disk.drive(&mut below, |io, below| {
        let crate::Io::Store(arrived) = io else { panic!("main map lane") };
        skein_kv::up(
            &mut store.stores.store,
            &env.with(fixtures::store_limits().store),
            arrived.event,
            &mut replies,
            &mut below.store,
        );
    });
    let mut disk = Disk::new(&disk.items());
    let mut store = disk.store();
    let mut above = skein_lib::Queue::uncapped();
    let mut below = fixtures::below();
    let available = disk.file(
        0,
        skein_io::file::Request::OpenDirectory {
            owner: OWNER,
            root: disk.root().fd,
            name: Box::from(b".".as_slice()),
            no_follow: true,
        },
    );
    let available = crate::directory::opened(available).unwrap();
    crate::restoring::begin(&mut store, &env, OWNER, disk.root(), source, [2; 16], &mut above, &mut below);
    for _ in 0_u32..100 {
        crate::fire(&mut store, &env, &mut above, &mut below);
        disk.drive(&mut below, |io, below| crate::up(&mut store, &env, io, &mut above, below));
        if !above.is_empty() {
            break;
        }
    }
    let events = fixtures::events(&mut above);
    let [crate::Event::RestoreRefused { owner: OWNER, why: RestoreRefusal::Open { why: crate::OpenRefusal::Failed } }] =
        events.as_ref()
    else {
        panic!("corrupt safety open: {events:?}")
    };
    assert!(store.restoring.is_none());
    assert!(store.checkpoint_open.is_none());
    let unlocked = disk.file(0, skein_io::file::Request::Lock { owner: OWNER, root: available.token });
    let skein_io::file::Event::Locked { .. } = unlocked.event else { panic!("deployment lock released") };
}
#[test]
fn restore_words_are_pinned_and_versioned_intents_round_trip() {
    use crate::{CheckpointRefusal, OpenRefusal};
    use skein_io::kernel::Error;
    let safety = name(2);
    let safety_text = core::str::from_utf8(&safety.bytes()).unwrap().to_owned();
    for (why, expected) in [
        (RestoreRefusal::Busy, "the deployment is in use by another process".to_owned()),
        (RestoreRefusal::Source { why: CheckpointRefusal::Missing }, "no such checkpoint".to_owned()),
        (RestoreRefusal::Source { why: CheckpointRefusal::InUse }, "in use: its lock is held".to_owned()),
        (RestoreRefusal::Deployment, "the checkpoint is another deployment's".to_owned()),
        (
            RestoreRefusal::Unreadable { path: "checkpoint".into() },
            "the checkpoint's record can't be read, checkpoint".to_owned(),
        ),
        (RestoreRefusal::Open { why: OpenRefusal::Failed }, "the deployment does not open".to_owned()),
        (
            RestoreRefusal::Safety { why: CheckpointRefusal::Files },
            "the safety checkpoint was refused: more than 65536 files to take".to_owned(),
        ),
        (
            RestoreRefusal::Staging { safety, path: "file".into(), error: Some(Error::NoSpace) },
            format!("staging failed, file: NoSpace, and nothing live changed; the safety checkpoint is {safety_text}"),
        ),
        (
            RestoreRefusal::Staging { safety, path: "file".into(), error: None },
            format!(
                "staging failed, file is shorter than its record says, and nothing live changed; the safety checkpoint is {safety_text}"
            ),
        ),
        (
            RestoreRefusal::Interrupted { safety, path: "file".into(), error: Error::NoSpace },
            format!(
                "interrupted after its intent, file: NoSpace; the next start finishes the restore; the safety checkpoint is {safety_text}"
            ),
        ),
    ] {
        assert_eq!(why.words().as_ref(), expected);
    }
    let values = skein_codegen::values(super::COMPONENT_SCHEMA);
    let goldens = values.goldens.clone();
    for name in ["record_restore_intent_smallest.bin", "record_restore_intent_full.bin"] {
        let bytes = goldens.iter().find(|value| value.name == name).expect("restore intent value").bytes.as_slice();
        let row = crate::component_v3::RestoreIntent::decode(&crate::component_v3::CEILINGS, &mut Reader::new(bytes))
            .unwrap();
        let mut out = Writer::new(usize::try_from(row.measure()).unwrap());
        row.encode(&mut out).unwrap();
        assert_eq!(out.finish().as_ref(), bytes);
    }
    let before = intent();
    let after = Intent::decode(&before.encode()).unwrap();
    assert_eq!(
        (after.generation, after.active, after.source, after.safety, after.made),
        (before.generation, before.active, before.source, before.safety, before.made)
    );
}
fn staging_fixture() -> (Vec<Item>, Checkpoint) {
    let mut items = vec![
        Item::directory(b"source"),
        Item::directory(b"source/generations"),
        Item::directory(b"source/generations/1"),
        Item::directory(b"source/generations/1/store"),
        Item::directory(b"source/generations/1/store/logs"),
        Item::directory(b"source/generations/1/store/logs/0000000000000000"),
        Item::directory(b"source/generations/1/secrets"),
        Item::directory(b"source/transcripts"),
        Item::directory(b"source/transcripts/0000000000000000"),
        Item::directory(b"generations"),
        Item::directory(b"generations/2"),
        Item::directory(b"generations/2/store"),
        Item::file(b"generations/2/store/partial", b"stray"),
        Item::directory(b"transcripts"),
        Item::file(b"active", b"1\n"),
        Item::file(b"transcripts/old", b"old"),
    ];
    let mut files = List::with_capacity(12);
    for (part, number, way, bytes) in [
        (1, 0, 1, b"abc".as_slice()),
        (2, 1, 1, b"abc"),
        (3, 1, 1, b"abc"),
        (3, 3, 1, b"abcdef"),
        (3, 5, 2, b"abc"),
        (3, 6, 2, b"abc"),
        (5, 0, 2, b"abc"),
        (6, 1, 2, b"abc"),
        (4, 7, 1, b"abc"),
        (4, 8, 2, b"abc"),
        (4, 9, 3, b"abc"),
    ] {
        let relative = crate::checkpoints::files::file_path(1, part, number);
        if way != 3 {
            items.push(Item::file(&[b"source/".as_slice(), &relative].concat(), bytes));
        }
        files
            .push(
                CheckpointFile::new(
                    &crate::component_v3::CEILINGS,
                    CheckpointFileParts { part, number, length: 3, way },
                )
                .unwrap(),
            )
            .unwrap();
    }
    (
        items,
        Checkpoint {
            store: 1,
            secrets: 0,
            made: 1,
            build: Box::default(),
            deployment: fixtures::DEPLOYMENT,
            levels: crate::format::NONE,
            files,
        },
    )
}
fn stage(disk: &mut Disk, record: Checkpoint) -> (crate::generations::Progress, Staging) {
    let arrived = disk.file(
        3,
        skein_io::file::Request::OpenDirectory {
            owner: OWNER,
            root: disk.root().fd,
            name: Box::from(b"source".as_slice()),
            no_follow: true,
        },
    );
    let source = crate::directory::opened(arrived).unwrap();
    let mut below = fixtures::below();
    let mut staging = Staging::begin(source, disk.root(), record, manifest(), 2, OWNER, &mut below.checkpoints);
    let mut progress = crate::generations::Progress::Pending;
    disk.drive(&mut below, |io, below| {
        let crate::Io::Checkpoints(arrived) = io else { panic!("staging lane") };
        progress = staging.arrived(arrived, 7, &mut below.checkpoints);
    });
    (progress, staging)
}
#[test]
fn staging_links_only_immutable_files_copies_extents_and_keeps_secrets_private() {
    for fallback in [false, true] {
        let (items, record) = staging_fixture();
        let mut disk = Disk::new(&items);
        if fallback {
            disk.refuse_links(skein_io::kernel::Error::Permission);
        }
        let (progress, staging) = stage(&mut disk, record);
        assert_eq!(progress, crate::generations::Progress::Done);
        assert_eq!(
            (staging.linked(), staging.copied(), staging.bytes()),
            if fallback { (0, 10, 30) } else { (4, 6, 18) }
        );
        for log in [3, 5, 6] {
            let path = crate::checkpoints::files::file_path(2, 3, log);
            assert_eq!(disk.bytes(&path).as_ref(), b"abc");
            assert_eq!(disk.links(&path), 1);
        }
        let secret = b"generations/2/secrets/snapshot";
        assert_eq!(disk.links(secret), 1);
        assert!(disk.tree().iter().any(|(path, _, mode)| path.as_ref() == secret && *mode == 0o600));
        assert_eq!(disk.bytes(b"transcripts/old").as_ref(), b"old");
        assert_eq!(disk.bytes(b"transcripts.staged/0000000000000000/0000000000000008").as_ref(), b"abc");
        assert!(
            !disk
                .tree()
                .iter()
                .any(|(path, ..)| path.as_ref() == b"transcripts.staged/0000000000000000/0000000000000009")
        );
        assert_eq!(Manifest::decode(&disk.bytes(b"generations/2/manifest")).unwrap().from, None);
    }
    for short in [false, true] {
        let (mut items, mut record) = staging_fixture();
        if short {
            let first = record.files.get(0).unwrap().clone().into_parts();
            record.files.get_mut(0).unwrap().clone_from(
                &CheckpointFile::new(&crate::component_v3::CEILINGS, CheckpointFileParts { length: 4, ..first })
                    .unwrap(),
            );
        } else {
            items.retain(|item| item.path != b"source/generations/1/store/snapshot");
        }
        let mut disk = Disk::new(&items);
        assert_eq!(stage(&mut disk, record).0, crate::generations::Progress::Failed);
        assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
        assert_eq!(disk.bytes(b"transcripts/old").as_ref(), b"old");
    }
}
