//! Focused step tests.

#![expect(clippy::disallowed_macros, reason = "tests use literal collections and variant assertions")]
#![expect(clippy::disallowed_types, reason = "tests enumerate a small fixed set of key tuples")]
#![expect(clippy::disallowed_methods, reason = "tests copy fixed byte vectors for comparisons")]

use crate::{Event, KeyReader, KeyWriter, Limits, Op, Page, Range, Request, Store, down, up};
use crate::{crc, frame};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use skein_io::file;
use skein_io::kernel::Fd;
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

fn limits() -> Limits {
    Limits {
        key: 100,
        value: 100,
        ops: 8,
        commit: 4096,
        queued: 8,
        queued_bytes: 32768,
        budget: 4096,
        segment: 8192,
        snapshot_after: 16384,
        chunk: 4096,
        page: Page { rows: 10, bytes: 4096 },
        deadline: Duration::from_secs(1),
    }
}

fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}

fn put(key: &[u8], value: &[u8]) -> Op {
    Op::Put { key: Box::from(key), value: Box::from(value) }
}

#[test]
fn crc_castagnoli_vector() {
    assert_eq!(crc::crc32c(b"123456789"), 0xe306_9283);
}

#[test]
fn keys_roundtrip_and_order() {
    let mut encoded = vec![];
    for tag in [0_u8, 1, 255] {
        for bytes in [&b""[..], &b"a"[..], &b"a\0"[..], &b"a\xff"[..], &b"b"[..]] {
            for number in [0_u64, 1, 256, u64::MAX] {
                let mut writer = KeyWriter::new(100);
                writer.tag(tag).bytes(bytes).u64(number);
                let key = writer.finish().expect("within limit");
                let mut reader = KeyReader::new(&key);
                assert_eq!(reader.tag(), Some(tag));
                assert_eq!(reader.bytes().as_deref(), Some(bytes));
                assert_eq!(reader.u64(), Some(number));
                assert!(reader.is_empty());
                encoded.push((tag, bytes.to_vec(), number, key));
            }
        }
    }
    for a in &encoded {
        for b in &encoded {
            assert_eq!(a.3.cmp(&b.3), (&a.0, &a.1, &a.2).cmp(&(&b.0, &b.1, &b.2)));
        }
    }
}

#[test]
fn frame_rejects_every_single_byte_corruption() {
    let bytes =
        frame::encode(7, &[put(b"a", b"one"), Op::Erase { key: Box::from(&b"b"[..]) }], 4096).expect("frame fits");
    let decoded = frame::decode(&bytes, 4096).expect("valid frame");
    assert_eq!(decoded.number, 7);
    assert_eq!(decoded.ops.len(), 2);
    for at in 0..bytes.len() {
        let mut bad = bytes.clone();
        *bad.get_mut(at).expect("byte in frame") ^= 1;
        assert!(frame::decode(&bad, 4096).is_none(), "corruption at {at}");
    }
}

#[test]
fn a_carried_frame_reserves_its_whole_length_once_its_header_arrives() {
    let bytes = frame::encode(1, &[put(b"a", b"value")], 4096).expect("frame fits");
    let header = usize::try_from(frame::HEADER).expect("header fits usize");
    let mut carried = Vec::with_capacity(header);
    carried.extend_from_slice(bytes.get(..header).expect("frame has a header"));
    frame::reserve_frame(&mut carried, 4096);
    assert!(carried.capacity() >= bytes.len());
    let capacity = carried.capacity();
    frame::reserve_frame(&mut carried, 4096);
    assert_eq!(carried.capacity(), capacity);

    let short = header.checked_sub(1).expect("header is nonempty");
    let mut carried = Vec::with_capacity(short);
    carried.extend_from_slice(bytes.get(..short).expect("frame has a header"));
    let capacity = carried.capacity();
    frame::reserve_frame(&mut carried, 4096);
    assert_eq!(carried.capacity(), capacity);
}

