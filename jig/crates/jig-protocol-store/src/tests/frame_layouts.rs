//! Registered transcript reads over real file drivers and a fake disk (protocol/store/transcripts.md 6).
#![expect(clippy::disallowed_methods, reason = "test cases use ordinary Rust collections (programming-model.md 10.2)")]
use super::fixtures::{self, ALICE, disk::Disk};
use crate::{Event, KeptFamily, KeptMap, Owner, OwnerSet, OwnerSets, Request, Store, TranscriptRead, codec, keys};
use jig_domain_core_vocabulary::{self as domain, reads};
use jig_domain_tasks as tasks;
use jig_test_layouts::{self as fixture, frames, layouts};
use skein_codec::schema::Schema;
use skein_fake_machine::Item;
use skein_lib::{Token, Wall, length::len_u64};

pub(super) const SETS: OwnerSets = OwnerSets {
    sets: &[
        OwnerSet {
            jobs: &[],
            owner: Owner::Jig,
            name: fixture::FIXTURE_JIG_OWNER,
            level: Some(2),
            families: &[KeptFamily {
                retired: &[],
                current: layouts::FRAME_BODY,
                types: &[],
                name: "fixture-frame",
                map: KeptMap::Store,
                keys: &[],
                frames: Some(crate::Frames {
                    root: frames::BODY_ROOT,
                    texts: &[layouts::FRAME_BODY_KEPT, layouts::FRAME_BODY],
                    number: 0,
                }),
            }],
        },
        OwnerSet {
            jobs: &[],
            owner: Owner::Smith,
            name: fixture::FIXTURE_SMITH_OWNER,
            level: Some(1),
            families: &[KeptFamily {
                retired: &[],
                current: layouts::FRAME_TURN,
                types: &[],
                name: "fixture-turn",
                map: KeptMap::Store,
                keys: &[],
                frames: Some(crate::Frames {
                    root: frames::TURN_ROOT,
                    texts: &[layouts::FRAME_TURN_KEPT, layouts::FRAME_TURN],
                    number: 0,
                }),
            }],
        },
    ],
};
const OWNER: Token = Token::new(71);
const PATH: &[u8] = b"0000000000000000/0000000000000015";
const SAID: &[u8] = b"hello";
const REPLAY: &[u8] = b"replay";
const POSITION: u64 = 1;
const ROWS: u32 = 2;
const BUDGET: u64 = 65_536;
// Transcript places key (two one-byte tags and u64 task) followed by a u64 offset.

