//! Payload durability, traversal and recovery (skein/docs/design/data/kv.md, 6 and 9).

use skein_io::file;
use skein_kv::{Direction, Event, Limits, LogRecord, Op, Page, Request};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Token};
use skein_sim::Faults;

pub(super) const OWNER: Token = Token::new(77);
pub(super) const PATH: &[u8] = b"logs/0000000000000000/0000000000000007";
pub(super) fn limits() -> Limits {
    Limits {
        key: 64,
        value: 64,
        ops: 16,
        commit: 2048,
        queued: 8,
        queued_bytes: 16384,
        budget: 8192,
        segment: 8192,
        snapshot_after: 16384,
        chunk: 256,
        page: Page { rows: 16, bytes: 1024 },
        record: 1024,
        log: 1 << 40,
        logs: 2,
        deadline: Duration::from_secs(1),
    }
}
pub(super) fn world() -> World {
    let mut world = World::new(limits());
    world.logs(b"e/");
    open(&mut world, 0);
    world
}
pub(super) fn open(world: &mut World, last: u64) {
    world.request(Request::Open { owner: OWNER, root: world.root });
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last });
}
pub(super) fn append(world: &World, log: u64, record: &[u8]) -> Op {
    Op::Append { log, frame: world.store.frame(record).expect("bounded record") }
}
pub(super) fn commit(world: &mut World, ops: Vec<Op>, number: u64) {
    world.request(Request::Commit { owner: OWNER, ops: ops.into_boxed_slice() });
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number });
}
pub(super) fn read(
    world: &mut World,
    log: u64,
    at: u64,
    direction: Direction,
    max: u32,
) -> (Box<[LogRecord]>, Option<u64>) {
    world.request(Request::ReadLog { owner: OWNER, log, at, direction, max });
    let Event::LogRead { records, next, .. } = world.pop() else { panic!("readable log") };
    (records, next)
}
pub(super) fn records(world: &mut World, log: u64) -> Vec<Vec<u8>> {
    let (records, next) = read(world, log, 0, Direction::Forward, 1024);
    assert_eq!(next, None);
    records.into_vec().into_iter().map(|record| record.record.expect("not cut").into_vec()).collect()
}
fn extent(world: &mut World, log: u64) -> u64 {
    let mut key = b"e/".to_vec();
    key.extend_from_slice(&log.to_be_bytes());
    world.request(Request::Get { owner: OWNER, key: key.into_boxed_slice() });
    let Event::Got { value, .. } = world.pop() else { panic!("extent row") };
    value.map_or(0, |value| u64::from_be_bytes(value.as_ref().try_into().expect("eight bytes")))
}

#[test]
fn two_logs_sync_before_the_commit_and_reads_keep_offsets_and_bounds() {
    let mut world = world();
    world.file_requests.clear();
    let a = append(&world, 7, b"a");
    let bb = append(&world, 7, b"bb");
    let c = append(&world, 9000, b"c");
    commit(&mut world, vec![c, a, bb], 1);
    assert_eq!(extent(&mut world, 7), 37);
    assert_eq!(extent(&mut world, 9000), 18);
    let requests = &world.file_requests;
    let first = requests
        .iter()
        .position(|r| r.starts_with("OpenWriteNoFollow") && r.contains(&format!("{PATH:?}")))
        .expect("first log open");
    let other_path = b"logs/0000000000000002/0000000000002328";
    let second = requests
        .iter()
        .position(|r| r.starts_with("OpenWriteNoFollow") && r.contains(&format!("{other_path:?}")))
        .expect("second log open");
    assert!(second < first, "logs follow their first append, not numeric or extent-key order");
    let writes: Vec<usize> =
        requests.iter().enumerate().filter_map(|(i, r)| r.starts_with("WriteAt").then_some(i)).collect();
    assert_eq!(writes.len(), 3, "two payload writes then one commit: {requests:?}");
    for &i in &writes {
        let request = &requests[i];
        let token = request.split("file: ").nth(1).expect("file").split(',').next().expect("token");
        let sync = requests
            .iter()
            .enumerate()
            .skip(i + 1)
            .find(|(_, r)| r.starts_with("Sync {") && r.contains(token))
            .expect("sync follows write")
            .0;
        let next_write = writes.iter().copied().find(|&next| next > i).unwrap_or(requests.len());
        assert!(sync < next_write, "file sync before next log or commit write");
    }
    let (forward, next) = read(&mut world, 7, 0, Direction::Forward, 3);
    assert_eq!(next, None);
    assert_eq!(
        forward.as_ref(),
        &[
            LogRecord { at: 0, end: 18, length: 1, record: Some(Box::from(&b"a"[..])) },
            LogRecord { at: 18, end: 37, length: 2, record: Some(Box::from(&b"bb"[..])) },
        ]
    );
    let (backward, next) = read(&mut world, 7, 37, Direction::Backward, 3);
    assert_eq!(next, None);
    assert_eq!(backward.as_ref(), forward.iter().rev().cloned().collect::<Vec<_>>());
    let (cut, next) = read(&mut world, 7, 37, Direction::Backward, 1);
    assert_eq!(cut.as_ref(), &[LogRecord { at: 18, end: 37, length: 2, record: None }]);
    assert_eq!(next, Some(18));
    let (page, next) = read(&mut world, 7, 0, Direction::Forward, 1);
    assert_eq!(page.len(), 1);
    assert_eq!(next, Some(18));
    assert!(read(&mut world, 88, 0, Direction::Forward, 10).0.is_empty());
    // An uncommitted suffix is invisible, even if it is a complete frame.
    let suffix = world.file(PATH).expect("log").get(..18).expect("first frame").to_vec();
    world.overwrite(PATH, 37, &suffix);
    assert_eq!(records(&mut world, 7), vec![b"a".to_vec(), b"bb".to_vec()]);
    world.overwrite(PATH, 27, b"X");
    for (at, direction) in [(18, Direction::Forward), (37, Direction::Backward)] {
        world.request(Request::ReadLog { owner: OWNER, log: 7, at, direction, max: 10 });
        assert_eq!(world.pop(), Event::LogUnread { owner: OWNER, at: 18 });
    }
    world.request(Request::Close { owner: OWNER });
    assert_eq!(world.pop(), Event::Closed { owner: OWNER });
    assert_eq!(world.io.open_files(), 0);
}