#[test]
fn memory_commits_reads_and_pages() {
    let env = env();
    let mut store = Store::memory(&env.limits);
    let mut above = Queue::with_capacity(20);
    let mut below = Queue::with_capacity(20);
    down(
        &mut store,
        &env,
        Request::Commit {
            owner: Token::new(1),
            ops: vec![put(b"a", b"1"), put(b"b", b"2"), put(b"c", b"3")].into_boxed_slice(),
        },
        &mut above,
        &mut below,
    );
    assert_eq!(above.pop(), Some(Event::Committed { owner: Token::new(1), number: 1 }));
    down(
        &mut store,
        &env,
        Request::Load { owner: Token::new(2), range: Range::prefix(b""), max: Page { rows: 2, bytes: 10 } },
        &mut above,
        &mut below,
    );
    let Event::Loaded { rows, next, .. } = above.pop().expect("page") else {
        panic!("expected page");
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(next.as_deref(), Some(&b"c"[..]));
    down(&mut store, &env, Request::Get { owner: Token::new(3), key: Box::from(&b"b"[..]) }, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Got { owner: Token::new(3), value: Some(Box::from(&b"2"[..])) }));
}

#[test]
fn a_page_too_small_for_one_row_is_refused() {
    let env = env();
    let mut store = Store::memory(&env.limits);
    let mut above = Queue::with_capacity(8);
    let mut below = Queue::with_capacity(8);
    down(
        &mut store,
        &env,
        Request::Commit { owner: Token::new(1), ops: vec![put(b"a", b"value")].into_boxed_slice() },
        &mut above,
        &mut below,
    );
    drop(above.pop());
    down(
        &mut store,
        &env,
        Request::Load { owner: Token::new(2), range: Range::prefix(b""), max: Page { rows: 1, bytes: 1 } },
        &mut above,
        &mut below,
    );
    assert_eq!(above.pop(), Some(Event::Refused { owner: Token::new(2), refusal: crate::Refusal::TooLarge }));
    down(
        &mut store,
        &env,
        Request::Load {
            owner: Token::new(3),
            range: Range { start: Box::from(&b"z"[..]), end: Some(Box::from(&b"a"[..])) },
            max: Page { rows: 1, bytes: 10 },
        },
        &mut above,
        &mut below,
    );
    assert_eq!(above.pop(), Some(Event::Refused { owner: Token::new(3), refusal: crate::Refusal::TooLarge }));
}

fn open_store(env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Queue<file::Request>) -> Store {
    let mut store = Store::new(&env.limits);
    down(&mut store, env, Request::Open { owner: Token::new(1), root: Fd::new(5) }, above, below);
    let Some(file::Request::List { .. }) = below.pop() else {
        panic!("list requested");
    };
    up(&mut store, env, file::Event::Listed { owner: Token::new(0x4b56), entries: Box::new([]) }, above, below);
    let Some(file::Request::Create { .. }) = below.pop() else {
        panic!("segment create requested");
    };
    up(&mut store, env, file::Event::Opened { owner: Token::new(0x4b56), file: Token::new(9), len: 0 }, above, below);
    let Some(file::Request::SyncDirectory { .. }) = below.pop() else {
        panic!("directory sync requested");
    };
    up(&mut store, env, file::Event::Synced { owner: Token::new(0x4b56) }, above, below);
    assert_eq!(above.pop(), Some(Event::Opened { owner: Token::new(1), last: 0 }));
    assert!(above.is_empty());
    assert!(below.is_empty());
    store
}

