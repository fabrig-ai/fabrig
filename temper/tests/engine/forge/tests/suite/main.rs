#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod hooks;
mod landing;
mod memory;
mod protection;
mod replay;
mod settle;
mod simulation;
mod tiers;
mod writer;

mod fuzzy {
    mod simulation;
}

mod kept_change;
