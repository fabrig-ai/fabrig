#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod brief;
mod changes;
mod green;
mod hooks;
mod landing;
mod memory;
mod protection;
mod public;
mod replay;
mod settle;
mod simulation;
mod tiers;
mod update_lost;
mod writer;

mod fuzzy {
    mod branch_patterns;
    mod gate_keys;
    mod simulation;
}

mod kept_change;