#[test]
fn put_and_erase_churn_on_distinct_keys_keeps_the_projection_bounded() {
    const ROUNDS: u32 = 1_024;
    let mut env = env();
    env.limits.segment = u64::from(ROUNDS) * u64::from(env.limits.commit);
    let bound = env.limits.queued * env.limits.ops;
    let mut above = Queue::with_capacity(20);
    let mut below = Queue::with_capacity(20);
    let mut store = open_store(&env, &mut above, &mut below);
    let mut committed = 0;
    let mut in_flight_last = 1;
    let mut syncs = 0;

    // Each round queues a put and erase on a fresh key before answering one request.
    // Every sync therefore has a successor batch waiting behind it.
    for round in 0..ROUNDS {
        let number = u64::from(round) + 1;
        let key = round.to_be_bytes();
        down(
            &mut store,
            &env,
            Request::Commit {
                owner: Token::new(number),
                ops: vec![put(&key, b"value"), Op::Erase { key: Box::from(key.as_slice()) }].into_boxed_slice(),
            },
            &mut above,
            &mut below,
        );
        assert!(above.is_empty(), "commit {number} waits for sync");
        assert_eq!(below.len(), 1, "one request is outstanding");
        if round % 2 == 0 {
            let Some(file::Request::WriteAt { owner, .. }) = below.pop() else {
                panic!("frame write requested");
            };
            up(&mut store, &env, file::Event::Written { owner }, &mut above, &mut below);
            assert!(above.is_empty(), "writing does not commit");
            let Some(file::Request::Sync { .. }) = below.iter().next() else {
                panic!("log sync requested");
            };
        } else {
            let Some(file::Request::Sync { owner, .. }) = below.pop() else {
                panic!("log sync requested");
            };
            assert!(in_flight_last < number, "a commit is queued at every sync");
            up(&mut store, &env, file::Event::Synced { owner }, &mut above, &mut below);
            for expected in committed + 1..=in_flight_last {
                assert_eq!(above.pop(), Some(Event::Committed { owner: Token::new(expected), number: expected }));
            }
            assert!(above.is_empty(), "only the synced batch commits");
            committed = in_flight_last;
            in_flight_last = number;
            syncs += 1;
            let Some(file::Request::WriteAt { .. }) = below.iter().next() else {
                panic!("next batch starts");
            };
        }
        assert_eq!(below.len(), 1);
        let keys = store.projected_keys();
        assert!(keys <= bound, "round {round}: {keys} projected keys exceed {bound}");
    }
    assert_eq!(syncs, ROUNDS / 2);
    assert_eq!(committed, u64::from(ROUNDS) - 2);
}

#[test]
fn map_budget_refusal_precedes_the_full_byte_queue() {
    // One row's entry charge, one-byte key and one-byte value.
    const BUDGET: u64 = crate::map::ENTRY + 2;
    // Frame header, put's lengths and tag, key and value.
    const FRAME: u32 = crate::FRAME_BYTES + 9 + 1 + 1;
    let mut env = env();
    env.limits.budget = BUDGET;
    env.limits.queued_bytes = u64::from(FRAME);
    let mut above = Queue::with_capacity(20);
    let mut below = Queue::with_capacity(20);
    let mut store = open_store(&env, &mut above, &mut below);
    for owner in [Token::new(2), Token::new(3)] {
        down(&mut store, &env, Request::Commit { owner, ops: Box::new([put(b"k", b"v")]) }, &mut above, &mut below);
    }
    assert!(above.is_empty(), "one frame is writing and one fills the byte queue");
    for (op, refusal) in [(put(b"x", b"v"), crate::Refusal::Full), (put(b"k", b"v"), crate::Refusal::Busy)] {
        let owner = Token::new(4);
        down(&mut store, &env, Request::Commit { owner, ops: Box::new([op]) }, &mut above, &mut below);
        assert_eq!(above.pop(), Some(Event::Refused { owner, refusal }));
        assert!(above.is_empty());
    }
}

#[test]
fn commit_is_invisible_until_sync() {
    let env = env();
    let mut above = Queue::with_capacity(20);
    let mut below = Queue::with_capacity(20);
    let mut store = open_store(&env, &mut above, &mut below);
    down(
        &mut store,
        &env,
        Request::Commit { owner: Token::new(2), ops: vec![put(b"k", b"v")].into_boxed_slice() },
        &mut above,
        &mut below,
    );
    let Some(file::Request::WriteAt { .. }) = below.pop() else {
        panic!("frame write requested");
    };
    down(&mut store, &env, Request::Get { owner: Token::new(3), key: Box::from(&b"k"[..]) }, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Got { owner: Token::new(3), value: None }));
    up(&mut store, &env, file::Event::Written { owner: Token::new(0x4b56) }, &mut above, &mut below);
    let Some(file::Request::Sync { .. }) = below.pop() else {
        panic!("log sync requested");
    };
    up(&mut store, &env, file::Event::Synced { owner: Token::new(0x4b56) }, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Committed { owner: Token::new(2), number: 1 }));
    down(&mut store, &env, Request::Get { owner: Token::new(4), key: Box::from(&b"k"[..]) }, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Got { owner: Token::new(4), value: Some(Box::from(&b"v"[..])) }));
}