pub(super) fn turn(version: u16, replay: &[u8]) -> Box<[u8]> {
    let mut out = version.to_be_bytes().to_vec();
    out.extend(u32::try_from(SAID.len()).expect("said").to_be_bytes());
    out.extend(SAID);
    if version == 1 {
        out.extend(u32::try_from(replay.len()).expect("replay").to_be_bytes());
        out.extend(replay);
    } else {
        out.extend(9_u32.to_be_bytes());
    }
    out.into_boxed_slice()
}
fn expected(version: u16) -> Box<[u8]> {
    let mut current = turn(2, &[]);
    if version == 1 {
        let at = current.len().checked_sub(4).expect("fixture tokens");
        current[at..].copy_from_slice(&0_u32.to_be_bytes());
    }
    current
}
fn activity() -> domain::Record {
    domain::Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
        task: frames::TASK,
        position: POSITION,
        at: Wall::EPOCH,
        line: tasks::Line::Turn { reply: POSITION, attempt: frames::ATTEMPT, turn: frames::TURN, read: None },
    }))
}
fn owner_record() -> domain::Record {
    let domain::Record::Tasks(tasks::Stored::Ended(mut owner)) =
        fixtures::ended_of(frames::TASK, tasks::Party::Person(ALICE))
    else {
        panic!("ended fixture")
    };
    owner.activity = POSITION;
    domain::Record::Tasks(tasks::Stored::Ended(owner))
}
fn put(record: &domain::Record) -> skein_kv::Op {
    skein_kv::Op::Put {
        key: keys::encode(&record.key(), fixtures::KEY_LIMIT).expect("key"),
        value: codec::encode(record, &fixtures::codec_limits(), &[]).expect("record"),
    }
}
fn scene(bytes: &[u8], sets: OwnerSets) -> (Disk, Store) {
    scene_with_schemas(bytes, sets, &[])
}
fn scene_with_schemas(bytes: &[u8], sets: OwnerSets, prepared: &'static [Schema]) -> (Disk, Store) {
    let disk = Disk::new(&[Item::directory(b"0000000000000000"), Item::file(PATH, bytes)]);
    let mut store = fixtures::over(fixtures::stores());
    store.owner_sets(sets);
    store.prepared_schemas(prepared);
    let opened = fixtures::run(
        &mut store,
        Request::Open {
            purpose: crate::Purpose::Serve,
            owner: Token::new(1),
            root: disk.root(),
            installation: fixtures::INSTALLATION,
        },
    );
    let [Event::Opened { .. }, Event::Census { .. }] = &*opened else { panic!("opened: {opened:?}") };
    fixtures::committed(&mut store, Box::new([fixtures::header(1)]));
    fixtures::plant(
        &mut store.stores,
        &[
            skein_kv::Op::Put {
                key: keys::encode_own(&keys::Own::Transcript { task: frames::TASK }, fixtures::KEY_LIMIT)
                    .expect("places"),
                value: codec::component::Transcript { length: len_u64(bytes.len()), window: 0 }
                    .encode()
                    .expect("places"),
            },
            skein_kv::Op::Put {
                key: keys::encode_own(
                    &keys::Own::Attempt { task: frames::TASK, attempt: frames::ATTEMPT },
                    fixtures::KEY_LIMIT,
                )
                .expect("attempt"),
                value: codec::component::Attempt { first: 0 }.encode().expect("attempt"),
            },
            put(&owner_record()),
            put(&activity()),
        ],
    );
    (disk, store)
}
fn request(kind: u8, budget: u64) -> Request {
    let read = match kind {
        0 | 4 => {
            TranscriptRead::Window { task: frames::TASK, attempt: frames::ATTEMPT, turn: frames::TURN, after: None }
        }
        1 => TranscriptRead::Turn { task: frames::TASK, attempt: frames::ATTEMPT, turn: frames::TURN },
        2 => {
            let mut range = fixtures::read(ALICE, reads::Scope::Activity { task: frames::TASK });
            range.backwards = true;
            range.bytes = budget;
            return Request::Read { owner: OWNER, range };
        }
        3 => TranscriptRead::Page { task: frames::TASK, backwards: false, after: None },
        5 => {
            let mut range = fixtures::read(
                ALICE,
                reads::Scope::Turn { task: frames::TASK, attempt: frames::ATTEMPT, turn: frames::TURN },
            );
            range.bytes = budget;
            return Request::Read { owner: OWNER, range };
        }
        _ => unreachable!("named read kind"),
    };
    Request::Transcript { owner: OWNER, read, rows: ROWS, bytes: budget }
}
fn payload(events: &[Event]) -> &[u8] {
    match events {
        [Event::Turns { turns, .. }] => &turns.first().expect("one turn").transcript,
        [Event::Read { result: reads::Loaded::Page { rows, .. }, .. }] => {
            let row = rows.first().expect("one row");
            let record = row.related.as_ref().unwrap_or(&row.value);
            let domain::Record::Core(domain::CoreRecord::Turn(turn)) = record else { panic!("joined turn {record:?}") };
            &turn.transcript
        }
        _ => panic!("read result {events:?}"),
    }
}
#[test]
fn a_kept_part_uses_its_live_key_and_converts_its_held_turn_on_read() {
    let text = smith_transcript::SEED[0];
    let old = skein_codegen::fixed_values(&skein_codegen::parse(text).expect("kept v7"))
        .into_iter()
        .find(|value| value.name == "record_turn_smallest.bin")
        .expect("turn")
        .bytes;
    let part = domain::PartRecord {
        key: domain::PartKey { task: 21, attempt: 2, number: 3 },
        turn: 4,
        thread: 0,
        call: 5,
        kind: jig_domain_link::PartKind::Answer,
        body: old.clone().into_boxed_slice(),
    };
    let row = domain::Record::Core(domain::CoreRecord::Part(part.clone()));
    let key = keys::encode(&row.key(), fixtures::KEY_LIMIT).unwrap();
    assert_eq!(key.get(..3), Some([1, 0, 10].as_slice()));
    assert_eq!(keys::decode(&key), Some(row.key()));
    let ranges = crate::intervals::restore(&domain::RestoreRange::Core);
    assert!(
        ranges
            .iter()
            .any(|range| key.as_ref() >= range.start.as_ref() && key.as_ref() < range.end.as_ref().unwrap().as_ref())
    );
    let mut disk = Disk::new(&[]);
    let mut store = disk.store();
    let old_family = KeptFamily {
        current: text,
        frames: Some(crate::Frames { root: smith_transcript::ROOT, number: 0, texts: Box::leak(Box::new([text])) }),
        ..crate::owners::SMITH_SET.families[0]
    };
    let old_set = OwnerSet { families: Box::leak(Box::new([old_family])), ..crate::owners::SMITH_SET };
    // Register this value type alone within the fixture's small commit edge.
    let mut core = *crate::patch_set::STORE_FAMILIES.iter().find(|family| family.name == "jig-store-core").unwrap();
    core.keys = &[crate::Keys { from: &[1, 0, 10], to: &[1, 0, 11], root: "Row" }];
    core.types = &[crate::ValueType {
        number: 8,
        root: "Row",
        variant: Some("part"),
        keys: &[crate::KeyRange { from: &[1, 0, 10], to: &[1, 0, 11] }],
    }];
    core.frames = None;
    // Only this variant's layout is needed; verify the small text against the
    // binary's layout instead of raising the fixture's first-commit budget.
    let actual = skein_codec::schema::parse(core.current.as_bytes()).unwrap();
    core.current = "family jig-store-core 6\nrecord PartKey { task: u64 attempt: u64 number: u32 }\nenum PartKind { answer, result, window }\nrecord PartRecord { key: PartKey turn: u32 thread: u32 call: u32 kind: PartKind body: bytes 33554432 holds smith-transcript Turn }\nenum Value { call, deployment, projection, run_proof, turn, terminal, proposal_decision, escalation_decision, part: PartRecord }\nversioned record Row { value: Value }\n";
    let small = skein_codec::schema::parse(core.current.as_bytes()).unwrap();
    let smith = skein_codec::schema::parse(crate::owners::SMITH_SET.families[0].current.as_bytes()).unwrap();
    assert_eq!(
        actual.layout(actual.root(b"Row", Some(b"part")).unwrap(), &[&smith]).unwrap().digest(),
        small.layout(small.root(b"Row", Some(b"part")).unwrap(), &[&smith]).unwrap().digest()
    );
    let jig = OwnerSet { jobs: &[], families: Box::leak(Box::new([core])), ..crate::patch_set::JIG_STORE_SET };
    store.owner_sets(OwnerSets { sets: Box::leak(Box::new([jig, old_set])) });
    let opened = disk.open(&mut store);
    let Some(Event::Opened { .. }) = opened.first() else { panic!("part writer open: {opened:?}") };
    let committed =
        disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1), fixtures::save(row.clone())])));
    let [Event::Committed { number: 1, .. }] = committed.as_ref() else { panic!("part commit: {committed:?}") };
    let mut store = disk.store();
    store.owner_sets(OwnerSets { sets: Box::leak(Box::new([jig, crate::owners::SMITH_SET])) });
    let opened = disk.open(&mut store);
    let Some(Event::Opened { .. }) = opened.first() else { panic!("part reader open: {opened:?}") };
    let before = fixtures::get_kv(&mut store, &key).unwrap();
    let events = disk.request(&mut store, Request::Get { owner: OWNER, key: crate::Get::Jig(row.key()) });
    let [Event::Got { row: Some(crate::Row::Jig(domain::Record::Core(domain::CoreRecord::Part(loaded)))), .. }] =
        events.as_ref()
    else {
        panic!("part read: {events:?}")
    };
    let mut expected = old;
    expected[1] = 8;
    assert_eq!(loaded.body.as_ref(), expected.as_slice());
    assert_eq!((loaded.key, loaded.turn, loaded.thread, loaded.call, loaded.kind), (part.key, 4, 0, 5, part.kind));
    assert_eq!(fixtures::get_kv(&mut store, &key).unwrap(), before, "patching never rewrites the kept part");
}

