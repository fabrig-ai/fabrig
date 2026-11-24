//! Independent acceptance checks for protocol/store/transcripts.md, 6, and protocol/store/opening-owners.md, 1, and protocol/store/README.md, 8.
use crate::frame_reads;

use jig_protocol_store::{
    self as store, Event, KeptFamily, Owner, OwnerSet, OwnerSets, Request, TranscriptRead, Write,
};
use jig_store_world::{World, patches, root};
use jig_test_layouts::frames;
use skein_lib::{Token, length::len_u64};

const READER: Token = Token::new(712);
const READ_BYTES: u64 = 256; // The world's core resume bound (root::CORE_LIMITS).
const BODY: KeptFamily = frame_reads::SETS.sets[0].families[0];
const TURN: KeptFamily = frame_reads::SETS.sets[1].families[0];
const BODY_FRAMES: store::Frames = BODY.frames.expect("body frames");
const TURN_FRAMES: store::Frames = TURN.frames.expect("turn frames");
fn selected() -> TranscriptRead {
    TranscriptRead::Turn { task: frames::TASK, attempt: frames::ATTEMPT, turn: frames::TURN }
}
pub(crate) fn kinds() -> [TranscriptRead; 4] {
    [
        selected(),
        TranscriptRead::Window { task: frames::TASK, attempt: frames::ATTEMPT, turn: frames::TURN, after: None },
        TranscriptRead::Page { task: frames::TASK, backwards: false, after: None },
        TranscriptRead::Page { task: frames::TASK, backwards: true, after: None },
    ]
}
fn ask(read: TranscriptRead, bytes: u64) -> Request {
    Request::Transcript { owner: READER, read, rows: patches::limits().store.page.rows, bytes }
}
pub(crate) fn log_frame_bytes(turn: &jig_domain_core::TurnRecord) -> u64 {
    let limits = root::limits();
    let codec = store::codec::limits(&limits.core).expect("codec");
    let kv = skein_kv::Store::memory(&limits.store);
    kv.frame(&store::codec::core::turn(turn, &codec).expect("turn body")).expect("log frame").len()
}
fn fixture(sets: OwnerSets, transcript: &[u8]) -> World {
    let mut world = World::unopened_observed(41, patches::limits(), None);
    let legacy = sets.sets.iter().any(|set| set.families.iter().any(|family| family.frames.is_some()));
    world.owner_sets(if legacy { patches::FRAME_LEGACY } else { sets });
    assert_eq!(world.open_switched(), Some(0));
    assert_eq!(world.commit(root::first()), Some(1));
    let mut writes = root::turn_commit(2, frames::TASK, frames::TURN, transcript).into_vec();
    for write in &mut writes {
        if let Write::Save(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))) = write {
            turn.spent = frames::SPENT;
        }
    }
    assert_eq!(world.commit(writes.into_boxed_slice()), Some(2));
    if legacy {
        world.crash(41);
        world.owner_sets(sets);
        world.open_switched().expect("register the frame layouts without rewriting the log");
    }
    world
}
fn bytes(events: &[Event]) -> &[u8] {
    let [Event::Turns { owner, turns, next: None, cut: None }] = events else { panic!("turn terminal: {events:?}") };
    assert_eq!(*owner, READER);
    assert_eq!(turns.len(), 1);
    let turn = &turns[0];
    assert_eq!((turn.task, turn.attempt, turn.turn), (frames::TASK, frames::ATTEMPT, frames::TURN));
    assert_eq!(turn.spent, frames::SPENT);
    assert_eq!(turn.read, None);
    assert!(turn.opens_window);
    &turn.transcript
}

#[test]
fn every_read_converts_again_without_writes() {
    for version in [1, 2] {
        let mut world = fixture(frame_reads::SETS, &frame_reads::turn(version, b"said", b"replay", 0));
        let before = world.contents();
        world.trace_writes();
        for read in kinds().into_iter().chain(kinds()) {
            let events = world.request(ask(read, READ_BYTES));
            assert_eq!(bytes(&events), frame_reads::turn(2, b"said", &[], 0).as_ref());
        }
        assert_eq!(world.contents(), before);
        assert!(world.changes().is_empty(), "a read issued a disk change");
    }
}

