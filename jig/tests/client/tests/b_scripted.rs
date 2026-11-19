//! Scripted person stories (jig/docs/design/client/domain/testing.md, 4).
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

use jig_client_world::stories;

fn scripted(
    _facts: stories::shared::b_seed::Facts,
) -> jig_client_world::World<jig_client_world::Test, jig_client_world::Scripted> {
    stories::seeded_world(8510)
}
mod scripted {
    jig_client_world::person_story_suite!(super::scripted;
        release: [ignore = "scripted stopped holds release without the Released notice emitted for placed holds"],
        compound_release: [],
        inbox_chat: [ignore = "scripted proposal decisions do not append the decision to their chat activity"],
        close: [], family: []);
}
