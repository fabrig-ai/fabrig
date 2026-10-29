//! Run integration tests.

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod jobs;
mod lease;
mod memory;
mod messages;
mod orders;
mod reserve;
mod simulation;
mod wind_down;

mod fuzzy {
    mod messages;
    mod simulation;
}