#[test]
fn the_real_smith_set_converts_kept_version_seven_on_read() {
    const REAL: OwnerSets = OwnerSets { sets: &[crate::owners::SMITH_SET] };
    let text = smith_transcript::SEED[0];
    let old = skein_codegen::fixed_values(&skein_codegen::parse(text).expect("kept v7"))
        .into_iter()
        .find(|value| value.name == "record_turn_smallest.bin")
        .expect("turn")
        .bytes;
    let bytes = frames::frame(6, &old);
    let (mut disk, mut store) = scene(&bytes, REAL);
    let events = disk.request(&mut store, request(1, BUDGET));
    let patched = payload(&events);
    let mut expected = old;
    expected[1] = 8;
    assert_eq!(patched, expected);
    assert_eq!(&*disk.bytes(PATH), &*bytes, "patching a read never writes the frame");
}

#[test]
fn every_read_kind_converts_each_kept_frame_without_writing() {
    const KINDS: u8 = 6; // resume, point, activity, page, tail's window path, party point
    for body in [5, 6] {
        for version in [1, 2] {
            let bytes = frames::frame(body, &turn(version, REPLAY));
            let (mut disk, mut store) = scene(&bytes, SETS);
            for kind in 0..KINDS {
                let events = disk.request(&mut store, request(kind, BUDGET));
                assert_eq!(payload(&events), &*expected(version), "body {body}, turn {version}, kind {kind}");
                assert_eq!(&*disk.bytes(PATH), &*bytes);
            }
        }
    }
}
fn log_scene(bytes: &[u8]) -> (Disk, Store) {
    let (disk, mut store) = scene(bytes, SETS);
    let frame = store.stores.store.frame(crate::transcripts::frame::body(bytes).unwrap()).unwrap();
    super::history_logs::raw_commit(
        &mut store,
        Box::new([
            skein_kv::Op::Erase {
                key: keys::encode_own(&keys::Own::Transcript { task: frames::TASK }, fixtures::KEY_LIMIT).unwrap(),
            },
            skein_kv::Op::Erase {
                key: keys::encode_own(
                    &keys::Own::Attempt { task: frames::TASK, attempt: frames::ATTEMPT },
                    fixtures::KEY_LIMIT,
                )
                .unwrap(),
            },
            skein_kv::Op::Append { log: crate::logs::turns(frames::TASK).unwrap(), frame },
            skein_kv::Op::Put {
                key: keys::encode_own(&keys::Own::TurnLog { task: frames::TASK }, fixtures::KEY_LIMIT).unwrap(),
                value: codec::component::TurnLog { window: Some(0) }.encode().unwrap(),
            },
            skein_kv::Op::Put {
                key: keys::encode_own(
                    &keys::Own::LogAttempt { task: frames::TASK, attempt: frames::ATTEMPT },
                    fixtures::KEY_LIMIT,
                )
                .unwrap(),
                value: codec::component::LogAttempt { first: 0 }.encode().unwrap(),
            },
        ]),
    );
    (disk, store)
}
#[test]
fn every_log_read_kind_converts_each_kept_body_without_writing() {
    for body in [5, 6] {
        for version in [1, 2] {
            let bytes = frames::frame(body, &turn(version, REPLAY));
            let (mut disk, mut store) = log_scene(&bytes);
            let log = crate::logs::turns(frames::TASK).unwrap();
            let extent = store.stores.store.log_end(log);
            for kind in 0..6 {
                let events = disk.request(&mut store, request(kind, BUDGET));
                assert_eq!(payload(&events), &*expected(version), "body {body}, turn {version}, kind {kind}");
                assert_eq!(store.stores.store.log_end(log), extent);
                assert_eq!(&*disk.bytes(PATH), &*bytes);
            }
        }
    }
}

