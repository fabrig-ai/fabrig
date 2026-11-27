//! Parts stay opaque between turns until a committed turn makes them moot
//! (jig/docs/design/domain/engine/executors-transcripts.md, 1).
use crate::tests::*;

const TASK: u64 = 1;
const OTHER_TASK: u64 = 2;

fn claim(core: &mut Core, task: u64, attempt: u64) {
    core.proofs.insert(task, RunProof::claimed(task, attempt, 1, None, fleet::HostKind::Worker)).unwrap();
}

fn part(attempt: u64, number: u32, turn: u32, thread: u32, call: u32, body: &[u8]) -> PartRecord {
    PartRecord {
        key: crate::PartKey { task: TASK, attempt, number },
        turn,
        thread,
        call,
        kind: jig_domain_link::PartKind::Answer,
        body: body.into(),
    }
}

#[test]
fn a_part_is_written_before_acknowledgement_and_a_copy_only_acknowledged() {
    const ROOM: u32 = 1; // The original part fills the room before its copy arrives.
    let mut limits = limits();
    limits.call_records = ROOM;
    let mut core = new_core(config(40), &limits).unwrap();
    claim(&mut core, TASK, 2);
    let row = part(2, 1, 1, 3, 4, b"opaque");
    kept(core.keep_part(row.clone()), &row);
    let mut out = core.keep_part(row.clone());
    ack(out.pop(), row.key);
    assert!(out.is_empty(), "a copy writes nothing even at capacity");
    assert_eq!(core.kept_parts.len(), ROOM);
    assert_eq!(core.kept_parts.get(&row.key), Some(&row));
    assert_eq!(core.transcript_bytes(), 6, "kept opaque bodies hold transcript room");
    let skipped = part(2, 3, 1, 0, 0, b"gap");
    let mut out = core.keep_part(skipped.clone());
    let Some(Request::Now(now)) = out.pop() else { panic!("a gap retries before advancing the prefix") };
    let Now::PartBusy { key } = *now else { panic!("part gap is busy") };
    assert_eq!(key, skipped.key);
    assert!(!core.kept_parts.contains_key(&skipped.key));
    core.retire_parts(TASK, None, &mut Queue::uncapped());
    ack(core.keep_part(row.clone()).pop(), row.key);
    assert!(core.kept_parts.is_empty(), "a retired copy cannot be resurrected");
    assert_eq!(core.last_part(TASK, 2), 1, "retiring bytes keeps the durable prefix");
}

#[test]
fn committing_a_turn_retires_earlier_attempts_and_its_prefix_but_not_later_parts() {
    use crate::routing::run::turns::route_accepted_turn;

    const ROOM: u32 = 3; // One old part, one at the commit, and one after it fill the room.
    let (mut core, mut limits) = make_tree();
    limits.call_records = ROOM;
    core.kept_parts.raise(ROOM);
    let old = part(1, 1, 9, 0, 0, b"old");
    let prefix = part(2, 1, 2, 0, 0, b"prefix");
    let later = part(2, 2, 3, 0, 0, b"later");
    claim(&mut core, TASK, old.key.attempt);
    drop(core.keep_part(old.clone()));
    claim(&mut core, TASK, prefix.key.attempt);
    drop(core.keep_part(prefix.clone()));
    drop(core.keep_part(later.clone()));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let record = TurnRecord {
        opens_window: true,
        task: TASK,
        attempt: prefix.key.attempt,
        turn: prefix.turn,
        spent: 0,
        read: None,
        at: Wall::EPOCH,
        transcript: b"turn".as_slice().into(),
    };
    let out = route_accepted_turn(&mut core, &env, tasks::Accepted::New, record.clone());
    assert_erased_parts(&out, &[old.key, prefix.key]);
    assert_eq!(core.kept_parts.len(), 1, "only the later part remains");
    assert_eq!(core.kept_parts.get(&later.key), Some(&later));

    let refill = part(2, 3, 3, 0, 0, b"refill");
    kept(core.keep_part(refill.clone()), &refill);
    let mut replay = route_accepted_turn(&mut core, &env, tasks::Accepted::Already, record.clone());
    let Some(Request::Held(held)) = replay.pop() else { panic!("replayed turn is only acknowledged") };
    let Held::TaskTurnKept { task, attempt, turn } = *held else { panic!("turn acknowledgement") };
    assert_eq!((task, attempt, turn), (record.task, record.attempt, record.turn));
    assert!(replay.is_empty(), "replayed turn writes nothing");

    let mut end = Queue::uncapped();
    core.retire_parts(TASK, None, &mut end);
    assert_erased_parts(&end, &[later.key, refill.key]);
    assert!(core.kept_parts.is_empty());
}

fn assert_erased_parts(out: &Queue<Request>, expected: &[crate::PartKey]) {
    let mut keys = expected.iter();
    for request in out {
        if let Request::Write(Write::Erase(Key::Core(CoreKey::Part(key)))) = request {
            assert_eq!(keys.next(), Some(key));
        }
    }
    assert!(keys.next().is_none(), "every expected kept part is erased");
}

