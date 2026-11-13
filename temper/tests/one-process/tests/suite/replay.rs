//! The kept visit judged by the forge referee (temper/docs/design/playground.md, 9).

use temper_one_process_world::replay::{Replayed, replay};

#[test]
#[ignore = "story A's kept log is recorded by hand, not per landing (2026-11-12): record it, then run this"]
fn story_a_kept_log_replays_under_the_referee() {
    let replayed =
        replay(&temper_playground::stories::A, include_bytes!("../../../playground/tests/fixtures/story-a.log"));
    assert!(matches!(replayed, Replayed::Complete), "{replayed:#?}");
}