#[test]
fn empty_sets_keep_the_opaque_turn_path() {
    for sets in [OwnerSets::NONE, patches::FRAME_LEGACY] {
        let mut world = fixture(sets, b"opaque turn");
        let before = world.contents();
        for read in kinds() {
            assert_eq!(bytes(&world.request(ask(read, READ_BYTES))), b"opaque turn");
        }
        assert_eq!(world.contents(), before);
    }
}

#[test]
fn converted_frames_past_bounds_match_current_boundaries() {
    let expected = frame_reads::turn(2, b"said", &[], 0);
    let charge = len_u64(frames::frame(6, &expected).len());
    let behind_transcript = frame_reads::turn(1, b"said", b"x", 0);
    let mut behind = fixture(frame_reads::SETS, &behind_transcript);
    let mut current = fixture(frame_reads::SETS, &expected);
    for read in kinds() {
        let reference = current.request(ask(read, charge - 1));
        let patched = behind.request(ask(read, charge - 1));
        if matches!(read, TranscriptRead::Window { .. }) {
            for events in [&reference, &patched] {
                let [Event::Turns { owner, turns, next: None, cut }] = events.as_slice() else {
                    panic!("cut window {read:?}: {events:?}")
                };
                assert_eq!(*owner, READER);
                assert_eq!(*cut, Some(u32::try_from(expected.len()).expect("fixture length")));
                let mut turn = root::turn(frames::TASK, frames::ATTEMPT, frames::TURN, &[]);
                turn.spent = frames::SPENT;
                assert_eq!(turns.as_ref(), &[turn]);
            }
        } else {
            assert!(
                matches!(reference.as_slice(), [Event::Unread { owner }] if *owner == READER),
                "{read:?}: {reference:?}"
            );
            assert!(
                matches!(patched.as_slice(), [Event::Unread { owner }] if *owner == READER),
                "{read:?}: {patched:?}"
            );
        }
    }
}

#[test]
fn a_malformed_old_turn_is_unread_in_every_transcript_read() {
    let mut truncated = frame_reads::turn(1, b"said", b"x", 0).into_vec();
    truncated.pop();
    let mut world = fixture(frame_reads::SETS, &truncated);
    for read in kinds() {
        let events = world.request(ask(read, READ_BYTES));
        assert!(matches!(events.as_slice(), [Event::Unread { .. }]), "{events:?}");
    }
}

