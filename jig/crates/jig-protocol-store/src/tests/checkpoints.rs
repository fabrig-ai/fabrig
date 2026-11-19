//! Names and kept checkpoint bytes (protocol/store/checkpoints.md, 2).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust checkpoint fixtures (programming-model.md, 10.2)"
)]
use super::fixtures;
use crate::{
    CheckpointName,
    checkpoints::FILES,
    codec::component::{Checkpoint, CheckpointFile, Manifest},
};
use crate::{CheckpointRefusal, Event, Request};
use skein_lib::{List, Reader, Token, Writer};
mod keeping;

#[test]
fn a_selection_distinguishes_live_and_ended_files_and_bounds_the_prefixes() {
    use crate::{checkpoints::selection::Selection, codec::component::Transcript, keys};
    let mut stores = fixtures::stores();
    let mut ops = Vec::new();
    let live = 7;
    for task in [live, 8] {
        ops.push(skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Transcript { task }, fixtures::KEY_LIMIT).unwrap(),
            value: Transcript { length: task * 10, window: 1 }.encode().unwrap(),
        });
        for log in [task * 2, task * 2 + 1] {
            ops.push(skein_kv::Op::Put {
                key: crate::logs::extent_key(log),
                value: Box::from((log * 20).to_be_bytes()),
            });
        }
    }
    ops.push(skein_kv::Op::Put {
        key: keys::encode(
            &jig_domain_core_vocabulary::Key::Core(jig_domain_core_vocabulary::CoreKey::RunProof(live)),
            fixtures::KEY_LIMIT,
        )
        .unwrap(),
        value: Box::new([]),
    });
    fixtures::plant(&mut stores, &ops);
    let mut store = fixtures::over(stores);
    let mut selection = Selection::new();
    selection
        .held(
            false,
            true,
            &[skein_kv::HeldSegment { first: 1, end: None }, skein_kv::HeldSegment { first: 2, end: Some(81) }],
        )
        .unwrap();
    selection
        .held(
            true,
            true,
            &[skein_kv::HeldSegment { first: 1, end: None }, skein_kv::HeldSegment { first: 3, end: Some(90) }],
        )
        .unwrap();
    let mut steps = 0_u32;
    while !selection.page(&mut store).unwrap() {
        steps += 1;
        assert!(steps < 20);
    }
    assert!(steps >= 6, "selection yields between pages");
    let found: Vec<_> =
        selection.files.iter().map(|file| (file.row.part, file.row.number, file.end, file.row.way)).collect();
    assert_eq!(
        found,
        vec![
            (1, 0, None, 1),
            (2, 1, None, 1),
            (2, 2, Some(81), 2),
            (5, 0, None, 2),
            (6, 1, None, 2),
            (6, 3, Some(90), 2),
            (4, 7, Some(70), 2),
            (4, 8, Some(80), 1),
            (3, 14, Some(280), 2),
            (3, 15, Some(300), 2),
            (3, 16, Some(320), 2),
            (3, 17, Some(340), 1)
        ]
    );
}

#[test]
fn a_selection_admits_files_at_the_bound_and_refuses_the_next_one() {
    let mut selection = crate::checkpoints::selection::Selection::new();
    for number in 0..FILES {
        selection.add(2, u64::from(number), None, true).expect("at FILES");
    }
    assert_eq!(selection.files.len(), FILES);
    assert_eq!(selection.add(2, u64::from(FILES), None, true), Err(CheckpointRefusal::Files));
    assert_eq!(selection.files.len(), FILES);
}

#[test]
fn checkpoint_refusals_are_terminal_and_have_the_design_words() {
    let owner = Token::new(45);
    let mut memory = fixtures::opened();
    let events = fixtures::run(&mut memory, Request::Checkpoint { owner, draw: [0; 16] });
    let [Event::CheckpointRefused { owner: found, why: CheckpointRefusal::Unavailable }] = &*events else {
        panic!("memory refusal: {events:?}");
    };
    assert_eq!(*found, owner);
    let (mut disk, _) = fixtures::directory();
    disk.phase = crate::Phase::Ready { next: 1, format_due: true };
    fixtures::run(&mut disk, Request::Checkpoint { owner, draw: [0; 16] });
    let events = fixtures::run(&mut disk, Request::Checkpoint { owner, draw: [1; 16] });
    let [Event::CheckpointRefused { why: CheckpointRefusal::InProgress, .. }] = &*events else {
        panic!("concurrent refusal: {events:?}");
    };
    for (why, words) in [
        (CheckpointRefusal::InProgress, "checkpoint in progress"),
        (CheckpointRefusal::Unavailable, "the store is not serving"),
        (CheckpointRefusal::Files, "more than 65536 files to take"),
        (CheckpointRefusal::Unrecognized { path: Box::from("store/odd") }, "an unrecognized file, store/odd"),
        (CheckpointRefusal::Vanished { path: Box::from("store/log") }, "a selected file vanished, store/log"),
        (
            CheckpointRefusal::Failed { path: Box::from("store/log"), error: skein_io::kernel::Error::Permission },
            "a file failed, store/log: Permission",
        ),
        (CheckpointRefusal::InUse, "in use: its lock is held"),
        (CheckpointRefusal::Missing, "no such checkpoint"),
    ] {
        assert_eq!(&*why.words(), words);
    }
}

