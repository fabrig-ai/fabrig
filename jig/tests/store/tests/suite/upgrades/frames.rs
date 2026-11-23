//! Independent acceptance checks for protocol/store/transcripts.md, 6, and protocol/store/opening-owners.md, 1, and protocol/store/README.md, 8.
use crate::frame_reads;

use jig_protocol_store::{
    self as store, Event, KeptFamily, Owner, OwnerSet, OwnerSets, Request, TranscriptRead, Write, keys,
};
use jig_store_world::{World, patches, root};
use jig_test_patches_jig::frames;
use jig_test_patches_smith::patches as smith;
use skein_codec::patch::Halt;
use skein_lib::{Token, length::len_u64};

const READER: Token = Token::new(712);
const OTHER: Token = Token::new(713);
const READ_BYTES: u64 = 256; // The world's core resume bound (root::CORE_LIMITS).
const BODY: KeptFamily = frame_reads::SETS.sets[0].families[0];
const TURN: KeptFamily = frame_reads::SETS.sets[1].families[0];
const BODY_FRAMES: store::Frames = BODY.frames.expect("body frames");
const TURN_FRAMES: store::Frames = TURN.frames.expect("turn frames");
const EMPTY: OwnerSets = OwnerSets {
    sets: &[
        OwnerSet {
            level: None,
            families: &[KeptFamily {
                frames: Some(jig_protocol_store::Frames { texts: &[], patches: &[], ..BODY_FRAMES }),
                ..BODY
            }],
            ..frame_reads::SETS.sets[0]
        },
        OwnerSet {
            level: None,
            families: &[KeptFamily {
                frames: Some(jig_protocol_store::Frames { texts: &[], patches: &[], ..TURN_FRAMES }),
                ..TURN
            }],
            ..frame_reads::SETS.sets[1]
        },
    ],
};

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
    world.owner_sets(sets);
    assert_eq!(world.open_switched(), Some(0));
    assert_eq!(world.commit(root::first()), Some(1));
    let mut writes = root::turn_commit(2, frames::TASK, frames::TURN, transcript).into_vec();
    for write in &mut writes {
        if let Write::Save(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))) = write {
            turn.spent = frames::SPENT;
        }
    }
    assert_eq!(world.commit(writes.into_boxed_slice()), Some(2));
    world
}
fn answer(world: &mut World, event: Event, halt: Option<Halt>, offset: u64) -> Vec<Event> {
    let Event::Site { owner, of, key, site, at, mut patcher } = event else {
        panic!("expected written site: {event:?}");
    };
    assert_eq!(owner, READER);
    assert_eq!(of, Owner::Smith);
    let mut expected = keys::encode_own(&keys::Own::TurnLog { task: frames::TASK }, 64).expect("task key").to_vec();
    expected.extend(offset.to_be_bytes());
    assert_eq!(key.as_ref(), expected);
    let result = if let Some(halt) = halt {
        Err(halt)
    } else {
        let (source, target) = patcher.parts();
        smith::written(site, source, target, at, &mut smith::Context { key })
    };
    world.request(Request::Site { owner, patcher, result })
}
fn settled(world: &mut World, request: Request) -> Vec<Event> {
    let mut events = world.request(request);
    for _ in 0..8 {
        if events.len() == 1 && matches!(events[0], Event::Site { .. }) {
            events = answer(world, events.remove(0), None, 0);
        } else {
            return events;
        }
    }
    panic!("fixture read did not settle");
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
fn every_read_patches_again_without_writes() {
    for version in [1, 2] {
        let mut world = fixture(frame_reads::SETS, &frame_reads::turn(version, b"said", b"replay", 24));
        let before = world.contents();
        world.trace_writes();
        for read in kinds().into_iter().chain(kinds()) {
            let events = settled(&mut world, ask(read, READ_BYTES));
            assert_eq!(bytes(&events), frame_reads::turn(2, b"said", &[], 24).as_ref());
        }
        assert_eq!(world.contents(), before);
        assert!(world.changes().is_empty(), "a read issued a disk change");
    }
}

#[test]
fn empty_sets_keep_the_opaque_turn_path() {
    for sets in [OwnerSets::NONE, EMPTY] {
        let mut world = fixture(sets, b"opaque turn");
        let before = world.contents();
        for read in kinds() {
            assert_eq!(bytes(&world.request(ask(read, READ_BYTES))), b"opaque turn");
        }
        assert_eq!(world.contents(), before);
    }
}

#[test]
fn patched_frames_past_bounds_match_current_boundaries() {
    let expected = frame_reads::turn(2, b"said", &[], 19);
    let charge = len_u64(frames::frame(6, &expected).len());
    let behind_transcript = frame_reads::turn(1, b"said", b"x", 0);
    let mut behind = fixture(frame_reads::SETS, &behind_transcript);
    let mut current = fixture(frame_reads::SETS, &expected);
    for read in kinds() {
        let reference = settled(&mut current, ask(read, charge - 1));
        let patched = settled(&mut behind, ask(read, charge - 1));
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
fn every_site_halt_is_unread_and_the_next_read_recovers() {
    {
        let mut world = fixture(frame_reads::SETS, &frame_reads::turn(1, b"said", b"replay", 0));
        let before = world.contents();
        world.trace_writes();
        for read in kinds() {
            for halt in [Halt::Room, Halt::Order, Halt::NoSite(skein_codec::patch::Site::new(1, 0))] {
                let mut events = world.request(ask(read, READ_BYTES));
                assert_eq!(events.len(), 1);
                events = answer(&mut world, events.remove(0), Some(halt), 0);
                assert!(matches!(events.as_slice(), [Event::Unread { owner }] if *owner == READER), "{events:?}");
                assert_eq!(
                    bytes(&settled(&mut world, ask(read, READ_BYTES))),
                    frame_reads::turn(2, b"said", &[], 24).as_ref()
                );
            }
        }
        assert_eq!(world.contents(), before);
        assert!(world.changes().is_empty(), "halted reads change no durable file");
        world.crash(43);
        world.open_switched().expect("durable frames reopen after site halts");
        assert_eq!(
            bytes(&settled(&mut world, ask(selected(), READ_BYTES))),
            frame_reads::turn(2, b"said", &[], 24).as_ref()
        );
    }
}

#[test]
fn declared_refusal_is_unread_in_every_transcript_read() {
    let mut world = fixture(frame_reads::SETS, &frame_reads::turn(1, b"said", &[], 0));
    for read in kinds() {
        let events = settled(&mut world, ask(read, READ_BYTES));
        assert!(matches!(events.as_slice(), [Event::Unread { .. }]), "{events:?}");
    }
}

#[test]
fn a_commit_waits_until_the_written_site_finishes() {
    let mut world = fixture(frame_reads::SETS, &frame_reads::turn(1, b"said", b"replay", 0));
    let mut events = world.request(ask(selected(), READ_BYTES));
    assert_eq!(events.len(), 1);
    let before = world.contents();
    world.trace_writes();
    let queued = world.request(Request::Commit { number: 3, writes: root::plain_commit(3) });
    assert!(queued.is_empty(), "commit answered ahead of the waiting read: {queued:?}");
    assert_eq!(world.contents(), before);
    assert_eq!((world.changes()).len(), 0);
    events = answer(&mut world, events.remove(0), None, 0);
    assert!(matches!(events.as_slice(), [Event::Turns { .. }, Event::Committed { number: 3, .. }]), "{events:?}");
}

fn replaced(owner: Owner, families: &'static [KeptFamily]) -> OwnerSets {
    let mut sets = frame_reads::SETS.sets.to_vec();
    let slot = sets.iter_mut().find(|set| set.owner == owner).expect("fixture owner");
    slot.families = families;
    OwnerSets { sets: Box::leak(sets.into_boxed_slice()) }
}
#[expect(clippy::too_many_lines, reason = "frame admission cases keep each current text beside its frame texts")]
fn invalid_sets() -> Vec<(&'static str, OwnerSets)> {
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
                        patches: &[],
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
                        patches: &[],
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
                        patches: &[],
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
                        patches: &[],
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
                        patches: &[],
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
                        patches: &[],
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
                        patches: &[],
                        ..TURN_FRAMES
                    }),
                    ..TURN
                }],
            ),
        ),
    ]
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
    let events = settled(&mut complete, ask(read, READ_BYTES));
    assert_eq!(bytes(&events), frame_reads::turn(2, b"said", &[], 24).as_ref());
    let operations = complete.operation_count() - start;
    assert!(operations > 0);
    let mut world = fixture(frame_reads::SETS, &transcript);
    let before = world.contents();
    crate::sweep::fork_case(&mut world, start, |waiting| {
        let events = waiting.request(ask(read, READ_BYTES));
        assert!(matches!(events.as_slice(), [Event::Site { .. }]));
        waiting.crash(43);
        assert_eq!(waiting.contents(), before, "power loss at a written site");
        true
    });
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
    let _events = settled(&mut world, ask(read, READ_BYTES));
    sweep.finish(&mut world);
}

