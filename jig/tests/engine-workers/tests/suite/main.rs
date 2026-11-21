#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod cascades;
mod grants;
mod hello;
mod jobs;
mod liveness;
mod memory;
mod opening;
mod pause;
mod parts;
mod prepared;
mod records;
mod relinks;
mod restarting;
mod retime;
mod sources;
mod stopping;

mod fuzzy {
    mod engine_side;
}

mod figures;
