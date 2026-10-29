#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod cascades;
mod grants;
mod hello;
mod liveness;
mod memory;
mod opening;
mod records;
mod sources;
mod stopping;

mod fuzzy {
    mod engine_side;
}
