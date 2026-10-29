//! Git protocol worlds (temper/docs/design/domain/testing.md, 2).

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod protocol_git;
mod transcripts;

mod fuzzy {
    mod protocol_git;
}