#[test]
fn a_patched_read_cut_at_its_first_and_last_file_operation_preserves_all_files() {
    for read in kinds() {
        a_read_cut_preserves_files(read, |operations| {
            eprintln!("{read:?}: before first file operation 0, finished read {operations}");
            vec![0, operations]
        });
    }
}

fn stray(owner: Token) -> Request {
    let capacity = skein_codec::patch::Capacity { versions: 0, patches: 0, text: 0, bytes: 0, nodes: 0 };
    Request::Site { owner, patcher: Box::new(skein_codec::patch::Patcher::new(capacity)), result: Err(Halt::Order) }
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
fn a_stray_site_is_unread_during_upgrades_idle_and_waiting_reads() {
    // Suspend before the copy's first directory creation.
    let mut complete = baseline(patches::FRAME_BASE);
    complete.owner_sets(patches::FRAME_JOBS);
    complete.trace_writes();
    assert_eq!(complete.open_switched(), Some(3));
    let copy = complete
        .changes()
        .iter()
        .find(|(_, change)| *change == jig_store_world::world::Change::Make)
        .expect("copy directory")
        .0;
    let mut patching = baseline(patches::FRAME_BASE);
    patching.owner_sets(patches::FRAME_JOBS);
    patching.trace_writes();
    patching.cut_at(copy);
    assert_eq!(patching.open_switched(), None);
    let before = patching.contents();
    let reference = patching.request(stray(OTHER));
    assert!(matches!(reference.as_slice(), [Event::Unread { owner: OTHER }]), "{reference:?}");
    assert_eq!(patching.contents(), before);
    let mut idle = fixture(frame_reads::SETS, &frame_reads::turn(1, b"said", b"replay", 0));
    assert_eq!(format!("{:?}", idle.request(stray(OTHER))), format!("{reference:?}"));
    let mut world = fixture(frame_reads::SETS, &frame_reads::turn(1, b"said", b"replay", 0));
    let mut events = world.request(ask(selected(), READ_BYTES));
    assert_eq!(events.len(), 1);
    let before = world.contents();
    world.trace_writes();
    let refused = world.request(stray(OTHER));
    assert_eq!(format!("{refused:?}"), format!("{reference:?}"));
    assert!(matches!(answer(&mut world, events.remove(0), None, 0).as_slice(), [Event::Turns { .. }]));
    let read = settled(&mut world, ask(selected(), READ_BYTES));
    assert!(matches!(read.as_slice(), [Event::Turns { .. }]));
    assert_eq!(world.contents(), before);
    assert_eq!((world.changes()).len(), 0);
}

#[test]
fn several_frames_suspend_at_their_own_offsets_in_both_directions() {
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
    let mut world = fixture(frame_reads::SETS, &transcript);
    for turn in 2..=COUNT {
        assert_eq!(
            world.commit(root::turn_commit(u64::from(turn) + 1, frames::TASK, turn, &transcript)),
            Some(u64::from(turn) + 1)
        );
    }
    let before = world.contents();
    world.trace_writes();
    for backwards in [false, true] {
        let order: Vec<_> = if backwards { (0..COUNT).rev().collect() } else { (0..COUNT).collect() };
        let mut after = None;
        let mut found = Vec::new();
        for page in order.chunks(skein_lib::length::usize_of(patches::limits().store.page.rows)) {
            let mut events =
                world.request(ask(TranscriptRead::Page { task: frames::TASK, backwards, after }, READ_BYTES));
            for index in page {
                assert_eq!(events.len(), 1, "{events:?}");
                events = answer(
                    &mut world,
                    events.remove(0),
                    None,
                    offsets[usize::try_from(*index).expect("bounded frame")],
                );
            }
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
                assert_eq!(turn.transcript, frame_reads::turn(2, b"said", &[], 24));
                found.push(turn.turn - 1);
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
        "family fixture-turn 2\nrecord Empty {}\nversioned record Turn { said: text 64 tokens: u32 nodes: list 512 Empty }\n",
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
        let mut expected = frame_reads::turn(2, b"", &[], 19).into_vec();
        expected.extend(count.to_be_bytes());
        let raw = frames::frame(5, &transcript);
        assert_eq!(frame_reads::decode(&mut component, &env, &raw, false).as_deref(), Some(expected.as_slice()));
    }
}

#[test]
fn a_party_turn_uses_patched_bytes() {
    let mut world = fixture(frame_reads::SETS, &frame_reads::turn(1, b"said", b"replay", 0));
    let events = settled(&mut world, party_read(patches::limits().core.load_bytes));
    let [Event::Read { owner, result: jig_domain_core::reads::Loaded::Page { rows, next: None } }] = events.as_slice()
    else {
        panic!("party turn: {events:?}");
    };
    assert_eq!(*owner, READER);
    assert_eq!(rows.len(), 1);
    let jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn)) = &rows[0].value else {
        panic!("party row: {:?}", rows[0]);
    };
    assert_eq!(turn.transcript, frame_reads::turn(2, b"said", &[], 24));
}