#[test]
fn exported_sizes_match_encoded_frames_and_single_row_chunks() {
    assert_eq!(frame::encoded_len(&[]), Some(usize::try_from(crate::FRAME_BYTES).expect("frame overhead fits")));
    for (key, value) in [(&b""[..], &b""[..]), (&b"a"[..], &b"one"[..]), (&b"longer key"[..], &b"value"[..])] {
        let key_len = u32::try_from(key.len()).expect("small key");
        let value_len = u32::try_from(value.len()).expect("small value");
        let put = put(key, value);
        let erase = Op::Erase { key: Box::from(key) };
        for (op, size) in [(put, crate::put_bytes(key_len, value_len)), (erase, crate::erase_bytes(key_len))] {
            let total = crate::FRAME_BYTES.checked_add(size.expect("small op")).expect("small frame");
            let total = usize::try_from(total).expect("frame fits usize");
            assert_eq!(frame::encoded_len(core::slice::from_ref(&op)), Some(total));
            assert_eq!(frame::encode(1, &[op], u32::MAX).expect("frame fits").len(), total);
        }
        let size = crate::chunk_bytes(key_len, value_len).expect("small chunk");
        let rows = [crate::Row { key: Box::from(key), value: Box::from(value) }];
        assert_eq!(crate::snapshot::chunk(&rows, size).expect("exact fit").len(), usize::try_from(size).expect("fits"));
        assert_eq!(crate::snapshot::chunk(&rows, size.checked_sub(1).expect("chunk overhead")), None);
    }
}

#[test]
fn exported_sizes_refuse_overflow_and_accept_the_largest_size() {
    assert_eq!(crate::FRAME_BYTES, 26);
    assert_eq!(crate::put_bytes(0, 0), Some(9));
    assert_eq!(crate::erase_bytes(0), Some(5));
    assert_eq!(crate::chunk_bytes(0, 0), Some(20));
    for (key, value) in [(u32::MAX, 0), (0, u32::MAX), (u32::MAX, u32::MAX)] {
        assert_eq!(crate::put_bytes(key, value), None);
        assert_eq!(crate::chunk_bytes(key, value), None);
    }
    assert_eq!(crate::erase_bytes(u32::MAX), None);
    assert_eq!(crate::put_bytes(u32::MAX - 9, 0), Some(u32::MAX));
    assert_eq!(crate::put_bytes(0, u32::MAX - 9), Some(u32::MAX));
    assert_eq!(crate::erase_bytes(u32::MAX - 5), Some(u32::MAX));
    assert_eq!(crate::chunk_bytes(u32::MAX - 20, 0), Some(u32::MAX));
    assert_eq!(crate::chunk_bytes(0, u32::MAX - 20), Some(u32::MAX));
    assert_eq!(crate::put_bytes(u32::MAX - 8, 0), None);
    assert_eq!(crate::erase_bytes(u32::MAX - 4), None);
    assert_eq!(crate::chunk_bytes(u32::MAX - 19, 0), None);
}

#[test]
fn snapshot_chunks_share_framing_across_rows() {
    let rows = [
        crate::Row { key: Box::from(&b"a"[..]), value: Box::from(&b"one"[..]) },
        crate::Row { key: Box::from(&b"b"[..]), value: Box::from(&b"two"[..]) },
    ];
    assert_eq!(crate::snapshot::chunk(&rows, 36).expect("two rows share twelve framing bytes").len(), 36);
    assert_eq!(crate::snapshot::chunk(&rows, 35), None);
}

