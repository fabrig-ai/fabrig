#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod authentication;
mod composition;
mod configuration;
mod events;
mod lending;
mod live_referee;
mod memory;
mod referee;
mod replay;
mod simulation;
mod terminal;

mod fuzzy {
    mod store;
    mod world;
}