fn unread(events: &[Event]) -> bool {
    match events {
        [
            Event::Unread { .. }
            | Event::Read { result: reads::Loaded::TurnProblem { why: reads::TurnFailure::Malformed, .. }, .. },
        ] => true,
        [Event::Read { result: reads::Loaded::Page { rows, .. }, .. }] => rows.first().is_some_and(|row| {
            row.unjoined
                == Some(reads::UnjoinedTurn::Unread {
                    task: frames::TASK,
                    attempt: frames::ATTEMPT,
                    turn: frames::TURN,
                })
        }),
        _ => false,
    }
}
#[test]
fn unknown_ids_and_refused_plans_are_unread() {
    const SHORT: &[u8] = &[0];
    for transcript in [turn(0, REPLAY), turn(3, REPLAY), turn(1, REPLAY)[..9].into(), Box::from(SHORT)] {
        for body in [5, 6] {
            let (mut disk, mut store) = scene(&frames::frame(body, &transcript), SETS);
            for kind in 0..6 {
                let events = disk.request(&mut store, request(kind, BUDGET));
                assert!(unread(&events), "body {body}, kind {kind}: {events:?}");
            }
        }
    }
    for body in [4, 7] {
        let (mut disk, mut store) = scene(&frames::frame(body, &turn(2, REPLAY)), SETS);
        for kind in 0..6 {
            let events = disk.request(&mut store, request(kind, BUDGET));
            assert!(unread(&events), "body {body}, kind {kind}: {events:?}");
        }
    }
}
#[test]
fn each_cut_of_a_behind_read_preserves_the_file() {
    let bytes = frames::frame(5, &turn(1, REPLAY));
    let (mut disk, mut store) = scene(&bytes, SETS);
    let events = disk.request(&mut store, request(0, BUDGET));
    drop(events);
    let points = disk.operations();
    for point in 0..=points {
        let (mut disk, mut store) = scene(&bytes, SETS);
        disk.cut_at(point);
        let events = disk.request(&mut store, request(0, BUDGET));
        drop(events);
        assert_eq!(&*disk.bytes(PATH), &*bytes, "cut {point}");
    }
}