#[test]
fn synced_tail_without_a_commit_is_overwritten_after_recovery() {
    let mut world = world();
    let a = append(&world, 7, b"a");
    commit(&mut world, vec![a], 1);
    let bb = append(&world, 7, b"bb");
    world.start(Request::Commit { owner: OWNER, ops: Box::new([bb]) });
    // Stop at the commit write: all payload and directory syncs have completed.
    for _ in 0..1000 {
        if matches!(world.below.iter().next(), Some(file::Request::WriteAt { bytes, .. }) if bytes.starts_with(b"SKVC"))
        {
            break;
        }
        assert!(world.step(false));
    }
    assert!(world.above.is_empty());
    assert_eq!(world.file(PATH).expect("tail file").len(), 37);
    world.crash(17);
    open(&mut world, 1);
    assert_eq!(extent(&mut world, 7), 18);
    let c = append(&world, 7, b"c");
    commit(&mut world, vec![c], 2);
    assert_eq!(records(&mut world, 7), vec![b"a".to_vec(), b"c".to_vec()]);
    assert_eq!(world.file(PATH).expect("tail remains untruncated").len(), 37);
}

#[test]
fn recovery_removes_dropped_and_never_committed_files() {
    for orphan in [false, true] {
        let mut world = world();
        let a = append(&world, 7, b"a");
        if orphan {
            world.start(Request::Commit { owner: OWNER, ops: Box::new([a]) });
            for _ in 0..1000 {
                if matches!(world.below.iter().next(), Some(file::Request::WriteAt { bytes, .. }) if bytes.starts_with(b"SKVC"))
                {
                    break;
                }
                assert!(world.step(false));
            }
        } else {
            commit(&mut world, vec![a], 1);
            world.start(Request::Commit { owner: OWNER, ops: Box::new([Op::Drop { log: 7 }]) });
            for _ in 0..1000 {
                if !world.above.is_empty() {
                    break;
                }
                assert!(world.step(false));
            }
            assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 2 });
        }
        assert!(world.file(PATH).is_some());
        world.crash(29);
        open(&mut world, if orphan { 0 } else { 2 });
        assert!(world.file(PATH).is_none(), "orphan {orphan} removed at recovery");
        assert_eq!(records(&mut world, 7), Vec::<Vec<u8>>::new());
    }
    let mut limits = limits();
    limits.ops = 64;
    limits.commit = 4096;
    limits.budget = 16384;
    let mut world = World::new(limits);
    world.logs(b"e/");
    open(&mut world, 0);
    let names: Vec<u64> = (1..=33).map(|n| n << 12).chain(0..40).collect();
    // Cross both directory-page boundaries: logs within a shard and shards within logs/.
    for (i, group) in names.chunks(40).enumerate() {
        let ops = group.iter().map(|&log| append(&world, log, b"a")).collect();
        commit(&mut world, ops, u64::try_from(i + 1).expect("commit number"));
    }
    // Erase all extents with two queued commits, crash before their removals finish.
    for group in names.chunks(40) {
        world.start(Request::Commit {
            owner: OWNER,
            ops: group.iter().map(|&log| Op::Drop { log }).collect::<Vec<_>>().into_boxed_slice(),
        });
        for _ in 0..10000 {
            if !world.above.is_empty() {
                break;
            }
            assert!(world.step(false));
        }
        let Event::Committed { .. } = world.pop() else { panic!("drop committed") };
        // Complete this removal batch before queuing the next, except for the final one.
        if group.len() == 40 {
            world.drive(true);
        }
    }
    world.crash(90);
    open(&mut world, 4);
    for log in names {
        let path = format!("logs/{:016x}/{log:016x}", log >> 12);
        assert!(world.file(path.as_bytes()).is_none(), "removed across pages: {path}");
    }
}

