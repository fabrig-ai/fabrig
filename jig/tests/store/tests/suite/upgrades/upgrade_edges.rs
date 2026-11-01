//! Independent upgrade contract checks through the public store world.
//! jig/docs/design/protocol/store.md 8.3, 8.6 and 10.2.
use jig_protocol_store::{Event, PatchSets, Request, TranscriptRead};
use jig_store_world::{World, patches, root};
use skein_lib::Token;

const SEED: u64 = 0x5141_4d31;
const OWNER: Token = Token::new(401);
const TASK: u64 = root::LIVE;
const ATTEMPT: u64 = 1;
const FIRST_TURN: u32 = 1;
const ONE_ROW: u32 = 1;
const TWO_ROWS: u32 = 2;
// root::limits() caps store pages at two rows; request one more.
const THREE_ROWS: u32 = 3;
const NATIVE: &[u8] = b"a complete native turn";
const SAID: &[u8] = b"same visible text";
const REPLAY: &[u8] = b"old replay bytes";
// Fixture v6 body: version(2), window(1), task(8), attempt(8), turn(4),
// spend(8), absent read(1), time(8), transcript length(4); framing adds 12.
// jig-test-patches-jig's public frames::body/wrap give 44 + 12 = 56.
// The clarified direct-read contract charges this plus the transcript for
// Window/Page reads, whether or not a machine is made.
const FRAME_OVERHEAD: u64 = 56;

fn direct_frame_bytes(transcript: &[u8]) -> u64 {
    FRAME_OVERHEAD + u64::try_from(transcript.len()).expect("fixture transcript bytes")
}

fn committed(sets: PatchSets, bodies: &[&[u8]]) -> World {
    let mut world = World::unopened(SEED);
    world.patch_sets(sets);
    world.open_switched().expect("fixture store opens");
    world.commit(root::first()).expect("live task committed");
    for (index, body) in bodies.iter().enumerate() {
        let turn = FIRST_TURN + u32::try_from(index).expect("fixture turn index");
        let commit = u64::from(turn) + 1;
        world.commit(root::turn_commit(commit, TASK, turn, body)).expect("whole turn committed");
    }
    world
}

fn starts() -> [TranscriptRead; 3] {
    [
        TranscriptRead::Window { task: TASK, attempt: ATTEMPT, turn: FIRST_TURN, after: None },
        TranscriptRead::Page { task: TASK, backwards: false, after: None },
        TranscriptRead::Page { task: TASK, backwards: true, after: None },
    ]
}

fn page(world: &mut World, read: TranscriptRead, rows: u32, bytes: u64) -> Vec<Event> {
    world.answered(Request::Transcript { owner: OWNER, read, rows, bytes })
}

fn continued(read: TranscriptRead, offset: u64) -> TranscriptRead {
    match read {
        TranscriptRead::Window { task, attempt, turn, .. } => {
            TranscriptRead::Window { task, attempt, turn, after: Some(offset) }
        }
        TranscriptRead::Page { task, backwards, .. } => TranscriptRead::Page { task, backwards, after: Some(offset) },
        TranscriptRead::Turn { .. } => panic!("these cases use paged complete-turn reads"),
    }
}

fn unread(events: &[Event], context: impl std::fmt::Debug) {
    assert!(matches!(events, [Event::Unread { owner }] if *owner == OWNER), "{context:?}: {events:?}");
}

#[test]
fn native_pages_without_room_end_as_unread() {
    let mut world = committed(PatchSets::NONE, &[NATIVE]);
    let bytes = direct_frame_bytes(NATIVE);
    let rooms = [("zero rows", 0, bytes), ("zero bytes", ONE_ROW, 0), ("one byte short", ONE_ROW, bytes - 1)];
    for read in starts() {
        for (label, rows, bytes) in rooms {
            unread(&page(&mut world, read, rows, bytes), (label, read));
        }
    }
}

