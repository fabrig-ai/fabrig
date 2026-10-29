//! Seeded direct Open openings keep the person's words under simulated I/O schedules.
use crate::direct_opening;

#[test]
fn seeded_open_chats_replay_the_whole_first_user_message() {
    for seed in 0..16 {
        for anthropic in [false, true] {
            skein_world::domain::assert_replays(seed, seed + 1, |seed| {
                let world = direct_opening::direct(seed, anthropic);
                (world.trace(), world.answer())
            });
        }
    }
}