#[test]
fn a_payload_sync_failure_fails_the_batch_and_recovers() {
    let mut world = world();
    let a = append(&world, 7, b"a");
    commit(&mut world, vec![a], 1);
    let bb = append(&world, 7, b"bb");
    world.start(Request::Commit { owner: OWNER, ops: Box::new([bb]) });
    let queued = append(&world, 7, b"queued");
    world.start(Request::Commit { owner: OWNER, ops: Box::new([queued]) });
    let mut payload_write = false;
    for _ in 0..1000 {
        if matches!(world.below.iter().next(), Some(file::Request::WriteAt { .. })) {
            payload_write = true;
        }
        if payload_write && matches!(world.below.iter().next(), Some(file::Request::Sync { .. })) {
            break;
        }
        assert!(world.step(false));
    }
    world.sim.set_faults(Faults { io_error: 1000, ..Faults::NONE });
    assert!(world.step(false));
    world.sim.set_faults(Faults::NONE);
    world.drive(true);
    assert_eq!(world.pop(), Event::Failed { owner: OWNER, number: 2 });
    assert_eq!(world.pop(), Event::Failed { owner: OWNER, number: 3 });
    assert_eq!(world.pop(), Event::Recovered { last: 1 });
    assert_eq!(extent(&mut world, 7), 18);
    assert_eq!(records(&mut world, 7), vec![b"a".to_vec()]);
    let c = append(&world, 7, b"c");
    commit(&mut world, vec![c], 2);
    assert_eq!(records(&mut world, 7), vec![b"a".to_vec(), b"c".to_vec()]);
}

#[test]
fn lru_closes_before_opening_another_payload_and_drop_then_append_starts_at_zero() {
    let mut world = world();
    for number in 1..=5 {
        let a = append(&world, number, b"a");
        commit(&mut world, vec![a], number);
        assert!(world.io.open_files() <= 3, "two payloads and the commit log");
    }
    for number in 1..=5 {
        assert_eq!(records(&mut world, number), vec![b"a".to_vec()]);
    }
    // Queue behind a first commit so the drop and subsequent append wait together.
    let first = append(&world, 7, b"old");
    world.start(Request::Commit { owner: OWNER, ops: Box::new([first]) });
    world.start(Request::Commit { owner: OWNER, ops: Box::new([Op::Drop { log: 7 }]) });
    let new = append(&world, 7, b"new");
    assert_eq!(world.store.log_end(7), 0);
    world.start(Request::Commit { owner: OWNER, ops: Box::new([new]) });
    world.drive(true);
    for number in 6..=8 {
        assert_eq!(world.pop(), Event::Committed { owner: OWNER, number });
    }
    assert_eq!(records(&mut world, 7), vec![b"new".to_vec()]);
    world.crash(34);
    open(&mut world, 8);
    assert_eq!(records(&mut world, 7), vec![b"new".to_vec()]);
}

#[test]
fn payload_golden_bytes_and_recovery() {
    const BODY: &[u8] = b"{\"model\":\"gpt-5-codex\",\"input\":[{\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"hello\"}]},{\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"hello again\"}]}],\"stream\":true}";
    const KEPT: &[u8] = include_bytes!("../../golden/payload-log");
    let mut world = World::new(Limits { snapshot_after: 1, ..limits() });
    world.logs(b"e/");
    open(&mut world, 0);
    let a = append(&world, 7, b"a");
    let deflated = append(&world, 7, BODY);
    commit(&mut world, vec![a, deflated], 1);
    assert_eq!(world.file(PATH).as_deref(), Some(KEPT), "payload log frame format (kv.md 6.1)");
    world.crash(98);
    open(&mut world, 1);
    assert_eq!(records(&mut world, 7), vec![b"a".to_vec(), BODY.to_vec()]);
}