fn replaced(owner: Owner, families: &'static [KeptFamily]) -> OwnerSets {
    let mut sets = frame_reads::SETS.sets.to_vec();
    let slot = sets.iter_mut().find(|set| set.owner == owner).expect("fixture owner");
    slot.families = families;
    OwnerSets { sets: Box::leak(sets.into_boxed_slice()) }
}
#[expect(clippy::too_many_lines, reason = "frame admission cases keep each current text beside its frame texts")]
fn invalid_sets() -> Vec<(&'static str, OwnerSets)> {
    const KEYS: store::Keys = store::Keys { from: &[1, 0, 0], to: &[1, 0, 1], root: "TurnRecord" };
    const RANGES: &[store::KeyRange] = &[store::KeyRange { from: KEYS.from, to: KEYS.to }];
    vec![
        (
            "missing body root",
            replaced(
                Owner::Jig,
                &[KeptFamily { frames: Some(jig_protocol_store::Frames { root: "Absent", ..BODY_FRAMES }), ..BODY }],
            ),
        ),
        (
            "missing turn root",
            replaced(
                Owner::Smith,
                &[KeptFamily { frames: Some(jig_protocol_store::Frames { root: "Absent", ..TURN_FRAMES }), ..TURN }],
            ),
        ),
        (
            "map type reuses the frame number",
            replaced(
                Owner::Jig,
                &[KeptFamily {
                    keys: &[KEYS],
                    types: &[store::ValueType {
                        number: BODY_FRAMES.number,
                        root: KEYS.root,
                        variant: None,
                        keys: RANGES,
                    }],
                    ..BODY
                }],
            ),
        ),
        (
            "retired type reuses the frame number",
            replaced(
                Owner::Jig,
                &[KeptFamily {
                    keys: &[KEYS],
                    retired: &[store::Retired { number: BODY_FRAMES.number, keys: RANGES }],
                    ..BODY
                }],
            ),
        ),
        ("duplicate body", replaced(Owner::Jig, &[BODY, BODY])),
        ("duplicate turn", replaced(Owner::Smith, &[TURN, TURN])),
        (
            "unversioned body",
            replaced(
                Owner::Jig,
                &[KeptFamily {
                    current: "family fixture-frame 6\nrecord TurnRecord { transcript: bytes 64 holds fixture-turn Turn }\n",
                    frames: Some(store::Frames {
                        texts: &[
                            "family fixture-frame 6\nrecord TurnRecord { transcript: bytes 64 holds fixture-turn Turn }\n",
                        ],
                        ..BODY_FRAMES
                    }),
                    ..BODY
                }],
            ),
        ),
        (
            "enumeration root",
            replaced(
                Owner::Smith,
                &[KeptFamily {
                    current: "family fixture-turn 2\nenum Turn { blank }\n",
                    frames: Some(store::Frames {
                        texts: &["family fixture-turn 2\nenum Turn { blank }\n"],
                        ..TURN_FRAMES
                    }),
                    ..TURN
                }],
            ),
        ),
        (
            "unversioned turn",
            replaced(
                Owner::Smith,
                &[KeptFamily {
                    current: "family fixture-turn 2\nrecord Turn { said: text 64 }\n",
                    frames: Some(store::Frames {
                        texts: &["family fixture-turn 2\nrecord Turn { said: text 64 }\n"],
                        ..TURN_FRAMES
                    }),
                    ..TURN
                }],
            ),
        ),
        (
            "wrong held family",
            replaced(
                Owner::Jig,
                &[KeptFamily {
                    current: "family fixture-frame 6\nversioned record TurnRecord { transcript: bytes 64 holds absent Turn }\n",
                    frames: Some(store::Frames {
                        texts: &[
                            "family fixture-frame 6\nversioned record TurnRecord { transcript: bytes 64 holds absent Turn }\n",
                        ],
                        ..BODY_FRAMES
                    }),
                    ..BODY
                }],
            ),
        ),
        (
            "no held turn",
            replaced(
                Owner::Jig,
                &[KeptFamily {
                    current: "family fixture-frame 6\nversioned record TurnRecord { transcript: bytes 64 }\n",
                    frames: Some(store::Frames {
                        texts: &["family fixture-frame 6\nversioned record TurnRecord { transcript: bytes 64 }\n"],
                        ..BODY_FRAMES
                    }),
                    ..BODY
                }],
            ),
        ),
        (
            "two held turns",
            replaced(
                Owner::Jig,
                &[KeptFamily {
                    current: "family fixture-frame 6\nversioned record TurnRecord { transcript: bytes 64 holds fixture-turn Turn extra: bytes 64 holds fixture-turn Turn }\n",
                    frames: Some(store::Frames {
                        texts: &[
                            "family fixture-frame 6\nversioned record TurnRecord { transcript: bytes 64 holds fixture-turn Turn extra: bytes 64 holds fixture-turn Turn }\n",
                        ],
                        ..BODY_FRAMES
                    }),
                    ..BODY
                }],
            ),
        ),
        (
            "hold cycle",
            replaced(
                Owner::Smith,
                &[KeptFamily {
                    current: "family fixture-turn 2\nversioned record Turn { back: bytes 64 holds fixture-frame TurnRecord }\n",
                    frames: Some(store::Frames {
                        texts: &[
                            "family fixture-turn 2\nversioned record Turn { back: bytes 64 holds fixture-frame TurnRecord }\n",
                        ],
                        ..TURN_FRAMES
                    }),
                    ..TURN
                }],
            ),
        ),
    ]
}
#[test]
fn a_same_version_layout_registers_and_writes_its_id_to_files_and_logs() {
    use jig_test_layouts::layouts;
    const FILE: &[u8] = b"transcripts/0000000000000000/0000000000000015";
    const SAID: &[u8] = b"said";
    let next = replaced(Owner::Smith, &[KeptFamily { current: layouts::FRAME_TURN_NEXT, ..TURN }]);
    for file in [false, true] {
        let tasks = if file { &[frames::TASK][..] } else { &[] };
        let mut world = crate::map_held::world_with_sets(41, &patches::limits(), tasks, patches::FRAME_LEGACY);
        let old = frame_reads::turn(1, SAID, b"replay", 0);
        assert_eq!(world.commit(root::turn_commit(2, frames::TASK, 1, &old)), Some(2));
        let before = world.contents();
        world.crash(42);
        world.owner_sets(frame_reads::SETS);
        let first = world.open_switched().expect("seed the pre-registry versions");
        assert!(first > 2);
        world.crash(43);
        world.owner_sets(next);
        let last = world.open_switched().expect("register same-version layout");
        assert!(last > first);
        let (source, target) = if file {
            (FILE.to_vec(), FILE.to_vec())
        } else {
            let log = skein_kv::log_path(store::logs::turns(frames::TASK).expect("turn log"));
            let active = world.active();
            let active = active.strip_suffix(b"\n").expect("generation pointer");
            ([b"generations/1/store/".as_slice(), &log].concat(), [b"generations/", active, b"/store/", &log].concat())
        };
        assert_eq!(world.contents().get(&target), before.get(&source));
        let mut turn = frame_reads::turn(2, SAID, &[], 4).into_vec();
        turn.extend(11_u32.to_be_bytes());
        assert_eq!(world.commit(root::turn_commit(last + 1, frames::TASK, 2, &turn)), Some(last + 1));
        let mut expected = frame_reads::turn(2, SAID, &[], 0).into_vec();
        expected.extend(7_u32.to_be_bytes());
        let read = TranscriptRead::Page { task: frames::TASK, backwards: false, after: None };
        let turns = world.transcript(read);
        assert_eq!(turns.len(), 2);
        assert_eq!(&*turns[0].transcript, expected);
        assert_eq!(&*turns[1].transcript, turn);
        if file {
            let contents = world.contents();
            let on_disk = contents[FILE].as_ref().expect("file frames");
            let at = before[FILE].as_ref().expect("legacy frame").len();
            let mut encoded =
                store::codec::core::turn(&turns[1], &store::codec::limits(&patches::limits().core).expect("codec"))
                    .expect("body");
            assert_eq!(encoded[..2], 6_u16.to_be_bytes());
            let held = encoded.len() - turn.len();
            encoded[held..held + 2].copy_from_slice(&3_u16.to_be_bytes());
            assert_eq!(on_disk[..at], *before[FILE].as_ref().expect("legacy frame"));
            assert_eq!(on_disk[at..], *frames::wrap(&encoded));
        }
    }
}

