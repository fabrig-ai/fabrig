#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod drop_project;
mod referee;
mod restart;
mod simulation;

mod fuzzy {
    mod simulation;
}
