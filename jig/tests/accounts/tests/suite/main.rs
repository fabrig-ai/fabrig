#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod changes;
mod ends;
mod finish;
mod forget_project;
mod lending;
mod lists;
mod memory;
mod model;
mod moves;
mod programs;
mod rates;
mod selection;
mod sign_in;
mod simulation;
mod start_restore;
mod terms;
mod usage_reads;

mod fuzzy {
    mod rates;
    mod simulation;
    mod start_restore;
}
