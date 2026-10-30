#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod changes;
mod ends;
mod finish;
mod memory;
mod model;
mod moves;
mod programs;
mod selection;
mod sign_in;
mod simulation;
mod start_restore;
mod terms;

mod fuzzy {
    mod simulation;
    mod start_restore;
}
