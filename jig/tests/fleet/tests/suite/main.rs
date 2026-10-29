#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod host_kinds;
mod memory;
mod models;
mod notices;
mod program_sources;
mod referee;
mod simulation;
mod turns;

mod fuzzy {
    mod simulation;
    mod turns;
}