#[test]
fn native_pages_accept_exactly_one_whole_turn() {
    let mut world = committed(PatchSets::NONE, &[NATIVE]);
    let bytes = direct_frame_bytes(NATIVE);
    for read in starts() {
        let events = page(&mut world, read, ONE_ROW, bytes);
        let [Event::Turns { owner, turns, next }] = events.as_slice() else {
            panic!("exact native room {read:?}: {events:?}")
        };
        assert_eq!(*owner, OWNER);
        assert_eq!(turns.as_ref(), &[root::turn(TASK, ATTEMPT, FIRST_TURN, NATIVE)]);
        assert_eq!(*next, None, "the only committed turn ends the transcript");
    }
}

#[test]
fn requested_rows_cannot_expand_the_store_page_limit() {
    let mut world = committed(PatchSets::NONE, &[NATIVE, NATIVE, NATIVE]);
    let bytes = direct_frame_bytes(NATIVE) * u64::from(THREE_ROWS);
    let read = TranscriptRead::Page { task: TASK, backwards: false, after: None };
    let events = page(&mut world, read, THREE_ROWS, bytes);
    let [Event::Turns { turns, next: Some(offset), .. }] = events.as_slice() else {
        panic!("store-bounded prefix: {events:?}")
    };
    assert_eq!(turns.iter().map(|turn| turn.turn).collect::<Vec<_>>(), [1, 2]);
    let events = page(&mut world, continued(read, *offset), THREE_ROWS, bytes);
    let [Event::Turns { turns, next: None, .. }] = events.as_slice() else {
        panic!("store-bounded continuation: {events:?}")
    };
    assert_eq!(turns.iter().map(|turn| turn.turn).collect::<Vec<_>>(), [3]);
}

fn mixed_frames() -> (World, u64) {
    let old = patches::fixture_turn(1, SAID, REPLAY, 0);
    let current = patches::fixture_turn(2, SAID, &[], 0);
    let charge = direct_frame_bytes(&current);
    (committed(patches::FRESH_AFTER, &[&old, &current]), charge)
}

#[test]
fn machine_pages_charge_framing_before_accepting_any_turn() {
    let (mut world, charge) = mixed_frames();
    // Forward/window encounter a patched turn first; backward encounters a
    // current turn. Both need the same complete current-body frame charge.
    for read in starts() {
        unread(&page(&mut world, read, TWO_ROWS, charge - 1), read);
    }
}

#[test]
fn accepted_prefix_continues_without_skipping_after_an_unread() {
    let (mut world, charge) = mixed_frames();
    let [window, forward, backward] = starts();
    let directions = [(window, 1, 2), (forward, 1, 2), (backward, 2, 1)];
    for (read, first, second) in directions {
        let events = page(&mut world, read, TWO_ROWS, charge);
        let [Event::Turns { owner, turns, next: Some(offset) }] = events.as_slice() else {
            panic!("one accepted prefix {read:?}: {events:?}")
        };
        assert_eq!(*owner, OWNER);
        assert_eq!(turns.iter().map(|turn| turn.turn).collect::<Vec<_>>(), [first]);
        assert_eq!(
            turns
                .iter()
                .map(|turn| u64::try_from(turn.transcript.len()).expect("fixture size") + FRAME_OVERHEAD)
                .collect::<Vec<_>>(),
            [charge]
        );
        let next = continued(read, *offset);
        unread(&page(&mut world, next, TWO_ROWS, charge - 1), next);
        let events = page(&mut world, next, TWO_ROWS, charge);
        let [Event::Turns { owner, turns, next: None }] = events.as_slice() else {
            panic!("retry keeps the next committed turn {read:?}: {events:?}")
        };
        assert_eq!(*owner, OWNER);
        assert_eq!(turns.iter().map(|turn| turn.turn).collect::<Vec<_>>(), [second]);
        assert_eq!(
            turns
                .iter()
                .map(|turn| u64::try_from(turn.transcript.len()).expect("fixture size") + FRAME_OVERHEAD)
                .collect::<Vec<_>>(),
            [charge]
        );
    }
}