#[test]
fn resume_parts_share_window_room_and_keep_whole_calls_in_call_order() {
    let mut limits = limits();
    limits.call_records = 8;
    let mut core = new_core(config(41), &limits).unwrap();
    let rows = [
        part(1, 1, 2, 0, 0, b"main"),
        part(1, 2, 2, 1, 5, b"ab"),
        part(1, 3, 2, 2, 4, b"abc"),
        part(1, 4, 2, 3, 4, b"de"),
        part(2, 1, 1, 0, 0, b"me"),
        part(2, 2, 1, 3, 1, b"f"),
    ];
    for row in rows.iter().rev() {
        core.kept_parts.insert(row.key, row.clone()).unwrap();
    }
    let cases: [(u32, &[&[u8]]); 4] = [
        (19, &[b"main", b"ab", b"abc", b"de", b"me", b"f"]),
        (16, &[b"main", b"abc", b"de", b"me"]),
        (15, &[b"main", b"me"]),
        (10, &[]),
    ];
    for (limit, bodies) in cases {
        core.settings.resume_bytes = limit;
        let selected = core.resumed_parts(1, 3, 5);
        let mut found = List::with_capacity(8);
        for part in &selected {
            found.push(part.body.as_ref()).unwrap();
            assert!(part.attempt < 3);
        }
        assert_eq!(found.as_slice(), bodies);
    }
    core.settings.resume_bytes = 19;
    assert_eq!(core.resumed_parts(1, 2, 5).len(), 4, "no current-attempt parts");
}

#[test]
fn restored_parts_have_a_room_apart_from_calls_and_name_the_part_on_overflow() {
    let limits = limits();
    let mut core = new_core(config(42), &limits).unwrap();
    let _begin = core.start_begin(false, Wall::EPOCH);
    let call = CallKey { task: 1, attempt: 1, completion: 1, position: 0 };
    assert!(core.restore_call(call, CallPart::Controlled));
    let rows = [part(1, 1, 2, 0, 0, b"opaque"), part(1, 2, 2, 0, 0, b"second")];
    for row in &rows {
        assert_eq!(core.restore_core(CoreRecord::Part(row.clone()), &limits), Restored::Live);
    }
    assert_eq!(core.resumed_parts(1, 2, 0)[0].body.as_ref(), b"opaque");
    let excess = part(1, 3, 2, 0, 0, b"extra");
    assert_eq!(core.restore_core(CoreRecord::Part(excess.clone()), &limits), Restored::Rejected);
    assert_eq!(
        core.start_failure(),
        Some(crate::start::Failure::LiveSet {
            set: crate::start::LiveSet::Parts,
            key: Key::Core(CoreKey::Part(excess.key)),
            held: 3,
            limit: 2,
        })
    );
    for row in &rows {
        assert_eq!(core.kept_parts.get(&row.key), Some(row), "refusal preserves the admitted part");
    }
    assert!(!core.kept_parts.contains_key(&excess.key));
    assert!(core.call_room(CallKey { position: 1, ..call }), "kept parts never take a named call's room");
    assert!(core.call_room(call), "a retained call replays");
}

#[test]
fn a_full_part_room_drops_the_tasks_parts_and_acknowledges_without_blocking() {
    let (mut core, limits) = make_tree();
    assert_eq!(limits.call_records, 2);
    let mine = part(2, 1, 1, 0, 0, b"answer");
    let other = PartRecord {
        key: crate::PartKey { task: OTHER_TASK, attempt: 5, number: 1 },
        ..part(5, 1, 1, 0, 0, b"theirs")
    };
    claim(&mut core, TASK, mine.key.attempt);
    claim(&mut core, OTHER_TASK, other.key.attempt);
    kept(core.keep_part(mine.clone()), &mine);
    kept(core.keep_part(other.clone()), &other); // The other task takes the last place.
    let fresh = CallKey { task: TASK, attempt: 2, completion: 1, position: 0 };
    assert!(core.call_room(fresh), "a full part room leaves named calls their room");

    let next = part(2, 2, 1, 3, 4, b"result");
    let mut out = core.keep_part(next.clone());
    let Some(Request::Write(Write::Erase(Key::Core(CoreKey::Part(erased))))) = out.pop() else {
        panic!("the task's kept parts go")
    };
    assert_eq!(erased, mine.key);
    let Some(Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof))))) = out.pop() else {
        panic!("the acknowledged prefix still advances")
    };
    assert_eq!(proof.kept_part, next.key.number);
    ack(out.pop(), next.key);
    assert!(out.is_empty(), "nothing waits and nothing of this part is written");
    assert!(!core.kept_parts.contains_key(&next.key));
    assert_eq!(core.kept_parts.get(&other.key), Some(&other), "another task keeps its parts");

    let after = part(2, 3, 1, 0, 0, b"later");
    kept(core.keep_part(after.clone()), &after);
}

fn kept(mut out: Queue<Request>, row: &PartRecord) {
    let Some(Request::Write(Write::Save(Record::Core(CoreRecord::Part(saved))))) = out.pop() else {
        panic!("part write before acknowledgement")
    };
    assert_eq!(&saved, row);
    let Some(Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof))))) = out.pop() else {
        panic!("kept prefix write")
    };
    assert_eq!(proof.kept_part, row.key.number);
    ack(out.pop(), row.key);
    assert!(out.is_empty(), "keeping a part writes only its record and prefix before acknowledging");
}

fn ack(request: Option<Request>, expected: crate::PartKey) {
    let Some(Request::Held(held)) = request else { panic!("held acknowledgement") };
    let Held::TaskPartKept { key } = *held else { panic!("part acknowledgement") };
    assert_eq!(key, expected);
}
