#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod simulation;
mod stories;

mod fuzzy {
    mod shared;
    mod stories;
}
