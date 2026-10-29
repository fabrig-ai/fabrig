#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod completion_retry;
mod memory;
mod referee;
mod restart;
mod simulation;

mod fuzzy {
    mod simulation;
}