#[test]
fn an_unplanned_frame_layout_refuses_open_without_a_census_exception() {
    use jig_test_layouts::layouts;
    let mut world = baseline(frame_reads::SETS);
    let sets = replaced(Owner::Smith, &[KeptFamily { current: layouts::FRAME_TURN_RETYPED, ..TURN }]);
    world.owner_sets(sets);
    let before = world.contents();
    assert_eq!(world.open_switched(), None);
    assert!(
        matches!(world.refusal(), Some(store::OpenRefusal::Layout {
        family, value_type: 0, id: 1, values: None, why: store::Unplanned::Underived(_)
    }) if family.as_ref() == TURN.name),
        "{:?}",
        world.refusal()
    );
    patches::kept(&before, &world.contents(), &"unplanned frame");
}

fn set_refused(world: &World, label: &str) {
    assert!(matches!(world.refusal(), Some(store::OpenRefusal::Set { .. })), "{label}: {:?}", world.refusal());
}

#[test]
fn invalid_frame_sets_refuse_before_first_start_writes() {
    for (label, sets) in invalid_sets() {
        let mut world = World::unopened_observed(41, patches::limits(), None);
        world.owner_sets(sets);
        let before = world.contents();
        world.trace_writes();
        assert_eq!(world.open_switched(), None, "{label}");
        set_refused(&world, label);
        assert_eq!(world.contents(), before, "{label}");
        assert!(world.changes().is_empty(), "{label}: {:?}", world.changes());
    }
}