#[test]
fn converted_frame_sizes_bound_resumes_turns_pages_and_tails() {
    // v6 body 44, v2 turn (2 + 4 + 5 + 4), framing 12.
    const CONVERTED_FRAME: u64 = 71;
    for (replay, budget, accepted) in
        [(b"r".as_slice(), 70, false), (b"r".as_slice(), CONVERTED_FRAME, true), (&[b'r'; 64], CONVERTED_FRAME, true)]
    {
        let raw = frames::frame(5, &turn(1, replay));
        for kind in [0, 1, 3, 4] {
            let (mut disk, mut store) = scene(&raw, SETS);
            let events = disk.request(&mut store, request(kind, budget));
            if accepted {
                let [Event::Turns { turns, .. }] = &*events else { panic!("turns: {events:?}") };
                assert!(!turns.is_empty(), "replay {}, kind {kind}, bound {budget}", replay.len());
            } else if kind == 0 || kind == 4 {
                let [Event::Turns { owner, turns, next: None, cut: Some(bytes) }] = &*events else {
                    panic!("cut window: {events:?}")
                };
                assert_eq!(*owner, OWNER);
                assert_eq!(u64::from(*bytes), len_u64(expected(1).len()));
                assert_eq!(
                    turns.as_ref(),
                    &[domain::TurnRecord {
                        task: frames::TASK,
                        attempt: frames::ATTEMPT,
                        turn: frames::TURN,
                        opens_window: true,
                        spent: frames::SPENT,
                        read: None,
                        at: Wall::EPOCH,
                        transcript: Box::new([]),
                    }]
                );
            } else {
                assert!(unread(&events), "replay {}, kind {kind}, bound {budget}: {events:?}", replay.len());
            }
        }
    }
}

#[test]
fn serving_opens_refuse_bad_frames_sets_without_io() {
    const WRONG: OwnerSets =
        OwnerSets { sets: &[OwnerSet { owner: Owner::Application, ..SETS.sets[0] }, SETS.sets[1]] };
    const TWO: OwnerSets = OwnerSets {
        sets: &[
            OwnerSet { families: &[SETS.sets[0].families[0], SETS.sets[0].families[0]], ..SETS.sets[0] },
            SETS.sets[1],
        ],
    };
    const OTHER: &str = "family other-frame 6\nversioned record TurnRecord { transcript: bytes 64 holds other Turn }\n";
    const MISMATCH: OwnerSets = OwnerSets {
        sets: &[
            OwnerSet {
                families: &[KeptFamily {
                    current: OTHER,
                    frames: Some(crate::Frames { texts: &[OTHER], number: 0, root: "TurnRecord" }),
                    name: "other-frame",
                    ..SETS.sets[0].families[0]
                }],
                level: Some(0),
                ..SETS.sets[0]
            },
            SETS.sets[1],
        ],
    };
    const MISSING: OwnerSets = OwnerSets { sets: &[SETS.sets[0]] };
    const BAD_ROOT: OwnerSets = OwnerSets {
        sets: &[
            SETS.sets[0],
            OwnerSet {
                families: &[KeptFamily {
                    frames: Some(crate::Frames {
                        root: "Missing",
                        ..SETS.sets[1].families[0].frames.expect("frames family")
                    }),
                    ..SETS.sets[1].families[0]
                }],
                ..SETS.sets[1]
            },
        ],
    };
    for (sets, owner) in [
        (WRONG, Owner::Application),
        (TWO, Owner::Jig),
        (MISMATCH, Owner::Jig),
        (MISSING, Owner::Jig),
        (BAD_ROOT, Owner::Smith),
    ] {
        let mut store = fixtures::over(fixtures::stores());
        store.owner_sets(sets);
        let events = fixtures::run(
            &mut store,
            Request::Open {
                purpose: crate::Purpose::Serve,
                owner: Token::new(1),
                root: fixtures::ROOT,
                installation: fixtures::INSTALLATION,
            },
        );
        let [Event::OpenRefused { why: crate::OpenRefusal::Set { owner: found }, .. }] = &*events else {
            panic!("set refusal {events:?}")
        };
        assert_eq!(*found, owner);
        assert_eq!(store.stores.store.last(), 0);
    }
}

