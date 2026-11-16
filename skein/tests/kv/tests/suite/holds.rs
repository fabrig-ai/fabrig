//! Files retained for a copy (skein/docs/design/data/kv.md, 2.3 and 9).

use super::logs::{self, OWNER};
use skein_io::file;
use skein_kv::{Event, HELD_REMOVALS, HeldSegment, Op, Refusal, Request, Store};
use skein_kv_tests::world::World;
use skein_lib::Token;
use skein_sim::Faults;

pub(super) fn put(n: u8) -> Op {
    Op::Put { key: Box::from(&b"key"[..]), value: Box::from([n]) }
}

fn hold(world: &mut World) -> (u64, bool, Box<[HeldSegment]>) {
    world.request_no_fire(Request::Hold { owner: OWNER });
    let Event::Held { last, snapshot, segments, owner } = world.pop() else { panic!("held") };
    assert_eq!(owner, OWNER);
    (last, snapshot, segments)
}

#[test]
fn a_hold_is_a_batch_barrier_and_names_only_the_committed_prefix() {
    let mut world = logs::world();
    for number in 1..=3 {
        logs::commit(&mut world, vec![put(1)], number);
    }
    world.file_requests.clear();
    world.start(Request::Commit { owner: OWNER, ops: Box::new([put(4)]) });
    world.start(Request::Commit { owner: OWNER, ops: Box::new([put(5)]) });
    world.start(Request::Hold { owner: OWNER });
    for request in [Request::Hold { owner: OWNER }, Request::Release { owner: OWNER }] {
        world.start(request);
        assert_eq!(world.pop(), Event::Refused { owner: OWNER, refusal: Refusal::Unavailable });
    }
    world.start(Request::Commit { owner: OWNER, ops: Box::new([put(6)]) });
    assert!(world.above.is_empty());
    world.drive(false);
    for number in 4..=5 {
        assert_eq!(world.pop(), Event::Committed { owner: OWNER, number });
    }
    // Each frame holds one put of a three-byte key and one-byte value.
    let end = 5 * (u64::from(skein_kv::FRAME_BYTES) + 1 + 4 + 3 + 4 + 1);
    assert_eq!(
        world.pop(),
        Event::Held {
            owner: OWNER,
            last: 5,
            snapshot: false,
            segments: Box::new([HeldSegment { first: 1, end: Some(end) }]),
        }
    );
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 6 });
    let bytes = world.file(b"log-00000000000000000001").expect("segment");
    assert_eq!(u64::try_from(bytes.len()).expect("length"), end / 5 * 6);
    let writes: Vec<_> = world.file_requests.iter().filter(|r| r.starts_with("WriteAt")).collect();
    assert_eq!(writes.len(), 3, "4, 5, and 6 are separate writes on opposite sides of the barrier");
    assert!(writes.last().expect("sixth commit").contains(&format!("offset: {end},")));
}