#[test]
fn invalid_frame_sets_refuse_at_serving_open() {
    for (label, sets) in invalid_sets() {
        let mut world = World::unopened_observed(41, patches::limits(), None);
        world.owner_sets(frame_reads::SETS);
        assert_eq!(world.open_switched(), Some(0));
        assert_eq!(world.commit(Box::new([Write::Save(root::header(1))])), Some(1));
        world.crash(42);
        world.owner_sets(sets);
        let before = world.contents();
        assert_eq!(world.open_switched(), None, "{label}");
        set_refused(&world, label);
        patches::kept(&before, &world.contents(), &label);
    }
}

pub(crate) fn a_read_cut_preserves_files(read: TranscriptRead, cuts: impl Fn(u32) -> Vec<u32>) {
    let transcript = frame_reads::turn(1, b"said", b"replay", 0);
    let mut complete = fixture(frame_reads::SETS, &transcript);
    complete.trace_writes();
    let start = complete.operation_count();
    let events = complete.request(ask(read, READ_BYTES));
    assert_eq!(bytes(&events), frame_reads::turn(2, b"said", &[], 0).as_ref());
    let operations = complete.operation_count() - start;
    assert!(operations > 0);
    let mut world = fixture(frame_reads::SETS, &transcript);
    let before = world.contents();
    world.trace_writes();
    let sweep = crate::sweep::Sweep::start(
        &mut world,
        cuts(operations).into_iter().map(|cut| start + cut).collect(),
        move |world, operation| {
            let cut = operation - start;
            assert_eq!(world.operation_count(), start + cut, "{read:?}: read cut reached");
            assert_eq!(world.contents(), before, "{read:?}: cut {cut}/{operations}");
            assert!(world.changes().is_empty(), "{read:?}: cut {cut}/{operations}");
            world.crash(43);
            assert_eq!(world.contents(), before, "{read:?}: power loss at cut {cut}/{operations}");
            true
        },
    );
    let _events = world.request(ask(read, READ_BYTES));
    sweep.finish(&mut world);
}

#[test]
fn a_converted_read_cut_at_its_first_and_last_file_operation_preserves_all_files() {
    for read in kinds() {
        a_read_cut_preserves_files(read, |operations| {
            eprintln!("{read:?}: before first file operation 0, finished read {operations}");
            vec![0, operations]
        });
    }
}

fn baseline(sets: OwnerSets) -> World {
    let mut world = World::unopened_observed(41, patches::limits(), None);
    world.owner_sets(sets);
    assert_eq!(world.open_switched(), Some(0));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(1))])), Some(1));
    world.crash(42);
    world
}