#[test]
fn party_and_activity_bounds_charge_the_converted_frame() {
    const CONVERTED_FRAME: u64 = 71; // v6 body 44 + v2 turn 15 + framing 12.
    const PAYLOAD: u64 = 15;
    for kind in [2, 5] {
        let raw = frames::frame(5, &turn(1, REPLAY));
        let (mut disk, mut store) = scene(&raw, SETS);
        let events = disk.request(&mut store, request(kind, BUDGET));
        let [Event::Read { result: reads::Loaded::Page { rows, .. }, .. }] = &*events else { panic!("row {events:?}") };
        let base = rows[0].owned_bytes().expect("row").checked_sub(PAYLOAD).expect("metadata");
        for (body, version) in [(5, 1), (6, 2)] {
            for replay in [b"r".as_slice(), &[b'r'; 64]] {
                let raw = frames::frame(body, &turn(version, replay));
                for (frame, accepted) in [(70, false), (CONVERTED_FRAME, true)] {
                    let (mut disk, mut store) = scene(&raw, SETS);
                    let events = disk.request(&mut store, request(kind, base.checked_add(frame).expect("bound")));
                    let carried = match &*events {
                        [Event::Read { result: reads::Loaded::Page { rows, .. }, .. }] => carries_turn(&rows[0]),
                        _ => false,
                    };
                    assert_eq!(carried, accepted, "kind {kind}, replay {}, frame {frame}: {events:?}", replay.len());
                    if !accepted {
                        let charged = match &*events {
                            [
                                Event::Read {
                                    result:
                                        reads::Loaded::TurnProblem { why: reads::TurnFailure::TooLarge { bytes }, .. },
                                    ..
                                },
                            ] => *bytes,
                            [Event::Read { result: reads::Loaded::Page { rows, .. }, .. }] => match rows[0].unjoined {
                                Some(reads::UnjoinedTurn::TooLarge { bytes, .. }) => bytes,
                                _ => panic!("large row"),
                            },
                            _ => panic!("large frame {events:?}"),
                        };
                        assert_eq!(charged, CONVERTED_FRAME);
                    }
                }
            }
        }
    }
}

fn carries_turn(row: &reads::Row) -> bool {
    if let Some(domain::Record::Core(domain::CoreRecord::Turn(turn))) = &row.related {
        return !turn.transcript.is_empty();
    }
    if let domain::Record::Core(domain::CoreRecord::Turn(turn)) = &row.value {
        return !turn.transcript.is_empty();
    }
    false
}

#[test]
fn cold_and_prepared_registry_plans_read_the_same_kept_frames() {
    const OTHER: &str = "family fixture-turn 1\nversioned record Other { said: text 64 replay: bytes 64 }\n";
    let schemas = SETS.prepare();
    let mut altered = schemas.to_vec();
    let old = altered.iter_mut().find(|schema| schema.text() == layouts::FRAME_TURN_KEPT.as_bytes()).unwrap();
    *old = skein_codec::schema::parse(OTHER.as_bytes()).unwrap();
    for prepared in [&[][..], &schemas, &altered] {
        let raw = frames::frame(5, &turn(1, REPLAY));
        let (mut disk, mut store) = scene_with_schemas(&raw, SETS, Box::leak(prepared.to_vec().into_boxed_slice()));
        let events = disk.request(&mut store, request(1, BUDGET));
        assert_eq!(payload(&events), expected(1).as_ref());
        assert_eq!(disk.bytes(PATH), raw);
    }
}

