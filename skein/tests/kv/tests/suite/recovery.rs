//! Recovery holds a read and one frame or snapshot chunk (skein/docs/design/data/kv.md, 3, 7 and 9).

use skein_fake_machine::Item;
use skein_heap::Span;
use skein_kv::{Event, Failure, Limits, Op, Page, Request};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Token};

const CHUNK: u32 = 4096;
const COMMIT: u32 = 512;
const SNAPSHOT_ROWS: u64 = 2048;
const LOG_KEYS: u64 = 4;
const LOG_SIZES: [usize; 2] = [16 * 1024, 256 * 1024];
const SMALL_CHUNK: u32 = 128;
const CUT_ROWS: u64 = 4;
const VALUE_BYTES: usize = 64;
const OWNER: Token = Token::new(1);
const LOG_NAME: &[u8] = b"log-00000000000000000001";

// The world's transient heap through an empty root's open, including its directory listing.
// Measured peak minus net: 7704 bytes; a quarter of headroom, rounded up to two significant digits.
const RECOVERY_ALLOWANCE: i64 = 9700;

fn limits() -> Limits {
    Limits {
        key: 8,
        value: 64,
        ops: 4,
        commit: COMMIT,
        queued: 8,
        queued_bytes: 8192,
        budget: 1_048_576,
        segment: 1_048_576,
        snapshot_after: u64::MAX,
        chunk: CHUNK,
        page: Page { rows: 16, bytes: 1024 },
        deadline: Duration::from_secs(1),
    }
}

fn value(number: u64) -> Box<[u8]> {
    number.to_be_bytes().repeat(VALUE_BYTES / 8).into_boxed_slice()
}

fn put(number: u64) -> Op {
    Op::Put { key: Box::from(number.to_be_bytes()), value: value(number) }
}

pub(super) fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 0 { crc >> 1 } else { (crc >> 1) ^ 0x82f6_3b78 };
        }
    }
    !crc
}

fn frame(number: u64, key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut body = vec![1_u8];
    body.extend_from_slice(&u32::try_from(key.len()).expect("key length").to_be_bytes());
    body.extend_from_slice(key);
    body.extend_from_slice(&u32::try_from(value.len()).expect("value length").to_be_bytes());
    body.extend_from_slice(value);
    let mut bytes = b"SKVC".to_vec();
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&number.to_be_bytes());
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    bytes.extend_from_slice(&u32::try_from(body.len()).expect("body length").to_be_bytes());
    bytes.extend_from_slice(&body);
    bytes.extend_from_slice(&crc32c(&bytes).to_be_bytes());
    bytes
}

fn open(world: &mut World) {
    world.request_no_fire(Request::Open { owner: OWNER, root: world.root });
}

fn read_row(world: &mut World, key: u64, expected: u64) {
    world.request_no_fire(Request::Get { owner: OWNER, key: Box::from(key.to_be_bytes()) });
    assert_eq!(world.pop(), Event::Got { owner: OWNER, value: Some(value(expected)) }, "key {key}");
}

fn snapshot_world(rows: u64, chunk: u32) -> World {
    let limits = Limits { snapshot_after: 1, chunk, ..limits() };
    let mut world = World::untraced(limits, &[]);
    open(&mut world);
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: 0 });
    for start in (0..rows).step_by(4) {
        let ops = (start..start + 4).map(put).collect();
        world.request_no_fire(Request::Commit { owner: OWNER, ops });
        assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: start / 4 + 1 });
    }
    world.drive(true);
    world.request(Request::Close { owner: OWNER });
    assert_eq!(world.pop(), Event::Closed { owner: OWNER });
    world
}

#[test]
fn a_snapshot_removes_a_recreated_empty_tail_once() {
    let mut world = World::with_items(Limits { snapshot_after: 1, ..limits() }, &[Item::file(LOG_NAME, b"")]);
    open(&mut world);
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: 0 });
    world.request_no_fire(Request::Commit { owner: OWNER, ops: Box::new([put(0)]) });
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 1 });

    let mut removed = 0;
    loop {
        if let Some(skein_io::file::Request::Remove { name, .. }) = world.below.iter().next()
            && name.as_ref() == LOG_NAME
        {
            removed += 1;
        }
        if !world.step(true) {
            break;
        }
    }
    assert_eq!(removed, 1, "the snapshot removes the recreated segment once");
    assert!(world.above.is_empty(), "snapshot cleanup fails no request: {:?}", world.above);
    let files = world.files();
    assert!(files.contains_key(&b"snapshot"[..]));
    assert!(!files.contains_key(LOG_NAME));
    read_row(&mut world, 0, 0);
    world.request(Request::Close { owner: OWNER });
    assert_eq!(world.pop(), Event::Closed { owner: OWNER });
    assert_eq!(world.io.open_files(), 0);
}

