#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod memory;
mod pieces;
mod simulation;
mod sources;
mod turns;

mod fuzzy {
    mod simulation;
    mod turns;
}
