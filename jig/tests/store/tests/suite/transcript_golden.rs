//! The transcripts' framing's golden bytes (protocol/store.md, 14).
//! One task's file keeps three fixed turns across two attempts. These tests
//! never write the kept file: a deliberate change copies the file named by
//! the writer test's failure over `golden/transcript`.

use jig_domain_core as core;
use jig_protocol_store::{Event, Request, TranscriptRead, sizes};
use jig_store_world::{World, root};
use skein_lib::{Token, Wall};

/// The kept file, as the writer leaves it after `turns()`.
const KEPT: &[u8] = include_bytes!("../../golden/transcript");
/// store.md 8.1: task 21's high bits (21 >> 12), then its number, in hex.
const PATH: &[u8] = b"transcripts/0000000000000000/0000000000000015";
const SEED: u64 = 0x0000_474f_4c44_454e;
const READER: Token = Token::new(715);
// Fixed above the world's roughly 323 KiB row floor with a margin, so a
// changed row worst case does not move the kept bytes. If that floor passes
// the fixed page below, sizes::check fails before the world opens.
const TRANSCRIPT_BYTES: u32 = 400_000;
// sizes::frame counts 52 bytes of core-v6 metadata (including Some(read)
// and the transcript's length) plus 12 wrapper bytes (store.md 8.1).
const FRAME_OVERHEAD: u32 = 64;
const PAGE: u32 = TRANSCRIPT_BYTES + FRAME_OVERHEAD;
// The disk driver needs the page plus the KV chunk's 20-byte envelope.
const KV_CHUNK_OVERHEAD: u32 = 20;
// store.md 7.1: decoded room is separate from the encoded page bound.
const ROOM: u64 = PAGE as u64 * 2;
const READ_ROWS: u32 = 1;
// store.md 8.1: leading length, CRC-32C and trailing length, each a u32.
const WORD_BYTES: usize = 4;
const WRAPPER_BYTES: usize = WORD_BYTES * 3;

/// The fixed turns, in commit order; every non-payload field is exercised.
fn turns() -> [core::TurnRecord; 3] {
    [
        core::TurnRecord {
            spent: 11,
            read: Some(12),
            at: Wall::from_nanos(13),
            ..root::turn(root::LIVE, 1, 1, b"first window")
        },
        core::TurnRecord {
            spent: 21,
            read: Some(22),
            at: Wall::from_nanos(23),
            ..root::turn(
                root::LIVE,
                1,
                2,
                &vec![b'x'; usize::try_from(TRANSCRIPT_BYTES).expect("fixed transcript size")],
            )
        },
        core::TurnRecord {
            spent: 31,
            read: Some(32),
            at: Wall::from_nanos(33),
            ..root::turn(root::LIVE, 2, 1, b"second window")
        },
    ]
}

/// A world with P exactly at the large frame, each turn committed separately.
fn written() -> World {
    let mut limits = root::limits();
    limits.store.page.bytes = PAGE;
    limits.store.chunk = limits.store.chunk.max(PAGE + KV_CHUNK_OVERHEAD);
    sizes::check(&limits.core, &root::APPLICATION, &limits.journal, &limits.store, limits.orphan_erases)
        .expect("fixed golden page still covers the world's row floor");
    let mut world = World::unopened_observed(SEED, limits, None);
    world.open().expect("golden world opens");
    world.commit(root::first()).expect("task committed");
    for (index, turn) in turns().into_iter().enumerate() {
        // first() is commit 1; the three turns are commits 2, 3 and 4.
        let number = u64::try_from(index).expect("three turns") + 2;
        world.commit(root::record_turn_commit(number, turn)).expect("turn committed");
    }
    world
}

/// Replace the crashed writer's file with the kept file before reopening.
fn kept() -> World {
    let mut world = written();
    world.crash(SEED + 1);
    world.rewrite(PATH, KEPT);
    world.reopen();
    world
}

/// Independently split store.md 8.1 by leading lengths, excluding wrappers.
/// Invalid lengths are themselves a framing change, never a reader error.
fn bodies(mut bytes: &[u8]) -> Option<Vec<&[u8]>> {
    let mut result = Vec::new();
    while !bytes.is_empty() {
        let word = bytes.get(..WORD_BYTES)?.try_into().ok()?;
        let length = usize::try_from(u32::from_be_bytes(word)).ok()?;
        if length < WRAPPER_BYTES {
            return None;
        }
        let frame = bytes.get(..length)?;
        result.push(&frame[WORD_BYTES..length - WORD_BYTES * 2]);
        bytes = &bytes[length..];
    }
    Some(result)
}

/// Compare bytes; on failure keep only the writer's output in target tmp.
fn assert_kept(made: &[u8]) {
    if made == KEPT {
        return;
    }
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("transcript-golden-made");
    std::fs::create_dir_all(path.parent().expect("target tmp parent")).expect("create target tmp");
    std::fs::write(&path, made).expect("write the writer's file for inspection");
    let changed_body = match (bodies(KEPT), bodies(made)) {
        (Some(kept), Some(made)) => kept
            .iter()
            .zip(&made)
            .position(|(kept, made)| kept != made)
            .or_else(|| (kept.len() != made.len()).then_some(kept.len().min(made.len()))),
        _ => None,
    };
    let reason = changed_body.map_or_else(
        || "the transcripts' framing changed (protocol/store.md 8.1 and 14): from the first release a deployment holds these bytes, and changing them needs an upgrade of its own, run before the records' patches".to_owned(),
        |index| format!("frame {}'s body changed: the core's `TurnRecord` or smith's turn, whose changes are the patches' and their goldens' (store.md 8.6 and 14); the kept file is replaced in place before the first release", index + 1),
    );
    let offset = KEPT.iter().zip(made).position(|(kept, made)| kept != made).unwrap_or(KEPT.len().min(made.len()));
    panic!(
        "{reason}; first differing offset {offset}; kept length {}, writer length {}; writer's file: {}",
        KEPT.len(),
        made.len(),
        path.display()
    );
}

fn walk(world: &mut World, backwards: bool) -> Vec<core::TurnRecord> {
    let mut result = Vec::new();
    let mut after = None;
    // One row per page, with at most one final empty page: a bad cursor
    // must fail rather than hang the focused suite.
    for _ in 0..=turns().len() {
        let events = world.request(Request::Transcript {
            owner: READER,
            read: TranscriptRead::Page { task: root::LIVE, backwards, after },
            rows: READ_ROWS,
            bytes: ROOM,
        });
        let [Event::Turns { owner: READER, turns, next }] = events.as_slice() else {
            panic!("kept transcript page must contain whole turns, never Unread: {events:?}");
        };
        result.extend(turns.iter().cloned());
        let Some(next) = next else { return result };
        assert_ne!(after, Some(*next), "transcript cursor must advance");
        after = Some(*next);
    }
    panic!("kept transcript walk did not end");
}

#[test]
fn the_writers_file_is_the_kept_file_byte_for_byte() {
    assert_kept(written().contents()[PATH].as_deref().expect("task's transcript file"));
}

#[test]
fn the_kept_file_reads_forwards_to_the_fixed_turns() {
    assert_eq!(walk(&mut kept(), false), turns());
}

#[test]
fn the_kept_file_reads_backwards_to_the_fixed_turns() {
    assert_eq!(walk(&mut kept(), true), turns().into_iter().rev().collect::<Vec<_>>());
}
