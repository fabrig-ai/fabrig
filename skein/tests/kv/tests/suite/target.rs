//! Snapshot chunks pack to a target, with heap for their rows and one encoding
//! (skein/docs/design/data/kv.md, 5, 7 and 9).

use skein_fake_machine::Item;
use skein_heap::Span;
use skein_kv::{Event, Failure, Limits, Op, Page, Request};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Token};

const TARGET: u32 = skein_kv::CHUNK_TARGET;
const CEILING: u32 = 16 * TARGET;
const OWNER: Token = Token::new(1);
// An eight-byte key, a kilobyte value and eight bytes of row lengths.
const VALUE_BYTES: u32 = 1024;
const ROW_BYTES: u32 = 1040;
const BATCH: u64 = 64;
// About 2.5 MiB: two full target chunks and a third, and rows and one whole
// encoding of them pass every bound these tests hold.
const ROWS: u64 = 2500;
const _: () = assert!(2 * ROWS * ROW_BYTES as u64 > (4 * TARGET + TARGET / 2) as u64);
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

fn large_row() -> (World, Vec<u8>, Vec<u8>) {
    let mut world = open(limits());
    let small = vec![b's'; usize::try_from(VALUE_BYTES).expect("small value fits")];
    let large = vec![b'b'; usize::try_from(3 * TARGET / 2).expect("large value fits")];
    let ops = Box::new([put(b"a", &small), put(b"b", &large), put(b"c", &small)]);
    world.request_no_fire(Request::Commit { owner: OWNER, ops });
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 1 });
    (world, small, large)
}

#[test]
fn a_row_past_the_target_has_a_chunk_of_its_own() {
    let (mut world, small, large) = large_row();
    let large_len = u32::try_from(large.len()).expect("large value fits");
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

/// Reopen with every recovery read and the whole world's transient heap measured.
fn recover(world: &mut World, last: u64, bound: u32) -> i64 {
    let files = world.files();
    let file_bytes: u64 = files
        .iter()
        .filter(|(name, _)| name.as_ref() == b"snapshot" || name.starts_with(b"log-"))
        .map(|(_, bytes)| u64::try_from(bytes.len()).expect("file size fits"))
        .sum();
    world.asked_reads = 0;
    world.asked_bytes = 0;
    world.asked_most = 0;
    let span = Span::start();
    world.reopen();
    let grown = span.end();
    let transient = grown.peak - grown.net;
    eprintln!(
        "recovery: peak {} net {}, transient {transient}, bound {bound}, asked_most {}, asked_reads {}, asked_bytes {}",
        grown.peak, grown.net, world.asked_most, world.asked_reads, world.asked_bytes
    );
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last });
    assert!(world.asked_most <= TARGET, "read asks {} > {TARGET}", world.asked_most);
    assert_eq!(world.asked_bytes, file_bytes, "reads total the snapshot and log bytes");
    assert!(transient <= i64::from(bound), "recovery transient {transient} > {bound}");
    transient
}

#[test]
fn recovery_stays_near_the_target_under_a_large_chunk() {
    let mut world = small_rows(ROWS);
    world.drive(true);
    close(&mut world);
    let bound = 4 * TARGET + TARGET / 2;
    let transient = recover(&mut world, ROWS.div_ceil(BATCH), bound);
    assert!(transient < i64::from(CEILING / 2));
    let value = vec![b'v'; usize::try_from(VALUE_BYTES).expect("value fits")];
    for key in (0..ROWS).step_by(64) {
        read_row(&mut world, &key.to_be_bytes(), &value);
    }
    close(&mut world);
}

#[test]
fn a_row_past_the_target_recovers_within_its_chunk() {
    let (mut world, small, large) = large_row();
    world.drive(true);
    close(&mut world);
    let chunk = skein_kv::chunk_bytes(1, u32::try_from(large.len()).expect("large value fits")).expect("chunk fits");
    recover(&mut world, 1, 2 * TARGET + 2 * chunk + TARGET / 2);
    for (key, value) in [(b"a", &small), (b"b", &large), (b"c", &small)] {
        read_row(&mut world, key, value);
    }
    close(&mut world);
}

#[test]
fn a_frame_longer_than_a_read_replays() {
    let mut world = open(Limits { snapshot_after: u64::MAX, ..limits() });
    let value_len = 3 * TARGET / 2;
    let value = vec![b'b'; usize::try_from(value_len).expect("value fits")];
    world.request_no_fire(Request::Commit { owner: OWNER, ops: Box::new([put(b"b", &value)]) });
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 1 });
    close(&mut world);
    let files = world.files();
    assert!(!files.contains_key(&b"snapshot"[..]));
    assert!(files.keys().any(|name| name.starts_with(b"log-")));
    let frame = skein_kv::FRAME_BYTES + skein_kv::put_bytes(1, value_len).expect("put fits");
    recover(&mut world, 1, 3 * TARGET + 2 * frame + TARGET / 2);
    read_row(&mut world, b"b", &value);
    close(&mut world);
}

/// A ceiling-packed snapshot: two 640 KiB rows together exceed the read target.
fn packed_snapshot(value: &[u8]) -> Vec<u8> {
    let mut snapshot = b"SKVS".to_vec();
    snapshot.extend_from_slice(&1_u16.to_be_bytes());
    snapshot.extend_from_slice(&1_u64.to_be_bytes());
    snapshot.extend_from_slice(&super::recovery::crc32c(&snapshot).to_be_bytes());
    let mut chunk = 2_u32.to_be_bytes().to_vec();
    let value_len = u32::try_from(value.len()).expect("value fits");
    chunk.extend_from_slice(&(2 * (8 + 1 + value_len)).to_be_bytes());
    for key in *b"ab" {
        chunk.extend_from_slice(&1_u32.to_be_bytes());
        chunk.extend_from_slice(&value_len.to_be_bytes());
        chunk.push(key);
        chunk.extend_from_slice(value);
    }
    chunk.extend_from_slice(&super::recovery::crc32c(&chunk).to_be_bytes());
    assert!(chunk.len() > usize::try_from(TARGET).expect("target fits"));
    assert!(chunk.len() <= usize::try_from(CEILING).expect("ceiling fits"));
    snapshot.extend_from_slice(&chunk);
    let mut trailer = b"SNPE".to_vec();
    trailer.extend_from_slice(&2_u64.to_be_bytes());
    trailer.extend_from_slice(&(2 * (1 + u64::from(value_len))).to_be_bytes());
    trailer.extend_from_slice(&super::recovery::crc32c(&trailer).to_be_bytes());
    trailer.extend_from_slice(b"END!");
    snapshot.extend_from_slice(&trailer);
    snapshot
}

#[test]
fn a_chunk_is_read_up_to_the_ceiling() {
    let value = vec![b'v'; 640 * 1024];
    let snapshot = packed_snapshot(&value);
    let mut world = World::untraced(limits(), &[Item::file(b"snapshot", &snapshot)]);
    recover(&mut world, 0, 2 * TARGET + 2 * (12 + 2 * (9 + 640 * 1024)) + TARGET / 2);
    for key in [b"a", b"b"] {
        read_row(&mut world, key, &value);
    }
    close(&mut world);

    let mut world = World::untraced(
        Limits { chunk: TARGET, value: TARGET - 28, ..limits() },
        &[Item::file(b"snapshot", &snapshot)],
    );
    world.request_no_fire(Request::Open { owner: OWNER, root: world.root });
    assert_eq!(world.pop(), Event::OpenFailed { owner: OWNER, failure: Failure::Corrupt });
    assert_eq!(world.io.open_files(), 0);
}
