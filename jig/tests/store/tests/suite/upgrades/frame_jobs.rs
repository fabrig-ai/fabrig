//! Frame reads over an equal-level job upgrade (protocol/store/transcripts.md, 6).
use jig_protocol_store::{Event, Request, TranscriptRead};
use jig_store_world::{root, upgrade_fixtures};
use skein_lib::{Token, length::len_u64};
const SEED: u64 = 719;
const TURNS_PER_TASK: usize = 2;
const FRAMING_BYTES: usize = 12;

#[test]
fn window_reads_convert_without_calling_the_shell() {
    let mut world = upgrade_fixtures::frame_committed(SEED, upgrade_fixtures::FRAME_WINDOWS, false);
    world.owner_sets(upgrade_fixtures::FRAME_JOBS);
    world.open_switched().expect("job copy opens");
    let frames = world.contents();
    let mut reads = vec![
        (TranscriptRead::Window { task: root::LIVE, attempt: 1, turn: 1, after: None, limits: None }, TURNS_PER_TASK),
        (TranscriptRead::Turn { task: root::LIVE, attempt: 1, turn: 1 }, 1),
        (TranscriptRead::Page { task: root::LIVE, backwards: false, after: None }, TURNS_PER_TASK),
        (TranscriptRead::Page { task: root::LIVE, backwards: true, after: None }, TURNS_PER_TASK),
    ];
    // Reverse order reads cannot depend on another frame's context.
    reads.reverse();
    for (read, count) in reads {
        let turns = world.transcript(read);
        assert_eq!(turns.len(), count, "each kept turn converts while serving");
        for turn in turns {
            assert_eq!(turn.transcript, upgrade_fixtures::fixture_turn(2, upgrade_fixtures::FRAME_SAID, &[], 0));
        }
    }
    assert_eq!(world.contents(), frames);
}

#[test]
fn converted_frame_charge_holds_at_bound_and_refuses_one_below() {
    let mut world = upgrade_fixtures::frame_committed(SEED, upgrade_fixtures::FRAME_WINDOWS, false);
    world.owner_sets(upgrade_fixtures::FRAME_JOBS);
    world.open_switched().expect("job copy opens");
    let converted = upgrade_fixtures::fixture_turn(2, upgrade_fixtures::FRAME_SAID, &[], 0);
    let size = len_u64(jig_test_layouts::frames::body(6, &converted).len() + FRAMING_BYTES);
    for (bytes, expected) in [(size, 1), (size - 1, 0)] {
        let events = world.request(Request::Transcript {
            owner: Token::new(719),
            read: TranscriptRead::Window { task: root::LIVE, attempt: 1, turn: 1, after: None, limits: None },
            rows: 1,
            bytes,
        });
        if expected == 0 {
            let [Event::Turns { owner, turns, next, cut }] = events.as_slice() else {
                panic!("cut converted frame charge {size}, budget {bytes}: {events:?}")
            };
            assert_eq!(*owner, Token::new(719));
            assert_eq!(turns.as_ref(), &[root::turn(root::LIVE, 1, 1, &[])]);
            assert_eq!(*cut, Some(u32::try_from(converted.len()).expect("fixture length")));
            let stored =
                upgrade_fixtures::fixture_turn(1, upgrade_fixtures::FRAME_SAID, upgrade_fixtures::FRAME_REPLAY, 0);
            let length = super::frames::log_frame_bytes(&root::turn(root::LIVE, 1, 1, &stored));
            assert_eq!(*next, Some(length), "continue past the stored frame");
        } else {
            let [Event::Turns { turns, .. }] = events.as_slice() else {
                panic!("bounded read: {events:?}");
            };
            assert_eq!(turns.len(), expected, "converted frame charge {size}, budget {bytes}");
        }
    }
}
