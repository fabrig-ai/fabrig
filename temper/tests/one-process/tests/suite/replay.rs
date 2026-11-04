//! The kept visit judged by the forge referee (temper/docs/design/playground.md, 9).

use temper_one_process_world::replay::{Replayed, replay};

#[test]
fn story_a_kept_log_replays_under_the_referee() {
    let replayed =
        replay(&temper_playground::stories::A, include_bytes!("../../../playground/tests/fixtures/story-a.log"));
    assert!(matches!(replayed, Replayed::Complete), "{replayed:#?}");
}