fn snapshot_world() -> World {
    let mut limits = logs::limits();
    limits.segment = 1;
    limits.snapshot_after = 32;
    let mut world = World::new(limits);
    world.logs(b"e/");
    logs::open(&mut world, 0);
    let payload = logs::append(&world, 7, b"retained");
    logs::commit(&mut world, vec![put(1), payload], 1);
    assert!(world.file(b"snapshot").is_some());
    for number in 2..=4 {
        world.request_no_fire(Request::Commit { owner: OWNER, ops: Box::new([put(2)]) });
        assert_eq!(world.pop(), Event::Committed { owner: OWNER, number });
    }
    world
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one replacement story checks both ways of ending a hold and their file ordering"
)]
fn snapshot_files_wait_under_their_names_until_release_or_close() {
    for close in [false, true] {
        let mut world = snapshot_world();
        let old = world.file(b"snapshot").expect("old snapshot");
        assert_eq!(
            hold(&mut world),
            (
                4,
                true,
                Box::from([
                    HeldSegment { first: 2, end: None },
                    HeldSegment { first: 3, end: None },
                    HeldSegment {
                        first: 4,
                        end: Some(
                            u64::try_from(world.file(b"log-00000000000000000004").expect("segment").len())
                                .expect("length")
                        )
                    },
                ])
            )
        );
        // No next snapshot starts during this hold.
        world.drive(true);
        assert!(world.file(b"snapshot.tmp").is_none());
        world.request_no_fire(Request::Release { owner: OWNER });
        assert_eq!(world.pop(), Event::Released { owner: OWNER });
        world.fire_once(); // Roll over to the snapshot's start, 5.
        assert_eq!(
            hold(&mut world),
            (
                4,
                true,
                Box::from([
                    HeldSegment { first: 2, end: None },
                    HeldSegment { first: 3, end: None },
                    HeldSegment { first: 4, end: None },
                ])
            )
        );
        world.file_requests.clear();
        world.drive(true);
        assert_eq!(world.file(b"snapshot"), Some(old.clone()));
        assert!(world.file(b"snapshot.tmp").is_some(), "finished temporary waits");
        assert_eq!(
            world.io.open_files(),
            2,
            "only the segment and cached payload stay open; the temporary is synced and closed"
        );
        assert!(!world.file_requests.iter().any(|r| r.starts_with("Rename")));
        for first in 2..=4 {
            assert!(world.file(format!("log-{first:020}").as_bytes()).is_some());
        }
        logs::commit(&mut world, vec![Op::Drop { log: 7 }], 5);
        assert_eq!(world.store.log_end(7), 0);
        assert!(world.file(logs::PATH).is_some(), "dropped payload remains beside held snapshot files");
        world.file_requests.clear();
        world.start(if close { Request::Close { owner: OWNER } } else { Request::Release { owner: OWNER } });
        if close {
            assert!(world.above.is_empty(), "close waits for held acts");
        } else {
            assert_eq!(world.pop(), Event::Released { owner: OWNER }, "release is immediate");
        }
        world.drive(true);
        if close {
            assert_eq!(world.pop(), Event::Closed { owner: OWNER });
            assert_eq!(world.io.open_files(), 0);
        }
        assert_ne!(world.file(b"snapshot"), Some(old));
        assert!(world.file(b"snapshot.tmp").is_none());
        for first in 2..=4 {
            assert!(world.file(format!("log-{first:020}").as_bytes()).is_none());
        }
        let requests = &world.file_requests;
        let rename = requests.iter().position(|r| r.starts_with("Rename")).expect("rename");
        let sync = requests
            .iter()
            .enumerate()
            .skip(rename + 1)
            .find(|(_, r)| r.starts_with("SyncDirectory"))
            .expect("directory sync")
            .0;
        let remove = requests.iter().position(|r| r.starts_with("Remove {")).expect("old segments removed");
        assert!(rename < sync && sync < remove);
        let payload_removal = requests.iter().position(|r| r.starts_with("RemoveIn")).expect("payload removed");
        let removals_before_payload =
            requests.iter().take(payload_removal).filter(|r| r.starts_with("Remove {")).count();
        assert_eq!(
            removals_before_payload, 3,
            "the snapshot removes segments 2, 3 and 4 before held payloads; a later snapshot may remove 5"
        );
        let last_segment_removal =
            requests.iter().take(payload_removal).rposition(|r| r.starts_with("Remove {")).expect("segments removed");
        assert!(
            requests
                .iter()
                .take(payload_removal)
                .skip(last_segment_removal + 1)
                .any(|r| r.starts_with("SyncDirectory")),
            "cleanup is synced before payload removal"
        );
        assert!(world.file(logs::PATH).is_none());
    }
}