#[test]
fn converted_frames_continue_at_their_own_offsets_in_both_directions() {
    const COUNT: u32 = 3;
    let transcript = frame_reads::turn(1, b"said", b"replay", 0);
    let mut offsets = Vec::new();
    let mut end = 0;
    for number in 1..=COUNT {
        offsets.push(end);
        let mut turn = root::turn(frames::TASK, frames::ATTEMPT, number, &transcript);
        if number == 1 {
            turn.spent = frames::SPENT;
        }
        end += log_frame_bytes(&turn);
    }
    let mut world = fixture(patches::FRAME_LEGACY, &transcript);
    for turn in 2..=COUNT {
        assert_eq!(
            world.commit(root::turn_commit(u64::from(turn) + 1, frames::TASK, turn, &transcript)),
            Some(u64::from(turn) + 1)
        );
    }
    world.crash(42);
    world.owner_sets(frame_reads::SETS);
    world.open_switched().expect("registry seeded");
    let before = world.contents();
    world.trace_writes();
    for backwards in [false, true] {
        let order: Vec<_> = if backwards { (0..COUNT).rev().collect() } else { (0..COUNT).collect() };
        let mut after = None;
        let mut found = Vec::new();
        for page in order.chunks(skein_lib::length::usize_of(patches::limits().store.page.rows)) {
            let events = world.request(ask(TranscriptRead::Page { task: frames::TASK, backwards, after }, READ_BYTES));
            let [Event::Turns { owner, turns, next, cut: None }] = events.as_slice() else {
                panic!("page terminal: {events:?}")
            };
            assert_eq!(*owner, READER);
            // protocol/store/transcripts.md 3 selects a page from either end without promising its turn order.
            let mut page_turns: Vec<_> = turns.iter().map(|turn| turn.turn - 1).collect();
            page_turns.sort_unstable();
            let mut expected = page.to_vec();
            expected.sort_unstable();
            assert_eq!(page_turns, expected);
            for turn in turns {
                assert_eq!(turn.transcript, frame_reads::turn(2, b"said", &[], 0));
                found.push(turn.turn - 1);
            }
            if let Some(next) = next {
                let index = if backwards { *page.last().expect("last") } else { page.last().expect("last") + 1 };
                assert_eq!(*next, offsets[usize::try_from(index).expect("bounded frame")]);
            }
            after = *next;
        }
        found.sort_unstable();
        let mut expected = order;
        expected.sort_unstable();
        assert_eq!(found, expected);
        assert_eq!(after, None);
    }
    assert_eq!(world.contents(), before);
    assert_eq!((world.changes()).len(), 0);
}

#[test]
fn kept_turn_nodes_are_not_limited_by_encoded_byte_count() {
    const TEXTS: &[&str] = &[
        "family fixture-turn 1\nrecord Empty {}\nversioned record Turn { said: text 64 replay: bytes 64 nodes: list 512 Empty }\n",
        "family fixture-turn 2\nrecord Empty {}\nversioned record Turn { said: text 64 tokens: u32 = 0 nodes: list 512 Empty }\n",
    ];
    const NODE_ITEMS: u32 = 512;
    const CURRENT: &str = TEXTS.last().expect("kept text");
    let sets = replaced(
        Owner::Smith,
        &[KeptFamily { current: CURRENT, frames: Some(store::Frames { texts: TEXTS, ..TURN_FRAMES }), ..TURN }],
    );
    let limits = patches::limits();
    let (mut component, env) = frame_reads::closed(sets, &limits);
    frame_reads::open(&mut component, &env);
    for count in [0, NODE_ITEMS] {
        let mut transcript = frame_reads::turn(1, b"", b"x", 0).into_vec();
        transcript.extend(count.to_be_bytes());
        let mut expected = frame_reads::turn(2, b"", &[], 0).into_vec();
        expected.extend(count.to_be_bytes());
        let raw = frames::frame(5, &transcript);
        assert_eq!(frame_reads::decode(&mut component, &env, &raw).as_deref(), Some(expected.as_slice()));
    }
}

#[test]
fn a_party_turn_uses_converted_bytes() {
    let mut world = fixture(frame_reads::SETS, &frame_reads::turn(1, b"said", b"replay", 0));
    let events = world.request(party_read(patches::limits().core.load_bytes));
    let [Event::Read { owner, result: jig_domain_core::reads::Loaded::Page { rows, next: None } }] = events.as_slice()
    else {
        panic!("party turn: {events:?}");
    };
    assert_eq!(*owner, READER);
    assert_eq!(rows.len(), 1);
    let jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn)) = &rows[0].value else {
        panic!("party row: {:?}", rows[0]);
    };
    assert_eq!(turn.transcript, frame_reads::turn(2, b"said", &[], 0));
}

