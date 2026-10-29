#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod budget;
mod memory;
mod messages;
mod referee;
mod simulation;

mod fuzzy {
    mod inline;
    mod simulation;
}