#[test]
fn snapshot_bound_follows_the_live_map_between_floor_and_ceiling() {
    let mut limits = limits();
    limits.snapshot_after = 1_000_000;
    assert_eq!(crate::SNAPSHOT_FLOOR, 65_536);
    assert_eq!(crate::SNAPSHOT_MULTIPLE, 2);
    assert_eq!(limits.snapshot_bound(0), crate::SNAPSHOT_FLOOR);
    assert_eq!(limits.snapshot_bound(40_000), 80_000);
    assert_eq!(limits.snapshot_bound(u64::MAX), limits.snapshot_after);
    limits.snapshot_after = u64::MAX;
    assert_eq!(limits.snapshot_bound(u64::MAX), u64::MAX);
    limits.snapshot_after = 1024;
    assert_eq!(limits.snapshot_bound(0), 1024);
    assert_eq!(limits.snapshot_bound(40_000), 1024);
}

#[test]
fn the_map_keeps_its_encoded_size_through_puts_overwrites_and_erases() {
    let limits = limits();
    let mut map = crate::map::KvMap::new(&limits);
    let mut model = alloc::collections::BTreeMap::new();
    assert_eq!(map.live(), 0);
    for ops in [
        vec![put(b"a", b"one"), put(b"bb", b"two")],
        vec![put(b"a", b"a longer value")],
        vec![put(b"bb", b"x")],
        vec![Op::Erase { key: Box::from(&b"a"[..]) }],
        vec![put(b"c", b"temporary"), Op::Erase { key: Box::from(&b"c"[..]) }],
        vec![put(b"bb", b"temporary"), Op::Erase { key: Box::from(&b"bb"[..]) }],
    ] {
        let used = map.check(&ops, &limits).expect("the batch fits");
        map.apply(&ops, used, None, true);
        for op in ops {
            match op {
                Op::Put { key, value } => {
                    model.insert(key, value);
                }
                Op::Erase { key } => {
                    model.remove(&key);
                }
            }
        }
        let expected: u64 = model
            .iter()
            .map(|(key, value)| {
                8_u64
                    .checked_add(skein_lib::len_u64(key.len()))
                    .and_then(|size| size.checked_add(skein_lib::len_u64(value.len())))
                    .expect("small row")
            })
            .sum();
        assert_eq!(map.live(), expected);
        assert_eq!(usize::try_from(map.len()).expect("small map"), model.len());
        for (key, value) in &model {
            assert_eq!(map.get(key), Some(value.as_ref()));
        }
    }
}

fn admit(
    queued: &mut crate::map::Queued,
    map: &crate::map::KvMap,
    ops: &[Op],
    limits: &Limits,
    number: u64,
) -> Result<(), crate::Refusal> {
    let projected = queued.check(map, ops, limits)?;
    queued.reserve(ops, projected, number);
    Ok(())
}