fn foreign(sets: OwnerSets, frames: Option<&'static str>) -> OwnerSets {
    const CURRENT: &str = "family fixture-foreign 1\nversioned record Other { content: bytes 64 }\n";
    let frames = frames.map(|root| store::Frames { root, texts: &[CURRENT], number: 0 });
    let mut sets = sets.sets.to_vec();
    sets.push(OwnerSet {
        jobs: &[],
        owner: Owner::Application,
        name: "fixture-foreign",
        level: Some(0),
        families: Box::leak(Box::new([KeptFamily {
            current: "family fixture-foreign 1\nversioned record Other { content: bytes 64 }\n",
            name: "fixture-foreign",
            frames,
            ..BODY
        }])),
    });
    OwnerSets { sets: Box::leak(sets.into_boxed_slice()) }
}

#[test]
fn foreign_frames_refuse_when_every_owner_level_compares() {
    let sets = foreign(frame_reads::SETS, Some("Other"));
    let mut first = World::unopened_observed(41, patches::limits(), None);
    first.owner_sets(sets);
    let before = first.contents();
    first.trace_writes();
    assert_eq!(first.open_switched(), None);
    set_refused(&first, "foreign frames at first start");
    assert_eq!(first.contents(), before);
    assert_eq!((first.changes()).len(), 0);
    let mut world = baseline(foreign(frame_reads::SETS, None));
    world.owner_sets(sets);
    let before = world.contents();
    assert_eq!(world.open_switched(), None);
    set_refused(&world, "foreign frames with comparable levels");
    patches::kept(&before, &world.contents(), &"foreign frames");
}

#[test]
fn a_frames_family_in_the_application_slot_refuses_admission() {
    let mut sets = frame_reads::SETS.sets.to_vec();
    sets[0].owner = Owner::Application;
    let sets = OwnerSets { sets: Box::leak(sets.into_boxed_slice()) };
    let mut world = baseline(frame_reads::SETS);
    world.owner_sets(sets);
    let before = world.contents();
    assert_eq!(world.open_switched(), None);
    assert!(
        matches!(world.refusal(), Some(store::OpenRefusal::Set { owner: Owner::Application })),
        "{:?}",
        world.refusal()
    );
    patches::kept(&before, &world.contents(), &"level precedes frames");
}

fn party_read(bytes: u64) -> Request {
    Request::Read {
        owner: READER,
        range: jig_domain_core::reads::Range {
            party: 1,
            scope: jig_domain_core::reads::Scope::Turn {
                task: frames::TASK,
                attempt: frames::ATTEMPT,
                turn: frames::TURN,
            },
            after: None,
            backwards: false,
            rows: 1,
            bytes,
        },
    }
}

#[test]
fn party_turn_too_large_names_the_converted_charge() {
    let expected = frame_reads::turn(2, b"said", &[], 0);
    let charge = len_u64(frames::frame(6, &expected).len());
    // This admitted request has no room for the turn after its owner's join.
    let bound = u64::from(patches::limits().store.page.bytes);
    for transcript in [frame_reads::turn(1, b"said", b"x", 0), expected] {
        let mut world = fixture(frame_reads::SETS, &transcript);
        let events = world.request(party_read(bound));
        assert!(
            matches!(events.as_slice(), [Event::Read { owner, result: jig_domain_core::reads::Loaded::TurnProblem { why: jig_domain_core::reads::TurnFailure::TooLarge { bytes }, .. } }] if *owner == READER && *bytes == charge),
            "{events:?}"
        );
    }
}

#[test]
fn old_and_current_frames_fit_exactly_the_converted_charge() {
    let expected = frame_reads::turn(2, b"said", &[], 0);
    let charge = len_u64(frames::frame(6, &expected).len());
    for transcript in [frame_reads::turn(1, b"said", b"x", 0), expected.clone()] {
        let mut world = fixture(frame_reads::SETS, &transcript);
        let before = world.contents();
        for read in kinds() {
            assert_eq!(bytes(&world.request(ask(read, charge))), expected.as_ref());
        }
        assert_eq!(world.contents(), before);
    }
}
