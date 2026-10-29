#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod jobs;
mod memory;
mod paths;
mod simulation;

mod fuzzy {
    mod jobs;
    mod memory;
    mod simulation;
}