#[test]
fn an_empty_root_opens_within_the_allowance() {
    let mut world = World::untraced(limits(), &[]);
    let span = Span::start();
    open(&mut world);
    let grown = span.end();
    let transient = grown.peak - grown.net;
    eprintln!(
        "empty open: peak {} net {}, transient {transient}, allowance {RECOVERY_ALLOWANCE}",
        grown.peak, grown.net
    );
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: 0 });
    assert!(transient <= RECOVERY_ALLOWANCE, "empty open: {transient} > {RECOVERY_ALLOWANCE}");
    assert!(RECOVERY_ALLOWANCE < i64::try_from(LOG_SIZES[1] / 4).expect("log length fits"));
    // Each snapshot row holds its 8-byte key and 64-byte value, before lengths and checksums.
    assert!(
        RECOVERY_ALLOWANCE
            < i64::try_from(SNAPSHOT_ROWS * (8 + u64::try_from(VALUE_BYTES).expect("value size fits")) / 4)
                .expect("snapshot size fits")
    );
}

#[test]
fn recovery_holds_a_read_and_a_frame_whatever_the_log() {
    let bound = 3 * i64::from(CHUNK) + i64::from(COMMIT) + RECOVERY_ALLOWANCE;
    for size in LOG_SIZES {
        let mut bytes = Vec::with_capacity(size);
        let mut last = 0;
        while bytes.len() < size {
            last += 1;
            bytes.extend_from_slice(&frame(last, &(last % LOG_KEYS).to_be_bytes(), &value(last)));
        }
        let mut world = World::untraced(limits(), &[Item::file(LOG_NAME, &bytes)]);
        let span = Span::start();
        open(&mut world);
        let grown = span.end();
        let transient = grown.peak - grown.net;
        eprintln!(
            "log {} bytes: peak {} net {}, transient {transient}, bound {bound}",
            bytes.len(),
            grown.peak,
            grown.net
        );
        assert_eq!(world.pop(), Event::Opened { owner: OWNER, last }, "log {} bytes", bytes.len());
        for key in 0..LOG_KEYS {
            let expected = last - (last + LOG_KEYS - key) % LOG_KEYS;
            read_row(&mut world, key, expected);
        }
        assert!(transient <= bound, "log {} bytes: {transient} > {bound}", bytes.len());
    }
}

#[test]
fn recovery_holds_a_read_and_a_chunk_whatever_the_snapshot() {
    let mut world = snapshot_world(SNAPSHOT_ROWS, CHUNK);
    let snapshot_bytes = world.files().get(&b"snapshot"[..]).expect("the store made a snapshot").len();
    assert!(RECOVERY_ALLOWANCE < i64::try_from(snapshot_bytes / 4).expect("snapshot size fits"));
    let span = Span::start();
    world.reopen();
    let grown = span.end();
    let transient = grown.peak - grown.net;
    let bound = 4 * i64::from(CHUNK) + RECOVERY_ALLOWANCE;
    eprintln!(
        "snapshot {snapshot_bytes} bytes: peak {} net {}, transient {transient}, bound {bound}",
        grown.peak, grown.net
    );
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: SNAPSHOT_ROWS / 4 });
    assert!(transient <= bound, "snapshot: {transient} > {bound}");
    for key in 0..SNAPSHOT_ROWS {
        read_row(&mut world, key, key);
    }
}

#[test]
fn a_snapshot_cut_short_fails_the_open() {
    let mut fixture = snapshot_world(CUT_ROWS, SMALL_CHUNK);
    let files = fixture.files();
    let snapshot = files.get(&b"snapshot"[..]).expect("the store made a snapshot");
    let limits = Limits { chunk: SMALL_CHUNK, ..limits() };
    for cut in 0..snapshot.len() {
        let mut world = World::with_items(limits, &[Item::file(b"snapshot", &snapshot[..cut])]);
        open(&mut world);
        assert_eq!(world.pop(), Event::OpenFailed { owner: OWNER, failure: Failure::Corrupt }, "cut {cut}");
        assert_eq!(world.io.open_files(), 0, "cut {cut}");
    }
    let mut world = World::with_items(limits, &[Item::file(b"snapshot", snapshot)]);
    open(&mut world);
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: CUT_ROWS / 4 });
    for key in 0..CUT_ROWS {
        read_row(&mut world, key, key);
    }
    eprintln!("snapshot cuts: {} rejected prefixes, whole file opened", snapshot.len());
}
