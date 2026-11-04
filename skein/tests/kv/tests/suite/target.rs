//! Snapshot chunks pack to a target, with heap for their rows and one encoding
//! (skein/docs/design/kv.md, 5, 7 and 9).

use skein_kv::{Event, Limits, Op, Page, Request};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Token};

const TARGET: u32 = skein_kv::CHUNK_TARGET;
const CEILING: u32 = 16 * TARGET;
const OWNER: Token = Token::new(1);
// An eight-byte key, a kilobyte value and eight bytes of row lengths.
const VALUE_BYTES: u32 = 1024;
const ROW_BYTES: u32 = 1040;
const BATCH: u64 = 64;
// About 3.3 MiB, enough to fill three target chunks and leave a fourth.
const ROWS: u64 = 3300;
// Rows and their boxes beside one encoding, with a quarter target of slack.
const SNAPSHOT_BOUND: u32 = 2 * TARGET + TARGET / 4;

/// Limits with a `CEILING` chunk, snapshotting whenever the store is fired.
fn limits() -> Limits {
    Limits {
        key: 8,
        value: CEILING - 28,
        ops: u32::try_from(BATCH).expect("batch fits u32"),
        commit: CEILING,
        queued: 8,
        queued_bytes: 2 * u64::from(CEILING),
        budget: 4 * u64::from(CEILING),
        segment: 2 * u64::from(CEILING),
        snapshot_after: 1,
        chunk: CEILING,
        page: Page { rows: 16, bytes: 1024 },
        deadline: Duration::from_secs(1),
    }
}

fn open(limits: Limits) -> World {
    let mut world = World::untraced(limits, &[]);
    world.request_no_fire(Request::Open { owner: OWNER, root: world.root });
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: 0 });
    world
}

fn close(world: &mut World) {
    world.request(Request::Close { owner: OWNER });
    assert_eq!(world.pop(), Event::Closed { owner: OWNER });
}

fn put(key: &[u8], value: &[u8]) -> Op {
    Op::Put { key: Box::from(key), value: Box::from(value) }
}

fn read_row(world: &mut World, key: &[u8], value: &[u8]) {
    world.request(Request::Get { owner: OWNER, key: Box::from(key) });
    assert_eq!(world.pop(), Event::Got { owner: OWNER, value: Some(Box::from(value)) });
}

/// A store of `rows` rows of a kilobyte, committed and not yet snapshotted, its world untraced.
fn small_rows(rows: u64) -> World {
    let mut world = open(limits());
    let value = vec![b'v'; usize::try_from(VALUE_BYTES).expect("value fits usize")];
    for start in (0..rows).step_by(usize::try_from(BATCH).expect("batch fits usize")) {
        let ops = (start..(start + BATCH).min(rows)).map(|key| put(&key.to_be_bytes(), &value)).collect();
        world.request_no_fire(Request::Commit { owner: OWNER, ops });
        assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: start / BATCH + 1 });
    }
    world
}

/// Each chunk of a snapshot file, its rows and its bytes, read from its lengths.
fn chunks(snapshot: &[u8]) -> Vec<(u32, u32)> {
    // The snapshot header is magic, version, start number and checksum.
    const HEADER: usize = 18;
    // A chunk's two lengths and checksum surround its body.
    const FRAMING: u32 = 12;
    let mut at = HEADER;
    let mut chunks = Vec::new();
    while &snapshot[at..at + 4] != b"SNPE" {
        let count = u32::from_be_bytes(snapshot[at..at + 4].try_into().expect("row count"));
        let body = u32::from_be_bytes(snapshot[at + 4..at + 8].try_into().expect("body length"));
        let bytes = FRAMING + body;
        chunks.push((count, bytes));
        at += usize::try_from(bytes).expect("chunk fits usize");
    }
    chunks
}

#[test]
fn many_small_rows_pack_to_the_target() {
    let mut world = small_rows(ROWS);
    let mut peak = 0;
    while world.ready() {
        peak = peak.max(world.fire_measured().peak);
    }
    close(&mut world);
    let files = world.files();
    let chunks = chunks(files.get(&b"snapshot"[..]).expect("the store made a snapshot"));
    eprintln!("snapshot: chunks {chunks:?}, step peak {peak}, bound {SNAPSHOT_BOUND}");
    let rows_per_chunk = (TARGET - 12) / ROW_BYTES;
    assert_eq!(chunks.len(), usize::try_from(ROWS.div_ceil(u64::from(rows_per_chunk))).expect("chunk count fits"));
    for &(_, bytes) in &chunks[..chunks.len() - 1] {
        assert!(bytes <= TARGET && bytes > TARGET - ROW_BYTES, "chunk bytes {bytes}");
    }
    assert_eq!(chunks.iter().map(|&(rows, _)| u64::from(rows)).sum::<u64>(), ROWS);
    assert!(peak <= i64::from(SNAPSHOT_BOUND), "snapshot peak {peak} > {SNAPSHOT_BOUND}");
    const { assert!(SNAPSHOT_BOUND < CEILING / 4) };
}

#[test]
fn a_row_past_the_target_has_a_chunk_of_its_own() {
    let mut world = open(limits());
    let small = vec![b's'; usize::try_from(VALUE_BYTES).expect("small value fits")];
    let large_len = 3 * TARGET / 2;
    let large = vec![b'b'; usize::try_from(large_len).expect("large value fits")];
    let ops = Box::new([put(b"a", &small), put(b"b", &large), put(b"c", &small)]);
    world.request_no_fire(Request::Commit { owner: OWNER, ops });
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 1 });
    world.drive(true);
    close(&mut world);
    let files = world.files();
    let chunks = chunks(files.get(&b"snapshot"[..]).expect("the store made a snapshot"));
    assert_eq!(
        chunks,
        vec![
            (1, VALUE_BYTES + 21),
            (1, skein_kv::chunk_bytes(1, large_len).expect("large row fits")),
            (1, VALUE_BYTES + 21)
        ]
    );
    world.reopen();
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: 1 });
    for (key, value) in [(b"a", &small), (b"b", &large), (b"c", &small)] {
        read_row(&mut world, key, value);
    }
    close(&mut world);
}

#[test]
fn a_row_at_the_ceiling_round_trips() {
    let ceiling = 2 * TARGET;
    let value_len = ceiling - 28;
    let mut world = open(Limits { chunk: ceiling, value: value_len, commit: ceiling + 64, ..limits() });
    let key = 0_u64.to_be_bytes();
    let value = vec![b'v'; usize::try_from(value_len).expect("ceiling value fits")];
    world.request_no_fire(Request::Commit { owner: OWNER, ops: Box::new([put(&key, &value)]) });
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 1 });
    world.drive(true);
    close(&mut world);
    let files = world.files();
    assert_eq!(chunks(files.get(&b"snapshot"[..]).expect("the store made a snapshot")), vec![(1, ceiling)]);
    assert_eq!(skein_kv::chunk_bytes(8, value_len), Some(ceiling));
    world.reopen();
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: 1 });
    read_row(&mut world, &key, &value);
    close(&mut world);
}
