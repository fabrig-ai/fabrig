#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod authentication;
mod configuration;
mod composition;
mod events;
mod lending;
mod memory;
mod referee;
mod replay;
mod simulation;
mod terminal;

mod fuzzy {
    mod store;
    mod world;
}