#[test]
fn a_hold_asked_during_rename_names_the_new_snapshot() {
    let mut world = snapshot_world();
    for _ in 0..30 {
        if matches!(world.below.iter().next(), Some(file::Request::Rename { .. })) {
            break;
        }
        if world.ready() {
            skein_kv::fire(&mut world.store, &world.env, &mut world.above, &mut world.below);
        } else {
            assert!(world.step(false));
        }
    }
    assert!(matches!(world.below.iter().next(), Some(file::Request::Rename { .. })));
    world.start(Request::Hold { owner: OWNER });
    world.start(Request::Commit { owner: OWNER, ops: Box::new([put(5)]) });
    assert!(world.above.is_empty(), "the rename is still in flight");
    world.drive(true);
    assert_eq!(world.pop(), Event::Held { owner: OWNER, last: 4, snapshot: true, segments: Box::new([]) });
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 5 });
    assert_eq!(u64::from_be_bytes(world.file(b"snapshot").expect("snapshot")[6..14].try_into().expect("start")), 5);
}

#[test]
fn dropped_payloads_wait_in_a_bounded_list_and_excess_is_swept_on_open() {
    let mut limits = logs::limits();
    limits.budget = 2 * 1024 * 1024;
    limits.snapshot_after = u64::MAX;
    let mut world = World::new(limits);
    world.logs(b"e/");
    logs::open(&mut world, 0);
    let total = u64::try_from(HELD_REMOVALS).expect("bound") + 1;
    let mut number = 0;
    for first in (0..total).step_by(16) {
        let ops = (first..(first + 16).min(total)).map(|log| logs::append(&world, log << 8, b"data")).collect();
        number += 1;
        logs::commit(&mut world, ops, number);
    }
    hold(&mut world);
    world.file_requests.clear();
    for first in (0..total).step_by(16) {
        let ops = (first..(first + 16).min(total)).map(|log| Op::Drop { log: log << 8 }).collect();
        number += 1;
        logs::commit(&mut world, ops, number);
    }
    assert_eq!(world.store.log_end(7 << 8), 0, "extent is gone although the file stays");
    assert!(world.file(b"logs/0000000000000000/0000000000000700").is_some());
    assert!(!world.file_requests.iter().any(|r| r.starts_with("RemoveIn")));
    world.file_requests.clear();
    world.start(Request::Release { owner: OWNER });
    assert_eq!(world.pop(), Event::Released { owner: OWNER });
    for _ in 0..100_000 {
        if !world.step(true) {
            break;
        }
    }
    assert!(!world.step(true), "bounded removals settle");
    assert!(world.file(b"logs/0000000000000000/0000000000000700").is_none());
    assert!(world.file(b"logs/0000000000000100/0000000000100000").is_some(), "overflow is left on disk");
    assert_eq!(world.file_requests.iter().filter(|r| r.starts_with("RemoveIn")).count(), HELD_REMOVALS);
    // Each removal is followed by its directory's sync before the next removal.
    let mut needs_sync = false;
    for request in &world.file_requests {
        if request.starts_with("RemoveIn") {
            assert!(!needs_sync);
            needs_sync = true;
        }
        if request.starts_with("Sync {") {
            needs_sync = false;
        }
    }
    assert!(!needs_sync);
    world.request(Request::Close { owner: OWNER });
    assert_eq!(world.pop(), Event::Closed { owner: OWNER });
    world.reopen();
    assert_eq!(world.pop(), Event::Opened { owner: Token::new(1), last: number });
    assert!(world.file(b"logs/0000000000000100/0000000000100000").is_none());
}

