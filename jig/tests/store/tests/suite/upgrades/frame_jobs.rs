//! Frame reads over an equal-level job upgrade (protocol/store/transcripts.md, 6).
use jig_protocol_store::{Event, Request, TranscriptRead};
use jig_store_world::{patches, root};
use skein_lib::{Token, length::len_u64};
const SEED: u64 = 719;
const TURNS_PER_TASK: usize = 2;
const FRAME_KEY_BYTES: u32 = 18;
const FRAMING_BYTES: usize = 12;

#[test]
fn serving_shell_answers_sites_for_window_reads() {
    let mut world = patches::frame_committed(SEED, patches::FRAME_WINDOWS, false);
    world.patch_sets(patches::FRAME_JOBS);
    world.open_switched().expect("job copy opens");
    let frames = world.contents();
    let mut reads = vec![
        (TranscriptRead::Window { task: root::LIVE, attempt: 1, turn: 1, after: None }, TURNS_PER_TASK),
        (TranscriptRead::Turn { task: root::LIVE, attempt: 1, turn: 1 }, 1),
        (TranscriptRead::Page { task: root::LIVE, backwards: false, after: None }, TURNS_PER_TASK),
        (TranscriptRead::Page { task: root::LIVE, backwards: true, after: None }, TURNS_PER_TASK),
    ];
    // Reverse order reads cannot depend on another frame's context.
    reads.reverse();
    for (read, count) in reads {
        let turns = world.transcript(read);
        assert_eq!(turns.len(), count, "each written site is answered while serving");
        for turn in turns {
            let key_tokens = u32::try_from(patches::FRAME_REPLAY.len()).expect("replay") + FRAME_KEY_BYTES;
            assert_eq!(turn.transcript, patches::fixture_turn(2, patches::FRAME_SAID, &[], key_tokens));
        }
    }
    assert_eq!(world.contents(), frames);
}

#[test]
fn patched_frame_charge_holds_at_bound_and_refuses_one_below() {
    let mut world = patches::frame_committed(SEED, patches::FRAME_WINDOWS, false);
    world.patch_sets(patches::FRAME_JOBS);
    world.open_switched().expect("job copy opens");
    let patched = patches::fixture_turn(
        2,
        patches::FRAME_SAID,
        &[],
        FRAME_KEY_BYTES + u32::try_from(patches::FRAME_REPLAY.len()).expect("replay size"),
    );
    let size = len_u64(jig_test_patches_jig::frames::body(6, &patched).len() + FRAMING_BYTES);
    for (bytes, expected) in [(size, 1), (size - 1, 0)] {
        let events = world.answered(Request::Transcript {
            owner: Token::new(719),
            read: TranscriptRead::Window { task: root::LIVE, attempt: 1, turn: 1, after: None },
            rows: 1,
            bytes,
        });
        if expected == 0 {
            let [Event::Turns { owner, turns, next, cut }] = events.as_slice() else {
                panic!("cut patched frame charge {size}, budget {bytes}: {events:?}")
            };
            assert_eq!(*owner, Token::new(719));
            assert_eq!(turns.as_ref(), &[root::turn(root::LIVE, 1, 1, &[])]);
            assert_eq!(*cut, Some(u32::try_from(patched.len()).expect("fixture length")));
            let stored = patches::fixture_turn(1, patches::FRAME_SAID, patches::FRAME_REPLAY, 0);
            let length = super::frames::log_frame_bytes(&root::turn(root::LIVE, 1, 1, &stored));
            assert_eq!(*next, Some(length), "continue past the stored frame");
        } else {
            let [Event::Turns { turns, .. }] = events.as_slice() else {
                panic!("bounded read: {events:?}");
            };
            assert_eq!(turns.len(), expected, "patched frame charge {size}, budget {bytes}");
        }
    }
}