#[test]
fn a_serving_checkpoint_writes_its_record_manifest_and_active_before_rename() {
    let mut disk = fixtures::disk::Disk::new(&[]);
    let mut store = disk.store();
    disk.open(&mut store);
    disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let events = disk.request(&mut store, Request::Checkpoint { owner: Token::new(55), draw: [0; 16] });
    let [Event::Checkpointed { checkpoint, .. }] = &*events else {
        panic!("checkpoint answer: {events:?}");
    };
    assert_eq!(checkpoint.name.store, 1);
    assert!(checkpoint.copied > 0);
    let base = [b"checkpoints/".as_slice(), &checkpoint.name.bytes()].concat();
    let record = Checkpoint::decode(&disk.bytes(&[base.as_slice(), b"/checkpoint"].concat())).unwrap();
    let manifest = Manifest::decode(&disk.bytes(&[base.as_slice(), b"/generations/1/manifest"].concat())).unwrap();
    assert_eq!(record.store, 1);
    assert_eq!(record.deployment, fixtures::DEPLOYMENT);
    assert_eq!(manifest.generation, 1);
    assert_eq!(manifest.from, None);
    assert_eq!(manifest.before, record.levels);
    assert_eq!(manifest.after, record.levels);
    assert_eq!(manifest.store, record.store);
    assert_eq!(manifest.secrets, Some(record.secrets));
    assert_eq!(manifest.written, record.made);
    assert!(manifest.counts.is_empty());
    assert_eq!(&*disk.bytes(&[base.as_slice(), b"/active"].concat()), b"1\n");
    assert!(!disk.tree().iter().any(|(path, _, _)| path.windows(7).any(|part| part == b".skein-")));
}

#[test]
fn checkpoint_names_and_draw_halves_have_one_form() {
    let (name, temporary) = CheckpointName::from_draw(1234, [0, 0, 0, 0, 0, 0, 0, 0xab, 1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(name, CheckpointName { store: 1234, suffix: 0xab });
    assert_eq!(temporary, 0x0102_0304_0506_0708);
    assert_eq!(&name.bytes(), b"00000000000000001234-00000000000000ab");
    assert_eq!(&name.temporary(temporary), b"00000000000000001234-00000000000000ab.skein-0102030405060708");
    for name in [name, CheckpointName { store: 0, suffix: 0 }, CheckpointName { store: u64::MAX, suffix: u64::MAX }] {
        assert_eq!(CheckpointName::parse(&name.bytes()), Some(name));
        assert_eq!(CheckpointName::parse_temporary(&name.temporary(temporary)), Some((name, temporary)));
        assert_eq!(CheckpointName::parse(&name.temporary(temporary)), None);
        assert_eq!(CheckpointName::parse_temporary(&name.bytes()), None);
    }
    for bytes in [
        b"0000000000000001234-00000000000000ab".as_slice(),
        b"000000000000000001234-00000000000000ab",
        b"00000000000000001234-00000000000000aB",
        b"00000000000000001234-00000000000000abx",
        b"00000000000000001234_00000000000000ab",
        b"0000000000000000123x-00000000000000ab",
        b"18446744073709551616-00000000000000ab",
        b"00000000000000001234-00000000000000ag",
    ] {
        assert_eq!(CheckpointName::parse(bytes), None, "{bytes:?}");
    }
    for bytes in [
        b"00000000000000001234-00000000000000ab.skein-010203040506070".as_slice(),
        b"00000000000000001234-00000000000000ab.skein-01020304050607080",
        b"00000000000000001234-00000000000000ab.skein-010203040506070F",
        b"00000000000000001234-00000000000000ab.skein-010203040506070g",
        b"00000000000000001234-00000000000000aB.skein-0102030405060708",
        b"00000000000000001234-00000000000000ab.Skein-0102030405060708",
    ] {
        assert_eq!(CheckpointName::parse_temporary(bytes), None, "{bytes:?}");
    }
}

#[test]
fn checkpoint_and_manifest_fixed_rule_values_round_trip_at_both_bounds() {
    let values = skein_codegen::values(super::COMPONENT_SCHEMA);
    let goldens = values.goldens.clone();
    let value = |record: &str, size: &str| {
        goldens
            .iter()
            .find(|value| value.name == format!("record_{record}_{size}.bin"))
            .expect("fixed-rule value")
            .bytes
            .as_slice()
    };
    for bytes in [value("checkpoint", "smallest"), value("checkpoint", "full")] {
        let record = Checkpoint::decode(bytes).expect("checkpoint value");
        assert_eq!(record.clone().encode().expect("bounded record").as_ref(), bytes);
        assert_eq!(Checkpoint::decode(&record.encode().expect("bounded record")), Checkpoint::decode(bytes));
    }
    let mut full = Checkpoint::decode(value("checkpoint", "full")).expect("full checkpoint");
    full.build = Box::from([b'a'; 128]);
    assert_eq!(crate::component_v3::CEILINGS.checkpoint_files, FILES);
    let file = full.files.get(0).expect("full value has a file").clone();
    let mut files = List::with_capacity(FILES);
    for _ in 0..FILES {
        files.push(file.clone()).expect("file list ceiling");
    }
    full.files = files;
    let bytes = full.clone().encode().expect("FILES entries admitted");
    assert_eq!(Checkpoint::decode(&bytes), Ok(full));
    for bytes in [value("checkpoint_file", "smallest"), value("checkpoint_file", "full")] {
        let record =
            CheckpointFile::decode(&crate::component_v3::CEILINGS, &mut Reader::new(bytes)).expect("file value");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("file size"));
        record.encode(&mut writer).expect("measured file");
        assert_eq!(writer.finish().as_ref(), bytes);
    }
    for bytes in [value("manifest", "smallest"), value("manifest", "full")] {
        let record = Manifest::decode(bytes).expect("manifest value");
        assert_eq!(record.encode().as_ref(), bytes);
    }
}