#[test]
fn queued_admission_matches_concatenated_ops_after_refusals() {
    use crate::map::{ENTRY, KvMap, Queued};
    use skein_lib::Rng;

    const SEED: u64 = 0x8b7c_1432_6ead_095f;
    const COMMITS: u32 = 512;
    const KEYS: u64 = 8;
    const VALUE: u64 = 8;
    // Room for three maximum-size rows, out of the eight possible keys.
    const BUDGET: u64 = 3 * (ENTRY + 1 + VALUE);
    // Apply the projection while more commits can arrive, then periodically drain it.
    const APPLY_EVERY: u32 = 7;
    const DRAIN_EVERY: u32 = 32;

    let mut limits = limits();
    limits.key = 1;
    limits.value = u32::try_from(VALUE).expect("small value limit");
    limits.budget = BUDGET;
    let mut rng = Rng::new(SEED);
    let mut map = KvMap::new(&limits);
    let initial = [put(&[0], b"seed"), put(&[1], b"map")];
    let used = map.check(&initial, &limits).expect("initial rows fit");
    map.apply(&initial, used, None, true);
    let mut admitted = Queued::new();
    let mut accepted = vec![];
    let mut full = 0_u32;
    let mut too_large = 0_u32;
    let mut successes = 0_u32;

    for step in 0..COMMITS {
        let number = u64::from(step);
        let mut ops = vec![];
        for _ in 0..rng.between(0, u64::from(limits.ops)) {
            let byte = u8::try_from(rng.below(KEYS)).expect("small key set");
            let key = if rng.chance(50) { vec![byte, byte] } else { vec![byte] };
            let op = if rng.chance(350) {
                Op::Erase { key: key.into_boxed_slice() }
            } else {
                let len = usize::try_from(rng.between(0, VALUE + 1)).expect("small value");
                put(&key, &vec![byte; len])
            };
            ops.push(op);
        }
        let mut concatenated = accepted.clone();
        concatenated.extend_from_slice(&ops);
        let expected = map.check(&concatenated, &limits).map(|_| ());
        assert_eq!(admit(&mut admitted, &map, &ops, &limits, number), expected, "seed {SEED:#x}, commit {step}");
        match expected {
            Ok(()) => {
                successes += 1;
                accepted = concatenated;
            }
            Err(refusal) => {
                match refusal {
                    crate::Refusal::Full => full += 1,
                    crate::Refusal::TooLarge => too_large += 1,
                    crate::Refusal::Busy | crate::Refusal::Unavailable => panic!("map admission refusal"),
                }
                // Rebuild the state before the refusal and compare the next admission.
                let mut unchanged = Queued::new();
                admit(&mut unchanged, &map, &accepted, &limits, number).expect("accepted projection fits");
                let key = [u8::try_from(rng.below(KEYS)).expect("small key set")];
                let probe = [put(&key, b"next")];
                let mut concatenated = accepted.clone();
                concatenated.extend_from_slice(&probe);
                let expected = map.check(&concatenated, &limits).map(|_| ());
                assert_eq!(admit(&mut unchanged, &map, &probe, &limits, number), expected, "reference probe at {step}");
                assert_eq!(
                    admit(&mut admitted, &map, &probe, &limits, number),
                    expected,
                    "refusal changed next answer at {step}"
                );
                if expected.is_ok() {
                    accepted = concatenated;
                }
            }
        }
        if step % APPLY_EVERY == 0 || step % DRAIN_EVERY == 0 {
            let used = map.check(&accepted, &limits).expect("admitted ops fit");
            map.apply(&accepted, used, None, true);
            admitted.applied(&accepted, number);
        }
        if step % DRAIN_EVERY == 0 {
            admitted.clear();
            accepted.clear();
        }
    }
    assert!(successes > 0, "seed exercises acceptance");
    assert!(full > 0, "seed exercises the map budget");
    assert!(too_large > 0, "seed exercises key and value limits");
}

#[test]
fn applying_an_earlier_commit_keeps_a_later_charge_on_the_same_key() {
    use crate::map::{KvMap, Queued};

    let limits = limits();
    let mut map = KvMap::new(&limits);
    let mut admitted = Queued::new();
    let first = [put(b"key", b"short")];
    let second = [put(b"key", b"a longer value")];
    let third = [put(b"other", b"third")];
    admit(&mut admitted, &map, &first, &limits, 1).expect("first commit fits");
    admit(&mut admitted, &map, &second, &limits, 2).expect("second commit fits");

    let used = map.check(&first, &limits).expect("first commit fits");
    map.apply(&first, used, None, true);
    admitted.applied(&first, 1);
    let mut concatenated = second.to_vec();
    concatenated.extend_from_slice(&third);
    assert_eq!(
        admitted.check(&map, &third, &limits),
        Queued::new().check(&map, &concatenated, &limits),
        "the later commit's charge survives the earlier commit applying",
    );
}

#[test]
fn chunk_target_is_the_target_under_a_higher_ceiling() {
    for (chunk, target) in
        [(4096, 4096), (crate::CHUNK_TARGET, crate::CHUNK_TARGET), (crate::CHUNK_TARGET + 1, crate::CHUNK_TARGET)]
    {
        assert_eq!(Limits { chunk, ..limits() }.chunk_target(), target);
    }
}