fn foreign(sets: OwnerSets, frames: Option<&'static str>) -> OwnerSets {
    const CURRENT: &str = "family fixture-foreign 1\nversioned record Other { content: bytes 64 }\n";
    let frames = frames.map(|root| store::Frames { root, texts: &[CURRENT], patches: &[] });
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
fn party_turn_too_large_names_the_patched_charge() {
    let expected = frame_reads::turn(2, b"said", &[], 19);
    let charge = len_u64(frames::frame(6, &expected).len());
    // This admitted request has no room for the turn after its owner's join.
    let bound = u64::from(patches::limits().store.page.bytes);
    for transcript in [frame_reads::turn(1, b"said", b"x", 0), expected] {
        let mut world = fixture(frame_reads::SETS, &transcript);
        let events = settled(&mut world, party_read(bound));
        assert!(
            matches!(events.as_slice(), [Event::Read { owner, result: jig_domain_core::reads::Loaded::TurnProblem { why: jig_domain_core::reads::TurnFailure::TooLarge { bytes }, .. } }] if *owner == READER && *bytes == charge),
            "{events:?}"
        );
    }
}

#[test]
fn old_and_current_frames_fit_exactly_the_patched_charge() {
    let expected = frame_reads::turn(2, b"said", &[], 19);
    let charge = len_u64(frames::frame(6, &expected).len());
    for transcript in [frame_reads::turn(1, b"said", b"x", 0), expected.clone()] {
        let mut world = fixture(frame_reads::SETS, &transcript);
        let before = world.contents();
        for read in kinds() {
            assert_eq!(bytes(&settled(&mut world, ask(read, charge))), expected.as_ref());
        }
        assert_eq!(world.contents(), before);
    }
}