#[test]
fn the_converted_frame_must_fit_the_store_page_not_just_the_stored_frame() {
    let raw = frames::frame(5, &turn(1, b""));
    let current = frames::frame(6, &expected(1));
    assert!(raw.len() < current.len());
    let bound = u32::try_from(current.len()).unwrap();
    for (limit, accepted) in [(bound, true), (bound.checked_sub(1).unwrap(), false)] {
        let (mut disk, mut store) = scene(&raw, SETS);
        store.limits.store.page.bytes = limit;
        for kind in [0, 1, 3] {
            let events = disk.request(&mut store, request(kind, BUDGET));
            if accepted {
                assert_eq!(payload(&events), expected(1).as_ref());
            } else {
                assert!(unread(&events), "converted frame past page bytes: {events:?}");
            }
        }
    }
}

#[test]
fn a_window_cuts_from_the_head_only_when_both_ids_keep() {
    use super::fixtures::disk::Seen;
    const KEEP: &str = "family fixture-turn 1\nversioned record Turn { said: text 64 tokens: u32 = 0 }\n";
    const SMITH: OwnerSet = OwnerSet {
        families: &[KeptFamily {
            frames: Some(crate::Frames { root: "Turn", number: 0, texts: &[KEEP, layouts::FRAME_TURN] }),
            ..SETS.sets[1].families[0]
        }],
        ..SETS.sets[1]
    };
    const KEEPING: OwnerSets = OwnerSets { sets: &[SETS.sets[0], SMITH] };
    let said = [b'a'; 64];
    let mut current = 2_u16.to_be_bytes().to_vec();
    current.extend_from_slice(&64_u32.to_be_bytes());
    current.extend_from_slice(&said);
    current.extend_from_slice(&0_u32.to_be_bytes());
    let mut keeping = current.clone();
    keeping[0..2].copy_from_slice(&1_u16.to_be_bytes());
    let mut older = keeping.clone();
    older.truncate(70);
    older.extend_from_slice(&64_u32.to_be_bytes());
    older.extend_from_slice(&said);
    for (sets, body, bytes, whole) in [
        (KEEPING, 6, current.as_slice(), false),
        (KEEPING, 6, keeping.as_slice(), false),
        (SETS, 6, older.as_slice(), true),
        (SETS, 5, current.as_slice(), true),
    ] {
        let (mut disk, mut store) = scene(&frames::frame(body, bytes), sets);
        let events = disk.request(&mut store, request(0, 8));
        let [Event::Turns { cut: Some(74), .. }] = events.as_ref() else { panic!("window at head: {events:?}") };
        assert_eq!(
            disk.requests()
                .iter()
                .any(|(_, request)| { if let Seen::Read { max } = request { *max > 36 } else { false } }),
            whole,
            "body {body}, tail {:?}",
            bytes.get(..2)
        );
    }
}

#[test]
fn a_file_conversion_finishes_its_read_before_a_queued_commit() {
    let raw = frames::frame(5, &turn(1, REPLAY));
    let (mut disk, mut store) = scene(&raw, SETS);
    store.phase = crate::Phase::Ready { next: 3, format_due: false };
    let mut above = skein_lib::Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(&mut store, &fixtures::env(), request(0, BUDGET), &mut above, &mut below);
    assert!(above.is_empty());
    let events = fixtures::run(&mut store, fixtures::commit(3, Box::new([fixtures::header(3)])));
    assert!(events.is_empty());
    assert_eq!(store.stores.store.last(), 2);
    disk.drive(&mut below, |io, below| crate::up(&mut store, &fixtures::env(), io, &mut above, below));
    let events = fixtures::events(&mut above);
    let [page @ Event::Turns { .. }, Event::Committed { number: 3, .. }] = events.as_ref() else {
        panic!("converted read precedes the commit: {events:?}");
    };
    assert_eq!(payload(core::slice::from_ref(page)), expected(1).as_ref());
    assert_eq!(disk.bytes(PATH), raw);
}
