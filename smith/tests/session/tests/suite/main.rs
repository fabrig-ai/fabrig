//! Session integration tests.

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod budget;
mod child_final;
mod compaction;
mod jobs;
mod memory;
mod paths;
mod recorded;
mod referee;
mod simulation;
mod startup_receiving;
mod wind_down;
mod window_room;

mod fuzzy {
    mod recorded;
    mod simulation;
}
