#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod jobs;
mod listing;
mod memory;
mod stories;

mod fuzzy {
    mod listing;
    mod worlds;
}
