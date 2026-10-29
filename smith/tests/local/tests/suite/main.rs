#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod memory;
mod messages;
mod person;
mod referee;
mod simulation;

mod fuzzy {
    mod simulation;
}
