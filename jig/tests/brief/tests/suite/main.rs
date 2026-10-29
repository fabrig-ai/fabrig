#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod referee;
mod simulation;

mod fuzzy {
    mod simulation;
}