#[test]
fn holds_are_unavailable_outside_the_open_life_and_failure_ends_them() {
    let mut world = World::new(logs::limits());
    for request in [Request::Hold { owner: OWNER }, Request::Release { owner: OWNER }] {
        world.request(request);
        assert_eq!(world.pop(), Event::Refused { owner: OWNER, refusal: Refusal::Unavailable });
    }
    world.start(Request::Open { owner: OWNER, root: world.root });
    world.start(Request::Hold { owner: OWNER });
    assert_eq!(world.pop(), Event::Refused { owner: OWNER, refusal: Refusal::Unavailable });
    world.drive(true);
    assert_eq!(world.pop(), Event::Opened { owner: OWNER, last: 0 });
    hold(&mut world);
    world.request(Request::Hold { owner: OWNER });
    assert_eq!(world.pop(), Event::Refused { owner: OWNER, refusal: Refusal::Unavailable });
    world.start(Request::Commit { owner: OWNER, ops: Box::new([put(1)]) });
    world.sim.set_faults(Faults { io_error: 1000, ..Faults::NONE });
    assert!(world.step(false));
    assert_eq!(world.pop(), Event::Failed { owner: OWNER, number: 1 });
    world.sim.set_faults(Faults::NONE);
    world.drive(true);
    assert_eq!(world.pop(), Event::Recovered { last: 0 });
    world.request(Request::Release { owner: OWNER });
    assert_eq!(world.pop(), Event::Refused { owner: OWNER, refusal: Refusal::Unavailable });
    hold(&mut world);
    world.start(Request::Close { owner: OWNER });
    world.start(Request::Hold { owner: OWNER });
    assert_eq!(world.pop(), Event::Refused { owner: OWNER, refusal: Refusal::Unavailable });
    world.drive(true);
    assert_eq!(world.pop(), Event::Closed { owner: OWNER });
}

#[test]
fn a_drop_already_opening_its_directory_is_deferred_by_a_hold() {
    let mut world = logs::world();
    let append = logs::append(&world, 7, b"retained");
    logs::commit(&mut world, vec![append], 1);
    world.start(Request::Commit { owner: OWNER, ops: Box::new([Op::Drop { log: 7 }]) });
    for _ in 0..100 {
        if matches!(world.below.iter().next(), Some(file::Request::OpenDirectory { name, .. }) if name.as_ref() == b"logs/0000000000000000")
        {
            break;
        }
        assert!(world.step(false));
    }
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number: 2 });
    assert!(matches!(world.below.iter().next(), Some(file::Request::OpenDirectory { .. })));
    world.start(Request::Hold { owner: OWNER });
    assert!(matches!(world.pop(), Event::Held { last: 2, .. }), "a hold needs no file request or completion");
    world.drive(true);
    assert!(world.file(logs::PATH).is_some(), "the directory open must not continue into removal");
    world.request(Request::Release { owner: OWNER });
    assert_eq!(world.pop(), Event::Released { owner: OWNER });
    assert!(world.file(logs::PATH).is_none());
}

#[test]
fn a_fatal_snapshot_failure_ends_the_hold_before_cleanup() {
    let mut world = snapshot_world();
    world.fire_once(); // Starting segment.
    world.fire_once(); // Temporary opened, header ready.
    hold(&mut world);
    skein_kv::fire(&mut world.store, &world.env, &mut world.above, &mut world.below);
    assert!(matches!(world.below.iter().next(), Some(file::Request::WriteAt { .. })));
    world.sim.set_faults(Faults { io_error: 1000, ..Faults::NONE });
    assert!(world.step(false));
    world.start(Request::Release { owner: OWNER });
    assert_eq!(world.pop(), Event::Refused { owner: OWNER, refusal: Refusal::Unavailable });
    assert!(matches!(world.below.iter().next(), Some(file::Request::Close { .. })), "cleanup is still pending");
    world.sim.set_faults(Faults::NONE);
    world.drive(true);
    assert_eq!(world.pop(), Event::Recovered { last: 4 });
    assert_eq!(hold(&mut world).0, 4);
}

#[test]
fn memory_holds_answer_at_once_without_files() {
    let mut world = World::new(logs::limits());
    world.store = Store::memory(&world.env.limits);
    logs::commit(&mut world, vec![put(1)], 1);
    world.start(Request::Hold { owner: OWNER });
    assert_eq!(world.pop(), Event::Held { owner: OWNER, last: 1, snapshot: false, segments: Box::new([]) });
    world.start(Request::Release { owner: OWNER });
    assert_eq!(world.pop(), Event::Released { owner: OWNER });
    assert!(world.below.is_empty());
}
